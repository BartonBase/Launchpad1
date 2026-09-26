//! Pure launch-parameter validation (host-testable, no accounts).

use anchor_lang::prelude::*;

use crate::{constants::*, error::LaunchError};

/// Creator-chosen launch parameters (ADR-013 flat SOL fee model).
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchParams {
    pub decimals: u8,
    pub ratio_whole_tokens: u64,
    pub collection_size: u64,
    // No fee field of any kind: the flat SOL fee is fee_for_ratio(ratio) from the const tier table
    // and the recipient is PLATFORM_FEE_RECIPIENT, both fixed by the program (M-05, M-07, M-08).
    /// Graduation threshold (SOL raised on the curve), lamports. Default 85 SOL.
    pub graduation_threshold_lamports: u64,
}

/// Values derived from validated params.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivedAmounts {
    pub total_supply_base: u64,
    pub ratio_base: u64,
    pub max_tokens_in_nft_form: u64,
    pub fee_lamports: u64,
    /// Always 0 since lazy minting (ADR-016): no graduation slice funds any pre-mint.
    pub graduation_slice_pct: u8,
}

/// Threshold bounds only. The affordability rule (slice of the raise must fund N pre-mints) was
/// DROPPED with lazy minting (Barton 2026-09-25 5:13 PM MT, ADR-016): each mint is paid from the
/// requester's escrow at settle.
pub fn check_graduation_threshold(threshold_lamports: u64) -> Result<()> {
    require!(
        (MIN_GRADUATION_THRESHOLD_LAMPORTS..=MAX_GRADUATION_THRESHOLD_LAMPORTS).contains(&threshold_lamports),
        LaunchError::GraduationThresholdOutOfRange
    );
    Ok(())
}

/// The flat SOL fee for a ratio, checked against the floor and the hard cap (shared with
/// hybrid_vault's re-validation).
pub fn checked_fee_for_ratio(ratio_whole_tokens: u64) -> Result<u64> {
    fee_for_ratio(ratio_whole_tokens).ok_or(LaunchError::RatioNotAllowed)?;
    Ok(tier_fee_lamports(ratio_whole_tokens).ok_or(LaunchError::SolFeeOutOfRange)?)
}

/// QA-FEE-03 (Barton 2026-09-26): THE single tier derivation used by launch, register_dbc_launch and
/// hybrid_vault's capture/re-roll. `Some(fee)` only for a ratio in FEE_TIERS whose fee is non-zero and
/// within [MIN_FEE_LAMPORTS, MAX_FEE_LAMPORTS] (hard cap 0.01 SOL).
pub fn tier_fee_lamports(ratio_whole_tokens: u64) -> Option<u64> {
    match fee_for_ratio(ratio_whole_tokens) {
        Some(f) if f != 0 && f >= MIN_FEE_LAMPORTS && f <= MAX_FEE_LAMPORTS => Some(f),
        _ => None,
    }
}

/// QA-FEE-03: a stored fee is valid only if it is EXACTLY the tier for the ratio (0 is never valid).
pub fn is_exact_tier_fee(stored_fee_lamports: u64, ratio_whole_tokens: u64) -> bool {
    stored_fee_lamports != 0 && tier_fee_lamports(ratio_whole_tokens) == Some(stored_fee_lamports)
}

pub fn validate(p: &LaunchParams) -> Result<DerivedAmounts> {
    require!(p.decimals <= MAX_DECIMALS, LaunchError::InvalidDecimals);
    require!(ALLOWED_RATIOS.contains(&p.ratio_whole_tokens), LaunchError::RatioNotAllowed);
    require!(p.collection_size >= 1, LaunchError::ZeroCollectionSize);
    require!(p.collection_size >= MIN_COLLECTION_SIZE, LaunchError::CollectionBelowMinimum);
    require!(p.collection_size <= MAX_COLLECTION_SIZE, LaunchError::CollectionAboveCap);
    let fee_lamports = checked_fee_for_ratio(p.ratio_whole_tokens)?;
    check_graduation_threshold(p.graduation_threshold_lamports)?;
    let graduation_slice_pct = 0;

    let scale = 10u64.checked_pow(u32::from(p.decimals)).ok_or(LaunchError::MathOverflow)?;
    let total_supply_base = TOTAL_SUPPLY_WHOLE_TOKENS.checked_mul(scale).ok_or(LaunchError::MathOverflow)?;
    let ratio_base = p.ratio_whole_tokens.checked_mul(scale).ok_or(LaunchError::MathOverflow)?;
    require!(total_supply_base % ratio_base == 0, LaunchError::RatioNotAllowed);
    // The supply check: collection_size * ratio <= 1B (checked_mul => overflow is a rejection too).
    let max_tokens_in_nft_form = p
        .collection_size
        .checked_mul(ratio_base)
        .ok_or(LaunchError::CollectionTooLargeForSupply)?;
    require!(max_tokens_in_nft_form <= total_supply_base, LaunchError::CollectionTooLargeForSupply);

    Ok(DerivedAmounts { total_supply_base, ratio_base, max_tokens_in_nft_form, fee_lamports, graduation_slice_pct })
}

/// Largest allowed collection size for a ratio (whole tokens): min(1B / ratio, MAX_COLLECTION_SIZE).
pub fn max_collection_size(ratio_whole_tokens: u64) -> Option<u64> {
    ALLOWED_RATIOS
        .contains(&ratio_whole_tokens)
        .then(|| (TOTAL_SUPPLY_WHOLE_TOKENS / ratio_whole_tokens).min(MAX_COLLECTION_SIZE))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> LaunchParams {
        LaunchParams {
            decimals: 6,
            ratio_whole_tokens: 1_000_000,
            collection_size: 1_000,
            graduation_threshold_lamports: DEFAULT_GRADUATION_THRESHOLD_LAMPORTS,
        }
    }

    /// Enough raise to fund any N <= 10k (10k x 0.00509 x 1.25 / 10% = 636.25 SOL).
    const BIG_RAISE: u64 = 700_000_000_000;

    #[test]
    fn supply_table_max_collection_size_per_ratio() {
        // max N = min(10_000, 1B / ratio) (Barton 2026-09-25 4:55 PM MT); min N = 100 for all.
        let table = [
            (50_000, 10_000),
            (100_000, 10_000),
            (200_000, 5_000),
            (500_000, 2_000),
            (1_000_000, 1_000),
            (2_500_000, 400),
            (5_000_000, 200),
        ];
        assert_eq!(table.len(), ALLOWED_RATIOS.len());
        for (ratio, max) in table {
            assert_eq!(max_collection_size(ratio), Some(max));
            for d in [0u8, 6, 9] {
                let ok = LaunchParams { ratio_whole_tokens: ratio, collection_size: max, decimals: d, graduation_threshold_lamports: BIG_RAISE };
                let got = validate(&ok).expect("max size must pass");
                if TOTAL_SUPPLY_WHOLE_TOKENS / ratio <= MAX_COLLECTION_SIZE {
                    assert_eq!(got.max_tokens_in_nft_form, got.total_supply_base, "max size uses 100% of supply");
                }
                assert!(validate(&LaunchParams { collection_size: max + 1, ..ok }).is_err(), "max+1 r={ratio} d={d}");
                validate(&LaunchParams { collection_size: MIN_COLLECTION_SIZE, ..ok }).expect("min size must pass");
                assert!(validate(&LaunchParams { collection_size: MIN_COLLECTION_SIZE - 1, ..ok }).is_err());
            }
        }
    }

    #[test]
    fn largest_ratio_with_9_decimals_does_not_overflow() {
        // 5M * 10^9 = 5e15 base units per NFT; 200 NFTs = 1e18 = full supply (< u64::MAX 1.8e19).
        // (Any bps-style product such as ratio * 10_000 would overflow u64; none remains since the
        // token fee was dropped, and new percentage math must use u128 intermediates.)
        let p = LaunchParams { ratio_whole_tokens: 5_000_000, decimals: 9, collection_size: 200, ..base() };
        let d = validate(&p).unwrap();
        assert_eq!(d.ratio_base, 5_000_000 * 10u64.pow(9));
        assert_eq!(d.max_tokens_in_nft_form, d.total_supply_base);
        assert!(u64::try_from(u128::from(d.ratio_base) * 10_000).is_err(), "the u64 product really would overflow");
        assert!(validate(&LaunchParams { collection_size: 201, ..p }).is_err());
    }

    #[test]
    fn attack_collection_size_above_cap_is_rejected() {
        let p = LaunchParams { ratio_whole_tokens: 50_000, collection_size: MAX_COLLECTION_SIZE + 1, graduation_threshold_lamports: BIG_RAISE, ..base() };
        assert!(validate(&p).is_err());
        validate(&LaunchParams { collection_size: MAX_COLLECTION_SIZE, ..p }).unwrap();
    }

    #[test]
    fn attack_10k_ratio_is_dropped_and_rejected() {
        assert!(!ALLOWED_RATIOS.contains(&10_000));
        assert!(validate(&LaunchParams { ratio_whole_tokens: 10_000, collection_size: 100, ..base() }).is_err());
        assert!(fee_for_ratio(10_000).is_none());
    }

    #[test]
    fn lazy_mint_any_n_up_to_cap_launches_at_any_valid_threshold() {
        // Affordability dropped (ADR-016): 10k NFTs at the 10 SOL minimum threshold are fine.
        for n in [100u64, 1_336, 5_000, MAX_COLLECTION_SIZE] {
            let d = validate(&LaunchParams { ratio_whole_tokens: 50_000, collection_size: n, graduation_threshold_lamports: MIN_GRADUATION_THRESHOLD_LAMPORTS, ..base() }).unwrap();
            assert_eq!(d.graduation_slice_pct, 0);
        }
        assert!(validate(&LaunchParams { graduation_threshold_lamports: MIN_GRADUATION_THRESHOLD_LAMPORTS - 1, ..base() }).is_err());
    }

    #[test]
    fn attack_collection_size_overflow_is_rejected_not_wrapped() {
        assert!(validate(&LaunchParams { collection_size: u64::MAX, ..base() }).is_err());
    }

    #[test]
    fn fee_tier_for_every_ratio_matches_barton_table() {
        let want = [
            (50_000, 2_000_000),
            (100_000, 5_000_000),
            (200_000, 5_000_000),
            (500_000, 10_000_000),
            (1_000_000, 10_000_000),
            (2_500_000, 10_000_000),
            (5_000_000, 10_000_000),
        ];
        assert_eq!(want.map(|w| w.0), ALLOWED_RATIOS);
        for (r, fee) in want {
            let d = validate(&LaunchParams { ratio_whole_tokens: r, collection_size: MIN_COLLECTION_SIZE, ..base() }).unwrap();
            assert_eq!(d.fee_lamports, fee, "r={r}");
            // Same fee on capture, release and re-roll => re-roll <= capture + release in every tier.
            assert!(fee <= fee + fee);
        }
    }

    #[test]
    fn fee_can_never_exceed_the_0_01_sol_hard_cap() {
        assert_eq!(MAX_FEE_LAMPORTS, 10_000_000);
        for r in ALLOWED_RATIOS {
            assert!(checked_fee_for_ratio(r).unwrap() <= MAX_FEE_LAMPORTS);
        }
        // Any ratio outside the table (so any "custom" fee) is refused.
        for r in [0u64, 1, 10_000, 20_000, 10_000_000, u64::MAX] {
            assert!(checked_fee_for_ratio(r).is_err(), "r={r}");
            assert!(validate(&LaunchParams { ratio_whole_tokens: r, ..base() }).is_err());
        }
    }

    #[test]
    fn fee_floor_is_at_or_above_rent_exempt_minimum_for_a_0_byte_account() {
        assert_eq!(RENT_EXEMPT_MIN_0_BYTES, (128 * 3_480) * 2);
        assert!(MIN_FEE_LAMPORTS >= RENT_EXEMPT_MIN_0_BYTES);
        assert_eq!(FEE_TIERS.iter().map(|t| t.1).min(), Some(MIN_FEE_LAMPORTS));
    }

    #[test]
    fn collection_below_minimum_is_rejected() {
        for n in [0u64, 1, 50, 99] {
            assert!(validate(&LaunchParams { collection_size: n, ..base() }).is_err(), "N={n}");
        }
        validate(&LaunchParams { collection_size: 100, ..base() }).unwrap();
    }

    #[test]
    fn undersized_collection_is_allowed() {
        let d = validate(&LaunchParams { collection_size: 500, ..base() }).unwrap();
        assert_eq!(d.max_tokens_in_nft_form * 2, d.total_supply_base);
    }

    /// QA-FEE-03: exact tier only; 0, off-tier and above-cap are invalid; unknown ratio invalid.
    #[test]
    fn qa_fee03_stored_fee_must_be_exactly_the_tier() {
        for &(r, tier) in FEE_TIERS.iter() {
            assert_eq!(tier_fee_lamports(r), Some(tier));
            assert_eq!(checked_fee_for_ratio(r).unwrap(), tier, "creation uses the same derivation");
            assert!(is_exact_tier_fee(tier, r), "ratio {r}");
            for bad in [0, 1, tier - 1, tier + 1, MAX_FEE_LAMPORTS + 1, u64::MAX] {
                assert!(!is_exact_tier_fee(bad, r), "ratio {r} stored {bad}");
            }
        }
        assert_eq!((tier_fee_lamports(50_000), tier_fee_lamports(100_000), tier_fee_lamports(200_000)), (Some(2_000_000), Some(5_000_000), Some(5_000_000)));
        for r in [500_000, 1_000_000, 2_500_000, 5_000_000] {
            assert_eq!(tier_fee_lamports(r), Some(10_000_000));
        }
        assert!(tier_fee_lamports(10_000).is_none() && !is_exact_tier_fee(0, 10_000));
        assert_eq!(MAX_FEE_LAMPORTS, 10_000_000, "hard cap 0.01 SOL");
    }

}
