use anchor_lang::prelude::*;

/// One vault per hybrid_launch LaunchConfig. PDA `["vault", launch_config]`.
/// Economics (ratio, fees, burn) are copied from the immutable LaunchConfig at init and never change.
#[account]
#[derive(InitSpace, Debug)]
pub struct Vault {
    pub version: u8,
    pub bump: u8,
    pub authority_bump: u8,
    pub randomness_authority_bump: u8,
    pub vault_tokens_bump: u8,
    pub fee_escrow_bump: u8,
    pub collection_bump: u8,
    pub sealed: bool,
    pub launch_config: Pubkey,
    pub mint: Pubkey,
    pub creator: Pubkey,
    /// Pubkey::default() = no guardian (no pause power at all).
    pub guardian: Pubkey,
    pub collection: Pubkey,
    pub pool: Pubkey,
    pub vault_tokens: Pubkey,
    pub fee_escrow: Pubkey,
    /// Merkle root over (index, name, uri) of every asset, committed before any deposit.
    pub trait_root: [u8; 32],
    pub collection_size: u32,
    pub deposited_count: u32,
    pub ratio_base: u64,
    pub capture_fee_amount: u64,
    pub reroll_fee_amount: u64,
    /// NFTs owned by users (not in pool/incoming, not held as a re-roll hand-in).
    pub assets_outside: u64,
    pub pending_captures: u64,
    pub pending_rerolls: u64,
    /// Sum of token fees escrowed for pending requests (burned at settle, refunded at expire).
    pub pending_fee_total: u64,
    /// Next request sequence number / next request that may be settled or expired (FIFO head).
    pub next_seq: u64,
    pub next_settle_seq: u64,
    pub paused_until_slot: u64,
    pub pause_cooldown_until_slot: u64,
    pub total_burned: u64,
    pub total_captures: u64,
    pub total_rerolls: u64,
    pub total_unwraps: u64,
}

impl Vault {
    pub fn pending_draws(&self) -> Result<u64> {
        self.pending_captures
            .checked_add(self.pending_rerolls)
            .ok_or_else(|| error!(crate::error::VaultError::MathOverflow))
    }
}

/// One pending capture or re-roll. PDA `["request", vault, seq_le]`. Closed (rent to `user`) at settle/expire.
#[account]
#[derive(InitSpace, Debug)]
pub struct Request {
    pub bump: u8,
    pub kind: u8,
    pub vault: Pubkey,
    pub seq: u64,
    pub user: Pubkey,
    /// Refund destination for tokens, fixed at request time.
    pub user_token: Pubkey,
    pub randomness: Pubkey,
    /// Switchboard seed slot recorded right after the vault's own commit CPI.
    pub seed_slot: u64,
    pub deadline_slot: u64,
    pub fee_amount: u64,
    /// Asset index handed in for a re-roll, NO_HANDED_IN for a capture.
    pub handed_in_index: u32,
}

/// Marks a Switchboard randomness account as bound to a pending request so the vault never
/// re-commits it (which would change the value) before settle/expire. PDA `["rand_lock", randomness]`.
#[account]
#[derive(InitSpace, Debug)]
pub struct RandLock {
    pub bump: u8,
    pub vault: Pubkey,
    pub seq: u64,
}
