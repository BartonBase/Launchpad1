//! `expire_request`: permissionless, never pausable, head-of-queue only, after `deadline_slot`, and
//! only if the randomness for this request's commit was NOT revealed (a revealed request must be
//! settled, so nobody can see the result and back out). Refunds exactly what was locked: the ratio
//! (capture) or the handed-in NFT (re-roll), plus the escrowed fee, to the destinations recorded at
//! request time.

use crate::{
    constants::*, core_cpi, error::VaultError, invariants, pool::PoolView, randomness,
    state::{RandLock, Request, Vault},
};
use super::vault_token_ops::pay_out;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct ExpireRequest<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump, has_one = mint)]
    pub vault: Box<Account<'info, Vault>>,

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
        has_one = user_token,
    )]
    pub request: Box<Account<'info, Request>>,

    #[account(mut, close = user, seeds = [RAND_LOCK_SEED, randomness.key().as_ref()], bump = rand_lock.bump)]
    pub rand_lock: Box<Account<'info, RandLock>>,

    /// CHECK: request.randomness (has_one); parsed to prove it was not revealed.
    pub randomness: UncheckedAccount<'info>,

    /// CHECK: request.user (has_one).
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    /// Refund destination recorded at request time (has_one).
    #[account(mut, token::mint = mint)]
    pub user_token: Box<Account<'info, TokenAccount>>,

    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut, seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [FEE_ESCROW_SEED, vault.key().as_ref()], bump = vault.fee_escrow_bump)]
    pub fee_escrow: Box<Account<'info, TokenAccount>>,

    /// CHECK: required for re-roll requests: the handed-in asset PDA (checked in handler).
    #[account(mut)]
    pub asset: Option<UncheckedAccount<'info>>,
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
    let req = &ctx.accounts.request;
    require!(req.seq == ctx.accounts.vault.next_settle_seq, VaultError::OutOfOrder);
    require!(Clock::get()?.slot > req.deadline_slot, VaultError::DeadlineNotReached);
    let snap = randomness::read(&ctx.accounts.randomness)?;
    require!(!randomness::is_revealed(&snap, req.seed_slot), VaultError::RandomnessAlreadyRevealed);
    let (seq, kind, fee, handed_in) = (req.seq, req.kind, req.fee_amount, req.handed_in_index);

    let a = &ctx.accounts;
    let bump = a.vault.authority_bump;
    let va = a.vault_authority.to_account_info();
    if kind == REQUEST_KIND_CAPTURE {
        pay_out(&a.token_program, &a.vault_tokens, &a.user_token.to_account_info(), &a.mint, &va, &vault_key, bump, a.vault.ratio_base)?;
    } else {
        let asset = a.asset.as_ref().ok_or_else(|| error!(VaultError::MissingAsset))?;
        let (expected, _) = Pubkey::find_program_address(&[ASSET_SEED, vault_key.as_ref(), &handed_in.to_le_bytes()], &crate::ID);
        require_keys_eq!(asset.key(), expected, VaultError::WrongAsset);
        let auth_seeds: &[&[u8]] = &[VAULT_AUTHORITY_SEED, vault_key.as_ref(), &[bump]];
        core_cpi::transfer_asset(
            &a.mpl_core_program.to_account_info(),
            &asset.to_account_info(),
            &a.collection.to_account_info(),
            &a.caller.to_account_info(),
            &va,
            &a.user.to_account_info(),
            &a.system_program.to_account_info(),
            Some(auth_seeds),
        )?;
        core_cpi::assert_asset_state(asset, &a.vault.collection, &a.user.key())?;
    }
    pay_out(&a.token_program, &a.fee_escrow, &a.user_token.to_account_info(), &a.mint, &va, &vault_key, bump, fee)?;

    let v = &mut ctx.accounts.vault;
    v.next_settle_seq = seq.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    v.pending_fee_total = v.pending_fee_total.checked_sub(fee).ok_or_else(|| error!(VaultError::MathOverflow))?;
    if kind == REQUEST_KIND_CAPTURE {
        v.pending_captures = v.pending_captures.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    } else {
        v.pending_rerolls = v.pending_rerolls.checked_sub(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
        v.assets_outside = v.assets_outside.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    }

    ctx.accounts.vault_tokens.reload()?;
    ctx.accounts.fee_escrow.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, &pool, ctx.accounts.vault_tokens.amount, ctx.accounts.fee_escrow.amount)
}
