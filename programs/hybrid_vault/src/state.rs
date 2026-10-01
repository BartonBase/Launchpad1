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

/// One Mode 3 vault per `BurnLaunchConfig`. PDA `["vault", launch_config]`.
///
/// Not a [`Vault`]. Mode 2 instructions cannot load it, and this mode has no release, re-roll, or
/// expire. The next NFT index is `minted_count` (not a VRF draw): burn and mint are one instruction,
/// so a failed reveal cannot destroy tokens that there is no release instruction to return.
#[account]
#[derive(InitSpace, Debug)]
pub struct PermanentVault {
    pub version: u8,
    pub bump: u8,
    pub authority_bump: u8,
    pub collection_bump: u8,
    pub open: bool,
    /// Always `hybrid_launch::LAUNCH_MODE_BURN`.
    pub launch_mode: u8,
    pub launch_config: Pubkey,
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub collection: Pubkey,
    pub trait_root: [u8; 32],
    pub trait_schema_hash: [u8; 32],
    pub minted_count: u32,
    /// Tokens burned by wraps. Not a balance anyone can withdraw.
    pub total_burned_base: u64,
    pub total_fee_lamports: u64,
    pub total_wraps: u64,
}

/// Mode 4 vault. Not a [`Vault`] and not a [`PermanentVault`].
///
/// Wrap locks `ratio` tokens (it does not burn them) and mints the next project NFT.
/// Transfer-tax withheld by Token-2022 is withdrawn by `tax_authority` and paid only to
/// the current owner of one of this collection's NFTs. There is no burn and no unwrap.
#[account]
#[derive(InitSpace, Debug)]
pub struct TaxVault {
    pub version: u8,
    pub bump: u8,
    pub authority_bump: u8,
    pub tax_authority_bump: u8,
    pub collection_bump: u8,
    /// Always `hybrid_launch::LAUNCH_MODE_TOKEN22`.
    pub launch_mode: u8,
    pub launch_config: Pubkey,
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub collection: Pubkey,
    pub trait_root: [u8; 32],
    pub trait_schema_hash: [u8; 32],
    pub minted_count: u32,
    /// Fixed at init. Caps how many NFTs can exist. Not used to reserve rewards.
    pub collection_size: u32,
    /// Next NFT index the program will pay. Callers cannot choose a different one.
    pub payout_cursor: u32,
    /// How many NFTs were already minted when this round was frozen.
    pub round_minted: u32,
    /// Exact tokens each of those NFTs is paid. Zero when no round is open.
    pub round_share: u64,
    /// 1 while a round is frozen. Nobody can change the set or the share while this is set.
    pub round_open: u8,
    /// Tokens assigned to minted NFTs. Claims cannot exceed this.
    pub credited_base: u64,
    /// Tokens already paid to holders of minted NFTs.
    pub paid_base: u64,
    /// Arrived tokens not yet assigned because the pot is under the market-cap tier,
    /// no NFT exists yet, or the split left rounding dust.
    pub pending_base: u64,
    /// Locked at init from the launch. Values the pot and the market cap. Not updatable.
    pub buyback_lamports_per_whole: u64,
    /// Locked at init. 1_000_000_000 * 10^decimals.
    pub total_supply_base: u64,
    pub decimals: u8,
    pub total_locked_base: u64,
    pub total_fee_lamports: u64,
    pub total_wraps: u64,
    pub tax_bps: u16,
    /// `KIND_SPLIT` (Mode 4) or `KIND_RAFFLE` (Mode 5). Fixed at init.
    pub kind: u8,
    /// Raffle only. Idle until a pot crosses the tier, then snapshot, commit, reveal.
    pub raffle_phase: u8,
    pub randomness_authority_bump: u8,
    /// Pinned Switchboard queue. Default for a split vault.
    pub sb_queue: Pubkey,
    /// Increments when a raffle round opens, before any seat is written.
    pub round_id: u64,
    /// Whole pot for the open raffle round. Zero for a split round.
    pub round_pot: u64,
    pub randomness: Pubkey,
    pub raffle_oracle: Pubkey,
    pub seed_slot: u64,
    pub revealed: [u8; 32],
    /// Raffle: seats written this round. A burned NFT is skipped (`skip_dead_nft`) and gets no seat,
    /// so seats are numbered densely 0..live_seats and the draw is uniform over live seats only.
    pub live_seats: u32,
    /// Raffle: commits made this round (1 + retries). At most 1 + MAX_RECOMMITS; after that a timed-out
    /// round is rolled back (pot returned to `pending_base`, nothing lost).
    pub raffle_commits: u8,
    /// Raffle: the oracle of each commit this round. A retry must be served by a different oracle.
    pub raffle_oracles: [Pubkey; 4],
    /// Raffle: after this slot an unrevealed commit may be retried by anyone (`retry_raffle`).
    pub raffle_deadline_slot: u64,
}
#[account]
#[derive(InitSpace, Debug)]
pub struct TaxShare {
    pub vault: Pubkey,
    pub asset: Pubkey,
    pub index: u128,
}

/// Owner of one NFT at the moment the raffle list was frozen. Written before randomness exists.
#[account]
#[derive(InitSpace, Debug)]
pub struct RaffleSeat {
    pub vault: Pubkey,
    pub round_id: u64,
    pub index: u32,
    pub owner: Pubkey,
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
    /// Lazy-mint escrow amount deposited at request (ADR-016); the lamports sit in the separate
    /// `["mint_escrow", vault, seq]` PDA, not in this account.
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
