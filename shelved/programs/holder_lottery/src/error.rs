//! Custom errors for `holder_lottery`.

use anchor_lang::prelude::*;

#[error_code]
pub enum LotteryError {
    #[msg("Ticket mint must be owned by the Token-2022 program")]
    MintNotToken2022,
    #[msg("ticket_threshold must be > 0")]
    ZeroTicketThreshold,
    #[msg("min_holding_seconds is outside the allowed range")]
    InvalidHoldingPeriod,
}
