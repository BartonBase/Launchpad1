//! Guardian pause (optional; guardian = Pubkey::default() disables it) and the permissionless
//! incoming-merge crank. The pause blocks ONLY request_capture / request_reroll, auto-expires after
//! at most MAX_PAUSE_SLOTS, has a cooldown, and never blocks unwrap, settle or expire.

use crate::{constants::*, error::VaultError, pool::PoolView, state::Vault};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct GuardianAction<'info> {
    pub guardian: Signer<'info>,
    #[account(
        mut,
        seeds = [VAULT_SEED, vault.launch_config.as_ref()],
        bump = vault.bump,
        constraint = vault.guardian != Pubkey::default() && vault.guardian == guardian.key() @ VaultError::NotGuardian,
    )]
    pub vault: Box<Account<'info, Vault>>,
}

pub fn handle_pause(ctx: Context<GuardianAction>, duration_slots: u64) -> Result<()> {
    require!(duration_slots > 0 && duration_slots <= MAX_PAUSE_SLOTS, VaultError::InvalidPauseDuration);
    let slot = Clock::get()?.slot;
    let v = &mut ctx.accounts.vault;
    require!(slot >= v.paused_until_slot && slot >= v.pause_cooldown_until_slot, VaultError::PauseNotAllowed);
    v.paused_until_slot = slot.checked_add(duration_slots).ok_or_else(|| error!(VaultError::MathOverflow))?;
    v.pause_cooldown_until_slot =
        v.paused_until_slot.checked_add(PAUSE_COOLDOWN_SLOTS).ok_or_else(|| error!(VaultError::MathOverflow))?;
    Ok(())
}

pub fn handle_unpause(ctx: Context<GuardianAction>) -> Result<()> {
    let slot = Clock::get()?.slot;
    let v = &mut ctx.accounts.vault;
    if v.paused_until_slot > slot {
        v.paused_until_slot = slot;
        v.pause_cooldown_until_slot = slot.checked_add(PAUSE_COOLDOWN_SLOTS).ok_or_else(|| error!(VaultError::MathOverflow))?;
    }
    Ok(())
}

#[derive(Accounts)]
pub struct MergeIncoming<'info> {
    #[account(seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, Vault>>,
    /// CHECK: validated by PoolView::load + address pinned.
    #[account(mut, address = vault.pool, owner = crate::ID)]
    pub pool: UncheckedAccount<'info>,
}

/// Merge incoming entries with tag <= next_settle_seq (eligible for every remaining request).
pub fn handle_merge_incoming(ctx: Context<MergeIncoming>, max: u32) -> Result<()> {
    let vault_key = ctx.accounts.vault.key();
    let mut data = ctx.accounts.pool.try_borrow_mut_data()?;
    let mut pool = PoolView::load(&mut data, &vault_key)?;
    pool.merge(ctx.accounts.vault.next_settle_seq, max.min(1024))?;
    Ok(())
}
