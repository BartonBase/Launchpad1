//! `settle_capture` / `settle_reroll`: PERMISSIONLESS (anyone, e.g. a crank; the user can't hold
//! it back). Strict FIFO (only the head request), merges incoming NFTs whose tag <= request.seq,
//! reads the revealed Switchboard value for the commit recorded in the request, picks uniformly
//! uniformly over EVERY index the vault holds (minted or not) and gives that exact asset to the user.
//! LAZY MINT (ADR-016): if the pick was never minted, the settler must pass its committed leaf +
//! proof; it is verified against the trait root and minted straight to the user, paid from the
//! request's mint escrow (never the settler's lamports, T-HV-16; no tip). Otherwise it's transferred.
//! No tokens move here: the ratio and fee were paid at request time (ADR-013); there is no burn (A-07).

use crate::{
    asset_source, config, constants::*, error::VaultError, invariants, pool::PoolView, selection,
    state::{RandLock, Request, Vault},
};
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
pub struct Settle<'info> {
    /// Anyone (crank). Pays nothing but the tx fee.
    #[account(mut)]
    pub settler: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: address-pinned; read raw by config::exit_view (M-41: user-exit path, upgrade-proof).
    #[account(address = vault.launch_config)]
    pub launch_config: UncheckedAccount<'info>,

    /// CHECK: validated by PoolView::load + address pinned.
    #[account(mut, address = vault.pool, owner = crate::ID)]
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

    /// CHECK: == request.randomness (has_one); only used to derive/close the rand_lock. The value
    /// is read from the request, where `reveal_randomness` recorded it (M-04 H1 / M-35).
    pub randomness: UncheckedAccount<'info>,

    /// CHECK: PDA; collection update authority (signs a lazy mint; never pays).
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: this request's mint escrow PDA `["mint_escrow", vault, seq]` (Core payer for a lazy
    /// mint); drained to the user at the end of settle.
    #[account(mut, seeds = [MINT_ESCROW_SEED, vault.key().as_ref(), &request.seq.to_le_bytes()], bump)]
    pub mint_escrow: UncheckedAccount<'info>,

    #[account(seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    /// CHECK: must be the asset PDA selected by the randomness (checked in handler).
    #[account(mut)]
    pub asset: UncheckedAccount<'info>,
    /// CHECK: the vault's collection.
    #[account(mut, address = vault.collection)]
    pub collection: UncheckedAccount<'info>,

    /// CHECK: request.user (has_one); receives the NFT and the request/lock rent.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_settle(ctx: Context<Settle>, expected_kind: u8, mint: Option<asset_source::MintArgs>) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let req = &ctx.accounts.request;
    require!(req.kind == expected_kind, VaultError::WrongRequestKind);
    require!(req.seq == ctx.accounts.vault.next_settle_seq, VaultError::OutOfOrder);
    require!(ctx.accounts.rand_lock.seq == req.seq && ctx.accounts.rand_lock.vault == vault_key, VaultError::RandomnessMismatch);
    require!(req.revealed, VaultError::RandomnessNotRevealed);
    let value = req.value;
    let econ = config::exit_view(&ctx.accounts.launch_config.to_account_info())?;
    let (seq, handed_in) = (req.seq, req.handed_in_index);

    // Select (pool fixed by FIFO + seq-bounded merge).
    let picked_index = {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        let mut pool = PoolView::load(&mut data, &vault_key)?;
        pool.merge(seq, MAX_MERGE_PER_IX)?;
        if let Some(t) = pool.incoming_front_tag() {
            require!(t > seq, VaultError::MergeBacklog);
        }
        let n = pool.pool_len();
        require!(n > 0, VaultError::NoAssetAvailable);
        let r = selection::request_randomness(&value, &vault_key, seq);
        let pos = selection::uniform_below(&r, n);
        let picked = pool.pool_swap_remove(pos)?;
        if expected_kind == REQUEST_KIND_REROLL {
            // The hand-in becomes drawable for later requests only (never for its own).
            pool.pool_push(handed_in)?;
        }
        picked
    };
    let (expected_asset, _) = asset_source::asset_address(&vault_key, picked_index);
    require_keys_eq!(ctx.accounts.asset.key(), expected_asset, VaultError::WrongAsset);

    let already_minted = {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        PoolView::load(&mut data, &vault_key)?.is_minted(picked_index)
    };
    let a = &ctx.accounts;
    if already_minted {
        asset_source::deliver(
            &a.mpl_core_program.to_account_info(),
            &a.asset.to_account_info(),
            &a.collection.to_account_info(),
            &a.settler.to_account_info(),
            &a.vault_authority.to_account_info(),
            &a.user.to_account_info(),
            &a.system_program.to_account_info(),
            &vault_key,
            a.vault.authority_bump,
        )?;
    } else {
        let m = mint.ok_or_else(|| error!(VaultError::MintArgsMissing))?;
        asset_source::check_leaf(&a.vault.trait_root, &a.vault.trait_schema_hash, &a.vault.launch_config, econ.collection_size, picked_index, &m.leaf, &m.proof)?;
        let seq_le = seq.to_le_bytes();
        let escrow_seeds: &[&[u8]] = &[MINT_ESCROW_SEED, vault_key.as_ref(), &seq_le, &[ctx.bumps.mint_escrow]];
        asset_source::mint_to_user(
            &a.mpl_core_program.to_account_info(),
            &a.asset.to_account_info(),
            &a.collection.to_account_info(),
            &a.vault_authority.to_account_info(),
            &a.mint_escrow.to_account_info(),
            &a.user.to_account_info(),
            &a.system_program.to_account_info(),
            &vault_key,
            picked_index,
            a.vault.authority_bump,
            escrow_seeds,
            asset_source::asset_name(picked_index),
            m.leaf.uri,
        )?;
        ctx.accounts.request.mint_escrow_lamports = 0;
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        PoolView::load(&mut data, &vault_key)?.mark_minted(picked_index)?;
        let v = &mut ctx.accounts.vault;
        v.minted_count = v.minted_count.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    }

    {
        // Whatever the escrow still holds (all of it on the transfer path) goes back to the user.
        let a = &ctx.accounts;
        let seq_le = seq.to_le_bytes();
        let escrow_seeds: &[&[u8]] = &[MINT_ESCROW_SEED, vault_key.as_ref(), &seq_le, &[ctx.bumps.mint_escrow]];
        asset_source::refund_escrow(&a.mint_escrow.to_account_info(), &a.user.to_account_info(), &a.system_program.to_account_info(), escrow_seeds)?;
    }
    ctx.accounts.request.mint_escrow_lamports = 0;

    let v = &mut ctx.accounts.vault;
    v.next_settle_seq = seq.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    v.assets_outside = v.assets_outside.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    if expected_kind == REQUEST_KIND_CAPTURE {
        v.pending_captures = v.pending_captures.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
        v.total_captures = v.total_captures.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    } else {
        v.pending_rerolls = v.pending_rerolls.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
        v.total_rerolls = v.total_rerolls.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    }

    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, econ.ratio_base, econ.collection_size, &pool, ctx.accounts.vault_tokens.amount)
}
