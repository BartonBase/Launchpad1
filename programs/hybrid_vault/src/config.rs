//! How the vault reads economics from the hybrid_launch LaunchConfig (audit M-01: no local copies).
//!
//! Two readers, on purpose (audit M-41):
//! - `econ` (init, mint, open, capture, re-roll): typed load, fail-closed on version and structure,
//!   M-22 cap (collection_size <= MAX_COLLECTION_SIZE). Capture and re-roll charge `request_fee`, which
//!   REQUIRES the stored fee to be exactly the tier for the ratio (QA-FEE-03, Barton 2026-09-26;
//!   supersedes the M-08 min(stored, tier, MAX) rule): 0 or any off-tier value reverts with FeeNotTier.
//!   Fees always go to today's PLATFORM_FEE_RECIPIENT constant (a rotated wallet doesn't strand old launches).
//! - `exit_view` (release, settle, expire): the user-exit paths. Reads ONLY ratio_base and
//!   collection_size as raw bytes at FROZEN offsets (hybrid_launch::stable_layout) after owner +
//!   discriminator checks. No version, fee, wallet or bounds check, so no launch-program upgrade
//!   (new tiers, rotated wallet, version bump, new MIN/MAX, appended fields) can freeze them.
//!   Release is FREE (Barton 5:04 PM MT), so exit paths never touch the fee.

use crate::error::VaultError;
use anchor_lang::{prelude::*, Discriminator};
use hybrid_launch::{
    is_exact_tier_fee, LaunchConfig, LC_DISCRIMINATOR_LEN, LC_OFF_COLLECTION_SIZE, LC_OFF_RATIO_BASE, LC_STABLE_PREFIX_LEN,
    MAX_COLLECTION_SIZE, PLATFORM_FEE_RECIPIENT,
};

pub const SUPPORTED_LAUNCH_CONFIG_VERSION: u8 = 4;

pub struct Econ {
    pub ratio_base: u64,
    pub collection_size: u32,
    /// Stored fee and ratio; charged on capture and re-roll only via `request_fee` (exact tier).
    pub stored_fee_lamports: u64,
    pub ratio_whole_tokens: u64,
    pub fee_recipient: Pubkey,
}

/// QA-FEE-03: the fee charged on capture and on re-roll (the same value, so re-roll <= capture +
/// release(0)). The stored fee must be EXACTLY the tier (hybrid_launch::tier_fee_lamports, the one
/// shared derivation); 0 or anything else reverts.
pub fn request_fee(stored: u64, ratio_whole_tokens: u64) -> Result<u64> {
    require!(is_exact_tier_fee(stored, ratio_whole_tokens), VaultError::FeeNotTier);
    Ok(stored)
}

impl Econ {
    pub fn request_fee(&self) -> Result<u64> {
        request_fee(self.stored_fee_lamports, self.ratio_whole_tokens)
    }
}

pub fn econ(cfg: &LaunchConfig) -> Result<Econ> {
    require!(cfg.version == SUPPORTED_LAUNCH_CONFIG_VERSION, VaultError::UnsupportedLaunchConfig);
    require!(cfg.collection_size <= MAX_COLLECTION_SIZE as u64, VaultError::CollectionAboveCap);
    let collection_size: u32 = cfg.collection_size.try_into().map_err(|_| error!(VaultError::UnsupportedLaunchConfig))?;
    require!(collection_size > 0 && cfg.ratio_base > 0, VaultError::UnsupportedLaunchConfig);
    Ok(Econ {
        ratio_base: cfg.ratio_base,
        collection_size,
        stored_fee_lamports: cfg.fee_lamports,
        ratio_whole_tokens: cfg.ratio_whole_tokens,
        fee_recipient: PLATFORM_FEE_RECIPIENT,
    })
}

pub struct ExitView {
    pub ratio_base: u64,
    pub collection_size: u32,
}

/// Minimal, upgrade-proof read for user-exit paths (M-41). `acc` must already be address-pinned to
/// `vault.launch_config`.
pub fn exit_view(acc: &AccountInfo) -> Result<ExitView> {
    require_keys_eq!(*acc.owner, hybrid_launch::ID, VaultError::UnsupportedLaunchConfig);
    let data = acc.try_borrow_data()?;
    exit_view_in(&data)
}

pub fn exit_view_in(data: &[u8]) -> Result<ExitView> {
    require!(
        data.len() >= LC_STABLE_PREFIX_LEN && &data[..LC_DISCRIMINATOR_LEN] == LaunchConfig::DISCRIMINATOR,
        VaultError::UnsupportedLaunchConfig
    );
    let u64_at = |o: usize| u64::from_le_bytes(data[o..o + 8].try_into().unwrap());
    let ratio_base = u64_at(LC_OFF_RATIO_BASE);
    let collection_size: u32 = u64_at(LC_OFF_COLLECTION_SIZE).try_into().map_err(|_| error!(VaultError::UnsupportedLaunchConfig))?;
    require!(ratio_base > 0 && collection_size > 0, VaultError::UnsupportedLaunchConfig);
    Ok(ExitView { ratio_base, collection_size })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hybrid_launch::{ALLOWED_RATIOS, MAX_FEE_LAMPORTS, MIN_FEE_LAMPORTS};

    #[test]
    fn request_fee_is_exactly_the_tier_or_reverts() {
        for &r in ALLOWED_RATIOS.iter() {
            let tier = hybrid_launch::tier_fee_lamports(r).unwrap();
            assert_eq!(request_fee(tier, r).unwrap(), tier);
            for bad in [0, 1, tier - 1, tier + 1, MIN_FEE_LAMPORTS - 1, MAX_FEE_LAMPORTS + 1, u64::MAX] {
                assert!(request_fee(bad, r).is_err(), "ratio {r} stored {bad}");
            }
        }
        assert!(request_fee(MAX_FEE_LAMPORTS, 10_000).is_err(), "ratio not in the table");
    }

    /// Re-roll fee == capture fee and release is free, so re-roll <= capture + release for every tier.
    #[test]
    fn reroll_le_capture_plus_release_every_tier() {
        let release = 0u64;
        for &r in ALLOWED_RATIOS.iter() {
            let f = request_fee(hybrid_launch::tier_fee_lamports(r).unwrap(), r).unwrap();
            let (capture, reroll) = (f, f);
            assert!(reroll <= capture + release, "ratio {r}");
            assert!(capture <= MAX_FEE_LAMPORTS);
        }
    }

    fn cfg_bytes(ratio_base: u64, n: u64, extra: usize) -> Vec<u8> {
        let mut d = vec![0u8; LC_STABLE_PREFIX_LEN + extra];
        d[..8].copy_from_slice(LaunchConfig::DISCRIMINATOR);
        d[LC_OFF_RATIO_BASE..LC_OFF_RATIO_BASE + 8].copy_from_slice(&ratio_base.to_le_bytes());
        d[LC_OFF_COLLECTION_SIZE..LC_OFF_COLLECTION_SIZE + 8].copy_from_slice(&n.to_le_bytes());
        d
    }

    #[test]
    fn exit_view_ignores_everything_but_ratio_and_n() {
        let mut d = cfg_bytes(7, 100, 64);
        d[hybrid_launch::LC_OFF_VERSION] = 99; // future version
        d[hybrid_launch::LC_OFF_FEE_LAMPORTS..hybrid_launch::LC_OFF_FEE_LAMPORTS + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        d[hybrid_launch::LC_OFF_FEE_RECIPIENT..hybrid_launch::LC_OFF_FEE_RECIPIENT + 32].copy_from_slice(&[9; 32]);
        let v = exit_view_in(&d).unwrap();
        assert_eq!((v.ratio_base, v.collection_size), (7, 100));
        assert!(exit_view_in(&d[..LC_STABLE_PREFIX_LEN - 1]).is_err());
        let mut bad = d.clone();
        bad[0] ^= 1;
        assert!(exit_view_in(&bad).is_err());
    }
}
