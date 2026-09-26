//! How the vault reads economics from the hybrid_launch LaunchConfig (audit M-01: no local copies).
//!
//! Two readers, on purpose (audit M-41):
//! - `econ` (init, mint, open, capture, re-roll): typed load, fail-closed on version and structure,
//!   M-22 cap (collection_size <= MAX_COLLECTION_SIZE). The SOL fee is min(stored, current tier, MAX)
//!   (M-08), so a later tier cut or cap cut applies to old launches and a stored value can never
//!   push a charge above today's rules. It never reverts over a fee mismatch. Fees always go to
//!   today's PLATFORM_FEE_RECIPIENT constant (a rotated wallet doesn't strand old launches).
//! - `exit_view` (release, settle, expire): the user-exit paths. Reads ONLY ratio_base and
//!   collection_size as raw bytes at FROZEN offsets (hybrid_launch::stable_layout) after owner +
//!   discriminator checks. No version, fee, wallet or bounds check, so no launch-program upgrade
//!   (new tiers, rotated wallet, version bump, new MIN/MAX, appended fields) can freeze them.
//!   Release is FREE (Barton 5:04 PM MT), so exit paths never touch the fee.

use crate::error::VaultError;
use anchor_lang::{prelude::*, Discriminator};
use hybrid_launch::{
    fee_for_ratio, LaunchConfig, LC_DISCRIMINATOR_LEN, LC_OFF_COLLECTION_SIZE, LC_OFF_RATIO_BASE, LC_STABLE_PREFIX_LEN,
    MAX_COLLECTION_SIZE, MAX_FEE_LAMPORTS, PLATFORM_FEE_RECIPIENT,
};

pub const SUPPORTED_LAUNCH_CONFIG_VERSION: u8 = 4;

pub struct Econ {
    pub ratio_base: u64,
    pub collection_size: u32,
    /// Charged on capture and on re-roll (the same value, so re-roll <= capture + release(0)).
    pub request_fee_lamports: u64,
    pub fee_recipient: Pubkey,
}

/// min(stored, current tier for the ratio, MAX). A ratio no longer in the table uses min(stored, MAX).
pub fn request_fee(stored: u64, ratio_whole_tokens: u64) -> u64 {
    let tier = fee_for_ratio(ratio_whole_tokens).unwrap_or(u64::MAX);
    stored.min(tier).min(MAX_FEE_LAMPORTS)
}

pub fn econ(cfg: &LaunchConfig) -> Result<Econ> {
    require!(cfg.version == SUPPORTED_LAUNCH_CONFIG_VERSION, VaultError::UnsupportedLaunchConfig);
    require!(cfg.collection_size <= MAX_COLLECTION_SIZE as u64, VaultError::CollectionAboveCap);
    let collection_size: u32 = cfg.collection_size.try_into().map_err(|_| error!(VaultError::UnsupportedLaunchConfig))?;
    require!(collection_size > 0 && cfg.ratio_base > 0, VaultError::UnsupportedLaunchConfig);
    Ok(Econ {
        ratio_base: cfg.ratio_base,
        collection_size,
        request_fee_lamports: request_fee(cfg.fee_lamports, cfg.ratio_whole_tokens),
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
    use hybrid_launch::{ALLOWED_RATIOS, MIN_FEE_LAMPORTS};

    #[test]
    fn request_fee_is_min_of_stored_tier_cap() {
        for &r in ALLOWED_RATIOS.iter() {
            let tier = fee_for_ratio(r).unwrap();
            assert_eq!(request_fee(tier, r), tier);
            assert_eq!(request_fee(u64::MAX, r), tier, "forged/huge stored fee capped at tier");
            assert_eq!(request_fee(MIN_FEE_LAMPORTS - 1, r), MIN_FEE_LAMPORTS - 1, "lower stored wins");
        }
        assert_eq!(request_fee(50_000_000, 10_000), MAX_FEE_LAMPORTS, "dropped ratio -> MAX cap");
    }

    /// Re-roll fee == capture fee and release is free, so re-roll <= capture + release for every tier.
    #[test]
    fn reroll_le_capture_plus_release_every_tier() {
        let release = 0u64;
        for &r in ALLOWED_RATIOS.iter() {
            let f = request_fee(fee_for_ratio(r).unwrap(), r);
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
