//! `settle_capture` / `settle_reroll`: permissionless. Strict FIFO (only the head request), merges
//! incoming NFTs whose tag <= request.seq, reads the revealed Switchboard value for the commit made
//! at request time, picks uniformly from the pool, sends that exact asset to the user and burns the fee.

use crate::{
    constants::*, core_cpi, error::VaultError, invariants, pool::PoolView, randomness, selection,
    state::{RandLock, Request, Vault},
};
use super::vault_token_ops::burn_fee;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Settle<'info> {
    /// Anyone (crank). Pays nothing but the tx fee.
    #[account(mut)]
    pub settler: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump, has_one = mint)]
    pub vault: Box<Account<'info, Vault>>,

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

    /// CHECK: must equal request.randomness (has_one); owner/layout checked in randomness::revealed_value.
    pub randomness: UncheckedAccount<'info>,

    /// CHECK: PDA.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut, seeds = [FEE_ESCROW_SEED, vault.key().as_ref()], bump = vault.fee_escrow_bump)]
    pub fee_escrow: Box<Account<'info, TokenAccount>>,

    #[account(seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub mint: Box<Account<'info, Mint>>,

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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_settle(ctx: Context<Settle>, expected_kind: u8) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let req = &ctx.accounts.request;
    require!(req.kind == expected_kind, VaultError::WrongRequestKind);
    require!(req.seq == ctx.accounts.vault.next_settle_seq, VaultError::OutOfOrder);
    let value = randomness::revealed_value(&ctx.accounts.randomness, req.seed_slot)?;
    let (seq, fee, handed_in) = (req.seq, req.fee_amount, req.handed_in_index);

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
    let (expected_asset, _) = Pubkey::find_program_address(
        &[ASSET_SEED, vault_key.as_ref(), &picked_index.to_le_bytes()],
        &crate::ID,
    );
    require_keys_eq!(ctx.accounts.asset.key(), expected_asset, VaultError::WrongAsset);

    let a = &ctx.accounts;
    let auth_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, vault_key.as_ref(), &[a.vault.authority_bump]];
    core_cpi::transfer_asset(
        &a.mpl_core_program.to_account_info(),
        &a.asset.to_account_info(),
        &a.collection.to_account_info(),
        &a.settler.to_account_info(),
        &a.vault_authority.to_account_info(),
        &a.user.to_account_info(),
        &a.system_program.to_account_info(),
        Some(auth_seeds),
    )?;
    core_cpi::assert_asset_state(&a.asset, &a.vault.collection, &a.user.key())?;
    burn_fee(&a.token_program, &a.fee_escrow, &a.mint, &a.vault_authority.to_account_info(), &vault_key, a.vault.authority_bump, fee)?;

    let v = &mut ctx.accounts.vault;
    v.next_settle_seq = seq.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    v.pending_fee_total = v.pending_fee_total.checked_sub(fee).ok_or_else(|| error!(VaultError::MathOverflow))?;
    v.total_burned = v.total_burned.checked_add(fee).ok_or_else(|| error!(VaultError::MathOverflow))?;
    v.assets_outside = v.assets_outside.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    if expected_kind == REQUEST_KIND_CAPTURE {
        v.pending_captures = v.pending_captures.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
        v.total_captures = v.total_captures.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    } else {
        v.pending_rerolls = v.pending_rerolls.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
        v.total_rerolls = v.total_rerolls.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    }

    ctx.accounts.fee_escrow.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, &pool, ctx.accounts.vault_tokens.amount, ctx.accounts.fee_escrow.amount)
}
