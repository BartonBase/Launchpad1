//! The permissionless incoming-merge crank. (The guardian pause was REMOVED, Barton 2026-09-25
//! 4:57 PM MT, ADR-015: no key can halt capture, re-roll, settle, expire or release.)

use crate::{constants::*, pool::PoolView, state::Vault};
use anchor_lang::prelude::*;

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
