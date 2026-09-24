//! Account state for `fee_treasury`.

use anchor_lang::prelude::*;

/// Per-fee-mint treasury configuration.
///
/// PDA: `["treasury_config", fee_mint]` (canonical bump stored in `bump`).
/// Anchor's 8-byte discriminator prevents type-confusion with other accounts.
#[account]
#[derive(InitSpace)]
pub struct TreasuryConfig {
    /// Layout version (for future migrations).
    pub version: u8,
    /// Canonical bump of this config PDA.
    pub bump: u8,
    /// Canonical bump of the `["vault_authority", config]` PDA.
    pub vault_authority_bump: u8,
    /// Emergency stop: when true, no funds may leave the vault.
    pub paused: bool,
    /// Admin allowed to update spending rules / pause (should be a multisig).
    pub admin: Pubkey,
    /// The Token-2022 mint whose withheld transfer fees fund this treasury.
    pub fee_mint: Pubkey,
    /// Hard cap for a single NFT purchase, in fee-mint base units.
    pub max_spend_per_purchase: u64,
    /// Rolling cap for all purchases inside one window, in base units.
    pub max_spend_per_window: u64,
    /// Window length in seconds.
    pub spend_window_seconds: i64,
    /// Unix timestamp at which the current window started.
    pub window_start_ts: i64,
    /// Amount spent in the current window.
    pub spent_in_window: u64,
    /// Lifetime amount withdrawn from Token-2022 withheld fees into the vault.
    pub total_harvested: u64,
    /// Lifetime amount spent on purchases.
    pub total_spent: u64,
}
