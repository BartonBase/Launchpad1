//! `request_capture` / `request_reroll`: take the payment, create a sequenced request, and commit
//! Switchboard randomness by CPI in the SAME instruction (nobody can know the value first).
//! No asset argument exists for a capture: the caller can't choose what they get. There is no
//! cancel/refund: the request can only end in `settle_*` (ADR-012).
//!
//! Payment (ADR-013): exactly `ratio` tokens into vault_tokens (capture only) and the flat SOL fee
//! min(stored, current tier, MAX) (M-08; the same for capture and re-roll) user -> PLATFORM_FEE_RECIPIENT by system
//! transfer. There is NO token fee. The fee is never refunded (M-04). The pool-floor check runs
//! BEFORE anything is charged. All amounts come from the immutable LaunchConfig (M-01).

use super::vault_token_ops::{sol_fee, user_pays};
use crate::{
    asset_source, config::{self, Econ}, constants::*, error::VaultError, invariants, pool::PoolView, randomness,
    state::{RandLock, Request, Vault},
};
use hybrid_launch::LaunchConfig;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct RequestCapture<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,

    /// Immutable economics (M-01); re-validated in config::econ.
    #[account(address = vault.launch_config)]
    pub launch_config: Box<Account<'info, LaunchConfig>>,

    /// CHECK: validated by PoolView::load + address pinned.
    #[account(address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    #[account(address = launch_config.mint @ VaultError::MintMismatch)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
        constraint = user_token.key() != vault.vault_tokens @ VaultError::AliasedTokenAccount,
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    #[account(mut, seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    /// CHECK: pinned to today's PLATFORM_FEE_RECIPIENT constant; receives the SOL fee by system
    /// transfer (F-06 / M-05). Not the LaunchConfig copy, so a wallet rotation never strands old launches.
    #[account(mut, address = hybrid_launch::PLATFORM_FEE_RECIPIENT @ VaultError::FeeRecipientMismatch)]
    pub fee_recipient: UncheckedAccount<'info>,

    #[account(init, payer = user, space = 8 + Request::INIT_SPACE, seeds = [REQUEST_SEED, vault.key().as_ref(), &vault.next_seq.to_le_bytes()], bump)]
    pub request: Box<Account<'info, Request>>,

    /// CHECK: data-less system-owned PDA `["mint_escrow", vault, seq]`; receives MINT_ESCROW_LAMPORTS (ADR-016).
    #[account(mut, seeds = [MINT_ESCROW_SEED, vault.key().as_ref(), &vault.next_seq.to_le_bytes()], bump)]
    pub mint_escrow: UncheckedAccount<'info>,

    #[account(init, payer = user, space = 8 + RandLock::INIT_SPACE, seeds = [RAND_LOCK_SEED, randomness.key().as_ref()], bump)]
    pub rand_lock: Box<Account<'info, RandLock>>,

    /// CHECK: Switchboard randomness account; owner + authority validated in randomness::commit_for_request.
    #[account(mut)]
    pub randomness: UncheckedAccount<'info>,
    /// CHECK: PDA; the randomness account's authority.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: pinned to vault.sb_queue in commit_for_request.
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

    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,

    /// Immutable economics (M-01); re-validated in config::econ.
    #[account(address = vault.launch_config)]
    pub launch_config: Box<Account<'info, LaunchConfig>>,

    /// CHECK: validated by PoolView::load + address pinned.
    #[account(address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,

    /// Read-only: solvency is asserted after the hand-in (no tokens move on a re-roll).
    #[account(seeds = [VAULT_TOKENS_SEED, vault.key().as_ref()], bump = vault.vault_tokens_bump)]
    pub vault_tokens: Box<Account<'info, TokenAccount>>,

    /// CHECK: see RequestCapture.
    #[account(mut, address = hybrid_launch::PLATFORM_FEE_RECIPIENT @ VaultError::FeeRecipientMismatch)]
    pub fee_recipient: UncheckedAccount<'info>,

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

    #[account(init, payer = user, space = 8 + Request::INIT_SPACE, seeds = [REQUEST_SEED, vault.key().as_ref(), &vault.next_seq.to_le_bytes()], bump)]
    pub request: Box<Account<'info, Request>>,

    /// CHECK: data-less system-owned PDA `["mint_escrow", vault, seq]`; receives MINT_ESCROW_LAMPORTS (ADR-016).
    #[account(mut, seeds = [MINT_ESCROW_SEED, vault.key().as_ref(), &vault.next_seq.to_le_bytes()], bump)]
    pub mint_escrow: UncheckedAccount<'info>,

    #[account(init, payer = user, space = 8 + RandLock::INIT_SPACE, seeds = [RAND_LOCK_SEED, randomness.key().as_ref()], bump)]
    pub rand_lock: Box<Account<'info, RandLock>>,

    /// CHECK: see RequestCapture.
    #[account(mut)]
    pub randomness: UncheckedAccount<'info>,
    /// CHECK: PDA.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: pinned in commit_for_request.
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

    pub system_program: Program<'info, System>,
}

/// Pool floor against cornering (graduation-design / B-06): a request is refused BEFORE any fee
/// is charged unless the drawable assets left after every pending draw is served are at least
/// max(MIN_POOL_FLOOR, POOL_FLOOR_BPS of the collection).
pub fn pool_floor(collection_size: u32) -> u64 {
    let pct = (u128::from(collection_size) * u128::from(POOL_FLOOR_BPS)).div_ceil(10_000) as u64;
    pct.max(MIN_POOL_FLOOR)
}

fn check_can_request(vault: &Vault, econ: &Econ, pool: &PoolView) -> Result<()> {
    require!(vault.open, VaultError::VaultNotOpen);
    // Every pool/incoming entry now has tag <= this request's seq, so it will be a candidate at
    // settle unless an earlier pending draw takes it.
    let available = (pool.pool_len() as u64)
        .checked_add(pool.incoming_len() as u64)
        .ok_or_else(|| error!(VaultError::MathOverflow))?;
    let free = available.saturating_sub(vault.pending_draws()?);
    // After this draw at least pool_floor(N) drawable assets must remain (anti-cornering).
    require!(free > pool_floor(econ.collection_size), VaultError::NoAssetAvailable);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn finish_request(
    vault: &mut Vault,
    request: &mut Request,
    rand_lock: &mut RandLock,
    request_bump: u8,
    lock_bump: u8,
    kind: u8,
    user: Pubkey,
    randomness: Pubkey,
    seed_slot: u64,
    handed_in_index: u32,
    vault_key: Pubkey,
    oracle: Pubkey,
    sol_fee_lamports: u64,
) -> Result<()> {
    let seq = vault.next_seq;
    let slot = Clock::get()?.slot;
    request.bump = request_bump;
    request.kind = kind;
    request.vault = vault_key;
    request.seq = seq;
    request.user = user;
    request.randomness = randomness;
    request.seed_slot = seed_slot;
    request.deadline_slot = slot.checked_add(REVEAL_TIMEOUT_SLOTS).ok_or_else(|| error!(VaultError::MathOverflow))?;
    request.commits = 1;
    request.oracles = [oracle, Pubkey::default(), Pubkey::default(), Pubkey::default()];
    request.revealed = false;
    request.value = [0u8; 32];
    request.handed_in_index = handed_in_index;
    rand_lock.bump = lock_bump;
    rand_lock.vault = vault_key;
    rand_lock.seq = seq;

    vault.next_seq = seq.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    vault.total_fee_lamports = vault.total_fee_lamports.checked_add(sol_fee_lamports).ok_or_else(|| error!(VaultError::MathOverflow))?;
    if kind == REQUEST_KIND_CAPTURE {
        vault.pending_captures = vault.pending_captures.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    } else {
        vault.pending_rerolls = vault.pending_rerolls.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
        vault.assets_outside = vault.assets_outside.checked_sub(1).ok_or_else(|| error!(VaultError::AssetAccountingBroken))?;
    }
    Ok(())
}

/// Lazy-mint escrow (ADR-016): the worst-case cost of minting whatever this request is assigned is
/// deposited in the request PDA up front, so settle never needs anyone else's lamports (T-HV-16) and
/// an underfunded request can't exist. Fails closed if live rent + the Core fee outgrew the constant.
fn escrow_mint_cost<'info>(system: &Program<'info, System>, user: &Signer<'info>, escrow: &AccountInfo<'info>) -> Result<()> {
    require!(escrow.data_is_empty() && *escrow.owner == anchor_lang::system_program::ID, VaultError::AssetStateMismatch);
    let live = Rent::get()?.minimum_balance(hybrid_launch::CORE_ASSET_SPACE_BYTES).saturating_add(hybrid_launch::CORE_CREATE_FEE_LAMPORTS);
    require!(live <= MINT_ESCROW_LAMPORTS, VaultError::MintCostConstantStale);
    anchor_lang::system_program::transfer(
        CpiContext::new(system.key(), anchor_lang::system_program::Transfer { from: user.to_account_info(), to: escrow.clone() }),
        MINT_ESCROW_LAMPORTS,
    )
}

pub fn handle_request_capture(ctx: Context<RequestCapture>) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let econ = config::econ(&ctx.accounts.launch_config)?;
    {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        let pool = PoolView::load(&mut data, &vault_key)?;
        check_can_request(&ctx.accounts.vault, &econ, &pool)?;
    }
    let a = &ctx.accounts;
    let (ratio, sol) = (econ.ratio_base, econ.request_fee_lamports);
    user_pays(&a.token_program, &a.user_token, &a.vault_tokens.to_account_info(), &a.user, &a.mint, ratio)?;
    sol_fee(&a.system_program, &a.user, &a.fee_recipient.to_account_info(), sol)?;
    escrow_mint_cost(&a.system_program, &a.user, &a.mint_escrow.to_account_info())?;

    let rseeds: &[&[u8]] = &[RANDOMNESS_AUTHORITY_SEED, vault_key.as_ref(), &[a.vault.randomness_authority_bump]];
    let seed_slot = randomness::commit_for_request(
        &a.randomness.to_account_info(),
        &a.sb_queue.to_account_info(),
        &a.sb_oracle.to_account_info(),
        &a.slot_hashes.to_account_info(),
        &a.randomness_authority.to_account_info(),
        &a.switchboard_program.to_account_info(),
        &a.vault.sb_queue,
        rseeds,
        &vault_key,
        a.vault.next_seq,
        &[],
        ctx.remaining_accounts,
    )?;

    let (user, rnd, oracle) = (a.user.key(), a.randomness.key(), a.sb_oracle.key());
    finish_request(
        &mut ctx.accounts.vault, &mut ctx.accounts.request, &mut ctx.accounts.rand_lock,
        ctx.bumps.request, ctx.bumps.rand_lock, REQUEST_KIND_CAPTURE, user, rnd, seed_slot, NO_HANDED_IN, vault_key, oracle, sol,
    )?;
    ctx.accounts.request.mint_escrow_lamports = MINT_ESCROW_LAMPORTS;

    ctx.accounts.vault_tokens.reload()?;
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, econ.ratio_base, econ.collection_size, &pool, ctx.accounts.vault_tokens.amount)
}

pub fn handle_request_reroll(ctx: Context<RequestReroll>, index: u32) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let econ = config::econ(&ctx.accounts.launch_config)?;
    {
        let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
        let pool = PoolView::load(&mut data, &vault_key)?;
        check_can_request(&ctx.accounts.vault, &econ, &pool)?;
    }
    require!(index < econ.collection_size, VaultError::IndexOutOfRange);
    let a = &ctx.accounts;
    let sol = econ.request_fee_lamports;

    // Hand the NFT in (user signs as owner). Held by the vault, NOT drawable for this request.
    asset_source::take_back(
        &a.mpl_core_program.to_account_info(),
        &a.asset.to_account_info(),
        &a.collection.to_account_info(),
        &a.user.to_account_info(),
        &a.vault_authority.to_account_info(),
        &a.system_program.to_account_info(),
    )?;
    sol_fee(&a.system_program, &a.user, &a.fee_recipient.to_account_info(), sol)?;
    escrow_mint_cost(&a.system_program, &a.user, &a.mint_escrow.to_account_info())?;

    let rseeds: &[&[u8]] = &[RANDOMNESS_AUTHORITY_SEED, vault_key.as_ref(), &[a.vault.randomness_authority_bump]];
    let seed_slot = randomness::commit_for_request(
        &a.randomness.to_account_info(),
        &a.sb_queue.to_account_info(),
        &a.sb_oracle.to_account_info(),
        &a.slot_hashes.to_account_info(),
        &a.randomness_authority.to_account_info(),
        &a.switchboard_program.to_account_info(),
        &a.vault.sb_queue,
        rseeds,
        &vault_key,
        a.vault.next_seq,
        &[],
        ctx.remaining_accounts,
    )?;

    let (user, rnd, oracle) = (a.user.key(), a.randomness.key(), a.sb_oracle.key());
    finish_request(
        &mut ctx.accounts.vault, &mut ctx.accounts.request, &mut ctx.accounts.rand_lock,
        ctx.bumps.request, ctx.bumps.rand_lock, REQUEST_KIND_REROLL, user, rnd, seed_slot, index, vault_key, oracle, sol,
    )?;
    ctx.accounts.request.mint_escrow_lamports = MINT_ESCROW_LAMPORTS;

    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let pool = PoolView::load(&mut data, &vault_key)?;
    invariants::check(&ctx.accounts.vault, econ.ratio_base, econ.collection_size, &pool, ctx.accounts.vault_tokens.amount)
}
