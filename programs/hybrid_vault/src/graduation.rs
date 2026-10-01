//! Graduation check (ADR-014): Meteora DBC. `open_vault` calls `verify` and nothing else.
//!
//! REAL verifier (every build): the proof must be the DBC pool recorded in the LaunchConfig by
//! `hybrid_launch::register_dbc_launch`. Owner, discriminator and length are checked by
//! `hybrid_launch::dbc::load_pool`, and the pool must name this mint and config and report it has
//! migrated (`is_migrated == 1` and `migration_progress == CreatedPool`; DBC sets both in the
//! instruction that creates the DAMM pool). A native `launch` records no DBC pool, so its vault can
//! never open.
//!
//! `test-mock-graduation` (TEST-ONLY, localnet LiteSVM) ADDITIONALLY accepts an account owned by
//! `MOCK_GRADUATION_OWNER` with layout `b"MOCKGRAD" | mint (32) | graduated (1)`. The feature can't be
//! combined with `mainnet` (compile_error). `scripts/build-test-sbf.sh` builds it into
//! `target/test-sbf/` only, and `scripts/deploy-devnet.sh` refuses any binary that contains the marker.

use crate::error::VaultError;
use anchor_lang::prelude::*;
use hybrid_launch::{dbc, LaunchConfig};

#[cfg(all(feature = "test-mock-graduation", feature = "mainnet"))]
compile_error!("test-mock-graduation is TEST-ONLY and can never be part of a mainnet build");

#[cfg(feature = "test-mock-graduation")]
pub const MOCK_GRADUATION_OWNER: Pubkey = pubkey!("grADWKnwMo64gj6DYGQEwuiEv9g5KkofdUT64WtWP2W");
#[cfg(feature = "test-mock-graduation")]
pub const MOCK_GRADUATION_DISCRIMINATOR: [u8; 8] = *b"MOCKGRAD";

/// Ok(()) iff `proof` proves the launch described by `cfg` has graduated.
pub fn verify(cfg: &LaunchConfig, proof: &AccountInfo) -> Result<()> {
    verify_recorded(&cfg.mint, &cfg.dbc_config, &cfg.dbc_pool, proof)
}

/// Same check as [`verify`], for a mode whose config is not `LaunchConfig` (Mode 3).
pub fn verify_recorded(
    mint: &Pubkey,
    dbc_config: &Pubkey,
    dbc_pool: &Pubkey,
    proof: &AccountInfo,
) -> Result<()> {
    #[cfg(feature = "test-mock-graduation")]
    {
        if *proof.owner == MOCK_GRADUATION_OWNER {
            return verify_mock(mint, proof);
        }
        // Test builds DO have a verifier for native launches (the mock), so a bad proof is "not verified".
        if *dbc_pool == Pubkey::default() {
            return err!(VaultError::GraduationNotVerified);
        }
    }
    verify_dbc_keys(mint, dbc_config, dbc_pool, proof)
}

fn verify_dbc_keys(
    mint: &Pubkey,
    dbc_config: &Pubkey,
    dbc_pool: &Pubkey,
    proof: &AccountInfo,
) -> Result<()> {
    // No DBC pool recorded (native launch): production has no graduation check for this launch at all.
    require!(
        *dbc_pool != Pubkey::default(),
        VaultError::GraduationCheckUnavailable
    );
    require_keys_eq!(proof.key(), *dbc_pool, VaultError::GraduationNotVerified);
    let pool = dbc::load_pool(proof).ok_or(error!(VaultError::GraduationNotVerified))?;
    require_keys_eq!(pool.base_mint, *mint, VaultError::GraduationNotVerified);
    require_keys_eq!(pool.config, *dbc_config, VaultError::GraduationNotVerified);
    require!(pool.graduated(), VaultError::GraduationNotVerified);
    Ok(())
}

#[cfg(feature = "test-mock-graduation")]
fn verify_mock(mint: &Pubkey, proof: &AccountInfo) -> Result<()> {
    msg!("hybrid_vault: TEST-ONLY mock graduation verifier");
    let d = proof.try_borrow_data()?;
    require!(d.len() >= 41, VaultError::GraduationNotVerified);
    require!(
        d[..8] == MOCK_GRADUATION_DISCRIMINATOR,
        VaultError::GraduationNotVerified
    );
    require!(
        d[8..40] == mint.to_bytes(),
        VaultError::GraduationNotVerified
    );
    require!(d[40] == 1, VaultError::GraduationNotVerified);
    Ok(())
}
