//! Custom errors for `fee_treasury`.

use anchor_lang::prelude::*;

#[error_code]
pub enum TreasuryError {
    #[msg("Fee mint must be owned by the Token-2022 program")]
    MintNotToken2022,
    #[msg("max_spend_per_purchase must be > 0")]
    ZeroPurchaseCap,
    #[msg("max_spend_per_purchase must be <= max_spend_per_window")]
    PurchaseCapExceedsWindowCap,
    #[msg("spend_window_seconds is outside the allowed range")]
    InvalidSpendWindow,
}
