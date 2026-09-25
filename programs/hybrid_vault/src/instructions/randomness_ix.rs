//! Randomness lifecycle instructions (ADR-012). All three are PERMISSIONLESS; nothing can halt them
//! (there is no pause of any kind, ADR-015):
//! - `init_randomness`: create a Switchboard randomness account whose authority is this vault's
//!   `randomness_authority` PDA (so only hybrid_vault can commit/reveal it).
//! - `reveal_randomness`: submit the oracle's signed reveal (fetched by anyone from the Switchboard
//!   gateway) for a pending request. The user holds no authority over the randomness and can't
//!   withhold or discard it.
//!   The value is RECORDED in the request in the reveal slot (M-04 H1, M-35); settle reads it.
//! - `recommit_randomness`: if a request's randomness is STILL UNREVEALED more than
//!   REVEAL_TIMEOUT_SLOTS after its commit, anyone re-commits fresh randomness for the SAME request,
//!   with an oracle not used before for it (H2), at most MAX_RECOMMITS times (H4). Refused once
//!   revealed (so nobody, including the user, can throw away a revealed draw) and before the
//!   deadline. No fee is refunded; see `expire_request` for the principal-only last resort.

use crate::{
    constants::*, error::VaultError, randomness::{self, RevealArgs},
    state::{RandLock, Request, Vault},
};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitRandomness<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK: new keypair account, created by Switchboard.
    #[account(mut)]
    pub randomness: Signer<'info>,
    /// CHECK: PDA; set as the randomness authority.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_reward_escrow: UncheckedAccount<'info>,
    /// CHECK: pinned to the vault's queue.
    #[account(mut, address = vault.sb_queue @ VaultError::WrongQueue)]
    pub sb_queue: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: address pinned.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = WRAPPED_SOL_MINT)]
    pub wrapped_sol_mint: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    pub sb_program_state: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    pub sb_lut_signer: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_lut: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = ADDRESS_LOOKUP_TABLE_PROGRAM_ID)]
    pub address_lookup_table_program: UncheckedAccount<'info>,
    /// CHECK: address pinned (+ executable check in randomness.rs).
    #[account(address = SWITCHBOARD_PROGRAM_ID)]
    pub switchboard_program: UncheckedAccount<'info>,
}

pub fn handle_init_randomness(ctx: Context<InitRandomness>, recent_slot: u64) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let seeds: &[&[u8]] = &[RANDOMNESS_AUTHORITY_SEED, vault_key.as_ref(), &[ctx.accounts.vault.randomness_authority_bump]];
    let a = &ctx.accounts;
    let accounts = [
        a.randomness.to_account_info(),
        a.sb_reward_escrow.to_account_info(),
        a.randomness_authority.to_account_info(),
        a.sb_queue.to_account_info(),
        a.payer.to_account_info(),
        a.system_program.to_account_info(),
        a.token_program.to_account_info(),
        a.associated_token_program.to_account_info(),
        a.wrapped_sol_mint.to_account_info(),
        a.sb_program_state.to_account_info(),
        a.sb_lut_signer.to_account_info(),
        a.sb_lut.to_account_info(),
        a.address_lookup_table_program.to_account_info(),
    ];
    randomness::init_account(&accounts, recent_slot, &a.switchboard_program.to_account_info(), seeds)
}

#[derive(Accounts)]
pub struct RevealRandomness<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,
    #[account(mut, seeds = [REQUEST_SEED, vault.key().as_ref(), &request.seq.to_le_bytes()], bump = request.bump, has_one = vault, has_one = randomness)]
    pub request: Box<Account<'info, Request>>,
    #[account(seeds = [RAND_LOCK_SEED, randomness.key().as_ref()], bump = rand_lock.bump, constraint = rand_lock.vault == vault.key() @ VaultError::RandomnessMismatch)]
    pub rand_lock: Box<Account<'info, RandLock>>,
    /// CHECK: == request.randomness; Switchboard validates the reveal.
    #[account(mut)]
    pub randomness: UncheckedAccount<'info>,
    /// CHECK: PDA.
    #[account(seeds = [RANDOMNESS_AUTHORITY_SEED, vault.key().as_ref()], bump = vault.randomness_authority_bump)]
    pub randomness_authority: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard (must be the oracle assigned at commit).
    pub sb_oracle: UncheckedAccount<'info>,
    /// CHECK: pinned.
    #[account(address = vault.sb_queue @ VaultError::WrongQueue)]
    pub sb_queue: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_stats: UncheckedAccount<'info>,
    /// CHECK: pinned; forwarded only.
    #[account(address = SLOT_HASHES_SYSVAR_ID)]
    pub slot_hashes: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: validated by Switchboard.
    #[account(mut)]
    pub sb_reward_escrow: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = WRAPPED_SOL_MINT)]
    pub wrapped_sol_mint: UncheckedAccount<'info>,
    /// CHECK: validated by Switchboard.
    pub sb_program_state: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = SWITCHBOARD_PROGRAM_ID)]
    pub switchboard_program: UncheckedAccount<'info>,
}

pub fn handle_reveal_randomness(ctx: Context<RevealRandomness>, args: RevealArgs) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let seed_slot = ctx.accounts.request.seed_slot;
    require!(!ctx.accounts.request.revealed, VaultError::RandomnessAlreadyRevealed);
    {
        let snap = randomness::read(&ctx.accounts.randomness)?;
        require!(snap.seed_slot == seed_slot, VaultError::RandomnessMismatch);
        require!(!randomness::is_revealed(&snap, seed_slot), VaultError::RandomnessAlreadyRevealed);
    }
    let seeds: &[&[u8]] = &[RANDOMNESS_AUTHORITY_SEED, vault_key.as_ref(), &[ctx.accounts.vault.randomness_authority_bump]];
    let a = &ctx.accounts;
    let accounts = [
        a.randomness.to_account_info(),
        a.sb_oracle.to_account_info(),
        a.sb_queue.to_account_info(),
        a.sb_stats.to_account_info(),
        a.randomness_authority.to_account_info(),
        a.payer.to_account_info(),
        a.slot_hashes.to_account_info(),
        a.system_program.to_account_info(),
        a.sb_reward_escrow.to_account_info(),
        a.token_program.to_account_info(),
        a.wrapped_sol_mint.to_account_info(),
        a.sb_program_state.to_account_info(),
    ];
    randomness::reveal(&accounts, &args, &a.switchboard_program.to_account_info(), seeds)?;
    let snap = randomness::read(&a.randomness)?;
    require!(randomness::is_revealed(&snap, seed_slot), VaultError::RandomnessNotRevealed);
    let r = &mut ctx.accounts.request;
    r.revealed = true;
    r.value = snap.value;
    Ok(())
}

#[derive(Accounts)]
pub struct RecommitRandomness<'info> {
    pub caller: Signer<'info>,
    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,
    #[account(mut, seeds = [REQUEST_SEED, vault.key().as_ref(), &request.seq.to_le_bytes()], bump = request.bump, has_one = vault, has_one = randomness)]
    pub request: Box<Account<'info, Request>>,
    /// CHECK: == request.randomness; validated in randomness::commit_for_request.
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
    /// CHECK: pinned; forwarded only.
    #[account(address = SLOT_HASHES_SYSVAR_ID)]
    pub slot_hashes: UncheckedAccount<'info>,
    /// CHECK: address pinned.
    #[account(address = SWITCHBOARD_PROGRAM_ID)]
    pub switchboard_program: UncheckedAccount<'info>,
}

pub fn handle_recommit_randomness(ctx: Context<RecommitRandomness>) -> Result<()> {
    let slot = Clock::get()?.slot;
    let req = &ctx.accounts.request;
    require!(!req.revealed, VaultError::RandomnessAlreadyRevealed);
    require!(slot > req.deadline_slot, VaultError::DeadlineNotReached);
    require!(req.commits <= MAX_RECOMMITS, VaultError::RecommitsExhausted);
    let oracle = ctx.accounts.sb_oracle.key();
    require!(!req.oracles[..req.commits as usize].contains(&oracle), VaultError::OracleReused);
    {
        let snap = randomness::read(&ctx.accounts.randomness)?;
        require!(snap.seed_slot == req.seed_slot, VaultError::RandomnessMismatch);
        require!(!randomness::is_revealed(&snap, req.seed_slot), VaultError::RandomnessAlreadyRevealed);
    }
    let vault_key = ctx.accounts.vault.key();
    let seeds: &[&[u8]] = &[RANDOMNESS_AUTHORITY_SEED, vault_key.as_ref(), &[ctx.accounts.vault.randomness_authority_bump]];
    let a = &ctx.accounts;
    let used: Vec<Pubkey> = a.request.oracles[..a.request.commits as usize].to_vec();
    let seed_slot = randomness::commit_for_request(
        &a.randomness.to_account_info(),
        &a.sb_queue.to_account_info(),
        &a.sb_oracle.to_account_info(),
        &a.slot_hashes.to_account_info(),
        &a.randomness_authority.to_account_info(),
        &a.switchboard_program.to_account_info(),
        &a.vault.sb_queue,
        seeds,
        &vault_key,
        a.request.seq,
        &used,
        ctx.remaining_accounts,
    )?;
    let r = &mut ctx.accounts.request;
    r.seed_slot = seed_slot;
    r.deadline_slot = slot.checked_add(REVEAL_TIMEOUT_SLOTS).ok_or_else(|| error!(VaultError::MathOverflow))?;
    let k = r.commits as usize;
    r.oracles[k] = oracle;
    r.commits = r.commits.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    let v = &mut ctx.accounts.vault;
    v.total_recommits = v.total_recommits.checked_add(1).ok_or_else(|| error!(VaultError::MathOverflow))?;
    Ok(())
}
