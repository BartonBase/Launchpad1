//! Account state for `holder_lottery`.

use anchor_lang::prelude::*;

/// Which external randomness provider a lottery uses. Never slot hashes.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum RandomnessProvider {
    /// Switchboard On-Demand randomness (commit/reveal, TEE oracles).
    SwitchboardOnDemand,
    /// ORAO VRF (multi-node EdDSA VRF).
    OraoVrf,
}

/// Per-ticket-mint lottery configuration.
///
/// PDA: `["lottery_config", ticket_mint]` (canonical bump stored in `bump`).
#[account]
#[derive(InitSpace)]
pub struct LotteryConfig {
    /// Layout version (for future migrations).
    pub version: u8,
    /// Canonical bump of this config PDA.
    pub bump: u8,
    /// Canonical bump of the `["prize_vault", config]` PDA.
    pub prize_vault_bump: u8,
    /// Emergency stop for new rounds/claims (cannot move prizes).
    pub paused: bool,
    /// Lottery admin (multisig). Cannot choose winners or withdraw prizes.
    pub admin: Pubkey,
    /// The Token-2022 mint whose holders get tickets.
    pub ticket_mint: Pubkey,
    /// Base units of `ticket_mint` per ticket. tickets = floor(balance / threshold).
    pub ticket_threshold: u64,
    /// Minimum continuous holding period before balance counts.
    pub min_holding_seconds: i64,
    /// External randomness source for draws.
    pub randomness_provider: RandomnessProvider,
    /// Monotonic round counter (next round id).
    pub current_round: u64,
}
