//! Pure launch-parameter validation (host-testable, no accounts).

use anchor_lang::prelude::*;

use crate::{constants::*, error::LaunchError};

/// Creator-chosen launch parameters.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaunchParams {
    pub decimals: u8,
    pub ratio_whole_tokens: u64,
    pub collection_size: u64,
    pub capture_fee_bps: u16,
    pub reroll_fee_bps: u16,
    /// Must be FEE_DESTINATION_BURN.
    pub fee_destination: u8,
}

/// Values derived from validated params.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivedAmounts {
    pub total_supply_base: u64,
    pub ratio_base: u64,
    pub max_tokens_in_nft_form: u64,
    pub capture_fee_amount: u64,
    pub reroll_fee_amount: u64,
}

pub fn validate(p: &LaunchParams) -> Result<DerivedAmounts> {
    require!(p.decimals <= MAX_DECIMALS, LaunchError::InvalidDecimals);
    require!(ALLOWED_RATIOS.contains(&p.ratio_whole_tokens), LaunchError::RatioNotAllowed);
    require!(p.collection_size >= 1, LaunchError::ZeroCollectionSize);
    require!(p.collection_size >= MIN_COLLECTION_SIZE, LaunchError::CollectionBelowMinimum);
    require!(p.capture_fee_bps <= MAX_TOKEN_FEE_BPS, LaunchError::FeeAboveCap);
    require!(p.reroll_fee_bps <= MAX_TOKEN_FEE_BPS, LaunchError::FeeAboveCap);
    require!(p.capture_fee_bps >= p.reroll_fee_bps, LaunchError::CaptureFeeBelowRerollFee);
    require!(p.fee_destination == FEE_DESTINATION_BURN, LaunchError::FeeDestinationNotBurn);

    let scale = 10u64.checked_pow(u32::from(p.decimals)).ok_or(LaunchError::MathOverflow)?;
    let total_supply_base = TOTAL_SUPPLY_WHOLE_TOKENS.checked_mul(scale).ok_or(LaunchError::MathOverflow)?;
    let ratio_base = p.ratio_whole_tokens.checked_mul(scale).ok_or(LaunchError::MathOverflow)?;
    // Ratio divides the supply exactly (true for every allowed ratio; asserted anyway).
    require!(total_supply_base % ratio_base == 0, LaunchError::RatioNotAllowed);
    // The supply check: collection_size * ratio <= 1B (checked_mul => overflow is a rejection too).
    let max_tokens_in_nft_form = p
        .collection_size
        .checked_mul(ratio_base)
        .ok_or(LaunchError::CollectionTooLargeForSupply)?;
    require!(max_tokens_in_nft_form <= total_supply_base, LaunchError::CollectionTooLargeForSupply);

    let fee = |bps: u16| -> Result<u64> {
        let v = u128::from(ratio_base) * u128::from(bps);
        // ratio_base is a multiple of 10_000, so this division is exact.
        require!(v % 10_000 == 0, LaunchError::MathOverflow);
        u64::try_from(v / 10_000).map_err(|_| error!(LaunchError::MathOverflow))
    };
    Ok(DerivedAmounts {
        total_supply_base,
        ratio_base,
        max_tokens_in_nft_form,
        capture_fee_amount: fee(p.capture_fee_bps)?,
        reroll_fee_amount: fee(p.reroll_fee_bps)?,
    })
}

/// Largest allowed collection size for a ratio (whole tokens): 1B / ratio.
pub fn max_collection_size(ratio_whole_tokens: u64) -> Option<u64> {
    ALLOWED_RATIOS
        .contains(&ratio_whole_tokens)
        .then(|| TOTAL_SUPPLY_WHOLE_TOKENS / ratio_whole_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> LaunchParams {
        LaunchParams {
            decimals: 6,
            ratio_whole_tokens: 1_000_000,
            collection_size: 1_000,
            capture_fee_bps: 200,
            reroll_fee_bps: 200,
            fee_destination: FEE_DESTINATION_BURN,
        }
    }

    #[test]
    fn supply_table_max_collection_size_per_ratio() {
        // Supply table (Barton 2026-09-24): ratio -> max N = 1B / ratio; min N = 100 for all.
        let table = [
            (10_000, 100_000),
            (50_000, 20_000),
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
                let ok = LaunchParams { ratio_whole_tokens: ratio, collection_size: max, decimals: d, ..base() };
                let got = validate(&ok).expect("max size must pass");
                assert_eq!(got.max_tokens_in_nft_form, got.total_supply_base, "max size uses 100% of supply");
                let over = LaunchParams { collection_size: max + 1, ..ok };
                assert!(validate(&over).is_err(), "max+1 must fail for ratio {ratio} decimals {d}");
                let min = LaunchParams { collection_size: MIN_COLLECTION_SIZE, ..ok };
                validate(&min).expect("min size must pass");
                let under = LaunchParams { collection_size: MIN_COLLECTION_SIZE - 1, ..ok };
                assert!(validate(&under).is_err(), "99 must fail for ratio {ratio} decimals {d}");
            }
        }
    }

    #[test]
    fn attack_collection_size_overflow_is_rejected_not_wrapped() {
        let p = LaunchParams { collection_size: u64::MAX, ..base() };
        assert!(validate(&p).is_err());
    }

    #[test]
    fn fee_amounts_are_exact_for_every_ratio_and_bps() {
        for ratio in ALLOWED_RATIOS {
            for bps in [0u16, 1, 7, 200, 999, 1_000] {
                let p = LaunchParams { ratio_whole_tokens: ratio, collection_size: MIN_COLLECTION_SIZE, capture_fee_bps: bps, reroll_fee_bps: 0, decimals: 0, ..base() };
                let d = validate(&p).unwrap();
                assert_eq!(u128::from(d.capture_fee_amount) * 10_000, u128::from(d.ratio_base) * u128::from(bps));
            }
        }
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
}
