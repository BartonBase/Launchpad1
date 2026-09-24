use anchor_lang::prelude::*;

pub const VAULT_SEED: &[u8] = b"vault";
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";
pub const RANDOMNESS_AUTHORITY_SEED: &[u8] = b"randomness_authority";
pub const VAULT_TOKENS_SEED: &[u8] = b"vault_tokens";
pub const FEE_ESCROW_SEED: &[u8] = b"fee_escrow";
pub const COLLECTION_SEED: &[u8] = b"collection";
pub const ASSET_SEED: &[u8] = b"asset";
pub const REQUEST_SEED: &[u8] = b"request";
pub const RAND_LOCK_SEED: &[u8] = b"rand_lock";

/// Metaplex Core program (hard-coded; every Core CPI goes to this id only).
pub const MPL_CORE_ID: Pubkey = pubkey!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

/// Switchboard On-Demand program that must own randomness accounts.
#[cfg(not(feature = "mainnet"))]
pub const SWITCHBOARD_PROGRAM_ID: Pubkey = pubkey!("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2");
#[cfg(feature = "mainnet")]
pub const SWITCHBOARD_PROGRAM_ID: Pubkey = pubkey!("SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv");

/// Switchboard `randomness_commit` instruction discriminator (switchboard-on-demand 0.13.0).
pub const SB_RANDOMNESS_COMMIT_DISCRIMINATOR: [u8; 8] = [52, 170, 152, 201, 179, 133, 242, 141];

/// SlotHashes sysvar. Passed through to Switchboard's commit only; hybrid_vault NEVER reads it.
pub const SLOT_HASHES_SYSVAR_ID: Pubkey = pubkey!("SysvarS1otHashes111111111111111111111111111");

/// A request whose randomness hasn't been revealed after this many slots (~10 min) can be expired.
pub const REQUEST_TIMEOUT_SLOTS: u64 = 1_500;
/// Guardian pause: max length (~7 days at 400 ms/slot) and cooldown before the next pause (~1 day).
pub const MAX_PAUSE_SLOTS: u64 = 1_512_000;
pub const PAUSE_COOLDOWN_SLOTS: u64 = 216_000;
/// Incoming entries merged per settle; larger backlogs are drained with `merge_incoming`.
pub const MAX_MERGE_PER_IX: u32 = 32;

pub const MAX_NAME_LEN: usize = 32;
pub const MAX_URI_LEN: usize = 200;
/// Merkle depth cap: 2^17 = 131,072 > max collection size (100,000).
pub const MAX_MERKLE_DEPTH: usize = 17;

pub const REQUEST_KIND_CAPTURE: u8 = 0;
pub const REQUEST_KIND_REROLL: u8 = 1;
pub const NO_HANDED_IN: u32 = u32::MAX;
