//! On-chain invariants, asserted at the END of every mutating instruction (Auditor A A-07).
//!
//! INV-1 (solvency): vault_tokens >= ratio * (assets_outside + pending_captures + pending_rerolls).
//!   Every NFT a user holds was paid for with exactly `ratio` tokens that sit in `vault_tokens`, and
//!   each pending capture has already paid its `ratio`. A pending re-roll's hand-in is back in the
//!   vault (assets_outside - 1) but its `ratio` stays escrowed for the replacement the user is owed. There is no burn, withdraw or fee path out
//!   of `vault_tokens`; unwrap pays exactly `ratio` and decrements `assets_outside`. `>=` rather than
//!   `==` only because anyone can donate tokens to the account; tests assert equality.
//! INV-2 (lazy-mint index accounting, ADR-016): pool + incoming + held re-roll hand-ins +
//!   assets_outside == N. Every index is exactly one of: drawable (minted or not), returning,
//!   held as a hand-in, or owned by a user.
//! INV-3: minted_count <= N, and every user-owned or held asset is minted
//!   (assets_outside + pending_rerolls <= minted_count). The bitmap makes each mint happen once.

use crate::{error::VaultError, pool::PoolView, state::Vault};
use anchor_lang::prelude::*;

pub fn required_vault_tokens(vault: &Vault, ratio_base: u64) -> Result<u64> {
    let owed_units = vault
        .assets_outside
        .checked_add(vault.pending_captures)
        .and_then(|x| x.checked_add(vault.pending_rerolls))
        .ok_or_else(|| error!(VaultError::MathOverflow))?;
    ratio_base.checked_mul(owed_units).ok_or_else(|| error!(VaultError::MathOverflow))
}

/// `ratio_base` / `collection_size` come from the LaunchConfig (config::econ or config::exit_view).
pub fn check(vault: &Vault, ratio_base: u64, collection_size: u32, pool: &PoolView, vault_tokens_amount: u64) -> Result<()> {
    require!(vault_tokens_amount >= required_vault_tokens(vault, ratio_base)?, VaultError::InsolventVault);

    let accounted = (pool.pool_len() as u64)
        .checked_add(pool.incoming_len() as u64)
        .and_then(|x| x.checked_add(vault.pending_rerolls))
        .and_then(|x| x.checked_add(vault.assets_outside))
        .ok_or_else(|| error!(VaultError::MathOverflow))?;
    require!(accounted == collection_size as u64, VaultError::AssetAccountingBroken);
    require!(pool.capacity() == collection_size, VaultError::AssetAccountingBroken);
    require!(vault.minted_count <= collection_size, VaultError::AssetAccountingBroken);
    let held_or_out = vault.assets_outside.checked_add(vault.pending_rerolls).ok_or_else(|| error!(VaultError::MathOverflow))?;
    require!(held_or_out <= vault.minted_count as u64, VaultError::AssetAccountingBroken);
    Ok(())
}
