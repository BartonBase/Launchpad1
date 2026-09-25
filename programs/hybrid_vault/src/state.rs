use anchor_lang::prelude::*;

/// One vault per hybrid_launch LaunchConfig. PDA `["vault", launch_config]`; every token account,
/// the collection and every asset are seeded by this vault key (never by an authority alone, A-06).
/// Economics (ratio, N, SOL fees, fee recipient) are READ from the immutable LaunchConfig on every
/// use (M-01); the vault keeps no copies. No instruction changes them.
#[account]
#[derive(InitSpace, Debug)]
pub struct Vault {
    pub version: u8,
    pub bump: u8,
    pub authority_bump: u8,
    pub randomness_authority_bump: u8,
    pub vault_tokens_bump: u8,
    pub collection_bump: u8,
    /// Converting is closed until `open_vault` (graduation verified + collection fully minted). One-way.
    pub open: bool,
    /// The immutable hybrid_launch LaunchConfig. Ratio, N, mint, SOL fees and fee recipient are READ
    /// FROM IT on every use (audit M-01: no local copies).
    pub launch_config: Pubkey,
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub collection: Pubkey,
    pub pool: Pubkey,
    pub vault_tokens: Pubkey,
    /// Switchboard queue pinned at init; every randomness account must use it.
    pub sb_queue: Pubkey,
    /// Leaf-v2 Merkle root (merkle.rs) and the trait schema hash, committed at init.
    pub trait_root: [u8; 32],
    pub trait_schema_hash: [u8; 32],
    pub minted_count: u32,
    /// NFTs owned by users (not in pool/incoming, not held as a re-roll hand-in).
    pub assets_outside: u64,
    pub pending_captures: u64,
    pub pending_rerolls: u64,
    /// Next request sequence number / next request that may be settled or expired (FIFO head).
    pub next_seq: u64,
    pub next_settle_seq: u64,
    pub opened_at_slot: u64,
    /// Capture + re-roll SOL fees paid directly to the fee recipient.
    pub total_fee_lamports: u64,
    pub total_captures: u64,
    pub total_rerolls: u64,
    pub total_unwraps: u64,
    pub total_recommits: u64,
    pub total_expired: u64,
}

impl Vault {
    pub fn pending_draws(&self) -> Result<u64> {
        self.pending_captures
            .checked_add(self.pending_rerolls)
            .ok_or_else(|| error!(crate::error::VaultError::MathOverflow))
    }
}

/// One pending capture or re-roll. PDA `["request", vault, seq_le]`. Closed (rent to `user`) only
/// by settle, or by the principal-only expire after MAX_RECOMMITS unrevealed re-commits. There is
/// no cancel instruction and the fees are never refunded.
#[account]
#[derive(InitSpace, Debug)]
pub struct Request {
    pub bump: u8,
    pub kind: u8,
    pub vault: Pubkey,
    pub seq: u64,
    pub user: Pubkey,
    pub randomness: Pubkey,
    /// Switchboard seed slot of the vault's latest commit for this request.
    pub seed_slot: u64,
    /// After this slot, if still unrevealed, anyone may re-commit (REVEAL_TIMEOUT_SLOTS).
    pub deadline_slot: u64,
    /// Number of commits so far (1 + re-commits), at most 1 + MAX_RECOMMITS.
    pub commits: u8,
    /// Oracle of each commit; a re-commit must use an oracle not used before for this request.
    pub oracles: [Pubkey; 4],
    /// Set by `reveal_randomness`: the value is recorded once and never changes (settle reads it).
    pub revealed: bool,
    pub value: [u8; 32],
    /// Asset index handed in for a re-roll, NO_HANDED_IN for a capture.
    pub handed_in_index: u32,
    /// Lazy-mint escrow held in this account's lamports on top of rent (ADR-016).
    pub mint_escrow_lamports: u64,
}

/// Binds a Switchboard randomness account to one pending request so no other request can commit it
/// while this one is pending. PDA `["rand_lock", randomness]`.
#[account]
#[derive(InitSpace, Debug)]
pub struct RandLock {
    pub bump: u8,
    pub vault: Pubkey,
    pub seq: u64,
}
