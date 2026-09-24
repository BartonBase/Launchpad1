//! On-chain invariants, asserted at the end of every instruction that moves tokens or NFTs.

use crate::{error::VaultError, pool::PoolView, state::Vault};
use anchor_lang::prelude::*;

/// INV (solvency): vault tokens >= ratio * (assets outside + pending captures).
/// Pending captures have already paid `ratio` into the vault; counting them keeps the check exact
/// for refunds on expire. Plus: fee escrow covers every pending fee, and every deposited NFT is
/// accounted for exactly once.
pub fn check(vault: &Vault, pool: &PoolView, vault_tokens_amount: u64, fee_escrow_amount: u64) -> Result<()> {
    let owed_units = vault
        .assets_outside
        .checked_add(vault.pending_captures)
        .ok_or_else(|| error!(VaultError::MathOverflow))?;
    let required = vault.ratio_base.checked_mul(owed_units).ok_or_else(|| error!(VaultError::MathOverflow))?;
    require!(vault_tokens_amount >= required, VaultError::InsolventVault);
    require!(fee_escrow_amount >= vault.pending_fee_total, VaultError::FeeEscrowShort);

    let in_vault = (pool.pool_len() as u64)
        .checked_add(pool.incoming_len() as u64)
        .and_then(|x| x.checked_add(vault.pending_rerolls)) // re-roll hand-ins held by the vault
        .and_then(|x| x.checked_add(vault.assets_outside))
        .ok_or_else(|| error!(VaultError::MathOverflow))?;
    require!(in_vault == vault.deposited_count as u64, VaultError::AssetAccountingBroken);
    Ok(())
}
