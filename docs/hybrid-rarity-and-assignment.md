# Hybrid collections: cosmetic rarity, assignment, escrow selection, re-rolls

> **CURRENT MODEL (2026-09-25; supersedes any burn / 2% / bps / token-fee / pause text below, which is kept for the
> record):** capture and re-roll each pay ONE flat SOL tier fee (0.002 / 0.005 / 0.01 SOL by ratio, cap 0.01) at request,
> to the fixed `PLATFORM_FEE_RECIPIENT`, never refunded (ADR-013). Release is free and returns exactly N tokens.
> Nothing is burned. No pause exists (ADR-015). Minting is lazy: the requester escrows the worst-case Core mint cost
> (0.0063381 SOL; actual ≈ 0.00507, the rest refunded at settle), and settle mints a never-minted pick straight to the
> user from that escrow (ADR-016, [lazy-mint-interface.md](lazy-mint-interface.md)). Expire refunds the principal +
> the escrow, never the fee. Core cost per asset is 0.0035–0.0067 SOL depending on plugins (ours ≈ 0.00509 all-in
> with plugins; lean/no plugins ≈ 0.00507 raw at the escrowed 385-byte size). Ratios: {50k … 5M}, 100 ≤ N ≤ 10,000.

Status: **ACCEPTED design (engine = `hybrid_vault`, ADR-008 accepted 2026-09-24).** Owner: Solana Program Engineer. Date: 2026-09-24, updated after the SCOPE CHANGE (ADR-009).
**Basis (decided):** Token-2022 is dropped/deferred (ADR-009, supersedes ADR-004's two-type model). The product is
SPL-404 hybrid launches only: classic SPL Token, untaxed, convertible, exact unwrap. There's no transfer fee anywhere in
the wrap/unwrap/re-roll path, so every amount below is exact. The engine-independent launch step (mint 1B, revoke
authorities, immutable LaunchConfig) is **built** as `programs/hybrid_launch` (ADR-010). The engine is **`hybrid_vault`**
(ADR-008 ACCEPTED, decided 2026-09-24 by the Solana Program Engineer because MPL-Hybrid can't meet Barton's stated
requirements; reversible if Barton objects). MPL-Hybrid is reference only. ~~Re-roll and capture fees = burn, at settle~~ **Superseded: flat SOL tier fee at
request (ADR-013); lazy mint from a per-request escrow (ADR-016).**

Inputs: BRIEF.md §Decisions (2026-09-24), [transfer-tax-vs-wrap.md](transfer-tax-vs-wrap.md), Auditor B findings B-01/B-12/B-16 and sim (`../security/auditor-b/sim/reroll_ev_output.txt`).
Decision record: [DECISIONS.md](DECISIONS.md) ADR-008. Threats: [THREAT_MODEL.md](THREAT_MODEL.md) T-HV-*.

## Summary

| Question | Recommendation |
|---|---|
| 1. Supply check | `collection_size × ratio ≤ 1,000,000,000`, checked with `checked_mul` in base units by the on-chain `initialize` (source of truth) and mirrored in the launch wizard. Undersized collections are allowed, and the copy says how much of the supply can be in NFT form at once. |
| 2. Rarity assignment | **Pre-committed trait list plus a VRF permutation.** The creator commits a Merkle root of the full trait list before launch. A VRF seed drawn at lock keys a Feistel permutation from NFT index to list position. Each NFT's metadata is bound on-chain with a Merkle proof. Anyone can recompute it. |
| 3. Escrow selection | **No configuration of MPL-Hybrid meets "no choosing, no peeking."** **Accepted (iii):** a minimal custom `hybrid_vault` program. Capture and reroll are two-step (lock payment → VRF → permissionless settle), settled in strict FIFO order over a sequenced pool. Release is instant and exact. MPL-Hybrid isn't kept in the swap path. |
| 4. Re-rolls | Re-roll = hand in your NFT, get a VRF-random *different* NFT from the pool, same two-step flow. The fee is **one flat SOL tier fee** (0.002/0.005/0.01 SOL by ratio, cap 0.01), the same as the capture fee, charged at request and never refunded (ADR-013). Release is free, so unwrap→rewrap is never a cheaper reroll. Fees are **fixed at init**. The request also escrows the worst-case mint cost (ADR-016), refunded at settle/expire except what a first mint spends. |

---

## 1. Supply check

Hard rule (BRIEF): total supply is exactly 1,000,000,000 tokens, and every NFT redeems for exactly `ratio` tokens, so at most
`floor(1B / ratio)` NFTs can exist in NFT form at once. The allowed ratios all divide 1B.

| Ratio (tokens per NFT) | Max collection size | Tokens in NFT form at max size |
|---:|---:|---:|
| 10,000 | **100,000** | 1,000,000,000 (100%) |
| 50,000 | **20,000** | 1,000,000,000 (100%) |
| 100,000 | **10,000** | 1,000,000,000 (100%) |
| 200,000 | **5,000** | 1,000,000,000 (100%) |
| 1,000,000 | **1,000** | 1,000,000,000 (100%) |

**Where it's enforced**

1. **On-chain (source of truth):** `hybrid_vault::initialize(ratio, collection_size, …)`, in base units:
   ```text
   require!(ratio ∈ {10_000, 50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000})  // whole tokens (Barton 2026-09-24)
   ratio_base  = ratio.checked_mul(10^decimals)?                             // u64, errors on overflow
   supply_base = 1_000_000_000u64.checked_mul(10^decimals)?
   require!(mint.supply == supply_base && mint.mint_authority == None && mint.freeze_authority == None)
   require!(supply_base % ratio_base == 0)                                    // always true for the allowed set; asserted anyway
   require!(collection_size >= 100)                                           // minimum collection size (Barton 2026-09-24)
   require!(collection_size.checked_mul(ratio_base)? <= supply_base)          // the supply check
   ```
   `ratio`, `collection_size`, and the mint are stored in the vault config and are **immutable**. No instruction changes
   them. A unit test enumerates every ratio at max and max+1 sizes, and a fuzz test covers overflow (e.g. `decimals = 11`
   overflows `u64` for 1B supply and must fail cleanly).
2. **Frontend (UX only, never trusted):** the launch wizard derives the max size from the ratio, clamps the input, and
   shows the "% of supply convertible" line below.
3. **Launch script/CI:** `assert_launch_ready` re-reads the mint and config on-chain before the sale opens (Auditor B
   R-03).

**When `collection_size × ratio < 1B`:** this is fine. The escrow only ever needs `ratio × NFTs outside the vault`
tokens, and the remaining tokens simply can't all be in NFT form at the same time. Copy for the launch wizard and token
page:

> **Up to {N × R} $TICKER ({pct}% of the 1B supply) can be held as NFTs at any one time.** The other {1B − N×R} $TICKER
> always stay tokens. Every NFT converts back to exactly {R} $TICKER.

(Example: R = 1,000,000, N = 500 → "Up to 500,000,000 $TICKER (50%) can be held as NFTs at any one time.")

---

## 2. How rarity is assigned

Constraint: cosmetic only. Every NFT redeems for exactly R tokens, and traits never change the payout (BRIEF rejects
rarity-weighted unwrap). The goal is that **neither the creator nor any user can steer which NFT gets which traits**,
and anyone can verify that afterwards.

### Options

| | VRF-at-reveal (traits generated from VRF) | Pre-committed list only (no VRF) | **Pre-committed list + VRF permutation (recommended)** |
|---|---|---|---|
| Creator steering | The creator can publish the trait list *after* seeing the seed and arrange it to taste, unless the list was committed first | The creator chooses index → trait, so they know where the rares are | Impossible: the list is fixed before the seed exists |
| User steering | None if the seed comes after lock | None if selection is VRF (§3) | None |
| Rarity distribution | Generated traits make counts random, so "exactly 10 legendaries" can't be promised | Exact counts | Exact counts (the permutation preserves the multiset) |
| Verifiability | Needs the generator code | Hash check | Hash check + permutation recompute |

### Recommended scheme

1. **Commit (before launch, stored immutably in the vault config):**
   - `trait_root` = Merkle root over `leaf_j = sha256("mintmark-trait-v1" ‖ j_le ‖ sha256(metadata_json_j))` for
     `j ∈ [0, N)`, where `metadata_json_j` includes the image URI (Arweave or another content-addressed store).
   - `leaf_count = N` (must equal `collection_size`), plus a published **trait census** (counts per trait/tier).
   - The full list (all N JSON files and images) is uploaded and its manifest published when the root is committed.
     This is a BAYC-style provenance commitment, but enforced on-chain.
2. **Lock:** once config is final (no further edits possible), anyone can call `request_permutation_seed`, which
   requests VRF with seed `vault ‖ trait_root ‖ "perm"`. Capture opens only after the seed is fulfilled and stored.
3. **Permutation:** `π = FeistelPermutation(key = vrf_seed, domain = N)`, a 4-round keyed Feistel network over
   `[0, 2^k)` with cycle-walking into `[0, N)`. It's a bijection, O(1) per index, and computable on-chain without storing
   an N-element shuffle. NFT index `i` gets list position `π(i)`.
4. **Binding on-chain (lazy):** the first time index `i` leaves the vault (§3 settle), the program mints the Core asset
   at PDA `["asset", vault, i_le]` with name/URI from list entry `π(i)`. The caller supplies `metadata_json` hash + URI
   + Merkle proof, and the program checks the proof against `trait_root` at leaf position `π(i)`. A bad proof fails the
   settle, and anyone can supply the correct one because the list is public. An optional permissionless
   `reveal_batch` crank can pre-mint or reveal everything for creators who want all NFTs visible on day one.
5. **Immutability after binding:** the Core collection's update authority is a `hybrid_vault` PDA. The program exposes
   **no** metadata-update or add-plugin instruction. `hybrid_vault` **creates the Core collection itself** at init
   (CPI), so no PermanentTransfer/PermanentBurn/PermanentFreeze delegate can exist (Core only allows Permanent plugins
   at creation), and no Royalties ruleset can deny transfers to or from the vault. The only plugin allowed is Royalties
   with a ruleset of `None`. No one can "upgrade" a rare later.

### How anyone verifies it (published script `scripts/verify-hybrid-provenance`, planned)

1. Download the manifest, recompute every leaf and the Merkle root, and compare it to `trait_root` stored on-chain.
   Check that the root was written in a slot *before* the VRF request slot.
2. Read the fulfilled VRF account (Switchboard/ORAO) referenced in the vault config and verify it's owned by the pinned
   VRF program and that its seed matches `vault ‖ trait_root ‖ "perm"`.
3. Recompute `π(i)` for all i, fetch each minted asset `["asset", vault, i]`, and check its URI and hash match list
   entry `π(i)`. Recount the census.

**Why this is enough, given §3:** because every NFT starts in the vault and leaves only via VRF selection, knowing
which index holds which trait gives nobody an advantage. The permutation is defense in depth. It also covers any future
flow where some NFTs are distributed non-randomly (which this design forbids; see Q-H6).

---

## 3. How the escrow picks the NFT a wrapper receives

Requirement: **no choosing, no peeking.** A wrapper must pay (irrevocably) before anything decides which NFT they get,
the decision must use randomness nobody knows at payment time, and the result must settle in a transaction the user
can't revert after seeing the outcome.

### 3.1 What MPL-Hybrid actually does (source-checked)

Repo https://github.com/metaplex-foundation/mpl-hybrid at `aacf1a53` (still latest `main` on 2026-09-24). Paths are
relative to `programs/mpl-hybrid/src/`.

- **The capturer picks the asset.** `CaptureV2Ctx.asset` is a caller-supplied `UncheckedAccount`
  (`instructions/capture_v2.rs:52-54`). The only check is that the asset belongs to the recipe's collection
  (`:177-181`). `capture.rs:42-44,156-160` is the same for V1. Every escrowed asset and its metadata is public on-chain,
  so a bot can cherry-pick any asset it likes.
- **Path flags** (`state/path.rs`, bit = enum position): `NoRerollMetadata`=bit 0, `BlockCapture`=1, `BlockRelease`=2,
  `BurnOnCapture`=3, `BurnOnRelease`=4.
- **"Reroll" is a metadata rewrite at capture, not a random pick of an asset.** If bit 0 is *unset*, capture rewrites
  the chosen asset's name/URI to `recipe.uri + idx + ".json"` with
  `idx = ((slot_hash_bytes[12..20] as u64) − unix_timestamp) × recipe.count mod (max − min) + min`
  (`capture_v2.rs:187-203`). Every input is public before execution, so an attacker program can CPI capture, inspect
  the new URI, and revert unless it's rare. Auditor B's sim estimates this is ~1000× cheaper than honest grinding, and
  when `(max − min)` divides 2^32 the hash term vanishes entirely (`reroll_ev_output.txt` Part B). The index is drawn
  *with replacement* from `[min, max)`, so the rarity census isn't preserved and index `max` is never picked (B-16).
  Release sets the asset to name "Captured", URI `…captured.json` (`release_v2.rs:211-250`).
- **Can the escrow authority gate capture?** Only incidentally:
  - The `authority` account is checked only as `if authority == recipe.authority { assert_signer }`
    (`capture_v2.rs:183-185`). With `NoRerollMetadata` set, any account can be passed, so **capture is fully
    permissionless**.
  - With `NoRerollMetadata` unset, the metadata update CPI needs either `recipe.authority`'s signature or the recipe
    PDA to be the asset's Core UpdateDelegate (`:229-237`). Without that delegate, only the authority can complete a
    capture, but the predictable URI rewrite still happens.
  - `BlockCapture` blocks everyone, including the authority (`:135-137`).
  - The authority *can* flip `path` with `update_recipe_v1` (only changes to bit 0 are frozen after the first swap,
    `update_recipe.rs:129-138`).
- **Authority powers (drain surface).** `update_recipe_v1` changes `amount`, all fees, `min/max`, `uri`, and `path` at
  any time, with no timelock. It also **unconditionally overwrites `recipe.token` and `recipe.fee_location` with
  whatever accounts are passed** (`update_recipe.rs:91-93`, not optional). A compromised authority can point the
  recipe at a worthless mint to capture every NFT, then point it back and release them against the real backing. See
  T-HY-01 and T-HV-10.
- **Upgradeability.** `MPL4o4wMzndgh8T1NVDxELQCj5UQfYTYEkabX3wNKtb` is upgradeable. Its upgrade authority on both
  devnet and mainnet is `mp14o4AQcmE5meFDxCscervMc1E4zyKEyDp3398PcwU` (read-only `getAccountInfo` of programdata
  `9RRs8kE5…`, 2026-09-24). **Audit Pending** per the README.

### 3.2 Does any MPL-Hybrid configuration meet "no choosing, no peeking"? **No.**

| Configuration | Choosing? | Peeking / revert-until-good? | Verdict |
|---|---|---|---|
| `NoRerollMetadata` set (static metadata) | **Yes**: the caller names the asset | Yes, metadata is visible | ✗ |
| Reroll on, recipe PDA as UpdateDelegate (permissionless) | The asset is chosen, but its metadata is rewritten | **Yes**: SlotHashes/timestamp/count are known, so CPI-and-revert works | ✗ |
| Reroll on, no delegate (authority must co-sign every capture) | Gated by whoever holds the authority | The rewrite is still predictable and overwrites per-NFT metadata, which breaks §2 | ✗ on its own. Only works as part of a wrapper (option i) |
| `BlockCapture` set | Nobody can capture | n/a | ✗ (no hybrid) |

MPL-Hybrid alone can't do it. Every workable option adds custom code that holds or controls value.

### 3.3 Options

All three use the same core pattern: **two-step VRF.**
- **Request tx:** the user's tokens (or NFT, for reroll) are moved into a vault PDA. The request stores the VRF
  account and seed. There's no cancel once VRF is fulfilled.
- ~~**Token fee timing (FINAL, 2026-09-24): burn at SETTLE.**~~ **Superseded by ADR-013/016 (no token fee, no burn; SOL fee at request, never refunded).** Old text: At request time the token fee is escrowed in the request
  PDA's token account (owned by the program, no withdraw path). `settle` burns it with an SPL Token `burn` (no wallet
  ever receives it). `expire` refunds it together with the locked tokens (or the handed-in NFT for a re-roll). A burn
  can't be undone, so burning at request would make a fair refund on expiry impossible.
- **Settle tx:** permissionless. It reads the fulfilled VRF, selects, and delivers to the stored recipient.

The user can't revert the outcome because selection happens in a transaction they don't need to sign and can't
pre-empt.

**The subtle part: pool manipulation between reveal and settle.** VRF output becomes public (Switchboard: the
requester can even fetch it off-chain before revealing; ORAO: at fulfilment) *before* settle executes. If the pool can
change in between (e.g. the attacker releases or deposits NFTs to shift which asset index `r mod len` lands on), the
attacker steers the pick. The fix is a **sequenced pool with FIFO settlement**:

- Every deposit (release, reroll hand-in) and every request gets a monotonically increasing `seq`.
- Requests settle **strictly in `seq` order**.
- Before settling request `s`, the program merges into the selectable pool only deposits with `seq < s`. Deposits made
  after the request (and the reroller's own hand-in) wait in an incoming FIFO.
- So the pool that request `s` selects from is fully determined when the request is made, before its randomness exists.
  Knowing `r` early is useless.
- Selection: `idx = uniform_below(r, pool_len)` with rejection sampling (no modulo bias, R-01.4), then swap-remove from
  a zero-copy `u32` index array.
- **Head-of-line liveness:** each request has `deadline_slot`. If its VRF account is still *unfulfilled* after the
  deadline, anyone can `expire` it: principal and the escrowed token fee are refunded in full (the SOL cost fee minus VRF cost is
  still open, N7), and the queue advances.
  A *fulfilled* request can only be settled. Settle is permissionless and our crank runs it, so a requester who
  withholds a Switchboard reveal gains nothing (anyone can reveal).
- **Reservations:** at request time, require `pool_len + incoming_len − pending_requests ≥ 1` (for reroll, not counting
  the caller's own hand-in). So a settle can never find an empty pool.

#### (i) Thin wrapper program that holds MPL-Hybrid's authority as a PDA

The wrapper PDA is both Core collection update authority and `recipe.authority`. Keep `NoRerollMetadata` set (no
predictable rewrite) and `BlockCapture` set. Settle does, atomically:
1. CPI `update_recipe` to clear `BlockCapture`.
2. CPI `capture_v2(asset = VRF-selected, owner = system-owned buyer PDA holding the locked tokens and SOL)`.
3. CPI `update_recipe` to set `BlockCapture` again.
4. Core transfer from the buyer PDA to the user.

Release either stays open (but then released NFTs enter MPL-Hybrid's escrow without our `seq`, so a permissionless
`sync` is needed) or is also wrapped via a `BlockRelease` toggle.
- *Audit surface:* the wrapper (~same size as iii) **plus** MPL-Hybrid (unaudited, upgradeable by a Metaplex key, and
  holding our backing tokens) **plus** a path-toggle trick that depends on undocumented upstream behaviour (e.g. that
  `update_recipe` keeps allowing bit 1 flips and keeps rewriting `token`/`fee_location` harmlessly).
- *MEV:* same as (iii) if the wrapper is correct.
- *Escrow drain:* closes the authority-drain hole (the authority is a PDA with no generic update path), but backing
  sits in a program we don't control. There are also 2 × 0.005 SOL Metaplex protocol fees per cycle and CPI depth
  user→wrapper→MPL-Hybrid→Core.
- *Verdict:* feasible, but the worst of both worlds.

#### (ii) Unrevealed metadata in escrow (every pooled NFT looks identical, traits bound by VRF on exit)

Traits attach to the *slot* at capture, and the trait returns to the pool on release. This is MPL-Hybrid's native
model, but done with VRF and sampling *without* replacement.
- It makes *choosing* pointless, but it **doesn't remove the need for (i)/(iii)**: binding must still be two-step VRF
  with FIFO settlement, or it's revert-until-rare again.
- It changes the product: an NFT's look changes every time it's unwrapped and rewrapped, so "#142 with laser eyes"
  isn't a stable identity. Barton's reroll wording ("swap an NFT for a different random one from the pool") implies
  stable NFTs with fixed traits.
- *Verdict:* not needed. Under (iii), pooled NFTs *can* be fully visible because nobody can target one.

#### (iii) Minimal custom swap program replacing MPL-Hybrid: `hybrid_vault` **ACCEPTED (2026-09-24)**

Instructions (all tokens are classic SPL Token, program pinned):

| Instruction | Who | Effect |
|---|---|---|
| `initialize` | creator (once) | §1 checks (or read them from the immutable `hybrid_launch::LaunchConfig`), store immutable `ratio`, `collection_size`, `mint`, `trait_root`, `leaf_count`, fee params (capped, immutable), token fee destination = burn. CPI-creates the Core collection with the program PDA as update authority and no Permanent delegates |
| `request_permutation_seed` / `store_seed` | anyone | §2 lock step. Capture stays closed until the seed is stored |
| `request_capture` | user | transfer exactly `ratio` tokens to the vault ATA, pay the flat SOL tier fee to `PLATFORM_FEE_RECIPIENT` (never refunded), move `MINT_ESCROW_LAMPORTS` into the `["mint_escrow", vault, seq]` PDA (ADR-016); create `Request{seq, kind=Capture, recipient, vrf_account, deadline}` |
| `request_reroll(asset)` | NFT owner | transfer the NFT into the vault (goes to incoming with this request's `seq`, so it's excluded from its own draw), fees as above |
| `settle` | anyone | head of queue only. Merge incoming `< seq`, verify pinned VRF account fulfilled, select, mint-on-first-exit (Merkle proof, §2) or transfer the existing asset to `recipient`, **burn the escrowed token fee**, close the request |
| `release(asset)` | NFT owner | single tx, deterministic: NFT → vault (incoming, new `seq`), exactly `ratio` tokens → owner. No randomness needed |
| `expire` | anyone | only if the VRF is unfulfilled after `deadline_slot`. Refund the locked tokens (or NFT) **and the escrowed token fee** |
| ~~`propose_fees` / `execute_fees`~~ | n/a | **Removed by ADR-009:** fees are fixed at init. The only admin power considered is `pause_new_requests` (multisig + timelock, never blocks `release`/`settle`/`expire`), see admin-multisig-timelock.md |

Invariants, asserted in tests and fuzzing after every instruction:
- `vault_token_balance ≥ ratio × nfts_outside_vault`. This is equality unless someone donates tokens.
- `pool_len + incoming_len + nfts_outside_vault + pending_reroll_handins == collection_size`, counting unminted
  indices as in the pool.
- `ratio`, `mint`, and `trait_root` never change.
- There is **no** instruction that moves vault tokens except `release`, and none that moves NFTs except `settle`.

- *Audit surface:* one small program that we own, pin, and audit. There's no dependency on MPL-Hybrid's pending audit
  or upgrade key. It's similar in size to (i)'s wrapper *without* the MPL-Hybrid surface under it.
- *MEV / front-running:*
  - Seeing a rare in the pool gives no edge. Every request draws uniformly from a pool fixed at request time, so racing
    to "grab" it is impossible.
  - Revert-if-bad is impossible, because settle is a separate permissionless tx and there's no cancel after
    fulfilment.
  - Pool shifting is impossible, because of FIFO + `seq` merge.
  - Residual: grinding at honest odds, priced by fees (§4).
- *Escrow drain:* no admin withdraw, immutable ratio/mint/fees/fee destination (burn). Upgrade
  authority is a multisig, then final after audit.
- *Cost:*
  - VRF per capture/reroll: ORAO ~0.001 SOL; Switchboard pays randomness-account rent (reclaimable) plus the oracle fee.
  - Lazy-minted Core asset: ~0.0016 SOL rent for ~180 bytes (`solana rent 180`), paid by the first capturer.
  - Pool index array at 100,000 NFTs: 400 KB, ~2.03 SOL rent, paid by the creator. It's created top-level (not via
    CPI) so the 10 KB CPI realloc limit doesn't apply.
- *UX:* capture and reroll take two transactions and a few seconds ("Drawing your NFT…"). The frontend or our crank
  submits the settle. Release is one transaction.

**Recommendation: (iii).** It's the only option where every property is enforced by code we control and audit. (i)
keeps an unaudited, externally upgradeable program under our backing and adds a fragile toggle trick. (ii) doesn't solve
selection by itself and changes the product. This **partially supersedes ADR-003** ("no custom 404 program"):
Barton's no-choosing/no-peeking requirement can't be met without custom code that controls NFT selection.

**Is MPL-Hybrid kept?** Not in the swap path. Metaplex **Core** stays the NFT standard. MPL-Hybrid remains a reference
design (and its SDK's account shapes are useful). A future "pick-your-NFT" mode *could* use MPL-Hybrid with
`NoRerollMetadata` if Barton ever wants choosing allowed, but that's explicitly not recommended.

---

## 4. Re-rolls

**Meaning under cosmetic rarity:** a holder hands in one NFT and receives a VRF-random *different* NFT from the pool.
There's no token movement except fees, and every NFT is still worth exactly `ratio` on unwrap.

**Flow:** `request_reroll(asset)` moves the NFT into the vault (incoming FIFO, tagged with this request's `seq`) and
takes the fees. `settle` (FIFO) selects from pool entries with `seq < s`, so **the handed-in NFT can't come back in
the same reroll**. The handed-in NFT becomes selectable for later requests.

### Fee denomination (SUPERSEDED: Barton chose a flat SOL tier fee, ADR-013; analysis kept for the record)

| | Token fee (% of ratio) | SOL fee (fixed) |
|---|---|---|
| Scales with NFT value | **Yes.** The NFT floor = R × token price, so a fee of `f × R` tokens stays the same fraction of floor at any price | No. A 0.02 SOL fee is huge at launch and negligible if the token 100×'s, which makes farming cheap exactly when rares are valuable |
| Covers VRF/rent costs (SOL) | No | Yes |
| User clarity | "Re-roll costs 2% of an NFT ({f×R} $TICKER)" | "0.02 SOL" (the design mock uses 0.01/0.02 SOL examples) |
| Supply impact | Burned (ADR-009): supply only decreases; backing unaffected (§4 burn safety) | None |

**Recommendation:** a **token fee `reroll_fee_bps` of R** (e.g. 200 bps = 2%, hard cap 1,000 bps), plus a **small SOL
fee** set to cover VRF and worst-case asset rent (~0.003 SOL, capped). Sizing rule: honest grinding for a trait with
pool frequency `p` and market premium `m × floor` has break-even `f ≥ p × m`. For example, a legendary at 0.1% of the
pool trading at 20× floor is break-even at f = 2%. The protocol never promises premiums, so treat this as a
disclosure-backed default, not a guarantee. Auditor B's sim shows why SOL-only fees fail when premiums rise (legendary
grinding goes profitable once the premium exceeds ~50 SOL at a 0.05 SOL cycle cost).

**Fee destination: BURN (decided, ADR-009).** Barton chose burn as the default on 2026-09-24. This **supersedes my
earlier recommendation** (creator/platform split, "no burn"). Burning means no program or person ever custodies a fee
pile (Stonk.fun lesson 2). Copy: "Fixed at 1,000,000,000 at launch. No one can mint more; re-roll burns can only
reduce it." Short form: "fixed 1B at launch; re-roll fees are burned". The SOL cost fee can't be burned; it pays the
VRF and rent and nothing else.

**Burn safety (why burning can't make the vault insolvent):**
1. Supply starts at exactly `S0 = 1B × 10^d` and can only go down: the mint authority is `None` (✅ `hybrid_launch`).
2. Backing is owed only for NFTs in circulation: `required = ratio × nfts_outside_vault`, held in the vault ATA.
3. The token fee is paid **from the user's own balance, on top of** the ratio. A re-roll moves no backing at all (NFT
   in, NFT out). A capture moves exactly `ratio` into the vault and burns a separate `capture_fee_amount`. Nothing
   ever burns from the vault, and no instruction lets anyone name the vault as the burn source.
4. So `vault ≥ ratio × nfts_outside_vault` is preserved by every instruction, and `release` always pays exactly
   `ratio`. Exact unwrap is unaffected.
5. Accounting becomes `circulating + vault + Σburned == S0`. `max_tokens_in_nft_form = N × R` stays a design cap; once
   burns push supply below `N × R`, not every NFT can be in circulation at once. That's harmless: NFTs still in the
   pool simply stay there, and every circulating NFT is fully backed. The copy "Up to {N×R} can be held as NFTs"
   needs "at launch" or a live figure (QA Q9).
6. **Engine caveat:** with `hybrid_vault` the burn is an inline `token::burn` CPI. MPL-Hybrid has no "burn the fee"
   option: fees go to `recipe.fee_location`, and its `BurnOnCapture`/`BurnOnRelease` paths burn the **backing**, not
   the fee, so they're forbidden. Under MPL-Hybrid the burn needs `fee_location` = a program PDA whose only
   instruction is a permissionless `burn_all` crank (transient custody, no withdraw path), and `update_recipe` can
   still overwrite `fee_location` (T-HY-01).

### Abuse cases

| Case | Handling |
|---|---|
| **Reroll loops to farm rares** | Priced by the fee rule above. Per-wallet cooldowns are sybil-able, so we don't rely on them. The SOL part covers VRF so loops never cost the protocol. Publish the trait census and the current pool census so odds are honest |
| **Unwrap→rewrap as a cheaper reroll** | `release` + `request_capture` ≡ reroll. Enforce `capture_fee ≥ reroll_fee` on-chain (both token-bps). Otherwise rational users bypass rerolls. Release has no token fee, which keeps "exactly R back" simple. SOL network costs apply |
| **Reroll into the same NFT** | Impossible: the hand-in is tagged `seq = s` and merged only for requests `> s` |
| **Pool empty / all NFTs out** | Reservations: `request_*` fails unless `pool_len + incoming_len − pending ≥ 1` (excluding the caller's hand-in). UI shows "No NFTs available to draw". Rerolling when you're the only NFT in circulation and the pool is empty is rejected |
| **Pool manipulation after VRF is known** | FIFO settlement + `seq`-bounded merge (§3.3) |
| **Revert after seeing the result** | Settle is separate and permissionless, with no cancel after fulfilment |
| **VRF never fulfils** | `expire` after `deadline_slot` returns the NFT or tokens. The fee is refunded minus VRF cost |
| **Escrow authority raises the fee** | Impossible: fees are fixed at init within hard-coded caps (≤ 10% of R token fee, ≤ 0.05 SOL) and no instruction changes them (ADR-009). There's no fee on release |
| **Fee destination swapped to an attacker** | The recipient is the code constant `PLATFORM_FEE_RECIPIENT`, checked as an account constraint (ADR-013) |
| **Queue flooding (DoS on head of line)** | Each request locks R tokens (or an NFT) plus fees, so flooding needs capital and pays fees. Settle is cheap and cranked. `expire` handles stuck heads |
| **Dust/partial fills** | Amounts are exact: `ratio` tokens, never partial |

---

## Open questions (new, for Barton)

- ~~**Q-H1:** Accept replacing MPL-Hybrid with a custom `hybrid_vault`?~~ **ACCEPTED (2026-09-24):** `hybrid_vault`,
  decided by the Solana Program Engineer (Barton's requirements rule out MPL-Hybrid); reversible if Barton objects.
  It's in audit scope.
- **Q-H2:** Re-roll and capture fees as a **% of the ratio in tokens** plus a small SOL cost fee (recommended), instead
  of the SOL-only fees in the current mock (0.01/0.02 SOL)? Default 2%?
- ~~**Q-H3:** Fee destination~~ **Resolved (ADR-009): BURN**, Barton's decision; supersedes my "no burn"
  recommendation. New follow-up: is the **capture** fee burned too (engineering default: yes, same path)?
- **Q-H4:** OK to require `capture fee ≥ reroll fee` (i.e. wrapping has a small fee) and keep unwrap token-fee-free?
- **Q-H5:** Lazy minting (first capturer pays ~0.0016 SOL rent per NFT) vs creator pre-mints everything (≈1.6 SOL per
  1,000 NFTs; ≈156 SOL at 100,000)?
- **Q-H6:** Forbid creator/team NFT allocations outside the pool (every NFT starts in the vault and leaves only via VRF)?
  Recommended.
- **Q-H7:** Show full pool contents and traits publicly (fine under this design and honest), or show only the census?
- **Q-H8:** Two-transaction capture/reroll with a few seconds of "drawing" delay: acceptable UX?
- ~~**Q-H9:** Is the 100,000-NFT maximum acceptable?~~ **Resolved by Barton's ratio set (2026-09-24):** ratios 10k,
  50k, 100k, 200k, 500k, 1M, 2.5M, 5M; max = 1B / ratio (100,000 … 200); minimum 100 for all.

## References

- MPL-Hybrid source @ aacf1a53: https://github.com/metaplex-foundation/mpl-hybrid (files cited inline). FAQ:
  https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/faq. Protocol fees:
  https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/protocol-fees
- Metaplex Core plugins (Permanent plugins only at creation, force-approve; Royalties rulesets can reject transfers):
  https://www.metaplex.com/docs/smart-contracts/core/plugins
- Switchboard On-Demand randomness: https://docs.switchboard.xyz/docs-by-chain/solana-svm/randomness
- ORAO VRF / Callback VRF: https://github.com/orao-network/solana-vrf (classic `VRFzZoJdhFWL8rkvu87LpKM3RbcVezpMEc6X5GVDr7y`,
  callback `VRFCBePmGTpZ234BhbzNNzmyg39Rgdd6VgdfhHwKypU`)
- Auditor B: `../security/auditor-b/threat-model.md` (B-01, B-12, B-16), `design-requirements.md` (R-01, R-12),
  `sim/reroll_ev_output.txt`
- Format-preserving permutation (Feistel + cycle walking): Black & Rogaway, "Ciphers with Arbitrary Finite Domains"
  (CT-RSA 2002)
