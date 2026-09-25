//! `expire_request`: the LAST-RESORT recovery (audit M-04 rule 5, B's H4). PERMISSIONLESS, head of
//! the FIFO only, and only when the request's randomness was NEVER revealed after 1 + MAX_RECOMMITS
//! commits (each with a different oracle) and EXPIRE_GRACE_SLOTS (~1 day) have passed since the
//! last deadline. It returns the PRINCIPAL ONLY: N tokens for a capture, the handed-in NFT for a
//! re-roll, plus the FULL lazy-mint escrow (ADR-016). The flat SOL fee is NEVER refunded (there is no token fee). A request whose value was revealed
//! can't be expired (settle is the only way out), so nobody can discard a revealed draw.
//!
//! `expire_requests(count)` (M-04 batch): the same rules applied to the next `count` consecutive
//! queue heads (1..=MAX_EXPIRE_PER_CALL) in ONE instruction. Each request's accounts come in
//! remaining_accounts, EXPIRE_BATCH_STRIDE (7) per request, in queue order:
//!   request (w), rand_lock (w), randomness, user (w), user_token (w), mint_escrow (w), asset (w).
//! Every account is validated by hand exactly as the Anchor constraints of `expire_request` do.
//! All-or-nothing: if any head isn't expirable, the whole instruction fails.

use super::vault_token_ops::pay_out;
use crate::{
    asset_source, config, constants::*, error::VaultError, invariants, pool::PoolView, randomness,
    state::{RandLock, Request, Vault},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct ExpireRequest<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,
    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK: address-pinned; read raw by config::exit_view (M-41: user-exit path, upgrade-proof).
    #[account(address = vault.launch_config)]
    pub launch_config: UncheckedAccount<'info>,
    /// CHECK: validated by PoolView::load + address pinned.
    #[account(address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,
    #[account(
        mut,
        close = user,
        seeds = [REQUEST_SEED, vault.key().as_ref(), &request.seq.to_le_bytes()],
        bump = request.bump,
        has_one = vault,
        has_one = user,
        has_one = randomness,
    )]
    pub request: Box<Account<'info, Request>>,
    #[account(mut, close = user, seeds = [RAND_LOCK_SEED, randomness.key().as_ref()], bump = rand_lock.bump)]
    pub rand_lock: Box<Account<'info, RandLock>>,
    /// CHECK: == request.randomness; read to confirm it was never revealed.
    pub randomness: UncheckedAccount<'info>,
    /// CHECK: request.user (has_one); receives principal + rent.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    /// The user's token account for the capture refund of N (principal only).
    #[account(mut, token::mint = mint, token::authority = user)]
    pub user_token: Box<Account<'info, TokenAccount>>,
    #[account(address = vault.mint @ VaultError::MintMismatch)]
    pub mint: Box<Account<'info, Mint>>,
    /// CHECK: PDA.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut, seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,
    /// CHECK: this request's mint escrow PDA; refunded IN FULL to the user (ADR-016).
    #[account(mut, seeds = [MINT_ESCROW_SEED, vault.key().as_ref(), &request.seq.to_le_bytes()], bump)]
    pub mint_escrow: UncheckedAccount<'info>,
    /// CHECK: for a re-roll, must be the handed-in asset PDA (checked in handler).
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,
    /// CHECK: the vault's collection.
    #[account(mut, address = vault.collection)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// One expirable request's accounts (already validated by the caller).
pub struct ExpireOne<'a, 'info> {
    pub request: &'a Request,
    pub randomness: &'a AccountInfo<'info>,
    pub user: &'a AccountInfo<'info>,
    pub user_token: &'a AccountInfo<'info>,
    pub mint_escrow: &'a AccountInfo<'info>,
    pub mint_escrow_bump: u8,
    pub asset: &'a AccountInfo<'info>,
}

/// Shared vault-side accounts.
pub struct ExpireShared<'a, 'info> {
    pub caller: &'a AccountInfo<'info>,
    pub vault_key: Pubkey,
    pub authority_bump: u8,
    pub mint: &'a Account<'info, Mint>,
    pub vault_authority: &'a AccountInfo<'info>,
    pub vault_tokens: &'a Account<'info, TokenAccount>,
    pub collection: &'a AccountInfo<'info>,
    pub mpl_core_program: &'a AccountInfo<'info>,
    pub token_program: &'a Program<'info, Token>,
    pub system_program: &'a AccountInfo<'info>,
    pub ratio_base: u64,
}

/// Checks that `r` is expirable as the queue head `expected_seq`, returns the principal (tokens or
/// the handed-in NFT) and the full mint escrow to the user, and updates the vault counters. The
/// caller closes the request + rand_lock to the user and runs the invariant check.
pub fn expire_one<'info>(sh: &ExpireShared<'_, 'info>, r: &ExpireOne<'_, 'info>, vault: &mut Vault) -> Result<()> {
    let req = r.request;
    require!(req.seq == vault.next_settle_seq, VaultError::OutOfOrder);
    require!(!req.revealed, VaultError::RandomnessAlreadyRevealed);
    {
        let snap = randomness::read(r.randomness)?;
        require!(!randomness::is_revealed(&snap, req.seed_slot), VaultError::RandomnessAlreadyRevealed);
    }
    require!(req.commits > MAX_RECOMMITS, VaultError::RecommitsRemaining);
    let earliest = req.deadline_slot.checked_add(EXPIRE_GRACE_SLOTS).ok_or_else(|| error!(VaultError::MathOverflow))?;
    require!(Clock::get()?.slot > earliest, VaultError::ExpiryNotReached);
    let (kind, handed_in, seq) = (req.kind, req.handed_in_index, req.seq);

    if kind == REQUEST_KIND_CAPTURE {
        pay_out(sh.token_program, sh.vault_tokens, r.user_token, sh.mint, sh.vault_authority, &sh.vault_key, sh.authority_bump, sh.ratio_base)?;
    } else {
        let (expected, _) = asset_source::asset_address(&sh.vault_key, handed_in);
        require_keys_eq!(r.asset.key(), expected, VaultError::WrongAsset);
        asset_source::deliver(
            sh.mpl_core_program, r.asset, sh.collection, sh.caller, sh.vault_authority, r.user, sh.system_program,
            &sh.vault_key, sh.authority_bump,
        )?;
    }
    {
        let seq_le = seq.to_le_bytes();
        let escrow_seeds: &[&[u8]] = &[MINT_ESCROW_SEED, sh.vault_key.as_ref(), &seq_le, &[r.mint_escrow_bump]];
        asset_source::refund_escrow(r.mint_escrow, r.user, sh.system_program, escrow_seeds)?;
    }
    vault.next_settle_seq = seq.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    vault.total_expired = vault.total_expired.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    if kind == REQUEST_KIND_CAPTURE {
        vault.pending_captures = vault.pending_captures.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    } else {
        vault.pending_rerolls = vault.pending_rerolls.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
        vault.assets_outside = vault.assets_outside.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    }
    Ok(())
}

pub fn handle_expire(ctx: Context<ExpireRequest>) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let econ = config::exit_view(&ctx.accounts.launch_config.to_account_info())?;
    {
        let a = &ctx.accounts;
        let (caller, va, coll, core, sys) = (
            a.caller.to_account_info(), a.vault_authority.to_account_info(), a.collection.to_account_info(),
            a.mpl_core_program.to_account_info(), a.system_program.to_account_info(),
        );
        let (rnd, user, ut, esc, asset) = (
            a.randomness.to_account_info(), a.user.to_account_info(), a.user_token.to_account_info(),
            a.mint_escrow.to_account_info(), a.asset.to_account_info(),
        );
        let sh = ExpireShared {
            caller: &caller, vault_key, authority_bump: a.vault.authority_bump, mint: &a.mint, vault_authority: &va,
            vault_tokens: &a.vault_tokens, collection: &coll, mpl_core_program: &core, token_program: &a.token_program,
            system_program: &sys, ratio_base: econ.ratio_base,
        };
        let one = ExpireOne {
            request: &a.request, randomness: &rnd, user: &user, user_token: &ut, mint_escrow: &esc,
            mint_escrow_bump: ctx.bumps.mint_escrow, asset: &asset,
        };
        let mut v: Vault = (**a.vault).clone();
        expire_one(&sh, &one, &mut v)?;
        ctx.accounts.vault.set_inner(v);
    }
    ctx.accounts.vault_tokens.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, econ.ratio_base, econ.collection_size, &pool, ctx.accounts.vault_tokens.amount)
}

/// Batch variant: shared accounts only; per-request accounts in remaining_accounts.
#[derive(Accounts)]
pub struct ExpireRequests<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,
    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK: address-pinned; read raw by config::exit_view (M-41).
    #[account(address = vault.launch_config)]
    pub launch_config: UncheckedAccount<'info>,
    /// CHECK: validated by PoolView::load + address pinned.
    #[account(address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,
    #[account(address = vault.mint @ VaultError::MintMismatch)]
    pub mint: Box<Account<'info, Mint>>,
    /// CHECK: PDA.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(mut, seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,
    /// CHECK: the vault's collection.
    #[account(mut, address = vault.collection)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_expire_batch<'info>(ctx: Context<'info, ExpireRequests<'info>>, count: u8) -> Result<()> {
    let n = count as usize;
    require!(count >= 1 && count <= MAX_EXPIRE_PER_CALL, VaultError::ExpireBatchInvalid);
    require!(ctx.remaining_accounts.len() == n * EXPIRE_BATCH_STRIDE, VaultError::ExpireBatchInvalid);
    let vault_key = ctx.accounts.vault.key();
    let econ = config::exit_view(&ctx.accounts.launch_config.to_account_info())?;
    let a = &ctx.accounts;
    let (caller, va, coll, core, sys) = (
        a.caller.to_account_info(), a.vault_authority.to_account_info(), a.collection.to_account_info(),
        a.mpl_core_program.to_account_info(), a.system_program.to_account_info(),
    );
    let sh = ExpireShared {
        caller: &caller, vault_key, authority_bump: a.vault.authority_bump, mint: &a.mint, vault_authority: &va,
        vault_tokens: &a.vault_tokens, collection: &coll, mpl_core_program: &core, token_program: &a.token_program,
        system_program: &sys, ratio_base: econ.ratio_base,
    };
    let mut v: Vault = (**a.vault).clone();
    let mismatch = || error!(VaultError::ExpireBatchAccountMismatch);
    for g in ctx.remaining_accounts.chunks(EXPIRE_BATCH_STRIDE) {
        let (req_ai, lock_ai, rnd, user, ut, esc, asset) = (&g[0], &g[1], &g[2], &g[3], &g[4], &g[5], &g[6]);
        for w in [req_ai, lock_ai, user, ut, esc, asset] {
            require!(w.is_writable, VaultError::ExpireBatchAccountMismatch);
        }
        // request: our program's Request at ["request", vault, seq] for the CURRENT head.
        let request: Account<'info, Request> = Account::try_from(req_ai).map_err(|_| mismatch())?;
        let seq_le = v.next_settle_seq.to_le_bytes();
        let want = Pubkey::create_program_address(&[REQUEST_SEED, vault_key.as_ref(), &seq_le, &[request.bump]], &crate::ID)
            .map_err(|_| mismatch())?;
        require_keys_eq!(req_ai.key(), want, VaultError::ExpireBatchAccountMismatch);
        require_keys_eq!(request.vault, vault_key, VaultError::ExpireBatchAccountMismatch);
        require_keys_eq!(request.user, user.key(), VaultError::ExpireBatchAccountMismatch);
        require_keys_eq!(request.randomness, rnd.key(), VaultError::ExpireBatchAccountMismatch);
        // rand_lock: ["rand_lock", randomness].
        let lock: Account<'info, RandLock> = Account::try_from(lock_ai).map_err(|_| mismatch())?;
        let want = Pubkey::create_program_address(&[RAND_LOCK_SEED, rnd.key.as_ref(), &[lock.bump]], &crate::ID)
            .map_err(|_| mismatch())?;
        require_keys_eq!(lock_ai.key(), want, VaultError::ExpireBatchAccountMismatch);
        // user_token: classic token account of `mint` owned by the user.
        {
            let t: Account<'info, TokenAccount> = Account::try_from(ut).map_err(|_| mismatch())?;
            require_keys_eq!(t.mint, a.mint.key(), VaultError::ExpireBatchAccountMismatch);
            require_keys_eq!(t.owner, user.key(), VaultError::ExpireBatchAccountMismatch);
        }
        // mint escrow PDA of this seq.
        let (want, esc_bump) = asset_source::mint_escrow_address(&vault_key, request.seq);
        require_keys_eq!(esc.key(), want, VaultError::ExpireBatchAccountMismatch);

        let one = ExpireOne { request: &request, randomness: rnd, user, user_token: ut, mint_escrow: esc, mint_escrow_bump: esc_bump, asset };
        expire_one(&sh, &one, &mut v)?;
        // close request + rand_lock to the user (same as Anchor's `close = user`).
        request.close(user.clone())?;
        lock.close(user.clone())?;
    }
    ctx.accounts.vault.set_inner(v);
    ctx.accounts.vault_tokens.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, econ.ratio_base, econ.collection_size, &pool, ctx.accounts.vault_tokens.amount)
}
