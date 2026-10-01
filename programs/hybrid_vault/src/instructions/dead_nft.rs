//! Fix for the review's finding 1 (2026-10-01): one burned NFT must not stall Mode 4/5 payouts.
//!
//! Payouts and raffle snapshots walk NFTs strictly in index order (`payout_cursor`). Core lets an owner
//! burn their NFT at any time, and before this fix the cursor could never pass a burned index.
//! `skip_dead_nft` is permissionless: it advances the cursor past the NFT at `payout_cursor` ONLY if
//! that asset (address derived here, not chosen by the caller) is dead (`core_cpi::asset_is_dead`).
//! - Mode 4: the burned NFT's share is forfeited back to `pending_base` (credited_base shrinks by the
//!   same amount), so it is redistributed by a later round. Nothing is paid, so nothing is double-paid.
//! - Mode 5: no seat is written for it. Seats are dense (0..live_seats), so the draw only lands on a
//!   live seat. If every NFT in the round is dead, the round is rolled back: the pot returns to
//!   `pending_base` and the vault goes idle.

use crate::{constants::*, core_cpi, error::VaultError, state::TaxVault};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SkipDeadNft<'info> {
    #[account(mut, seeds = [VAULT_SEED, vault.launch_config.as_ref()], bump = vault.bump)]
    pub vault: Box<Account<'info, TaxVault>>,

    /// CHECK: must be the asset PDA at `payout_cursor` (derived below) and must be dead.
    pub asset: UncheckedAccount<'info>,
}

pub fn handle_skip_dead_nft(ctx: Context<SkipDeadNft>) -> Result<()> {
    let v = &ctx.accounts.vault;
    let cursor = v.payout_cursor;
    match v.kind {
        KIND_SPLIT => require!(v.round_open == 1, VaultError::OutOfOrder),
        KIND_RAFFLE => require!(v.raffle_phase == RAFFLE_SNAPSHOT, VaultError::OutOfOrder),
        _ => return err!(VaultError::WrongRequestKind),
    }
    require!(cursor < v.round_minted, VaultError::OutOfOrder);
    let (expected, _) = Pubkey::find_program_address(
        &[ASSET_SEED, v.key().as_ref(), &cursor.to_le_bytes()],
        &crate::ID,
    );
    require_keys_eq!(ctx.accounts.asset.key(), expected, VaultError::OutOfOrder);
    require!(
        core_cpi::asset_is_dead(&ctx.accounts.asset.to_account_info()),
        VaultError::AssetNotDead
    );

    let v = &mut ctx.accounts.vault;
    v.payout_cursor = cursor.checked_add(1).ok_or(VaultError::MathOverflow)?;
    let done = v.payout_cursor == v.round_minted;
    if v.kind == KIND_SPLIT {
        let share = v.round_share;
        v.credited_base = v.credited_base.checked_sub(share).ok_or(VaultError::MathOverflow)?;
        require!(v.credited_base >= v.paid_base, VaultError::DistributionExceeded);
        v.pending_base = v.pending_base.checked_add(share).ok_or(VaultError::MathOverflow)?;
        if done {
            v.round_open = 0;
            v.round_share = 0;
        }
        return Ok(());
    }
    if done {
        finish_snapshot(v)?;
    }
    Ok(())
}

/// End of a raffle snapshot: READY if any live seat exists, else roll the round back (pot kept).
pub fn finish_snapshot(v: &mut TaxVault) -> Result<()> {
    if v.live_seats > 0 {
        v.raffle_phase = RAFFLE_READY;
        Ok(())
    } else {
        rollback_raffle_round(v)
    }
}

/// Close the open raffle round without paying it: the whole pot goes back to `pending_base`
/// (credited_base shrinks by the same amount), so a later round redistributes it. Nothing is lost.
pub fn rollback_raffle_round(v: &mut TaxVault) -> Result<()> {
    let pot = v.round_pot;
    v.credited_base = v.credited_base.checked_sub(pot).ok_or(VaultError::MathOverflow)?;
    require!(v.credited_base >= v.paid_base, VaultError::DistributionExceeded);
    v.pending_base = v.pending_base.checked_add(pot).ok_or(VaultError::MathOverflow)?;
    v.round_pot = 0;
    v.round_open = 0;
    v.raffle_phase = 0;
    v.raffle_commits = 0;
    v.raffle_oracles = [Pubkey::default(); 4];
    v.raffle_deadline_slot = 0;
    Ok(())
}
