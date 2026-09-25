use anchor_lang::prelude::*;

pub const VAULT_SEED: &[u8] = b"vault";
pub const VAULT_AUTHORITY_SEED: &[u8] = b"vault_authority";
pub const RANDOMNESS_AUTHORITY_SEED: &[u8] = b"randomness_authority";
pub const VAULT_TOKENS_SEED: &[u8] = b"vault_tokens";
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

/// Build marker: logged by `init_vault`, so it's in the binary's rodata and deploy scripts can verify
/// which Switchboard cluster id was compiled in (pubkey constants are inlined as immediates and
/// can't be grepped). An exported `#[no_mangle]` static breaks SBPF v2 loading, hence the log.
#[cfg(not(feature = "mainnet"))]
pub const SWITCHBOARD_CLUSTER_MARKER: &str = "hybrid_vault:switchboard=devnet:Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2";
#[cfg(feature = "mainnet")]
pub const SWITCHBOARD_CLUSTER_MARKER: &str = "hybrid_vault:switchboard=mainnet:SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv";

/// Switchboard instruction discriminators (Anchor: sha256("global:<name>")[..8]). The commit value
/// matches switchboard-on-demand 0.13.0's `RandomnessCommit`; init/reveal match the codama-generated
/// `switchboard_on_demand_sol` builders (accounts documented in randomness.rs).
pub const SB_RANDOMNESS_COMMIT_DISCRIMINATOR: [u8; 8] = [52, 170, 152, 201, 179, 133, 242, 141];
pub const SB_RANDOMNESS_INIT_DISCRIMINATOR: [u8; 8] = [9, 9, 204, 33, 50, 116, 113, 15];
pub const SB_RANDOMNESS_REVEAL_DISCRIMINATOR: [u8; 8] = [197, 181, 187, 10, 30, 58, 20, 73];

/// SlotHashes sysvar. Passed through to Switchboard only; hybrid_vault NEVER reads it.
pub const SLOT_HASHES_SYSVAR_ID: Pubkey = pubkey!("SysvarS1otHashes111111111111111111111111111");
pub const WRAPPED_SOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const ADDRESS_LOOKUP_TABLE_PROGRAM_ID: Pubkey = pubkey!("AddressLookupTab1e1111111111111111111111111");

/// If a request's randomness is still unrevealed this many slots (~1 h, about Switchboard's own
/// expiry) after its commit, ANYONE may re-commit fresh randomness for the SAME request with a
/// different oracle (`recommit_randomness`). Fees are never refunded (ADR-012).
pub const REVEAL_TIMEOUT_SLOTS: u64 = 9_000;
/// Incoming entries merged per settle; larger backlogs are drained with `merge_incoming`.
pub const MAX_MERGE_PER_IX: u32 = 32;

pub const MAX_NAME_LEN: usize = 32;
pub const MAX_URI_LEN: usize = 200;
/// Metadata and collection URIs must be content-addressed (Auditor B R1-B-11): the committed leaf
/// then pins the exact JSON bytes, and the JSON pins the image by CID / Arweave tx id.
pub const ALLOWED_URI_PREFIXES: [&str; 2] = ["ipfs://", "ar://"];
/// Merkle depth cap: 2^17 = 131,072 > max collection size (100,000).
pub const MAX_MERKLE_DEPTH: usize = 17;

pub const REQUEST_KIND_CAPTURE: u8 = 0;
pub const REQUEST_KIND_REROLL: u8 = 1;
pub const NO_HANDED_IN: u32 = u32::MAX;

pub const VAULT_VERSION: u8 = 2;
/// Re-commits allowed per request after the first commit (each with a different oracle), before
/// the principal-only expire becomes possible (audit M-04 rule 4, B's H4).
pub const MAX_RECOMMITS: u8 = 3;
/// `expire_requests` (M-04 batch expire): at most this many consecutive queue heads per call.
/// Bound: Solana's 64-account-lock limit per tx (11 shared + 7 per request = 60 at K = 7, 61 with a
/// compute-budget ix). Compute is not the limit (~26k CU per head measured). More than 3 heads
/// exceed a legacy tx's 1,232 bytes, so K > 3 needs a v0 tx with an address lookup table.
pub const MAX_EXPIRE_PER_CALL: u8 = 7;
/// Accounts per request in `expire_requests`' remaining_accounts.
pub const EXPIRE_BATCH_STRIDE: usize = 7;
/// After the last re-commit's deadline, wait this long (~1 day) before anyone may expire.
pub const EXPIRE_GRACE_SLOTS: u64 = 216_000;
/// M-04 staleness filter: an oracle whose Switchboard `last_heartbeat` is older than this (seconds)
/// may be skipped by oracle selection, but only when the caller supplies its account as proof.
/// Tune on devnet against the queue's real heartbeat cadence.
pub const MAX_ORACLE_HEARTBEAT_AGE_SECS: i64 = 3_600;
/// LAZY MINT (ADR-016): every capture/re-roll request escrows the worst-case cost of minting the
/// asset it may be assigned: rent for CORE_ASSET_SPACE_BYTES + the Core create fee, x 125%. Settle
/// spends it only if the pick was never minted and refunds the rest to the user; expire refunds all.
/// `["mint_escrow", vault, seq_le]`: data-less, system-owned PDA holding one request's mint escrow;
/// it signs as the Core payer at settle and is drained to the user at settle/expire.
pub const MINT_ESCROW_SEED: &[u8] = b"mint_escrow";
pub const MINT_ESCROW_MARGIN_PCT: u64 = 125;
pub const MINT_ESCROW_LAMPORTS: u64 =
    (hybrid_launch::CORE_ASSET_RENT_LAMPORTS + hybrid_launch::CORE_CREATE_FEE_LAMPORTS) * MINT_ESCROW_MARGIN_PCT / 100;
const _: () = assert!(MINT_ESCROW_LAMPORTS == 6_338_100);
/// The settler (crank) gets NOTHING from escrow (T-HV-16); it pays only its tx fee.
pub const SETTLE_TIP_LAMPORTS: u64 = 0;
/// Pool floor (graduation-design, anti-cornering): requests need at least
/// max(MIN_POOL_FLOOR, POOL_FLOOR_BPS of N) drawable assets left. NEEDS BARTON (values).
pub const MIN_POOL_FLOOR: u64 = 5;
pub const POOL_FLOOR_BPS: u64 = 200;
