//! `unwrap`: one tx, never pausable. The user returns vault asset `index` and receives exactly
//! `ratio` tokens. No token fee exists on this path. The returned NFT enters `incoming` tagged with
//! next_seq, so it can't be drawn by any request made before it came back.

use crate::{constants::*, core_cpi, error::VaultError, invariants, pool::PoolView, state::Vault};
use super::vault_token_ops::pay_out;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
#[instruction(index: u32)]
pub struct Unwrap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump, has_one = mint)]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: validated by PoolView::load + address pinned.
    #[account(mut, address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: PDA; new owner of the NFT, signs the token payout.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(mut, seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    #[account(seeds = [FEE_ESCROW_SEED, vault.key().as_ref()], bump = vault.fee_escrow_bump)]
    pub fee_escrow: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
        constraint = user_token.key() != vault.vault_tokens && user_token.key() != vault.fee_escrow @ VaultError::AliasedTokenAccount,
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    /// CHECK: must be this vault's asset PDA for `index`; a foreign-collection asset can't match.
    #[account(mut, seeds = [ASSET_SEED, vault.key().as_ref(), &index.to_le_bytes()], bump)]
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

pub fn handle_unwrap(ctx: Context<Unwrap>, index: u32) -> Result<()> {
    // Deliberately NO pause check and NO fee: unwrap must always work and return exactly `ratio`.
    let vault_key = ctx.accounts.vault.key();
    require!(index < ctx.accounts.vault.collection_size, VaultError::IndexOutOfRange);
    let a = &ctx.accounts;

    core_cpi::transfer_asset(
        &a.mpl_core_program.to_account_info(),
        &a.asset.to_account_info(),
        &a.collection.to_account_info(),
        &a.user.to_account_info(),
        &a.user.to_account_info(),
        &a.vault_authority.to_account_info(),
        &a.system_program.to_account_info(),
        None,
    )?;
    core_cpi::assert_asset_state(&a.asset, &a.vault.collection, &a.vault_authority.key())?;

    let ratio = a.vault.ratio_base;
    pay_out(
        &a.token_program,
        &a.vault_tokens,
        &a.user_token.to_account_info(),
        &a.mint,
        &a.vault_authority.to_account_info(),
        &vault_key,
        a.vault.authority_bump,
        ratio,
    )?;

    let tag = a.vault.next_seq;
    {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        let mut pool = PoolView::load(&mut data, &vault_key)?;
        pool.incoming_push(index, tag)?;
    }
    let v = &mut ctx.accounts.vault;
    v.assets_outside = v.assets_outside.checked_sub(1).ok_or_else(|| error!(VaultError::AssetAccountingBroken))?;
    v.total_unwraps = v.total_unwraps.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;

    ctx.accounts.vault_tokens.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, &pool, ctx.accounts.vault_tokens.amount, ctx.accounts.fee_escrow.amount)
}
