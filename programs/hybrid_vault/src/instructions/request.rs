//! `request_capture` / `request_reroll`: lock the payment, create a sequenced request, and commit
//! Switchboard randomness via CPI in the SAME instruction (so nobody can know the value first).
//! No asset argument exists for a capture: the caller can't choose what they get.

use crate::{
    constants::*, core_cpi, error::VaultError, invariants, pool::PoolView, randomness,
    state::{RandLock, Request, Vault},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

#[derive(Accounts)]
pub struct RequestCapture<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump, has_one = mint)]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: validated by PoolView::load + address pinned.
    #[account(address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
        constraint = user_token.key() != vault.vault_tokens && user_token.key() != vault.fee_escrow @ VaultError::AliasedTokenAccount,
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [FEE_ESCROW_SEED, vault.key().as_ref()], bump = vault.fee_escrow_bump)]
    pub fee_escrow: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = user,
        space = 8 + Request::INIT_SPACE,
        seeds = [REQUEST_SEED, vault.key().as_ref(), &vault.next_seq.to_le_bytes()],
        bump
    )]
    pub request: Box<Account<'info, Request>>,

    #[account(
        init,
        payer = user,
        space = 8 + RandLock::INIT_SPACE,
        seeds = [RAND_LOCK_SEED, randomness.key().as_ref()],
        bump
    )]
    pub rand_lock: Box<Account<'info, RandLock>>,

    /// CHECK: Switchboard randomness account; owner + authority validated in randomness::commit_for_request.
    #[account(mut)]
    pub randomness: UncheckedAccount<'info>,
    /// CHECK: PDA; the randomness account's authority.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    pub sb_queue: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_oracle: UncheckedAccount<'info>,
    /// CHECK: address pinned; only forwarded to Switchboard, never read here.
    #[account(address = SLOT_HASHES_SYSVAR_ID)]
    pub slot_hashes: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = SWITCHBOARD_PROGRAM_ID)]
    pub switchboard_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(index: u32)]
pub struct RequestReroll<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump, has_one = mint)]
    pub vault: Box<Account<'info, Vault>>,

    /// CHECK: validated by PoolView::load + address pinned.
    #[account(address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
        constraint = user_token.key() != vault.vault_tokens && user_token.key() != vault.fee_escrow @ VaultError::AliasedTokenAccount,
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    #[account(seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [FEE_ESCROW_SEED, vault.key().as_ref()], bump = vault.fee_escrow_bump)]
    pub fee_escrow: Box<Account<'info, TokenAccount>>,

    /// CHECK: PDA; receives the handed-in asset.
    #[account(seeds = [VAULT_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.authority_bump)]
    pub vault_authority: UncheckedAccount<'info>,

    /// CHECK: must be this vault's asset PDA for `index` (a foreign asset can't match).
    #[account(mut, seeds = [ASSET_SEED, vault.key().as_ref(), &index.to_le_bytes()], bump)]
    pub asset: UncheckedAccount<'info>,
    /// CHECK: the vault's collection.
    #[account(mut, address = vault.collection)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + Request::INIT_SPACE,
        seeds = [REQUEST_SEED, vault.key().as_ref(), &vault.next_seq.to_le_bytes()],
        bump
    )]
    pub request: Box<Account<'info, Request>>,

    #[account(
        init,
        payer = user,
        space = 8 + RandLock::INIT_SPACE,
        seeds = [RAND_LOCK_SEED, randomness.key().as_ref()],
        bump
    )]
    pub rand_lock: Box<Account<'info, RandLock>>,

    /// CHECK: see RequestCapture.
    #[account(mut)]
    pub randomness: UncheckedAccount<'info>,
    /// CHECK: PDA.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    pub sb_queue: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_oracle: UncheckedAccount<'info>,
    /// CHECK: address pinned; only forwarded to Switchboard.
    #[account(address = SLOT_HASHES_SYSVAR_ID)]
    pub slot_hashes: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = SWITCHBOARD_PROGRAM_ID)]
    pub switchboard_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

fn check_open(vault: &Vault, pool: &PoolView) -> Result<()> {
    require!(vault.sealed, VaultError::VaultNotSealed);
    require!(Clock::get()?.slot >= vault.paused_until_slot, VaultError::Paused);
    // Every pool/incoming entry at this moment has tag <= this request's seq, so it will be a
    // candidate at settle unless an earlier pending draw takes it.
    let available = (pool.pool_len() as u64)
        .checked_add(pool.incoming_len() as u64)
        .ok_or_else(|| error!(VaultError::MathOverflow))?;
    require!(available > vault.pending_draws()?, VaultError::NoAssetAvailable);
    Ok(())
}

fn pay<'info>(
    token_program: &Program<'info, Token>,
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    authority: &Signer<'info>,
    mint: &Account<'info, Mint>,
    amount: u64,
) -> Result<()> {
    if amount == 0 {
        return Ok(());
    }
    token::transfer_checked(
        CpiContext::new(
            token_program.key(),
            TransferChecked {
                from: from.to_account_info(),
                mint: mint.to_account_info(),
                to: to.to_account_info(),
                authority: authority.to_account_info(),
            },
        ),
        amount,
        mint.decimals,
    )
}

pub fn handle_request_capture(ctx: Context<RequestCapture>) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        let pool = PoolView::load(&mut data, &vault_key)?;
        check_open(&ctx.accounts.vault, &pool)?;
    }
    let a = &ctx.accounts;
    let (ratio, fee) = (a.vault.ratio_base, a.vault.capture_fee_amount);
    pay(&a.token_program, &a.user_token, &a.vault_tokens, &a.user, &a.mint, ratio)?;
    pay(&a.token_program, &a.user_token, &a.fee_escrow, &a.user, &a.mint, fee)?;

    let rbump = a.vault.randomness_authority_bump;
    let rseeds: &[&[u8]] = &[RANDOMNESS_AUTHORITY_SEED, vault_key.as_ref(), &[rbump]];
    let seed_slot = randomness::commit_for_request(
        &a.randomness.to_account_info(),
        &a.sb_queue.to_account_info(),
        &a.sb_oracle.to_account_info(),
        &a.slot_hashes.to_account_info(),
        &a.randomness_authority.to_account_info(),
        &a.switchboard_program.to_account_info(),
        rseeds,
    )?;

    finish_request(
        &mut ctx.accounts.vault,
        &mut ctx.accounts.request,
        &mut ctx.accounts.rand_lock,
        ctx.bumps.request,
        ctx.bumps.rand_lock,
        REQUEST_KIND_CAPTURE,
        ctx.accounts.user.key(),
        ctx.accounts.user_token.key(),
        ctx.accounts.randomness.key(),
        seed_slot,
        fee,
        NO_HANDED_IN,
        vault_key,
    )?;

    ctx.accounts.vault_tokens.reload()?;
    ctx.accounts.fee_escrow.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, &pool, ctx.accounts.vault_tokens.amount, ctx.accounts.fee_escrow.amount)
}

pub fn handle_request_reroll(ctx: Context<RequestReroll>, index: u32) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        let pool = PoolView::load(&mut data, &vault_key)?;
        check_open(&ctx.accounts.vault, &pool)?;
    }
    require!(index < ctx.accounts.vault.collection_size, VaultError::IndexOutOfRange);
    let a = &ctx.accounts;
    let fee = a.vault.reroll_fee_amount;

    // Hand the NFT in (user signs as owner). Held by the vault, NOT drawable for this request.
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
    pay(&a.token_program, &a.user_token, &a.fee_escrow, &a.user, &a.mint, fee)?;

    let rbump = a.vault.randomness_authority_bump;
    let rseeds: &[&[u8]] = &[RANDOMNESS_AUTHORITY_SEED, vault_key.as_ref(), &[rbump]];
    let seed_slot = randomness::commit_for_request(
        &a.randomness.to_account_info(),
        &a.sb_queue.to_account_info(),
        &a.sb_oracle.to_account_info(),
        &a.slot_hashes.to_account_info(),
        &a.randomness_authority.to_account_info(),
        &a.switchboard_program.to_account_info(),
        rseeds,
    )?;

    finish_request(
        &mut ctx.accounts.vault,
        &mut ctx.accounts.request,
        &mut ctx.accounts.rand_lock,
        ctx.bumps.request,
        ctx.bumps.rand_lock,
        REQUEST_KIND_REROLL,
        ctx.accounts.user.key(),
        ctx.accounts.user_token.key(),
        ctx.accounts.randomness.key(),
        seed_slot,
        fee,
        index,
        vault_key,
    )?;
    let v = &mut ctx.accounts.vault;
    v.assets_outside = v.assets_outside.checked_sub(1).ok_or_else(|| error!(VaultError::AssetAccountingBroken))?;

    ctx.accounts.fee_escrow.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, &pool, ctx.accounts.vault_tokens.amount, ctx.accounts.fee_escrow.amount)
}

#[allow(clippy::too_many_arguments)]
fn finish_request(
    vault: &mut Vault,
    request: &mut Request,
    lock: &mut RandLock,
    request_bump: u8,
    lock_bump: u8,
    kind: u8,
    user: Pubkey,
    user_token: Pubkey,
    randomness: Pubkey,
    seed_slot: u64,
    fee: u64,
    handed_in_index: u32,
    vault_key: Pubkey,
) -> Result<()> {
    let seq = vault.next_seq;
    let slot = Clock::get()?.slot;
    request.bump = request_bump;
    request.kind = kind;
    request.vault = vault_key;
    request.seq = seq;
    request.user = user;
    request.user_token = user_token;
    request.randomness = randomness;
    request.seed_slot = seed_slot;
    request.deadline_slot = slot.checked_add(REQUEST_TIMEOUT_SLOTS).ok_or_else(|| error!(VaultError::MathOverflow))?;
    request.fee_amount = fee;
    request.handed_in_index = handed_in_index;

    lock.bump = lock_bump;
    lock.vault = request.vault;
    lock.seq = seq;

    vault.next_seq = seq.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    vault.pending_fee_total = vault.pending_fee_total.checked_add(fee).ok_or_else(|| error!(VaultError::MathOverflow))?;
    if kind == REQUEST_KIND_CAPTURE {
        vault.pending_captures = vault.pending_captures.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    } else {
        vault.pending_rerolls = vault.pending_rerolls.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    }
    Ok(())
}
