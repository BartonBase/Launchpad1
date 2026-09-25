//! Graduation check behind a small interface. `open_vault` calls `verify` and nothing else, so the
//! bonding-curve integration (Meteora DBC vs Raydium LaunchLab, not chosen yet; see
//! docs/graduation-design.md) plugs in here without touching the vault logic.
//!
//! Builds:
//! - DEFAULT / RELEASE: no verifier is compiled in, `verify` always fails
//!   (`GraduationCheckUnavailable`). Fail-closed: a vault can never open until the real curve check
//!   exists.
//! - `test-mock-graduation` (TEST-ONLY, localnet LiteSVM): accepts an account owned by
//!   `MOCK_GRADUATION_OWNER` with layout `b"MOCKGRAD" | mint (32) | graduated (1)`. The feature can't be
//!   combined with `mainnet` (compile_error), `scripts/build-test-sbf.sh` builds it into
//!   `target/test-sbf/` only, and `scripts/deploy-devnet.sh` refuses any binary that contains the
//!   marker string.

use anchor_lang::prelude::*;

#[cfg(all(feature = "test-mock-graduation", feature = "mainnet"))]
compile_error!("test-mock-graduation is TEST-ONLY and can never be part of a mainnet build");

#[cfg(feature = "test-mock-graduation")]
pub const MOCK_GRADUATION_OWNER: Pubkey = pubkey!("grADWKnwMo64gj6DYGQEwuiEv9g5KkofdUT64WtWP2W");
#[cfg(feature = "test-mock-graduation")]
pub const MOCK_GRADUATION_DISCRIMINATOR: [u8; 8] = *b"MOCKGRAD";

/// Ok(()) iff `proof` proves the launch for `mint` has graduated.
#[cfg(feature = "test-mock-graduation")]
pub fn verify(mint: &Pubkey, proof: &AccountInfo) -> Result<()> {
    use crate::error::VaultError;
    msg!("hybrid_vault: TEST-ONLY mock graduation verifier");
    require_keys_eq!(*proof.owner, MOCK_GRADUATION_OWNER, VaultError::GraduationNotVerified);
    let d = proof.try_borrow_data()?;
    require!(d.len() >= 41, VaultError::GraduationNotVerified);
    require!(d[..8] == MOCK_GRADUATION_DISCRIMINATOR, VaultError::GraduationNotVerified);
    require!(d[8..40] == mint.to_bytes(), VaultError::GraduationNotVerified);
    require!(d[40] == 1, VaultError::GraduationNotVerified);
    Ok(())
}

#[cfg(not(feature = "test-mock-graduation"))]
pub fn verify(_mint: &Pubkey, _proof: &AccountInfo) -> Result<()> {
    err!(crate::error::VaultError::GraduationCheckUnavailable)
}
