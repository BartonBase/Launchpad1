# Launchpad QA Test Plan: SPL-404 hybrid launches

Status: **DRAFT v0.3** (2026-09-24). Owner: QA.
Networks: **localnet and devnet only.** Nothing in this plan deploys to mainnet or uses real funds or keys.

**Changelog**
- **v0.3 (2026-09-24):** Barton's scope change (docs/BRIEF.md "SCOPE CHANGE", 2:51 PM MT).
  - **Token-2022 is dropped for now** (deferred, not dropped for good). That removes Track B entirely: transfer tax, tax treasury and NFT buys, holder raffle, the Token-2022 fee math and extension matrix, and the track-separation section (old INV-13…16, SEP-xx, E2E-S0x).
  - The product is **SPL-404 hybrid launches only**.
  - Added: fixed-supply launch checks, blind NFT assignment, the VRF re-roll with burned fee, multisig + timelock governance, and bonding-curve anti-sniping.
  - Invariants renumbered INV-01…16 (old IDs don't carry over).
  - v0.2 is archived at `qa/archive/TEST_PLAN.v0.2.md`, and shelved Track B notes are in `tests/_shelved/track-b/README.md`.
- v0.2 (2026-09-24): two tracks (A: SPL-404, B: Token-2022 tax). Archived.
- v0.1 (2026-09-24): initial draft. Archived.

---

## 1. Scope and assumptions

### 1.1 Product (only this)

An **SPL-404 hybrid launch** works like this:
- A **classic SPL Token** mint (Token program, no Token-2022) with a fixed supply of **exactly 1,000,000,000** tokens.
- Mint authority and freeze authority are revoked at launch.
- A **token ⇄ NFT converter** at a creator-chosen ratio R ∈ {10k, 50k, 100k, 200k, 1M} tokens per NFT, with a creator-chosen collection size N, and `N × R ≤ 1B`.
- **Exact unwrap**: an NFT always converts back to exactly R tokens.
- **Cosmetic rarity** only.
- **Blind NFT assignment**: nobody chooses or peeks at the NFT they'll receive.
- A **VRF re-roll** whose token fee is **burned**.
- **Multisig + timelock** on every setting that can change at all.
- A **bonding-curve launch window** with anti-sniping controls.

| Component | Short name | Status |
|-----------|------------|--------|
| Launch: mint 1B, revoke authorities, `assert_launch_ready` / `open_sale` | `launch` | no code yet |
| Swap engine / converter: capture, release, re-roll, settle, expire | `hybrid` | no code yet. **Engine not final, see §1.2 and Q1** |
| Governance: multisig proposals + timelock for fees and any other mutable setting | `gov` | no code yet |
| Bonding curve / launch window (our program or a third party) | `curve` | no code yet, venue is Q4 |
| Next.js site (launch wizard, token page, convert, re-roll) | `app` | `app/` exists; E2E not started |

Out of scope: Token-2022 anything (deferred), lottery/raffle (shelved), mainnet deployment, running the audits themselves.

### 1.2 Assumptions and sources (read first)

- **Sources:**
  - `docs/BRIEF.md`, including the SCOPE CHANGE section.
  - `docs/stonkfun-lessons.md`.
  - `docs/hybrid-rarity-and-assignment.md` and `docs/DECISIONS.md` ADR-004 and ADR-008.
  - `docs/THREAT_MODEL.md` T-HY/T-HV/T-MKT rows.
  - `security/auditor-b/design-requirements.md` R-01, R-03, R-10, R-11 and R-12.
  - `docs/qa-answers.md` **does not exist yet**.
- **Swap engine is NOT final.**
  - The BRIEF scope change says "MPL-Hybrid-based token<>NFT conversion".
  - ADR-008 (**Proposed**, Barton's decision pending as Q-H1) replaces MPL-Hybrid with a custom **`hybrid_vault`**. The reason: the engineer found, from MPL-Hybrid source @ `aacf1a53`, that **no MPL-Hybrid configuration gives blind assignment**. The capturer names the asset (`capture_v2.rs:52-54`), the built-in reroll is predictable (SlotHashes/timestamp, `:187-203`), and `update_recipe` can change amounts/fees and overwrite the mint and fee destination with no timelock (`update_recipe.rs:91-138`).
  - This plan writes tests against the **required behaviour**, not a specific engine. Provisional instruction names follow ADR-008 (`initialize`, `request_permutation_seed`/`store_seed`, `request_capture`, `settle`, `release`, `request_reroll`, `expire`, `propose_fees`/`execute_fees`) and are marked _(prov.)_.
  - **If stock MPL-Hybrid is kept, the blind-assignment tests (§5) and immediate-path governance tests (§7) are expected to fail by design.** QA would log them as blocking findings, not as test bugs.
- **No SPL-404 program code exists yet** (checked 2:5x PM MT). `programs/` contains only the engineer's shelved Track B programs (`fee_treasury`, `holder_lottery`). No tests in this plan are written yet.
- **Supply after burns:**
  - The token re-roll fee is burned (BRIEF scope change: "default is BURN"; "re-roll burns can only reduce it").
  - So the rule is **exactly 1B at launch, and afterwards `supply = 1B − Σ recorded burns`**, never more.
  - Burns must never touch the vault or exact unwrap.
- **Freeze authority revoked:**
  - BRIEF hard requirement #1 only says *mint* authority is revoked.
  - Freeze authority `None` comes from ADR-004, ARCHITECTURE "Launch types", hybrid-rarity §1 (`require!(… freeze_authority == None)`) and Auditor B R-03.1. This plan tests both.
- **No lottery on hybrid launches, no transfer tax, no discretionary platform wallet** (stonkfun lessons 1–2).

### 1.3 Test levels and tooling

| Level | Tool | Where it runs |
|-------|------|---------------|
| Unit (supply/ratio math, fee bps, Feistel permutation, rejection sampling, timelock arithmetic) | `cargo test` | local |
| Program integration | Rust + LiteSVM 0.10 (same as the engineer's harness: Anchor 1.2.0, Rust 1.89, SBPF v2) and the localnet validator via `anchor test --validator legacy` | local |
| VRF | Mock VRF accounts in LiteSVM (owner and layout of the chosen provider, Q5); devnet queue for smoke tests only | localnet / devnet |
| Upstream programs | Metaplex Core (and MPL-Hybrid, only if Q1 keeps it), dumped read-only from devnet | localnet |
| Fuzzing | Trident 0.12 | local |
| Security regression | one test per finding ID (§11) | CI + local |
| E2E / UI | Playwright + mocked wallet adapter (§12) | localnet, then devnet |

---

## 2. Invariants

Every integration test ends with `assert_invariants()`, and every Trident flow asserts them after each instruction. Notation: `D = 10^decimals`, `R_base = R × D`, `S0 = 1_000_000_000 × D`.

| ID | Invariant |
|----|-----------|
| **INV-01** Fixed launch supply | At launch, `mint.supply == S0`, `mint_authority == None`, `freeze_authority == None`, and the mint is owned by the classic Token program. The sale can't open otherwise. |
| **INV-02** Supply only shrinks, only by burns | After launch, `mint.supply == S0 − Σ recorded re-roll burns` (and any other documented burn). No instruction ever mints. |
| **INV-03** Exact per-instruction token deltas | capture: user −(R_base + token fee), vault +R_base, fee handled per its policy (Q2). release: vault −R_base, user +R_base. reroll: user −fee (burned), vault ±0. No other token balance changes. |
| **INV-04** Escrow backing | `vault_token_balance == R_base × nfts_outside_vault`. If unsolicited donations are possible, it's `≥`, with the donation surplus tracked separately (Q11). |
| **INV-05** NFT conservation | `pool + incoming + nfts_outside_vault + pending_reroll_handins == collection_size`, counting unminted indices as pool. No NFT is created beyond N, and none is lost. |
| **INV-06** Exact unwrap | `release` always pays exactly `R_base`: no fee, no rounding, independent of fees, burns, pool state or pending requests. |
| **INV-07** Supply cap at config | `R ∈ {10k, 50k, 100k, 200k, 1M}`, and `N ≥ 1`, and `N × R_base ≤ S0`, computed with checked math (overflow fails cleanly). R, N and the mint are immutable after init. |
| **INV-08** Blind assignment | The recipient never chooses the NFT. The pick depends only on a VRF value produced **after** payment is locked, and on a candidate pool fixed at request time (deposits with `seq < s`). |
| **INV-09** Single-use randomness | Each request consumes exactly one fulfilled VRF result from the account pinned at request time, exactly once. There's no cancel or refund after fulfilment, and `expire` works only if the request is unfulfilled past `deadline_slot`. |
| **INV-10** Re-roll fee burned | A re-roll lowers `mint.supply` by exactly the token fee. No token account's balance goes up because of it, and the vault is untouched. |
| **INV-11** Fee destinations public and fixed | Every fee token or lamport goes to a destination recorded on-chain at init (or is burned), matching the site copy. No instruction takes a free destination argument, and no operator wallet ever custodies user value. |
| **INV-12** Governance: multisig + timelock | Every mutable setting changes only by multisig proposal → delay ≥ configured timelock → execute, within hard caps. Immutable fields (R, N, mint, trait root, vault account, fee destinations unless governed) never change. There's no immediate path. |
| **INV-13** Access control and account validation | Privileged instructions need the documented signer. Every PDA is checked (seeds, bump, owner, discriminator), and every external program is pinned. |
| **INV-14** No stuck value / close safety | No account holding user tokens, NFTs or escrowed fees can be closed. `expire` returns the principal. Closing returns rent only to the recorded payer. |
| **INV-15** Launch fairness window | No trade or capture happens before `open_slot` (stored at init). During the anti-snipe window, per-wallet, per-tx and per-slot caps hold (Q4). |
| **INV-16** UI truthfulness | Ratio, collection size, "% convertible", fees, fee destinations, supply, authorities and pending timelocked changes on the site equal the on-chain values. |

---

## 3. Launch and supply (INV-01, 02, 07)

| ID | Case | Expected |
|----|------|----------|
| SUP-01 | Launch happy path | supply == S0, mint & freeze authority None, classic Token owner (INV-01) |
| SUP-02 | Mint authority still set | `assert_launch_ready`/`open_sale` fails |
| SUP-03 | Freeze authority still set | fails |
| SUP-04 | Supply ≠ S0 (S0−1, S0+1, 0) | fails |
| SUP-05 | Token-2022 mint (with or without extensions) passed as the hybrid mint | rejected (classic Token only) |
| SUP-06 | Upgrade authority is a single EOA instead of the multisig | `assert_launch_ready` fails (R-03.1) |
| SUP-07 | Attempt to mint after launch (any path) | impossible; INV-02 |
| SUP-08 | Many re-rolls | supply decreases by exactly Σ fees; recorded burn counter matches (INV-02, INV-10) |
| CFG-01 | Every allowed R at max N (`S0 / R_base`) | accepted |
| CFG-02 | Every allowed R at max N + 1 | rejected |
| CFG-03 | R not in the allowed set (0, 1, 9_999, 3, 1B+1) | rejected |
| CFG-04 | N = 0 | rejected |
| CFG-05 | Overflow edges: `N × R_base` overflows u64 (e.g. N = u64::MAX); decimals high enough that `S0` overflows (e.g. decimals ≥ 11) | clean error, never wraps |
| CFG-06 | Undersized collection (`N × R < 1B`) | accepted; UI copy shows "% convertible" (INV-16) |
| CFG-07 | Change R, N or mint after init | no instruction exists; direct attempts fail (INV-07, INV-12) |
| CFG-08 | Burns push supply below `N × R_base` | releases stay exact; captures limited only by users' tokens; no insolvency (INV-04/06); UI copy still accurate (Q9) |

---

## 4. Wrap / unwrap exactness (INV-03…06)

| ID | Case | Expected |
|----|------|----------|
| WR-01 | Capture happy path (two-step: request → VRF → settle) | exact deltas (INV-03); NFT delivered to the stored recipient; INV-04/05 |
| WR-02 | Release happy path | exactly R_base back in the same tx (INV-06); NFT enters incoming with a new `seq` |
| WR-03 | Capture/release loop × 10k (random users) | INV-03…06 after every step; no drift of even 1 base unit |
| WR-04 | Release of an NFT not owned by the signer / from another collection / a fake Core asset | rejected |
| WR-05 | Release while requests are pending, and after many burns | still exactly R_base |
| WR-06 | Capture with balance = R_base + fee exactly, and 1 base unit less | success / clean failure with no partial transfer |
| WR-07 | Double release of the same asset; double settle of the same request | second fails (T-HV-17) |
| WR-08 | Unsolicited tokens sent to the vault ATA | no effect on payouts; accounting per Q11 |
| WR-09 | Rounding: fee bps on R for every allowed R and fee value | integer, no dust left anywhere; `Σ out == Σ in` |
| WR-10 | Slippage guards (R-11.2): capture/release pass `expected_amount`/`expected_fee` | mismatch after a config change reverts |

---

## 5. Blind NFT assignment (INV-08)

| ID | Case | Expected |
|----|------|----------|
| BA-01 | IDL review: `request_capture` has no asset/index argument and no caller-selectable asset account | pass (T-HV-01). **Fails on stock MPL-Hybrid** (`capture_v2` takes the asset) |
| BA-02 | `settle` with a caller-supplied asset ≠ the VRF-selected index | rejected |
| BA-03 | Simulation peek: simulate `request_capture` | reveals no selection (none happens in the request tx) |
| BA-04 | Simulation peek: simulate `settle` before fulfilment | fails; after fulfilment the outcome is computable but can't be changed (the request is irrevocable) |
| BA-05 | Revert-until-rare: attacker program CPIs `request_capture` (+ tries `settle`) in one tx and reverts if unhappy | can't observe a pick in the same tx; settle needs a VRF fulfilled in a later tx (T-HV-02) |
| BA-06 | Same-slot tricks: request, fulfil and settle in the same slot / one Jito-style bundle; reuse a pre-fulfilled VRF account | rejected: VRF account created/pinned at request with a seed tied to the request; a pre-existing or already-fulfilled account fails |
| BA-07 | Pool shifting after the VRF value is public: release/deposit NFTs before settle | new deposits (`seq ≥ s`) never enter request `s`'s candidates; out-of-order settle fails (T-HV-03) |
| BA-08 | Predictability: selection must not depend on slot, timestamp, SlotHashes, count or blockhash | same VRF value + same pool ⇒ same pick at any slot/time; CI grep lint on selection code (T-HV-05) |
| BA-09 | Uniformity: ≥1M mock-VRF picks over pool sizes including non-powers of two | chi-square passes; no modulo bias (rejection sampling) |
| BA-10 | Trait provenance: seed requested only after `trait_root` is committed; wrong Merkle leaf/proof at mint-on-exit | rejected (T-HV-06) |
| BA-11 | Feistel permutation | bijection over [0, N) for many N and keys (unit + property test) |
| BA-12 | Creator/team capture outside VRF, or pre-allocation (Q-H6) | impossible |
| BA-13 | Seeing a rare in the pool, racing to grab it | no edge: pick is uniform over a pool fixed at request time |

---

## 6. VRF re-roll (INV-09, INV-10)

| ID | Case | Expected |
|----|------|----------|
| RR-01 | Happy path: `request_reroll(asset)` → fulfil → `settle` | hand-in moves to incoming; a **different** random NFT is delivered (T-HV-13) |
| RR-02 | Fee burned | `mint.supply` −fee exactly; no account's balance goes up; vault unchanged (INV-10) |
| RR-03 | Stale fulfilment: VRF account from an earlier or other request, wrong seed, foreign owner, unfulfilled | rejected (INV-09) |
| RR-04 | Duplicate fulfilment / double settle / reuse of a consumed VRF result | rejected; request closed after settle |
| RR-05 | Cancel or `expire` after fulfilment (the "saw the result, want out" path) | rejected (T-HV-02/04) |
| RR-06 | Abandon: requester never settles | anyone (crank) can settle; outcome identical; requester can't block it |
| RR-07 | Recipient griefing: requester makes delivery fail (closes accounts, changes wallet state) hoping to reach `expire` | settle still succeeds (Core asset owner set directly) or can't be turned into a refund; INV-09 |
| RR-08 | VRF never fulfils | `expire` after `deadline_slot` returns the NFT; fee handling per policy (burned, or refunded minus VRF cost?, Q3); not before the deadline |
| RR-09 | Pool has only the caller's own NFT / reservations exhausted | request rejected (T-HV-14) |
| RR-10 | Fee changes (timelocked) while a request is pending | the request pays the fee in force at request time (T-HV-11) |
| RR-11 | `capture_fee ≥ reroll_fee` rule, if a capture fee exists (Q2 / Q-H4) | config violating it is rejected; unwrap→rewrap is never cheaper than a re-roll |
| RR-12 | Re-roll loop × 1k | INV-02/04/05/10 after each; burned total matches the counter |
| RR-13 | SOL part of the fee (VRF + rent) | covers VRF and lazy-mint rent; a zero-balance cranker can settle (T-HV-16); destination per Q3 |

---

## 7. Governance: multisig + timelock (INV-12, 13)

| ID | Case | Expected |
|----|------|----------|
| GOV-01 | Propose a change with a non-multisig signer (EOA, creator key, random key) | rejected |
| GOV-02 | Execute at `delay − 1s`, exactly at `delay`, and after | fails / succeeds / succeeds |
| GOV-03 | Values over the hard-coded caps (e.g. token fee > cap bps, SOL fee > cap) | rejected at propose and at execute |
| GOV-04 | Execute the same proposal twice; replay after cancel; execute a cancelled proposal | rejected |
| GOV-05 | **No immediate path:** enumerate every instruction in the IDL and try to change each governed or immutable field (R, N, mint, trait root, vault account, fee values, fee destinations, VRF provider, open slot, curve params) without the timelock | every attempt fails. Includes the MPL-Hybrid T-HY-01 class ("escrow authority overwrites the mint / fee destination / amount"), as a regression test if Q1 keeps MPL-Hybrid |
| GOV-06 | Pending user requests during a fee change | keep their original fee (RR-10) |
| GOV-07 | Program upgrade authority | is the multisig (launch check SUP-06); an upgrade by an EOA is impossible |
| GOV-08 | Multisig config (Squads v4): signers/threshold match the documented set (Q6) | launch-script assertion |
| GOV-09 | Visibility: every proposal emits an event and is readable on-chain before execution | the site shows pending changes and the countdown (E2E-G01) |
| GOV-10 | Settings declared immutable in the docs | IDL has no setter for them |

---

## 8. Bonding-curve launch window and anti-sniping (INV-15) _(prov.; venue and mechanism are Q4)_

Source of the specifics:
- `docs/stonkfun-lessons.md` says only this (lesson 5, plus the ZCAT example): snipers churning in the first minutes are measurable, so design anti-sniping for the curve's first minutes.
- The concrete controls below come from Auditor B R-10 (commit window + single clearing price + per-wallet deposit cap, **or** max buy per wallet + max total per slot for the first N slots; open slot stored at init), B-10, and `research-sources.md`:
  - Jito bundles are ≤ 5 txs, same-slot and atomic.
  - 36.5% of supply is held by bundled accounts at migration (MELT dataset).
  - StonkFun admitted single-wallet launches and changed its dev-buy limits.
  - The pump.fun 2024 incident came from a privileged migration key plus flash loans.

| ID | Case | Expected |
|----|------|----------|
| SN-01 | Buy or capture at `open_slot − 1` | rejected; open slot comes from config, not an admin tx (R-10.2) |
| SN-02 | Per-wallet cap during the window: buy cap−1, cap, cap+1 (single tx and split across txs) | the last one fails |
| SN-03 | Per-tx limit | over-limit tx reverts |
| SN-04 | Per-slot total cap for the first N slots: many wallets in one slot | total in the slot ≤ cap; excess reverts |
| SN-05 | Same-slot bundle `[open/first-buy, buy×4]` from one funder or many fresh wallets | caps enforced across the bundle (LiteSVM: many txs at the same slot) |
| SN-06 | Sybil pattern: 50 fresh wallets funded by one source buy in the window | per-slot/window caps bound the total; report the share acquired (compare to the 36.5% MELT benchmark) as a metric |
| SN-07 | If commit window + batch clearing: deposits in the first and last slot of the window | same clearing price; over-cap deposit rejected; excess refunded exactly |
| SN-08 | Window edges: `open_slot`, `open_slot + N − 1`, `open_slot + N` | caps apply / apply / lifted |
| SN-09 | Creator/dev buy at launch | within the documented limit and visible on the site (StonkFun single-wallet lesson) |
| SN-10 | Slippage: every buy/sell has `max_in`/`min_out` (R-11.1) | a sandwich pushing past the bound reverts |
| SN-11 | Snipe NFTs in the window: buy tokens + capture immediately | capture is still blind (R-10.3); no rare-targeting edge |
| SN-12 | Flash pattern in one tx/bundle: buy → capture → release → sell | no profit beyond honest odds minus fees; no supply or backing drift |
| SN-13 | Graduation/migration | no privileged withdraw/migrate key; migration is permissionless and deterministic; flash-buying to 100% can't extract liquidity (pump.fun precedent) |
| SN-14 | Fee/curve params | fixed at init or governed by INV-12; no operator wallet receives curve funds (stonkfun lessons 2–4) |

If the curve is a **third-party** venue (e.g. a Raydium/Meteora launch product), SN-xx is reduced to what that venue enforces, plus configuration checks. Any gap goes to the Security Auditors as a finding.

---

## 9. Generic program matrix (all instructions, provisional names)

Columns: **HP** happy path · **AC** access control/signer · **PDA** spoofing/wrong accounts · **PROG** wrong token program / wrong mint / fake Core or VRF program · **MATH** overflow/rounding · **CPI** ordering/reentrancy · **PAUSE** paused/closed (Q7) · **EDGE** zero/max.

| Instruction | HP | AC | PDA | PROG | MATH | CPI | PAUSE | EDGE |
|-------------|----|----|-----|------|------|-----|-------|------|
| `initialize` | CFG-01, SUP-01 | creator only; re-init fails | vault/config seeds and bump | T22 mint rejected; existing Core collection rejected (program creates it, T-HV-07) | CFG-05 | Core collection CPI with no Permanent plugins | n/a | CFG-01…04 |
| `request_permutation_seed` / `store_seed` | seed stored once | anyone / pinned VRF | VRF account pinned | foreign VRF owner rejected | n/a | n/a | capture closed until the seed is stored | second store fails |
| `request_capture` | WR-01 | user signs | vault ATA / request PDA spoofing | classic Token only; wrong mint | R_base + fee overflow | tokens locked before anything else | per Q7 | exact balance; empty pool (RR-09) |
| `settle` | WR-01, RR-01 | permissionless | request/VRF/asset PDA checks | pinned VRF + Core | index bounds | mint-on-exit CPI order; Merkle check before mint | per Q7 (settle of already-paid requests shouldn't be blockable) | last NFT in pool |
| `release` | WR-02 | NFT owner only | fake asset / collection | fake Core program | exact R_base | NFT in before tokens out, atomic | **should release ever be pausable? (Q7)** | last NFT; after burns |
| `request_reroll` | RR-01 | NFT owner | asset from another collection | pinned programs | fee bps rounding | hand-in before fee burn, atomic | per Q7 | only NFT in pool |
| `expire` | RR-08 | permissionless | request PDA | n/a | refund exact | n/a | n/a | deadline ± 1 slot |
| `propose_*` / `execute_*` / `cancel_*` | GOV-02 | multisig only | proposal PDA | n/a | cap checks | n/a | n/a | delay edges |
| launch / `assert_launch_ready` / `open_sale` | SUP-01 | launcher | config | classic Token | S0 math | n/a | n/a | SUP-02…06 |
| curve `buy` / `sell` (if ours) | SN-xx | user | curve PDAs | classic Token | curve math overflow/rounding favours the pool | n/a | window/pause | SN-02…08 |

Close safety (INV-14): try to close every program-owned account while it holds tokens, NFTs, escrowed lamports or a pending request. All attempts must fail, and rent may go only to the recorded payer.

---

## 10. Fuzzing plan (Trident 0.12)

- Set up with `trident init` once the hybrid program exists. Run `trident fuzz run <target> [seed] --with-exit-code` in CI; reproduce with `trident fuzz debug <target> <seed>`.
- The VRF is a **mock account owned by the pinned VRF program id**. The fuzzer controls fulfilment timing and value, including never fulfilling, fulfilling late, and fulfilling twice.

| Target | Instructions | Assertions after every instruction |
|--------|--------------|------------------------------------|
| `fuzz_hybrid` | request_capture, settle (random order attempts), release, request_reroll, expire, random VRF fulfilment, random donations to the vault | INV-02…06, 08, 09, 10, 14; out-of-order settle always fails; settle never finds an empty pool |
| `fuzz_config` | initialize with random R, N, decimals (incl. overflow edges) | INV-01, 07; no panic, only clean errors |
| `fuzz_governance` | propose/execute/cancel with random signers, values and clock warps, interleaved with user requests | INV-11, 12, 13; pending requests keep their fee |
| `fuzz_launch_curve` (only if the curve is ours) | buys/sells from many wallets in the same and adjacent slots around `open_slot` | INV-15; curve reserves reconcile; no value created |

Exit criteria: ≥ 1M iterations (or 1 h) per target with no invariant failure or panic. Crash seeds go to `tests/fuzz/regressions/` and each becomes a deterministic regression test.

---

## 11. Security-finding regression workflow

1. The Security Auditors write findings to `security/auditor-a/` and `security/auditor-b/`. The merged, de-duplicated list goes in `security/merged/` with stable IDs (e.g. `SEC-001`).
2. For each merged finding, QA adds a row to `qa/FINDINGS_TRACKER.md` and writes a test **named after the ID**:
   - Rust: `tests/regression/qa-sec_001_<slug>.rs`, `#[test] fn sec_001_<slug>()`
   - TS: `tests/regression/qa-SEC-001-<slug>.test.ts`
   - Fuzz seed (if any): `tests/fuzz/regressions/SEC-001.seed`
3. The test must **fail on the pre-fix commit**. Record the commit hash and "FAIL" in "status before fix".
4. After the fix, the test must **pass**. Record the commit and "PASS" in "status after fix". The finding is only closed when both columns are filled.
5. Regression tests run on every CI build and are never deleted. If the behavior changes on purpose, update the test and note it in the tracker.
6. Non-testable findings get the note "N/A – non-testable" plus a reviewer sign-off.
7. If Q1 keeps MPL-Hybrid, the documented MPL-Hybrid behaviours (T-HY-01 authority overwrite, T-HY-04 caller-chosen asset / predictable reroll) get regression tests that demonstrate them. They stay open findings until mitigated.

---

## 12. End-to-end plan: Next.js site

- **Stack:** Playwright (`@playwright/test`) run with yarn.
- **Environments:**
  - localnet: validator with our program(s), Metaplex Core, and a mock-VRF fulfiller script.
  - devnet: nightly/manual, throwaway airdropped keys only.
- **Wallet mocking:** a custom `MockWalletAdapter` for `@solana/wallet-adapter` (connect, signTransaction/signAllTransactions/signMessage) backed by a localnet-only keypair from `tests/.keys/`. Also cover: rejecting a signature, disconnecting mid-flow, and a wallet on mainnet (a blocking warning; nothing is sent).
- **Viewports:** Desktop Chrome 1440×900, Firefox/WebKit smoke, Pixel 7, iPhone 14.

| ID | Flow | Checks |
|----|------|--------|
| E2E-01 | Connect / disconnect wallet | truncated address; reconnect after reload |
| E2E-L01 | **Launch wizard** | ratio picker limited to {10k, 50k, 100k, 200k, 1M}; collection size clamped to `1B / R`; "Up to {N×R} ({pct}%) can be held as NFTs" copy; summary shows 1B supply, mint + freeze authority revoked, timelock length, fee values and destinations (burn for re-roll); no Token-2022 or tax option anywhere |
| E2E-L02 | Post-launch token page, "What nobody can change" panel | values equal on-chain state (INV-16); supply shows "Fixed at 1,000,000,000 at launch… re-roll burns can only reduce it" with the live burned total |
| E2E-C01 | Capture (two-step) | price + fees shown before signing; a "Drawing your NFT…" state until settle; the result is shown only after settle; no UI lets the user pick an NFT |
| E2E-C02 | Release | "You get exactly {R}" shown and matched on-chain |
| E2E-C03 | Re-roll | fee and "burned" disclosed before signing; drawing state; the new NFT differs; burned counter goes up |
| E2E-C04 | VRF slow / expired request | clear status; refund path after the deadline; no double submission |
| E2E-G01 | Pending governance change | banner with old → new value and countdown; the change applies only after it |
| E2E-S01 | Launch window | countdown to `open_slot`; per-wallet cap shown; an over-cap buy is blocked in the UI and reverts on-chain if forced |
| E2E-04 | Insufficient balance / empty pool / paused (Q7) | clear error, disabled button, no tx sent |
| E2E-08 | Tx failure / timeout / blockhash expired | retry UX, no duplicate submissions |
| E2E-09 | Network guard | test builds refuse mainnet RPC |

**Usability checks:**
- Axe accessibility scan (`@axe-core/playwright`) per page.
- Keyboard-only launch wizard, capture, re-roll and release.
- Tap targets ≥ 44px on mobile.
- No horizontal scroll at 375px.
- A pending state for every tx.
- Human-readable amounts with raw amounts in details.
- Visual snapshots per viewport.
- Lighthouse smoke test.

---

## 13. Proposed `tests/` layout

QA-authored files are prefixed `qa-` or live in `qa/` subfolders, so they never collide with the engineer's tests. Folder names will be aligned with the engineer's once the hybrid program lands.

```
tests/
├── .keys/                   # LOCALNET/DEVNET-ONLY throwaway keypairs (gitignored)
├── shared/                  # engineer's shared crate
│   └── qa/                  # QA helpers: SPL supply/authority checker (INV-01/02), vault-backing
│                            #   and NFT-conservation checkers (INV-04/05), balance-diff (INV-03/10/11),
│                            #   mock VRF account builder, IDL "no immediate path" scanner (GOV-05)
├── hybrid/                  # SPL-404 program tests
│   ├── qa-launch.rs         # SUP-xx, CFG-xx
│   ├── qa-wrap.rs           # WR-xx
│   ├── qa-blind.rs          # BA-xx
│   ├── qa-reroll.rs         # RR-xx
│   └── qa-governance.rs     # GOV-xx
├── curve/                   # SN-xx (depends on Q4)
├── regression/              # qa-sec_xxx (one file per finding ID)
├── fuzz/                    # Trident targets
│   └── regressions/         # crash seeds
├── e2e/                     # Playwright specs, wallet mock, playwright.config.ts
└── _shelved/track-b/        # shelved Token-2022 QA notes (2026-09-24 scope change)
```

---

## 14. Open questions

- **Q1 Swap engine:** MPL-Hybrid (BRIEF scope-change wording) or the custom `hybrid_vault` (ADR-008, Proposed, Barton's Q-H1)? The required blind assignment, safe VRF re-roll and no-immediate-path governance can't pass on stock MPL-Hybrid, per the engineer's source review.
- **Q2 Capture fee:** is there one? Is it a token bps of R, burned like the re-roll fee or sent to a fixed destination? Keep `capture_fee ≥ reroll_fee` (Q-H4)?
- **Q3 SOL part of fees:** the VRF + lazy-mint rent charge can't be burned. Who receives it, and is it refunded (minus VRF cost) on `expire`? Is the token fee burned or refunded on `expire`?
- **Q4 Bonding curve:** our own program or a third-party venue? Which anti-snipe mechanism (commit window + single clearing price vs per-wallet / per-tx / per-slot caps)? What are the window length, cap values and creator/dev-buy limit? What is the graduation/migration mechanism?
- **Q5 VRF provider:** Switchboard On-Demand or ORAO (DECISIONS Q1)? What are the request deadline and expire policy?
- **Q6 Multisig and timelock:** Squads v4 signers and threshold; timelock length (ADR-008 proposes ≥ 72h; stonkfun lesson 3 prefers long and visible, or immutable); the exact list of settings that are changeable at all; hard caps.
- **Q7 Pause:** is there one? Which instructions can it block? Release and settle of already-paid requests arguably should never be blockable.
- **Q8 Hybrid design details:** lazy mint vs pre-mint (Q-H5); forbid creator/team allocations (Q-H6); public pool contents (Q-H7); max collection size (Q-H9); two-tx capture UX (Q-H8).
- **Q9 Supply copy after burns:** once burns push supply below N × R, "Up to N×R can be held as NFTs" is no longer reachable. How should the site phrase it?
- **Q10 Decimals:** fixed value (6 or 9)? This determines the overflow edges in CFG-05.
- **Q11 Vault donations:** treat unsolicited tokens sent to the vault as surplus (`≥` backing) or reject/sweep them?

Removed from v0.2 as no longer applicable: Track B treasury buying (old Q2), raffle exclusions/randomness/prizes (old Q4/Q5/Q7/Q10), existing-collection NFT standards (old Q8), cross-track registry (old Q9), Token-2022 mint onboarding (old Q11). Old Q3 (pause), Q5 (randomness) and Q6 (authorities) survive as Q7, Q5 and Q6.

---

## 15. Coverage status

No SPL-404 program code exists yet, so every row is "not started".

| Area | Section | Status |
|------|---------|--------|
| QA helpers (supply/authority, backing, NFT conservation, balance diff, mock VRF, IDL scanner) | §13 | not started |
| Launch & supply (SUP-01…08) | §3 | not started |
| Ratio × size config (CFG-01…08) | §3 | not started |
| Wrap/unwrap exactness (WR-01…10) | §4 | not started |
| Blind assignment (BA-01…13) | §5 | not started (blocked on Q1) |
| VRF re-roll (RR-01…13) | §6 | not started (blocked on Q1, Q5) |
| Governance multisig + timelock (GOV-01…10) | §7 | not started (blocked on Q6) |
| Anti-sniping launch window (SN-01…14) | §8 | not started (blocked on Q4) |
| Generic matrix + close safety | §9 | not started |
| Fuzz: fuzz_hybrid / fuzz_config / fuzz_governance / fuzz_launch_curve | §10 | not started |
| Security regression tests | §11 | not started (security/merged is empty) |
| E2E desktop + mobile | §12 | not started |
| Usability / a11y | §12 | not started |
