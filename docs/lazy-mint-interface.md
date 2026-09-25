# hybrid_vault lazy-mint interface (for QA)

Status: **interface frozen for QA** (Barton decided lazy minting at 5:13 PM MT on 2026-09-25). The implementation
is in the audit-baseline-r1 commit (tables below are generated from the built IDL, `target/idl/hybrid_vault.json`) on `wip/hybrid-vault`; if the code and this doc ever differ, that's a bug, so report it.
Localnet/LiteSVM only.

## What changed vs batch pre-mint

- No pre-mint: `mint_assets` (the crank) is **removed**, and so are the graduation mint fund and the launch-time affordability rule.
  The 100 minimum and 10,000 cap on N stay.
- `open_vault` gate: graduation verified AND the vault's collection exists (created by `init_vault`, update
  authority = vault_authority PDA). No `minted_count == N` requirement.
- Asset `i` is minted by **settle** the first time VRF picks it, straight to the user, paid from the mint escrow
  the requester deposited in the per-request mint-escrow PDA. A settler (crank) pays only its tx fee, never mint cost, and gets
  **no tip** (T-HV-16).
- The pick is uniform over **all indices the vault holds**: never-minted plus minted-and-returned. Minting is
  lazy, so assignment isn't biased by it.
- Minted bitmap: each index is minted at most once. `minted_count <= N` always.
- Release (`unwrap`) is free and unchanged: no mint, no burn. A re-roll hand-in goes back to the vault as-is
  (no re-mint, no burn).

## Constants (hybrid_launch `needs_barton.rs` / hybrid_vault `constants.rs`)

| const | value | meaning |
|---|---|---|
| `CORE_ASSET_SPACE_BYTES` | 385 | worst-case asset size we escrow for. Our assets have no plugins: 83 + name (<= 5, `#9999`) + URI (<= `MAX_URI_LEN` = 200) = 288 bytes (+8 with seq), so 385 has headroom |
| `CORE_ASSET_RENT_LAMPORTS` | 3_570_480 | rent for 385 bytes = (128 + 385) x 6_960 |
| `CORE_CREATE_FEE_LAMPORTS` | 1_500_000 | Metaplex Core create protocol fee (0.0015 SOL) |
| `MINT_ESCROW_MARGIN_PCT` | 125 | margin |
| `MINT_ESCROW_LAMPORTS` | 6_338_100 | (3_570_480 + 1_500_000) x 125 / 100, escrowed per capture/re-roll request |
| `SETTLE_TIP_LAMPORTS` | 0 | the settler gets nothing from escrow |

`request_*` also fails closed (`MintCostConstantStale`, vault error) if live `Rent::minimum_balance(385) +
CORE_CREATE_FEE_LAMPORTS > MINT_ESCROW_LAMPORTS`, so an underfunded request can't be created.

## Accounts

### Pool account (raw, zero-copy, owner = hybrid_vault), layout v2, discriminator `b"hvpool02"`

The creator creates it with `system_program::create_account` (space = `pool_account_size(N)`, owner = hybrid_vault)
in the same tx as, and immediately before, `init_vault`, which initialises it. Creator-funded.

```
[0..8)    "hvpool02"
[8..40)   vault pubkey
[40..44)  capacity N (u32)
[44..48)  pool_len (u32)             = N after init (every index drawable)
[48..52)  incoming_head (u32)
[52..56)  incoming_len (u32)
[56..64)  reserved
[64 .. 64+4N)         pool slots u32: slot s holds (index + 1); 0 means "index s" (lazy Fisher-Yates, O(1) init)
[64+4N .. 64+16N)     incoming ring (u32 index, u64 tag)
[64+16N .. +ceil(N/8)) minted bitmap: bit i (byte i/8, bit i%8, LSB first) = asset i exists
```
`pool_account_size(N) = 64 + 16N + ceil(N/8)`. For N = 10,000 that's 161,314 bytes (rent ~1.1236 SOL, paid by the creator).
The bitmap is 1,250 bytes at 10k. The available-index array is u32 (40 KB at 10k); packing it as u16 later would save ~0.28 SOL of rent.

### Request PDA `["request", vault, seq: u64 LE]` (Anchor account `Request`, with one field APPENDED)

The existing fields are unchanged, plus `mint_escrow_lamports: u64` (= `MINT_ESCROW_LAMPORTS` at request time, for
display and audit). The Request holds only its own rent.

### Mint-escrow PDA `["mint_escrow", vault, seq: u64 LE]` (NEW; system-owned, no data)

It holds exactly `MINT_ESCROW_LAMPORTS` from request until settle/expire. Why a separate PDA: Core's create needs a
system-owned, data-less payer, and hybrid_vault can't debit a program-owned account inside the same ix that CPIs
with it (UnbalancedInstruction). At settle this PDA is the Core **payer** (signed with its seeds). Whatever is left
after the mint (or all of it, if the pick was already minted) goes to the user in the same ix, so the account ends
at 0 lamports and is garbage-collected. Expire refunds it in full.

## Instructions (Anchor, discriminator = sha256("global:<name>")[..8])

Seeds are all under hybrid_vault's program id. `seq` / `next_seq` / `request.seq` seeds are u64 little-endian;
`index` is u32 LE. `vault.launch_config`, `vault.next_seq` and `request.seq` are read from the already-loaded account.
Extra constraints (not in the IDL) are listed under each table.

### 1. `request_capture()`: no args

| # | account | signer | writable | constraint / PDA seeds |
|---|---|---|---|---|
| 0 | `user` | ✔ | ✔ |  |
| 1 | `vault` |  | ✔ | PDA ["vault", vault.launch_config] |
| 2 | `launch_config` |  |  |  |
| 3 | `pool` |  |  |  |
| 4 | `mint` |  |  |  |
| 5 | `user_token` |  | ✔ |  |
| 6 | `vault_tokens` |  | ✔ | PDA ["vault_tokens", vault] |
| 7 | `fee_recipient` |  | ✔ | address 7J3AajxfajAMgGmmwzfZeYGNieRtHRCuEgNTfd4gTjDN |
| 8 | `request` |  | ✔ | PDA ["request", vault, vault.next_seq] |
| 9 | `mint_escrow` |  | ✔ | PDA ["mint_escrow", vault, vault.next_seq] |
| 10 | `rand_lock` |  | ✔ | PDA ["rand_lock", randomness] |
| 11 | `randomness` |  | ✔ |  |
| 12 | `randomness_authority` |  |  | PDA ["randomness_authority", vault] |
| 13 | `sb_queue` |  |  |  |
| 14 | `sb_oracle` |  | ✔ |  |
| 15 | `slot_hashes` |  |  | address SysvarS1otHashes111111111111111111111111111 |
| 16 | `switchboard_program` |  |  | address Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2 |
| 17 | `token_program` |  |  | address TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA |
| 18 | `system_program` |  |  | address 11111111111111111111111111111111 |
| 19.. | stale-oracle proofs (remaining_accounts) |  |  | optional: Switchboard oracle accounts proving a queue candidate stale (M-04) |

Extra constraints: launch_config == vault.launch_config (typed, fail-closed on version != 3); pool == vault.pool,
owner hybrid_vault; mint == launch_config.mint; user_token has token::mint = mint and token::authority = user, and
!= vault_tokens; randomness is Switchboard-owned with authority = randomness_authority; sb_queue == vault.sb_queue;
sb_oracle == the program-selected oracle with a fresh heartbeat.

Effects, in order: pool floor check (`NoAssetAvailable`, strict: free > floor) and the live mint-cost check
(`MintCostConstantStale`); then N tokens user -> vault_tokens; then the tier fee `min(stored, tier, MAX)` user ->
fee_recipient (never refunded); then `MINT_ESCROW_LAMPORTS` user -> mint_escrow (system transfer); then the
Switchboard commit. The user signs and pays for everything; the request can't be created underfunded.

### 2. `request_reroll(index: u32)`

| # | account | signer | writable | constraint / PDA seeds |
|---|---|---|---|---|
| 0 | `user` | ✔ | ✔ |  |
| 1 | `vault` |  | ✔ | PDA ["vault", vault.launch_config] |
| 2 | `launch_config` |  |  |  |
| 3 | `pool` |  |  |  |
| 4 | `vault_tokens` |  |  | PDA ["vault_tokens", vault] |
| 5 | `fee_recipient` |  | ✔ | address 7J3AajxfajAMgGmmwzfZeYGNieRtHRCuEgNTfd4gTjDN |
| 6 | `vault_authority` |  |  | PDA ["vault_authority", vault] |
| 7 | `asset` |  | ✔ | PDA ["asset", vault, arg:index] |
| 8 | `collection` |  | ✔ |  |
| 9 | `mpl_core_program` |  |  | address CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d |
| 10 | `request` |  | ✔ | PDA ["request", vault, vault.next_seq] |
| 11 | `mint_escrow` |  | ✔ | PDA ["mint_escrow", vault, vault.next_seq] |
| 12 | `rand_lock` |  | ✔ | PDA ["rand_lock", randomness] |
| 13 | `randomness` |  | ✔ |  |
| 14 | `randomness_authority` |  |  | PDA ["randomness_authority", vault] |
| 15 | `sb_queue` |  |  |  |
| 16 | `sb_oracle` |  | ✔ |  |
| 17 | `slot_hashes` |  |  | address SysvarS1otHashes111111111111111111111111111 |
| 18 | `switchboard_program` |  |  | address Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2 |
| 19 | `system_program` |  |  | address 11111111111111111111111111111111 |
| 20.. | stale-oracle proofs (remaining_accounts) |  |  | optional (M-04) |

Extra constraints: the user owns asset(index), which must already be minted (it's the user's); vault_tokens is used
only for the solvency check (no tokens move).
Effects: pool floor check, then the hand-in (asset -> vault_authority, held and NOT drawable for this request),
then the tier fee (== the capture fee), then `MINT_ESCROW_LAMPORTS` -> mint_escrow, then the commit.

### 3. `settle_capture(mint: Option<MintArgs>)` / `settle_reroll(mint: Option<MintArgs>)` (identical account lists)

```rust
pub struct MintArgs { pub leaf: LeafPreimage, pub proof: Vec<[u8; 32]> }
pub struct LeafPreimage { pub trait_values: [u16; 8], pub salt: [u8; 32], pub image_sha256: [u8; 32],
                          pub json_sha256: [u8; 32], pub uri: String }
```

| # | account | signer | writable | constraint / PDA seeds |
|---|---|---|---|---|
| 0 | `settler` | ✔ | ✔ |  |
| 1 | `vault` |  | ✔ | PDA ["vault", vault.launch_config] |
| 2 | `launch_config` |  |  |  |
| 3 | `pool` |  | ✔ |  |
| 4 | `request` |  | ✔ | PDA ["request", vault, request.seq] |
| 5 | `rand_lock` |  | ✔ | PDA ["rand_lock", randomness] |
| 6 | `randomness` |  |  |  |
| 7 | `vault_authority` |  |  | PDA ["vault_authority", vault] |
| 8 | `mint_escrow` |  | ✔ | PDA ["mint_escrow", vault, request.seq] |
| 9 | `vault_tokens` |  |  | PDA ["vault_tokens", vault] |
| 10 | `asset` |  | ✔ |  |
| 11 | `collection` |  | ✔ |  |
| 12 | `user` |  | ✔ |  |
| 13 | `mpl_core_program` |  |  | address CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d |
| 14 | `system_program` |  |  | address 11111111111111111111111111111111 |

Extra constraints: the settler can be anyone and pays only the tx fee; launch_config gets a raw exit_view read (M-41);
pool == vault.pool; request.seq == vault.next_settle_seq and it's revealed; request and rand_lock close -> user;
randomness == request.randomness; vault_authority is **read-only** (it signs Core create/transfer as authority but
pays nothing); asset == asset(picked) = PDA `["asset", vault, picked: u32 LE]`; collection == vault.collection;
user == request.user.

Selection: merge `incoming` entries with tag <= seq, then `pos = uniform_below(request_randomness(value, vault,
seq), pool_len)`, `picked = pool[pos]` (decoded), swap-remove. For a re-roll the handed-in index is pushed back
AFTER the pick. Off-chain: replay the same code (`hybrid_vault::selection`, `PoolView`) to know `picked` before
building the tx.

- **picked already minted** (bitmap bit set): Core transfer vault_authority -> user. `mint` is ignored (pass None).
  The whole escrow is refunded to the user.
- **picked never minted**: `mint` is required (`MintArgsMissing`). The leaf must verify against `vault.trait_root`,
  which is committed before the curve (leaf v2 binds launch_config, index, trait hash and art hash; the URI must be
  content-addressed and <= 200). Then the program:
  1. requires mint_escrow >= live rent(385) + Core fee (`MintEscrowShort`),
  2. drains any pre-funded lamports at asset(picked) into mint_escrow (T-GRAD-01),
  3. Core `CreateV2`: asset = asset(picked) (PDA signer), collection = vault.collection (set + verified), authority =
     vault_authority, **payer = mint_escrow (PDA signer)**, **owner = user**, name `#<picked>`, uri = leaf.uri
     (immutable), no plugins,
  4. asserts the asset is in the collection and owned by the user, sets the bitmap bit (`AlreadyMinted` if it was
     already set), `minted_count += 1`,
  5. transfers everything left in mint_escrow to the user (escrow - rent - Core fee + any drained pre-fund).
- The request and rand_lock close to the user. Settler tip: 0 (`SETTLE_TIP_LAMPORTS`). Settle can't strand funds:
  mint_escrow always ends at 0.

Tx size: with N = 10,000 (proof depth 14) and a ~70-char URI, a legacy tx is ~1.2 KB, right at the 1,232-byte
limit. Clients SHOULD send settle-with-mint as a v0 tx with an address lookup table (static accounts: vault,
launch_config, pool, vault_authority, vault_tokens, collection, Core, System).

### 4. `expire_request()`: no args

| # | account | signer | writable | constraint / PDA seeds |
|---|---|---|---|---|
| 0 | `caller` | ✔ | ✔ |  |
| 1 | `vault` |  | ✔ | PDA ["vault", vault.launch_config] |
| 2 | `launch_config` |  |  |  |
| 3 | `pool` |  |  |  |
| 4 | `request` |  | ✔ | PDA ["request", vault, request.seq] |
| 5 | `rand_lock` |  | ✔ | PDA ["rand_lock", randomness] |
| 6 | `randomness` |  |  |  |
| 7 | `user` |  | ✔ |  |
| 8 | `user_token` |  | ✔ |  |
| 9 | `mint` |  |  |  |
| 10 | `vault_authority` |  |  | PDA ["vault_authority", vault] |
| 11 | `vault_tokens` |  | ✔ | PDA ["vault_tokens", vault] |
| 12 | `mint_escrow` |  | ✔ | PDA ["mint_escrow", vault, request.seq] |
| 13 | `asset` |  | ✔ |  |
| 14 | `collection` |  | ✔ |  |
| 15 | `mpl_core_program` |  |  | address CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d |
| 16 | `token_program` |  |  | address TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA |
| 17 | `system_program` |  |  | address 11111111111111111111111111111111 |

Extra constraints: request is the head of the FIFO, never revealed, commits > MAX_RECOMMITS, past deadline + grace;
randomness == request.randomness and confirmed unrevealed; user == request.user; user_token has token::mint = mint
and token::authority = user; mint == vault.mint; asset = asset(handed_in) for a re-roll (unused for a capture);
collection == vault.collection.

Refunds the principal (N tokens, or the handed-in NFT), the **full** mint escrow (mint_escrow -> user) and rents (the
request and rand_lock close to the user). The tier fee is **never** refunded. To batch, put several `expire_request`
ixs in one tx. There is no K-per-call variant yet.

## Errors appended (hybrid_vault; append-only, never renumbered)

`MintArgsMissing` 6055, `MintEscrowShort` 6056, `AlreadyMinted` 6057, `MintCostConstantStale` 6058 (after
`CollectionAboveCap` 6054). `CollectionNotFullyMinted` (6035) and `GraduationFundShortfall` (6052) became
`Reserved*` (unused, numbers kept). hybrid_launch: `GraduationUnfundable` (6015) became `ReservedGraduationUnfundable`.

## Invariants (asserted on-chain at the end of every mutating ix)

- INV-1: vault_tokens >= ratio x (assets_outside + pending_captures + pending_rerolls).
- INV-2 (lazy): pool_len + incoming_len + pending_rerolls + assets_outside == N (every index is exactly one of: drawable,
  returning, held as a re-roll hand-in, or owned by a user); pool capacity == N.
- INV-3: minted_count <= N; assets_outside + pending_rerolls <= minted_count; an index is minted only when its bitmap
  bit was clear, then set.
- INV-4: every pending request's mint_escrow PDA holds `MINT_ESCROW_LAMPORTS` until it settles or expires.

## Test coverage (tests/track-a-hybrid/vault/vault.rs, all passing)

Escrow held in a PDA at request; first pick minted to the user with the settler paying only the tx fee; bad leaf/proof
rejected; a returned asset is transferred, not re-minted; a re-roll hand-in is never burned; all N indices are drawable;
expire refunds principal + full escrow but not the fee; minted_count <= N; pre-funded asset/collection griefing.
