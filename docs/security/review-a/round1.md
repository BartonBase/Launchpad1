# Review A — Round 1 (independent) — Mintmark SPL-404 hybrid launchpad

- **Author:** Review A (independent). `docs/security/review-b/` was **not** read at any point.
- **Date:** 2026-09-24 (MT). **Environment:** local `solana-test-validator` (Agave 4.1.2) only. No mainnet/devnet
  writes, no real funds, no real keys. Read-only RPC (`getAccountInfo`/`program show`) was used to check the
  deployed MPL-Hybrid program's upgrade authority.
- **Primary target:** upstream **MPL-Hybrid** at pinned commit
  `aacf1a53c8a43bf395db7b562c02cf016ce604c5` of `github.com/metaplex-foundation/mpl-hybrid` (README: **"Audit
  Pending"**). Source read at `/workspace/launchpad/reference/mpl-hybrid-aacf1a53.../programs/mpl-hybrid/src/`;
  all `file:line` refs are that tree.
- **Also covered:** the team's proposed safeguards (blind assignment, VRF re-roll with burned fee, multisig +
  timelock) from `docs/hybrid-rarity-and-assignment.md`, `docs/DECISIONS.md`, `docs/THREAT_MODEL.md`; and a
  fresh threat model for the classic-SPL 1B token + bonding-curve launch.
- **Docs read first:** BRIEF.md (incl. SCOPE CHANGE), DECISIONS.md, transfer-tax-vs-wrap.md,
  stonkfun-lessons.md, hybrid-rarity-and-assignment.md, ARCHITECTURE.md, THREAT_MODEL.md.
- **Scope note (per SCOPE CHANGE 2026-09-24):** Token-2022 / Rewards / transfer-tax / holder-lottery are
  SHELVED. This round is 100% SPL-404 hybrid: classic SPL Token, 1B fixed supply, mint+freeze revoked,
  token⇄NFT convert with exact unwrap, cosmetic rarity, blind assignment, VRF re-roll, burned re-roll fee,
  multisig+timelock on any mutable setting. The obsolete Token-2022 sections of
  `docs/security/review-a/00-threat-model.md` are superseded by this file.

## Severity key
**Critical** = an unprivileged attacker (or a single non-timelocked key) can drain pooled value or take an
authority, or the design is fundamentally broken. **High** = realistic loss of funds/NFTs or a reliably biased
draw. **Medium** = economic leakage, meaningful DoS, or loss needing an unusual precondition. **Low** = minor
bias/griefing/hygiene. **Info** = context, no direct exploit.

---

## Findings summary table

### A. Confirmed in code (upstream MPL-Hybrid @ aacf1a53)

| ID | Sev | Title | Location | PoC |
|---|---|---|---|---|
| A-01 | Critical | Escrow drain: `update_recipe_v1` changes `amount`/fees with **no timelock**, single signer | `update_recipe.rs:54-138` (`amount` 114-116), `capture_v2.rs:284`, `release_v2.rs:287` | **Executed (on-chain)** — PASS |
| A-02 | Critical | `update_recipe_v1` **unconditionally overwrites** `recipe.token` and `recipe.fee_location` (mint/fee-dest swap) | `update_recipe.rs:91-93` | **Executed (on-chain)** — PASS |
| A-03 | High | Capturer **cherry-picks** which NFT they get; with `NoRerollMetadata` capture is **fully permissionless** | `capture_v2.rs:52-54, 177-185`; `capture.rs:42-44,156-164` | **Executed (on-chain)** — PASS |
| A-04 | High | "Reroll" metadata index is **predictable/grindable** (SlotHashes−timestamp×count), revert-until-rare, not VRF | `capture_v2.rs:187-203`; `capture.rs:167-183` | **Executed (on-chain, PoC 13, 2026-09-25)** + model 11/11 |
| A-05 | High | Reroll index math is **biased & lossy**: `max` never selected, with-replacement, power-of-two collapse | `capture_v2.rs:200-203` (`checked_rem(max-min)+min`) | **Executed (model)** — part of A-04 |
| A-06 | High | Shared V2 escrow `["escrow", authority]` pools backing across recipes → cross-collection insolvency | `init_escrow_v2.rs:11-14`, `capture_v2.rs:44-48`, `release_v2.rs:45-49` | **Executed (model)** — PASS |
| A-07 | High | `BurnOnCapture`/`BurnOnRelease` burn backing/supply → escrow insolvency, breaks exact unwrap & fixed 1B | `capture_v2.rs:261-272`, `release_v2.rs:191-208`; `state/path.rs` | **Executed (model)** — PASS |
| A-08 | Medium | No supply/ratio/backing invariant in upstream: `init_recipe`/`update_recipe` never bind `amount` to a ratio or check escrow solvency | `init_recipe.rs:66-131`, `update_recipe.rs` | Covered by A-06/A-07 model |
| A-09 | Medium | `migrate_nft_v1` **ignores the CPI transfer result** (`let _ = ...invoke_signed`) — silent failure | `migrate_nft_v1.rs:91-92` | Static |
| A-10 | Medium | `init_recipe`/`init_escrow` use `init_if_needed` fee ATA + `create_or_allocate_account_raw`; no re-init guard on data (raw memcpy) | `init_recipe.rs:53-59, 66-79`, `init_escrow_v2.rs:25-53` | Static |
| A-11 | Low | Handlers carry `// Need to add account checks for security` / `//Need to add account checks` TODOs in shipped code | `release_v2.rs:110`, `release.rs:99`, `update_recipe.rs:58`, `update_escrow.rs:56`, `migrate_tokens_v1.rs:74` | Static |
| A-12 | Low | `authority` is a bare `AccountInfo`/optional signer; correctness depends entirely on `NoRerollMetadata` + delegate config | `capture_v2.rs:28-30,183-185` | Static |
| A-13 | Info | Deployed mainnet binary length (630,752 B) ≠ local build of pinned commit (521,696 B); upgradeable by `mp14o4AQ…` | read-only `program show` 2026-09-24 | Read-only |

### B. Design risk / not yet implemented (team's `hybrid_vault` + curve plans)

| ID | Sev | Title | Location (doc §) | Status |
|---|---|---|---|---|
| B-01 | High (until built) | Entire "no-choosing/no-peeking" convert depends on an **unwritten custom `hybrid_vault`** (ADR-008 Proposed); no code exists yet | DECISIONS ADR-008; hybrid-rarity §3.3(iii) | Design |
| B-02 | Medium | VRF integration is the real risk surface (foreign/pre-revealed randomness acct, selective reveal, modulo bias) — spec exists, unimplemented | hybrid-rarity §3.3; ARCH "Randomness" | Design |
| B-03 | Medium | FIFO/`seq` sequenced-pool anti-manipulation is subtle; correctness rests on merge/reservation invariants that are unfuzzed | hybrid-rarity §3.3 | Design |
| B-04 | Medium | Feistel-permutation + Merkle trait binding: security depends on root committed **before** VRF seed and a bug-free FPE | hybrid-rarity §2 | Design |
| B-05 | Critical (curve) | Bonding-curve venue undecided; **launch-window sniping / Jito bundles / sandwiching** unmitigated (Stonk.fun S-5) | THREAT_MODEL T-MKT-01/02; this doc §6 | Design |
| B-06 | High (curve) | Graduation/migration to DEX: who triggers, LP custody, liquidity-drain path all unspecified | THREAT_MODEL T-MKT-03; this doc §6 | Design |
| B-07 | Medium | Launch supply destination = a person by default (T-HL-11) → creator rug of the full supply pre-curve | THREAT_MODEL T-HL-11 | Design |
| B-08 | Medium | Re-roll **burn** fee (ADR-009) must burn from the user's own balance only, never escrow; unverified until built | BRIEF SCOPE CHANGE; THREAT_MODEL T-BURN-01/02 | Design |

> Note on consistency: `docs` are internally inconsistent about the re-roll fee. BRIEF SCOPE CHANGE +
> THREAT_MODEL (ADR-009) say the fee is **BURNED**; `hybrid-rarity-and-assignment.md` §4 and ADR-008 recommend
> a **creator/platform split, no burn**. This must be reconciled before implementation (see B-08).

---

## PoC status (what ran, and results)

| PoC file | Type | Ran? | Result |
|---|---|---|---|
| `poc/harness/10_escrow_drain.js` (+ `poc/10_escrow_drain.out`) | **On-chain**, local validator, pinned MPL-Hybrid build | **Yes** | **PASS.** Authority set `amount=0` instantly (no timelock), captured 2 NFTs for 0 backing, then raised `amount` and drained 3,000,000 backing tokens to 0. Confirms A-01. |
| `poc/harness/12_token_swap.js` (+ `poc/12_token_swap.out`) | **On-chain**, local validator | **Yes** | **PASS.** An `update_recipe_v1` with *every* option field `None` still replaced `recipe.token` with a worthless attacker mint; the authority then captured an escrowed NFT paying only worthless tokens and restored the real mint. Confirms A-02. |
| `poc/harness/11_cherrypick.js` (+ `poc/11_cherrypick.out`) | **On-chain**, local validator | **Yes** | **PASS.** A non-authority bot captured a *named* rare NFT out of escrow with no authority signature (`NoRerollMetadata` set). Confirms A-03. |
| `poc/01_reroll_predictability.py` (+ `.out`) | Off-chain model of `capture_v2.rs:187-203` | **Yes** (model) | **11/11 PASS.** T2 now computes guard/program agreement, committed-rare rate within a bounded budget, mean attempts and an honest-odds control (was a hard-coded True before 2026-09-25). Revert-until-rare (~100 tries for top-1%), power-of-two span collapse, `max` never selected, with-replacement duplicates/missing, zero-hash→min. Confirms A-04/A-05. |
| `poc/02_escrow_accounting_model.py` (+ `.out`) | Off-chain model of escrow accounting | **Yes** (model) | **5/5 PASS.** `B_burn_paths_violate_fixed_supply` now computes mint supply after burns (was a hard-coded True before 2026-09-25). Shared-escrow cross-collection drain (A-06), BurnOnCapture insolvency (A-07), and control: classic-SPL no-burn keeps `escrow == R×nfts_outside` over 10k random ops. |
| `poc/harness/13_reroll_predict_abort.js` (+ `poc/13_reroll_predict_abort.out`; attacker program `poc/guard-src/`, `poc/programs/reroll_guard.so` sha256 `41a5ee4f…d8ed`) | **On-chain**, local validator, pinned MPL-Hybrid build (added 2026-09-25) | **Yes** | **PASS.** A non-authority bot's program CPIs `capture_v2` in reroll mode (recipe PDA = collection UpdateDelegate), reads the new URI in the same tx and fails unless index < 10 of 100. 18 attempts: 15 reverted on-chain with tokens and NFT unchanged, 3 committed, all rare (indices 4, 4, 6) vs 10% honest odds. Confirms A-04 on-chain. |
| `poc/03_fee_model.py`, `poc/04_flat_sol_fee.py` (+ `.out`) | Off-chain economic models (fee design; 4:49 PM flat SOL fee) | **Yes** | 8/8 and 7/7 PASS. Models, not exploits. |
| `poc/superseded/poc1_reroll_randomness.py` | Earlier partial reroll draft | superseded | Folded into 01 (kept for provenance). |
| `poc/superseded/02_escrow_state_model.py` | Earlier draft, truncated/syntax-broken | superseded | Rewritten as `02_escrow_accounting_model.py`. |

Build reproduction (for auditors): upstream builds with `cargo build-sbf` (Agave 4.1.2 platform tools) after a
**local-only** patch of `ahash` 0.7.8/0.8.5 `build.rs` to drop the removed nightly `stdsimd` feature
(`[patch.crates-io]` in `poc/build-mplh/Cargo.toml`, patched crates in `poc/build-mplh/patches/`). **No
MPL-Hybrid source was modified.** Artifact `poc/programs/mpl_hybrid.so`
sha256 `9e9982251982c0c1818e2b8c47aab27ddd720b2de7c9542ac84256afe8bcc830`. MPL Core (`poc/programs/mpl_core.so`)
was dumped read-only from devnet.

---

## Confirmed-in-code findings (detail)

### A-01 — Critical — Escrow drain via untimelocked `update_recipe_v1`
**Location.** `update_recipe.rs:54-138` (the whole handler is instant, single-signer); `amount` write at
`:114-116`; capture/release honor `recipe.amount` at `capture_v2.rs:284` and `release_v2.rs:287`.
**What the code does.** The only gate is `collection_data.update_authority == authority.key()`
(`update_recipe.rs:70-73`). There is no timelock, no bound, no pending-change buffer. Every economic field
(`amount`, all token/SOL fees, `min`/`max`, `uri`, `path`) is rewritten in place immediately.
**Exploit scenario (executed).** With one signer (the collection update authority — an EOA on upstream):
1. `update_recipe_v1(amount = 0)` — instant.
2. `capture_v2` each escrowed NFT: the user (here the attacker) pays `amount = 0` and receives the NFT.
   Every NFT leaves the escrow for free.
3. `update_recipe_v1(amount = escrow_balance)` — instant.
4. `release_v2` one NFT back: the escrow pays out `amount` = the whole backing balance to the attacker.
PoC `10_escrow_drain.js` did exactly this on-chain: 3,000,000 backing → 0, attacker balance → 3,000,000.
This is the MPL-Hybrid form of the Stonk.fun "retained power to change economics after launch" failure
(stonkfun-lessons §3; S-3).
**Fix.** Do **not** expose upstream `update_recipe`/`update_escrow` to a human key. Either (a) hold the
authority in a Squads multisig behind a governor PDA that only permits bounded, timelocked changes (≥72h) and
**never** allows `amount`, `token`, or `fee_location` to change post-launch; or (b, recommended, matches
ADR-008) replace the swap path with a custom `hybrid_vault` where `ratio`, `mint`, and fee destination are
immutable at init and there is no generic update instruction. Add a monitor that alerts on any
`UpdateRecipe`/`UpdateEscrow`.

### A-02 — Critical — `update_recipe_v1` overwrites `recipe.token` and `recipe.fee_location`
**Location.** `update_recipe.rs:91-93` — `recipe.authority`, `recipe.token`, `recipe.fee_location` are
assigned **unconditionally** from the passed accounts (not `Option`-gated like the other fields).
**Exploit.** The authority points `recipe.token` at a worthless mint, captures every NFT paying worthless
tokens, then points `token` back to the real mint and releases against real backing — a second, independent
drain path (and a fee-destination hijack: `fee_location` can be swapped to an attacker ATA so all token/SOL
project fees flow to them). **Executed:** PoC `12_token_swap.js` sent an update with every option field `None`;
`recipe.token` was still replaced by a worthless mint, an escrowed NFT was captured with worthless tokens, and the
real mint was restored afterwards. Note the overwrite happens even on an "innocent" update of an unrelated field,
so any tooling that passes the wrong `token`/`fee_location` account silently re-points the recipe.
**Fix.** Same as A-01. In any custom program, `mint`/`token` and fee destinations must be immutable and stored
at init, never instruction arguments (ARCHITECTURE Principle 4; matches ADR-008 "destination fixed at init").

### A-03 — High — Caller-chosen asset + permissionless capture
**Location.** `capture_v2.rs:52-54` (`asset: UncheckedAccount`, caller-supplied); the only asset check is
collection membership at `:177-181`; the authority signer check at `:183-185` runs **only if**
`authority == recipe.authority`. With `Path::NoRerollMetadata` set (bit 0), the metadata-update branch
(`:188-238`) is skipped entirely, so any account may be passed as `authority` and **no privileged signature is
required**. Same shape in V1 (`capture.rs:42-44,156-164`).
**Exploit (executed).** Every escrowed NFT and its metadata are public. A bot reads the pool, picks the rarest
asset, and calls `capture_v2` naming that asset, signing only as the token-paying `owner`. PoC
`11_cherrypick.js` had an unprivileged bot pull a specifically-named "rare" NFT out of escrow with no
authority signature. This defeats "blind assignment" for any static-metadata MPL-Hybrid config.
**Fix.** MPL-Hybrid cannot provide no-choosing/no-peeking in any configuration (see §Safeguards). Use the
custom `hybrid_vault` two-step VRF selection where `request_capture` takes **no asset argument** and selection
happens in a separate permissionless `settle`.

### A-04 — High — Predictable, grindable "reroll" randomness (not VRF)
**Location.** `capture_v2.rs:187-203` (V1 `capture.rs:167-183`):
`seed = u64::from_le(slothashes[12..20]) .saturating_sub(unix_ts) .wrapping_mul(count);
remainder = seed.checked_rem(max-min) + min`.
**Why it's broken.** Every input (SlotHashes sysvar, `Clock`, `recipe.count`) is readable by any program in
the same transaction *before* the capture CPI executes. An attacker program CPIs capture, inspects the
resulting URI, and reverts the whole transaction unless the index is rare — free retries under a Jito bundle
(failed attempts never land). PoC 01 measures ~100 tries to force a top-1% index at honest odds, i.e. the
guard succeeds 100% of the time. Only 32 bits of hash entropy are used, and they sit in the high half of the
word (`slothashes[12..16]` are the high bytes of the slot, which are 0 today).
**Fix.** VRF only for any value-bearing selection. Never derive selection from SlotHashes/Clock/count. (Team
already plans this in `hybrid_vault`; see B-02.)

### A-05 — High — Reroll index is biased and lossy
**Location.** `capture_v2.rs:200-203`.
**Findings (all reproduced in PoC 01):** (1) range is `[min, max)` so index `max` is **never** selectable;
(2) drawn *with replacement*, so over N rerolls ~37% of indices are never assigned and others repeat — any
"fixed census / 1-of-1" rarity promise is false; (3) when `(max-min)` is a power of two the hash term vanishes
and the index is a pure function of `(unix_ts, count)`; when `count % (max-min) == 0` the index is constant
(`= min`); for the documented 0..9999 example, `index mod 16` is fully determined by `(ts, count)`.
**Fix.** VRF + rejection sampling (no modulo bias) over a permutation/without-replacement pool, as in the
`hybrid_vault` design.

### A-06 — High — Shared V2 escrow pools backing across recipes/collections
**Location.** `init_escrow_v2.rs:11-14` seeds the escrow `["escrow", authority]` (per-authority, **not**
per-collection); `capture_v2.rs:44-48` and `release_v2.rs:45-49` derive the escrow from `recipe.authority`.
The escrow token ATA is `ATA(escrow, token)` and is shared by every recipe that shares an authority and token.
**Exploit (model, PoC 02).** One authority runs recipes A and B over the same mint. Backing deposited by
captures of collection A sits in the same ATA that collection-B releases draw from; `release_v2` only checks
the escrow ATA's owner and mint (`utils.rs:34-46`), never that the backing "belongs" to the releasing
collection. B releases drain A's backing, leaving A's NFTs unbacked. (V1 escrows are per-collection, so this is
a V2-specific regression.)
**Fix.** One dedicated authority (and therefore one escrow) per collection, asserted by the launch script; or
a custom vault that accounts backing per collection. Add the invariant `escrow_balance ≥ R × NFTs_outside` and
test it.

### A-07 — High — Burn paths destroy backing and break fixed supply
**Location.** `capture_v2.rs:261-272` (BurnOnCapture burns the user's `amount` instead of escrowing it);
`release_v2.rs:191-208` (BurnOnRelease burns the NFT); `state/path.rs` bits 3/4.
**Exploit (model, PoC 02).** With BurnOnCapture, captures fund **no** backing, but `release_v2` still pays
`amount` from the escrow (`:287`) — the first release finds an empty pool and fails / would underflow;
meanwhile mint supply has dropped below 1B, so "fixed 1,000,000,000 / exact unwrap" is already false.
**Fix.** Forbid `BurnOnCapture`/`BurnOnRelease` for hybrid launches (already called out in THREAT_MODEL
T-BURN-02). If a re-roll burn is wanted (BRIEF), it must burn from the **user's own** balance in a separate
instruction and never touch escrow backing (B-08).

### A-08 — Medium — No ratio/supply/solvency binding in upstream
`init_recipe.rs:66-131` stores `amount` as a free u64 with only `max > min` checked; nothing ties `amount` to
an allowed ratio, checks `collection_size × ratio ≤ 1B`, or verifies the escrow is solvent before a sale.
Mint supply and revoked authorities are never checked. All of Mintmark's supply guarantees would live in the
(unwritten) `hybrid_vault`/launch layer, not upstream. **Fix:** enforce ratio set, `checked_mul`, supply and
authority checks at init (as ADR-008 §1 / T-HL-* already specify).

### A-09 — Medium — `migrate_nft_v1` ignores the transfer result
`migrate_nft_v1.rs:91-92`: `let _transfer_nft_result = transfer_nft_ix.invoke_signed(...);` — the `Result`
is discarded, so a failed Core transfer returns `Ok(())`. A caller could believe an NFT migrated when it did
not. **Fix:** propagate with `?`.

### A-10 — Medium — Raw account init / `init_if_needed`
`init_escrow_v2.rs:25-53` and `init_recipe.rs:66-79` allocate via `create_or_allocate_account_raw` and
`sol_memcpy` the discriminator+data manually rather than Anchor `init` (no automatic "already initialized"
guard on the data region), and the fee ATA uses `init_if_needed` (`init_recipe.rs:53-59`). Re-invocation is
blocked in practice because the PDA already holds lamports/data, but this is fragile and worth a negative test.
**Fix:** prefer Anchor `init` with discriminator checks; assert accounts are empty before allocate.

### A-11 / A-12 / A-13 — Low / Info
- **A-11:** Shipped handlers contain `// Need to add account checks for security` (`release_v2.rs:110`,
  `update_recipe.rs:58`, `update_escrow.rs:56`, `migrate_tokens_v1.rs:74`) — the authors flag the validation as
  incomplete. Treat as unfinished code, consistent with "Audit Pending".
- **A-12:** `authority` is a bare `AccountInfo` whose meaning flips with `path`/delegate config; easy to
  misconfigure into fully-permissionless capture (A-03).
- **A-13 (Info):** read-only `program show` on 2026-09-24 shows the live program is upgradeable by
  `mp14o4AQcmE5meFDxCscervMc1E4zyKEyDp3398PcwU` on both devnet and mainnet; mainnet data length (630,752 B)
  differs from a local build of the pinned commit (521,696 B). Pin behavior to a locally-built, reviewed
  binary; do not assume the deployed program equals `aacf1a53`. An external upgrade key over code that holds
  your backing is itself a Critical trust dependency if MPL-Hybrid is ever used in the swap path.

---

## Assessment of the team's proposed safeguards

### 1. Blind assignment (NFT identity not chosen or visible before capture)
- **Does it close the upstream issue?** Only if implemented as the custom `hybrid_vault` two-step VRF where
  `request_capture` takes **no asset argument** and selection happens in a separate permissionless `settle`.
  It **cannot** be done with MPL-Hybrid in any configuration: `NoRerollMetadata` lets the caller name the
  asset (A-03, executed); reroll-on makes metadata predictable and revert-until-rare (A-04, executed). So
  blind assignment closes A-03/A-04 **iff** MPL-Hybrid leaves the swap path, which ADR-008 already proposes.
- **Residual risk:** the anti-manipulation depends on (a) VRF drawn *after* payment is irrevocably locked,
  (b) FIFO `seq`-bounded pool merge so the candidate set is fixed at request time (defeats pool-shifting after
  the value is public), (c) rejection sampling (no modulo bias — do not repeat A-05), and (d) no cancel after
  fulfilment. Each is a place to reintroduce the bug. Publish the pool census so odds are honest.
- **How it must be implemented:** exactly ADR-008 §3.3(iii)/§3.3 — request locks funds + pins the VRF account
  and `seq`; settle is permissionless, merges only deposits with `seq < request.seq`, verifies the VRF account
  owner + seed, selects with `uniform_below` rejection sampling, then delivers. No instruction may reveal or
  let the requester veto the outcome.

### 2. VRF re-roll paid with a burned token fee
- **Does it close the issue?** The burn addresses two things: it removes the SlotHashes reroll (A-04/A-05) by
  moving selection to VRF, and it prices rare-farming. Burning per se is orthogonal to the drain bugs.
- **Residual risk / correctness:** (a) the burn **must** come from the user's own balance, never from escrow
  backing — otherwise it recreates A-07 insolvency (B-08). (b) A flat token burn that scales with the ratio
  (bps of R) is scale-invariant; a SOL-only fee goes cheap exactly when rares get valuable (matches the doc's
  own analysis). (c) **Fee-model inconsistency in the docs must be resolved:** BRIEF SCOPE CHANGE + THREAT_MODEL
  (ADR-009) say **burn**; hybrid-rarity §4 + ADR-008 say **split, no burn**. Pick one; if burn, the copy
  "re-roll burns can only reduce supply" is fine, but every burn must be provably from the user and the
  `vault ≥ R×NFTs_outside` invariant must be fuzzed to hold after each op.
- **How it must be implemented:** inline `token::burn` on the caller's ATA in `request_reroll`; enforce
  `capture_fee_bps ≥ reroll_fee_bps` so unwrap→rewrap is never a cheaper reroll; charge the fee in force at
  request time; hard-cap the bps in code.

### 3. Multisig + timelock for authorities
- **Does it close the issue?** It is the correct mitigation for A-01/A-02 **only if** the timelocked path
  cannot change the drain-relevant fields at all. A Squads multisig that still calls upstream
  `update_recipe_v1` does **not** help, because that instruction is atomic and unbounded — a compromised /
  colluding multisig set drains in one approved tx with no delay. Timelock must be enforced *in the program*,
  not merely by who holds the key.
- **Residual risk:** upstream has no timelock primitive, so multisig-over-upstream still allows instant
  `amount`/`token`/`fee_location` changes (A-01/A-02). The only robust closure is the custom program where
  `ratio`/`mint`/fee-destination are immutable and only bounded fee tweaks exist behind a ≥72h on-chain
  timelock (`propose_fees`/`execute_fees`).
- **How it must be implemented:** program-enforced `propose → wait ≥ timelock → execute`, hard-coded caps,
  no field outside the bounded set mutable, multisig as the proposer, upgrade authority → multisig → `--final`
  after audit. Pending requests keep the fee in force at request time.

**Bottom line on safeguards:** all three are sound *as designed in ADR-008/§ hybrid-rarity*, but every one of
them **requires abandoning MPL-Hybrid as the swap engine and shipping the custom `hybrid_vault`**. None of the
three can be retrofitted onto upstream MPL-Hybrid. The custom program then becomes the highest-risk,
heaviest-scrutiny component and must be audited (B-01).

---

## §6. Threat model — classic-SPL 1B token + bonding-curve launch

Assumes well-funded bots (Jito bundles run up to 5 txns atomically in one slot; MEV searchers co-locate).

### 6.1 Mint / freeze authority
- **T-CURVE-01 (Critical if missed):** mint authority must be `None` and freeze authority must be `None` at
  launch, verified on-chain by `assert_launch_ready` *before* the sale opens — not merely intended. A retained
  mint authority = infinite dilution; retained freeze = holder lockout. Verify supply == 1B × 10^decimals and
  decimals ≤ 9 (so 1B×10^d < u64::MAX). Fresh mint keypair only; reject onboarding an existing mint (retained
  authority / pre-mint). (Matches T-TOK-01/02, T-HL-01/04 — good; make the on-chain gate mandatory.)

### 6.2 Curve math
- **T-CURVE-02 (High):** rounding must always favor the pool, never the trader, and be consistent between
  buy and sell (a buy/sell round trip must not net the trader tokens or SOL). Use `checked_*` throughout with
  `overflow-checks = true`; fuzz the buy/sell inverse for monotonicity and no free round-trip.
- **T-CURVE-03 (High):** price-manipulation / first-buyer advantage — the deployer or a sniper buying the
  first block gets tokens at the floor and dumps on the curve. Mitigate with a launch delay after pool
  creation, per-wallet buy caps in the first N slots, and/or a decaying anti-sniper fee in the opening window.
- **T-CURVE-04 (High):** integer overflow at the extremes of the curve (near-empty and near-graduation
  reserves) — test both boundaries explicitly.

### 6.3 Sniper bots / Jito bundles / sandwiching
- **T-CURVE-05 (Critical, Stonk.fun S-5):** the launch window is where holders are harmed most. Well-funded
  bots will bundle create+buy or race the first buy. Required: no privileged pre-list allocation; a fair-launch
  opening (commit-reveal or batch/uniform-price first window, or a hard per-wallet cap + opening-fee decay);
  publish the exact opening rules. This is currently an **open design item** (B-05) and must be closed before
  any curve ships.
- **T-CURVE-06 (High):** sandwiching user swaps — every value-moving instruction needs on-chain
  `min_out`/`max_in` slippage bounds (T-MKT-02). Do not rely on the frontend.

### 6.4 Graduation / migration to a DEX
- **T-CURVE-07 (Critical, B-06):** who triggers graduation and where the liquidity goes is unspecified. A
  discretionary/off-chain trigger that routes LP to an operator wallet is the Stonk.fun trust failure in a new
  form. Required: graduation is **permissionless and deterministic** (fires at a fixed reserve/supply
  threshold), LP tokens are **burned or locked in a program PDA** (never sent to a person), and the migration
  route (Raydium/Meteora) and `min_out` are pinned on-chain. No human hand touches the liquidity.
- **T-CURVE-08 (High):** partial-graduation / re-entrancy — migration must be atomic and one-shot (a `graduated`
  flag set before any transfer), so it cannot be run twice or interleaved to skim reserves.

### 6.5 Creator / platform rug paths
- **T-CURVE-09 (High, B-07):** if the 1B supply is minted to a person (T-HL-11), that person can dump the
  entire supply pre- or post-curve. Supply must go to the curve/sale vault PDA, not an EOA; any creator
  allocation must be explicit, capped, and disclosed (Q-H6 recommends forbidding out-of-pool creator NFTs).
- **T-CURVE-10 (High):** no discretionary platform wallet may custody or route user value at any point
  (stonkfun-lessons §2). Every fee destination on-chain, public, immutable or timelocked, and equal to the
  site copy (§4, S-2/S-4). Prefer immutable.
- **T-CURVE-11 (Medium):** re-roll burn (if adopted) must not be spun as "buyback"; publish live supply +
  cumulative burned and a reconciliation script (S-4).

### 6.6 Stonk.fun lessons applied
No transfer tax (already dropped) removes the "tax sells into every chart" harm (S-1). The residual Stonk.fun
patterns that still apply to a curve launch are S-2 (no operator wallet routes value → T-CURVE-07/10), S-3 (no
retained power over economics → A-01/A-02 must be closed; curve params immutable/timelocked), S-4 (on-chain,
reconcilable ledger), and S-5 (anti-sniping in the first minutes → T-CURVE-05). The `update_recipe` drain
(A-01) is the on-chain embodiment of the S-3 "retained power" failure and is the single most important thing to
eliminate.

---

## Launch-readiness statement (blunt)

**Mintmark is NOT launch-ready, and MPL-Hybrid must not be used as the swap engine as-is.** Upstream
MPL-Hybrid @ `aacf1a53` is explicitly "Audit Pending" and I confirmed, with executed on-chain PoCs against a
locally-built copy of the pinned commit, two Critical drain paths (A-01 untimelocked `update_recipe` amount
change; A-02 mint/fee-destination overwrite, PoCs 10 and 12) and one High cherry-pick/permissionless-capture path (A-03), plus
modeled High-severity predictable-randomness (A-04/A-05), shared-escrow (A-06) and burn-insolvency (A-07)
issues. Every one of the team's three safeguards (blind assignment, VRF burn re-roll, multisig+timelock) is
sound in the ADR-008 design but **cannot be retrofitted onto upstream MPL-Hybrid** — they all require shipping
the custom `hybrid_vault`, which does not exist in code yet (B-01). The bonding-curve half of the product has no
chosen venue and no anti-sniping, no deterministic permissionless graduation, and no LP-custody design
(B-05/B-06/T-CURVE-05/07), which are exactly the Stonk.fun-class failures the BRIEF is trying to avoid.

Minimum bar before any devnet/mainnet consideration: (1) commit to the custom `hybrid_vault` (or a
timelock-wrapped, field-restricted authority over MPL-Hybrid that provably blocks A-01/A-02) and implement it;
(2) reconcile the burn-vs-split fee inconsistency; (3) specify and implement anti-sniping + deterministic
graduation + program-custodied LP for the curve; (4) fuzz the backing invariant `vault ≥ R × NFTs_outside` and
the curve round-trip; (5) enforce mint/freeze revocation and supply on-chain before sale open.

**A professional third-party audit of the full on-chain system (the custom `hybrid_vault`, the launch/curve
programs, and any MPL-Hybrid configuration if it is kept) is REQUIRED before mainnet. This internal round is
not a substitute for that audit.**

---

## Errata (2026-09-25, after Review B's cross-check)
- **E1. `poc/harness/12_reroll_peek.js` was a truncated 11-line stub** (it cut off mid-function) that was never run, but its
  header said "EXECUTED, on-chain". **Deleted.** It is replaced by `13_reroll_predict_abort.js`, which executed on the local
  validator (PASS, see the PoC table). A-04 is now on-chain evidence, not model-only.
- **E2. `01` test T2 and `02` test `B_burn_paths_violate_fixed_supply` asserted a literal `True`.** Both now compute their
  result (see the PoC table). Re-run 2026-09-25: 11/11 and 5/5 PASS. 01 is still a model of the upstream formula. The on-chain
  proof of A-04 is PoC 13.
- **E3. A-11 missed a fifth TODO**: `release.rs:99` (V1 release) carries the same `//Need to add account checks for security`.
- **E4. B's re-run** (`docs/security/review-b/poc-crosscheck/a_pocs_10_11_12_rerun.out`) reproduced PoCs 10, 11 and 12 PASS on an
  independent local validator. B's `b13_token_swap_full_drain.js` extends PoC 12 to a full drain (escrow 1,000,000 → 0) plus a
  third party's capture fee rerouted to the attacker's fee ATA.
- **E5. "hybrid_vault does not exist in code yet (B-01)"** in the bottom line was already stale when written:
  `wip/hybrid-vault` existed locally. See merged M-11.
