//! `expire_request`: the LAST-RESORT recovery (audit M-04 rule 5, B's H4). PERMISSIONLESS, head of
//! the FIFO only, and only when the request's randomness was NEVER revealed after 1 + MAX_RECOMMITS
//! commits (each with a different oracle) and EXPIRE_GRACE_SLOTS (~1 day) have passed since the
//! last deadline. It returns the PRINCIPAL ONLY: N tokens for a capture, the handed-in NFT for a
//! re-roll, plus the FULL lazy-mint escrow (ADR-016). The flat SOL fee is NEVER refunded (there is no token fee). A request whose value was revealed
//! can't be expired (settle is the only way out), so nobody can discard a revealed draw.

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

pub fn handle_expire(ctx: Context<ExpireRequest>) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let econ = config::exit_view(&ctx.accounts.launch_config.to_account_info())?;
    let req = &ctx.accounts.request;
    require!(req.seq == ctx.accounts.vault.next_settle_seq, VaultError::OutOfOrder);
    require!(!req.revealed, VaultError::RandomnessAlreadyRevealed);
    {
        let snap = randomness::read(&ctx.accounts.randomness)?;
        require!(!randomness::is_revealed(&snap, req.seed_slot), VaultError::RandomnessAlreadyRevealed);
    }
    require!(req.commits > MAX_RECOMMITS, VaultError::RecommitsRemaining);
    let earliest = req.deadline_slot.checked_add(EXPIRE_GRACE_SLOTS).ok_or_else(|| error!(VaultError::MathOverflow))?;
    require!(Clock::get()?.slot > earliest, VaultError::ExpiryNotReached);
    let (kind, handed_in, seq) = (req.kind, req.handed_in_index, req.seq);

    let a = &ctx.accounts;
    if kind == REQUEST_KIND_CAPTURE {
        pay_out(
            &a.token_program, &a.vault_tokens, &a.user_token.to_account_info(), &a.mint,
            &a.vault_authority.to_account_info(), &vault_key, a.vault.authority_bump, econ.ratio_base,
        )?;
    } else {
        let (expected, _) = asset_source::asset_address(&vault_key, handed_in);
        require_keys_eq!(a.asset.key(), expected, VaultError::WrongAsset);
        asset_source::deliver(
            &a.mpl_core_program.to_account_info(), &a.asset.to_account_info(), &a.collection.to_account_info(),
            &a.caller.to_account_info(), &a.vault_authority.to_account_info(), &a.user.to_account_info(),
            &a.system_program.to_account_info(), &vault_key, a.vault.authority_bump,
        )?;
    }

    {
        let seq_le = seq.to_le_bytes();
        let escrow_seeds: &[&[u8]] = &[MINT_ESCROW_SEED, vault_key.as_ref(), &seq_le, &[ctx.bumps.mint_escrow]];
        let a = &ctx.accounts;
        asset_source::refund_escrow(&a.mint_escrow.to_account_info(), &a.user.to_account_info(), &a.system_program.to_account_info(), escrow_seeds)?;
    }

    let v = &mut ctx.accounts.vault;
    v.next_settle_seq = seq.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    v.total_expired = v.total_expired.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    if kind == REQUEST_KIND_CAPTURE {
        v.pending_captures = v.pending_captures.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    } else {
        v.pending_rerolls = v.pending_rerolls.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
        v.assets_outside = v.assets_outside.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    }
    ctx.accounts.vault_tokens.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, econ.ratio_base, econ.collection_size, &pool, ctx.accounts.vault_tokens.amount)
}
