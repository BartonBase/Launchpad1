# Launchpad QA Test Plan

Status: **DRAFT v0.1** (2026-09-24). Owner: QA.
Networks: **localnet and devnet only.** Nothing in this plan deploys to mainnet or uses real funds or keys.

---

## 1. Scope and assumptions

### 1.1 What's in scope

| # | Component | Short name | What it does |
|---|-----------|------------|--------------|
| A | Hybrid swap wrapper: Anchor program(s) around Metaplex **MPL-Hybrid** | `swap` | **Capture**: a user pays fungible tokens (plus fees) and gets an NFT from escrow. **Release**: a user returns an NFT to escrow and gets tokens back (minus fees). |
| B | Token-2022 "tax" treasury | `treasury` | A Token-2022 mint with the **TransferFee** extension. Withheld fees are harvested and withdrawn into a treasury, and the treasury buys NFTs. |
| C | Holder lottery | `lottery` | `tickets = floor(balance / threshold)` at a snapshot. Winners are drawn with randomness and prizes are paid out. |
| D | Next.js site | `app` | Wallet connect, swap UI, treasury/lottery dashboards, claim flow. |

Out of scope for now: mainnet deployment, audits themselves (those belong to the Security Auditors; QA only turns their findings into regression tests), and load testing beyond basic compute-unit budgets.

### 1.2 Assumptions (read first)

- **No code or spec exists yet.** `programs/`, `app/` and `docs/` are empty. **Every instruction, account and PDA name in this document is PROVISIONAL** and marked _(prov.)_. Rename the tests when the real IDL lands, but keep the test IDs.
- Upstream facts checked against `metaplex-foundation/mpl-hybrid` @ `aacf1a5` (2026-05-27):
  - Program ID `MPL4o4wMzndgh8T1NVDxELQCj5UQfYTYEkabX3wNKtb`.
  - Instructions: `init_recipe_v1`, `init_escrow_v1/v2`, `init_nft_data_v1`, `capture_v1/v2`, `release_v1/v2`, `update_recipe_v1`, `update_escrow_v1`, `update_new_data_v1`, `migrate_nft_v1`, `migrate_tokens_v1`.
  - `RecipeV1` fields: `amount`, `fee_amount_capture`, `fee_amount_release`, `sol_fee_amount_capture`, `sol_fee_amount_release`, `fee_location`, `min`/`max`, `count`, `path`.
  - Capture and release also charge a fixed Metaplex protocol SOL fee.
  - NFTs are **MPL Core** assets.
  - Optional re-roll randomness is derived from the **SlotHashes sysvar plus the timestamp**.
  - **Upstream accounts use `Program<'info, Token>` (classic SPL Token), not Token-2022.** If the tax token (B) is also the swap token (A), the upstream program can't transfer it as written. Also, a transfer fee would make the escrow receive less than `recipe.amount` while release pays out the full `recipe.amount`. **Open question Q1** (§11) must be settled before swap tests are finalized.
- Toolchain on the shared box:
  - Agave/Solana CLI 4.1.2. Anchor 1.2.0 pinned it; the installer's `stable` was 4.2.2.
  - Anchor CLI 1.2.0 via avm, Trident 0.12.0, yarn 1.22.22, Node 20.19.2, SBF platform-tools v1.57.
  - rustup `stable` is 1.98.1. The distro Rust 1.85.1 is still at `/usr/bin` and is available as rustup toolchain `system`.
- Randomness for the lottery (C) is **not chosen yet**. Candidates are Switchboard On-Demand randomness, ORAO VRF, or commit-reveal. The tests below cover each option as _(prov.)_.
- Treasury "buys NFTs" is assumed to mean a capture through the swap wrapper (A), using treasury-owned tokens. _(prov., see Q2)_

### 1.3 Test levels and tooling

| Level | Tool | Where it runs |
|-------|------|---------------|
| Unit (pure math: fees, tickets, rounding) | `cargo test` on host | local |
| Program integration | Rust + LiteSVM (Anchor 1.2 `litesvm` test template), plus the Anchor TS test runner against `solana-test-validator` | local |
| Upstream programs | MPL-Hybrid, MPL Core and Token-2022 loaded into the validator with `[[test.genesis]]` or `solana program dump -u devnet` (read-only dump) | localnet |
| Fuzzing | Trident 0.12 (`trident fuzz run <target>`) | local |
| Security regression | one test per finding ID (§8) | CI + local |
| E2E / UI | Playwright + mocked wallet adapter (§9) | localnet, then devnet |

---

## 2. Global invariants

Every integration test ends by calling a shared `assert_invariants()` helper, and the Trident fuzz targets check the same invariants after every flow. IDs are referenced throughout this plan.

| ID | Invariant | Check |
|----|-----------|-------|
| **INV-01** Token supply conserved across wrap/unwrap | `tokens_in_swap_escrow + circulating_tokens + tokens_withheld (T22) + tokens_burned == mint.supply_at_init (+ minted − burned)` | Read every token account for the mint plus `mint.supply` before and after each ix. The delta must be exactly the recipe amounts and fees. |
| **INV-02** NFT count conserved | `nfts_in_escrow + nfts_held_by_users (+ nfts_burned if path=burn) == collection.num_minted` | Count MPL Core assets by owner (escrow vs others). |
| **INV-03** Escrow backing | While there's no transfer fee: `escrow_token_balance == recipe.amount × nfts_currently_out_of_escrow + initial_seed`. With a transfer fee, see Q1. | Compare derived vs actual balances. |
| **INV-04** Vault/treasury reconciliation | Every vault/treasury token and SOL balance equals the recorded counters in its state account (`treasury.total_harvested − total_spent == balance`, and so on) _(prov. fields)_ | Compare the on-chain account with the state account after each ix. |
| **INV-05** Tickets exact | For every holder in a snapshot: `tickets == floor(balance_at_snapshot / threshold)`. `total_tickets == Σ tickets`. No holder gets tickets for a balance after the snapshot. | Recompute off-chain from snapshot data. |
| **INV-06** No prize paid twice | Each `(round, winner_slot)` pays at most once. `Σ prizes_paid ≤ prize_pool_funded`. | Check claim-receipt PDAs and pool balance. |
| **INV-07** No double draw | Each round goes through `Open → Snapshotted → RandomnessRequested → Drawn → Closed` exactly once. `draw` can't run twice or with stale or reused randomness. | Check the state machine and randomness account. |
| **INV-08** Fees only to configured recipients | Every lamport and token of fees ends up in `config.fee_recipient(s)`, `recipe.fee_location`, the Metaplex protocol fee wallet, or the T22 withheld/treasury path. Nothing else changes balance except payer, escrow and user. | Diff all touched accounts' balances per tx. |
| **INV-09** Authority immutability | Only the configured admin can change config, fees, pause or authorities. Authority changes need the current authority's signature. | Negative tests plus fuzz. |
| **INV-10** No stuck funds on close | Closing any PDA returns its rent and balances to the configured destination. Nothing can close while it holds user value. | Close tests. |

---

## 3. Component A: swap wrapper (MPL-Hybrid) _(prov.)_

### 3.1 Provisional instruction list

`initialize_config`, `create_recipe` (CPI `init_recipe_v1` / `init_escrow_v2`), `fund_escrow_nfts`, `fund_escrow_tokens`, `capture` (CPI `capture_v2`), `release` (CPI `release_v2`), `update_fees`, `set_paused`, `withdraw_fees`, `close_recipe`.

### 3.2 Test matrix

Columns: **HP** happy path · **AC** access control/signer · **PDA** wrong accounts/PDA spoofing · **MINT** wrong mint/token program · **MATH** overflow/rounding · **CPI** reentrancy/CPI ordering · **PAUSE** paused/closed state · **EDGE** zero/max amounts.

| Instruction | HP | AC | PDA | MINT | MATH | CPI | PAUSE | EDGE |
|-------------|----|----|-----|------|------|-----|-------|------|
| `initialize_config` | config created, admin set | non-payer or admin mismatch fails; can't re-init | config PDA with wrong seeds or bump rejected | n/a | fee bps > 10_000 rejected | n/a | n/a | fee = 0 allowed? (spec) |
| `create_recipe` | recipe and escrow created, CPI succeeds | only admin; collection update authority must match | fake collection, fake escrow, recipe for another collection | token ≠ configured mint; T22 vs SPL program (Q1) | `min > max`, `max − min` overflow | CPI to MPL-Hybrid uses the expected program ID (reject spoofed program account) | fails when paused | amount = 0, amount = u64::MAX |
| `fund_escrow_nfts` / `_tokens` | escrow counters go up, INV-02/03 hold | only admin (or open, per spec) | NFT from another collection rejected; escrow must be the recipe's PDA | wrong mint rejected | counter overflow | n/a | fails when closed | 0 NFTs / 0 tokens |
| `capture` | user pays `amount + fee_amount_capture` (+ SOL fees) and receives exactly 1 NFT. `recipe.count` += 1. INV-01/02/03/08 hold. | user must sign; can't capture on another user's behalf with their tokens | spoofed escrow, recipe, fee_location or SlotHashes account rejected (upstream checks `SLOT_HASHES` address) | user ATA for wrong mint; Token-2022 ATA passed to a classic-Token ix, and vice versa | `amount + fee` overflow; insufficient balance fails cleanly with no partial transfer | CPI order: token pull → NFT transfer → fees, all atomic; the failure of any leg reverts everything; no callback into the wrapper (MPL Core plugins/hooks) | fails when paused or recipe closed | escrow empty (0 NFTs) → clean error; balance exactly equal to the price; `fee = 0` |
| `release` | user returns their NFT and gets `amount − fee_amount_release` (per spec) tokens. The NFT goes to escrow, or is burned if `path` says burn. | only the NFT owner; delegate handling per spec | NFT from another collection; fake escrow token account | wrong mint | fee > amount → reject, no underflow | same atomicity as capture | fails when paused (users may need release while paused → spec Q3) | escrow token balance < amount → clean failure; last NFT |
| `update_fees` | new fees apply to the next tx only | only admin | wrong config PDA | n/a | bps > max rejected | n/a | allowed when paused? | 0 and max caps |
| `set_paused` | toggles and emits an event | only admin / pauser | spoofed config | n/a | n/a | n/a | idempotent | n/a |
| `withdraw_fees` | only to the configured recipient, INV-08 | only admin | recipient ≠ configured rejected | wrong mint vault | withdraw > balance | n/a | allowed when paused (spec) | withdraw 0 / all |
| `close_recipe` | rent returned, INV-10 | only admin | n/a | n/a | n/a | n/a | only when escrow is empty or drained per spec | n/a |

Extra swap cases:
- **Price/fee front-running:** admin changes the fee in the same slot as a user capture. The user should pass `max_price` / `expected_fee` _(prov.)_ and the tx must fail if it's exceeded.
- **Re-roll randomness** (if the `path` flag is used): SlotHashes + timestamp can be predicted or influenced by the leader. Test that a user can't pick their index by simulating first and retrying. Documented as a known-weak source; accepted only for cosmetic traits.
- **MPL Core plugin interaction:** an NFT with a freeze or transfer-delegate plugin gets returned. Release must fail, or must not let the user keep control.
- **Compute budget:** capture and release stay under the CU limit with max-length name/uri.

---

## 4. Component B: Token-2022 tax treasury _(prov.)_

### 4.1 Provisional instruction list

`initialize_treasury`, `harvest_withheld` (wraps `HarvestWithheldTokensToMint` or `WithdrawWithheldTokensFromAccounts`), `withdraw_withheld_to_treasury` (wraps `WithdrawWithheldTokensFromMint`, needs the `withdraw_withheld_authority`, which should be a program PDA), `buy_nft` (treasury → swap `capture`), `update_treasury_params`, `set_paused`.

### 4.2 Transfer-fee math (unit + integration)

Token-2022 computes `fee = min(ceil(amount × bps / 10_000), maximum_fee)`, and `fee = 0` when `bps == 0` or `amount == 0`. The recipient receives `amount − fee`, and the fee is **withheld in the recipient's token account**.

| ID | Case | Expected |
|----|------|----------|
| T22-FEE-01 | amount = 1, bps = 1 | fee = 1 (ceil), recipient gets 0 |
| T22-FEE-02 | amount where `amount × bps / 10_000` is exactly an integer | no rounding up |
| T22-FEE-03 | fee would exceed `maximum_fee` | fee == maximum_fee exactly |
| T22-FEE-04 | bps = 10_000 (max) | recipient gets 0 or `amount − max_fee` |
| T22-FEE-05 | amount = u64::MAX | no overflow (Token-2022 uses u128 internally); our off-chain/on-chain mirrors must too |
| T22-FEE-06 | `transfer_checked_with_fee` with the wrong expected fee | rejected |
| T22-FEE-07 | fee schedule change: new `TransferFeeConfig` only takes effect at `newer_transfer_fee.epoch` (≥ 2 epochs later) | program reads the **current epoch's** fee, not the newest config |
| T22-FEE-08 | our program's fee mirror (UI preview, treasury accounting) vs actual withheld amount | equal for 10k random (amount, bps, max_fee) triples |

### 4.3 Harvest / withdraw

| ID | Case | Expected |
|----|------|----------|
| T22-HV-01 | harvest from N accounts to the mint (permissionless) | `mint.withheld` += Σ; each account's withheld amount goes to 0; INV-01 holds |
| T22-HV-02 | harvest with accounts of a **different mint** in the list | skipped or rejected; no cross-mint movement |
| T22-HV-03 | withdraw from the mint to the treasury by a non-authority | fails |
| T22-HV-04 | withdraw to a destination ≠ treasury PDA ATA | rejected by our wrapper (INV-08) |
| T22-HV-05 | treasury counters after harvest + withdraw | INV-04: `treasury.total_withdrawn == Δ treasury balance` |
| T22-HV-06 | the treasury ATA itself receives a transfer (so it has withheld fees) | accounted for, not double counted |
| T22-HV-07 | closing a token account with withheld > 0 | Token-2022 blocks it; UI explains |
| T22-HV-08 | harvest with a very long account list | stays under tx size/CU limits; chunking works |
| T22-BUY-01 | `buy_nft` happy path | treasury tokens → capture → NFT lands in the treasury-owned account; INV-01/02/04 hold |
| T22-BUY-02 | `buy_nft` by a non-authorized caller / cranker | per spec (permissionless crank or admin); never able to redirect the NFT |
| T22-BUY-03 | treasury balance < price + fees | clean failure, no partial spend |
| T22-BUY-04 | transfer fee applied on the treasury → escrow leg | escrow receives `price − fee`. Check INV-03 impact (Q1). |

### 4.4 Token-2022 extensions that can break assumptions

Each test creates a mint with the extension and asserts the program **rejects** the mint at `initialize_*` time, or handles it safely:

| Extension | Risk | Test |
|-----------|------|------|
| TransferHook | arbitrary CPI during transfers → reentrancy, or a hook that blocks the escrow | the mint's hook program is in an allowlist, or transfers fail safely; reentrancy attempt via a malicious hook fixture |
| PermanentDelegate | delegate can drain escrow/treasury at any time | reject mints with a permanent delegate (or document the trust assumption); test that a drain breaks INV-04 and is detected |
| DefaultAccountState = Frozen / FreezeAuthority | escrow or treasury account can be frozen → funds stuck | detect at init; test behavior when frozen |
| MintCloseAuthority | mint closed after supply hits 0 | init rejects, or documents it |
| ConfidentialTransfer / ConfidentialTransferFee | balances/fees not visible → snapshot and ticket math wrong | reject for lottery/treasury mints |
| InterestBearing / ScaledUiAmount | UI amount ≠ raw amount → ticket threshold confusion | tickets use **raw** amounts; UI shows both |
| NonTransferable / CpiGuard / MemoTransfer on user accounts | CPI transfers fail | clear error messages; CpiGuard'd user account can't be pulled via CPI |
| Pausable (if present in the deployed Token-2022) | mint paused → all flows stop | graceful failure |
| TransferFee config authority changes the fee | tax rate can change later | T22-FEE-07 plus a UI warning |

---

## 5. Component C: holder lottery _(prov.)_

### 5.1 Provisional instruction list

`initialize_lottery`, `open_round`, `snapshot` / `register_tickets`, `fund_prize_pool`, `request_randomness`, `settle_randomness` / `draw_winners`, `claim_prize`, `sweep_unclaimed`, `close_round`, `set_paused`.

### 5.2 Test matrix (same columns as §3.2)

| Instruction | HP | AC | PDA | MINT | MATH | CPI | PAUSE | EDGE |
|-------------|----|----|-----|------|------|-----|-------|------|
| `initialize_lottery` | config created | only admin | seeds/bump | T22 vs SPL; extensions (§4.4) | threshold = 0 rejected (div by zero) | n/a | n/a | threshold = 1, u64::MAX |
| `open_round` | round N+1, state Open | only admin | round PDA with wrong index | n/a | round counter overflow | n/a | fails when paused | n/a |
| `snapshot`/`register_tickets` | INV-05 exact | holder signs own registration (or crank) | registering someone else's account; token account not owned by claimed holder | wrong mint's token account | `floor` correctness; Σ tickets overflow (u64) | n/a | only while Open/before cutoff | balance = threshold−1 → 0 tickets; = threshold → 1; = 2·threshold−1 → 1; u64::MAX |
| `request_randomness` | state → RandomnessRequested | only admin/crank | spoofed oracle/randomness account | n/a | n/a | oracle CPI ordering | fails when paused | request twice → rejected |
| `draw_winners` | winners chosen from ticket ranges, state → Drawn, INV-07 | anyone/crank per spec | randomness account not the one committed at request | n/a | `rand % total_tickets` bias check; total_tickets = 0 → no draw / rollover | n/a | n/a | 1 ticket total; winners > ticket holders |
| `claim_prize` | winner paid once, receipt PDA created, INV-06 | only the winner (or their ATA as destination) | fake receipt, wrong round | prize mint mismatch | prize split rounding (dust goes to pool/treasury, not lost) | pay after marking claimed (checks-effects-interactions) | allowed when paused? (Q3) | claim twice → second fails |
| `sweep_unclaimed` | after deadline, unclaimed → configured destination | only admin | destination ≠ config | n/a | n/a | n/a | n/a | sweep before deadline fails; claim after sweep fails |
| `close_round` | rent returned, INV-10 | only admin | n/a | n/a | n/a | n/a | only after Drawn + claims/sweep | n/a |

### 5.3 Lottery-specific risks

- **Randomness manipulation:**
  - SlotHashes / clock / recent blockhash must be **rejected** as the lottery source. A test shows that a leader or a simulate-then-send attacker could predict or retry them.
  - For VRF / Switchboard: the randomness account is bound at request time, can't be swapped at settle time, can't be reused across rounds, and the revealed value can't be seen before the commit.
  - For commit-reveal: a withheld reveal (admin refuses to reveal) → timeout path.
- **Snapshot timing / flash-balance gaming:**
  - A user borrows or buys tokens right before the snapshot and sells after. Mitigations: time-weighted or min-hold rule, or snapshot at an unannounced/random slot _(prov.)_.
  - Test: the same tokens moved between wallets A→B during the registration window must not count twice. Register A, transfer to B, register B → total tickets must not exceed `floor(supply/threshold)`. Registration must be finalized at one slot, or re-verified at draw.
- **Threshold edges:** threshold−1, threshold, threshold+1, k·threshold−1, balance 0, threshold > supply.
- **Rounding:** `floor` only, never round up. Prize splits: `Σ payouts + dust == pool`.
- **Withheld fees and balances:** does "balance" include Token-2022 withheld amounts? Decide and test (expected: no, only the spendable `amount`).
- **Excluded accounts:** escrow, treasury, LP/AMM pools and program-owned accounts should not get tickets (Q4).
- **Duplicate claims:** double claim in the same tx (two ixs), in parallel txs, and after a round is closed and re-opened.
- **Unclaimed prizes:** deadline, sweep destination, and what happens to the next round's pool.

---

## 6. Cross-component flows (integration)

1. Tax transfers → harvest → withdraw → treasury `buy_nft` via swap → NFT in treasury. INV-01/02/04/08 hold after every step.
2. User capture → transfer NFT → second user release. Escrow balances reconcile (INV-03).
3. Lottery round funded from treasury fees → draw → claims → sweep. INV-04/06/07 hold.
4. Pause everything mid-flow → only allowed ops succeed (Q3) → unpause → state consistent.

---

## 7. Fuzzing plan (Trident 0.12)

- Setup: `trident init` in the Anchor workspace once `programs/` exists. One fuzz target per component, plus one combined target: `fuzz_swap`, `fuzz_treasury`, `fuzz_lottery`, `fuzz_e2e`. Run with `trident fuzz run <target> [seed] --with-exit-code` in CI. Reproduce with `trident fuzz debug <target> <seed>`.
- Upstream programs (MPL-Hybrid, MPL Core, Token-2022) are loaded into the Trident SVM as program binaries dumped from devnet _(prov., verify loading support in 0.12 docs)_.
- Actors: admin, 3–5 users, attacker (random signer), cranker.
- Inputs: random amounts biased toward edges (0, 1, threshold±1, max_fee boundary, u64::MAX), random account substitution (swap in a wrong PDA or mint 5% of the time), random ordering.

| Target | Instructions fuzzed | Invariant assertions after each flow |
|--------|--------------------|--------------------------------------|
| `fuzz_swap` | capture, release, update_fees, set_paused, withdraw_fees, fund_escrow_* | INV-01, INV-02, INV-03, INV-08, INV-09; no instruction succeeds with a spoofed account |
| `fuzz_treasury` | token transfers with fee, harvest_withheld, withdraw_withheld_to_treasury, buy_nft, fee schedule change | INV-01 (incl. withheld), INV-04, INV-08; our fee mirror == Token-2022 fee |
| `fuzz_lottery` | open_round, register_tickets, token transfers during window, request/settle/draw, claim_prize, sweep_unclaimed | INV-05, INV-06, INV-07, INV-10; Σ tickets ≤ floor(eligible_supply/threshold) |
| `fuzz_e2e` | all of the above interleaved | all INV-* |

Exit criteria: each target runs ≥ 1M iterations (or 1 h) with no invariant failures or panics. Every crash seed gets saved under `tests/fuzz/regressions/` and turned into a deterministic regression test.

---

## 8. Security-finding regression workflow

1. The Security Auditors write findings to `security/auditor-a/` and `security/auditor-b/`. The merged, de-duplicated list goes in `security/merged/` with stable IDs (e.g. `SEC-001`).
2. For each merged finding, QA adds a row to `qa/FINDINGS_TRACKER.md` and writes a test **named after the ID**:
   - Rust: `tests/regression/sec_001_<slug>.rs`, `#[test] fn sec_001_<slug>()`
   - TS: `tests/regression/SEC-001-<slug>.test.ts`
   - Fuzz seed (if any): `tests/fuzz/regressions/SEC-001.seed`
3. The test must **fail on the pre-fix commit**. Record the commit hash and "FAIL" in "status before fix".
4. After the fix is merged, the test must **pass**. Record the commit and "PASS" in "status after fix". The finding is only closed when both columns are filled.
5. Regression tests run on every CI build forever and are never deleted. If the behavior changes on purpose, update the test and note it in the tracker.
6. Findings that can't be tested (docs or process issues) get the note "N/A – non-testable" plus a reviewer sign-off.

---

## 9. End-to-end plan: Next.js site _(prov.; `app/` is empty)_

- **Stack:** Playwright (`@playwright/test`) run with yarn.
- **Environments:**
  - (1) localnet: `solana-test-validator` with the programs and upstream clones, seeded by a fixture script.
  - (2) devnet: nightly/manual, using only devnet airdropped throwaway keys.
- **Wallet mocking:** inject a test wallet into `@solana/wallet-adapter` (a custom `MockWalletAdapter` implementing the `connect` and `signTransaction`/`signAllTransactions`/`signMessage` interface, backed by a localnet-only keypair from `tests/.keys/`). Wallet Standard mock as an alternative. Also cover: user rejects signing, wallet disconnects mid-flow, wrong network (mainnet-selected wallet must show a blocking warning and never send).
- **Viewports:** Desktop Chrome 1440×900, Desktop Firefox/WebKit smoke, Mobile Pixel 7 and iPhone 14 (Playwright device profiles).
- **Flows:**

| ID | Flow | Checks |
|----|------|--------|
| E2E-01 | Connect / disconnect wallet | address shown truncated; reconnect after reload |
| E2E-02 | Capture (buy NFT with tokens) | price + token fee + SOL fees + Token-2022 transfer fee shown **before** signing, matching the on-chain result; NFT appears after confirmation |
| E2E-03 | Release (NFT → tokens) | expected tokens after fees shown; NFT disappears; balance updates |
| E2E-04 | Insufficient balance / escrow empty / paused | clear error, disabled button, no tx sent |
| E2E-05 | Treasury dashboard | harvested, withdrawn, spent and NFT count match on-chain state (INV-04) |
| E2E-06 | Lottery: tickets view | `floor(balance/threshold)` shown; tooltip explains threshold; updates after a transfer (before snapshot only) |
| E2E-07 | Lottery: claim prize | only winners see claim; double-click sends a single tx; already-claimed state |
| E2E-08 | Tx failure / timeout / blockhash expired | retry UX, no duplicate submissions |
| E2E-09 | Network guard | app refuses mainnet RPC in test builds |

- **Usability checks:**
  - Axe accessibility scan (`@axe-core/playwright`) on each page.
  - Keyboard-only navigation of the swap and claim flows.
  - Tap targets ≥ 44px on mobile.
  - No horizontal scroll at 375px.
  - Loading and pending states for every tx.
  - Human-readable amounts (decimals) with raw amounts in details.
  - Visual snapshots per viewport.
  - Lighthouse performance smoke.

---

## 10. Proposed `tests/` layout

```
tests/
├── .keys/                  # LOCALNET/DEVNET-ONLY throwaway keypairs (gitignored)
├── fixtures/               # mint/collection/recipe setup, dumped upstream .so files
│   └── programs/           # mpl_hybrid.so, mpl_core.so (dumped from devnet)
├── common/                 # shared helpers: assert_invariants(), balance diff, fee math mirror
├── unit/                   # pure math: fee calc, tickets, prize split
├── swap/                   # component A integration (Rust/LiteSVM + TS)
├── treasury/               # component B integration, Token-2022 extension matrix
├── lottery/                # component C integration
├── integration/            # cross-component flows (§6)
├── regression/             # SEC-xxx tests (one file per finding ID)
├── fuzz/                   # Trident targets (generated by `trident init`/`fuzz add`)
│   └── regressions/        # crash seeds
└── e2e/                    # Playwright specs, mock wallet, playwright.config.ts
    ├── specs/
    └── wallet-mock/
```

---

## 11. Open questions (block finalizing tests)

- **Q1** Is the swap token the same Token-2022 tax mint? Upstream MPL-Hybrid uses classic SPL Token, and a transfer fee breaks escrow backing (INV-03). Options: a separate non-taxed swap mint, a custom Token-2022-aware swap program, or fee-exempt escrow routing.
- **Q2** How does the treasury buy NFTs: via swap capture, a marketplace, or direct mint?
- **Q3** Which operations are allowed while paused (release, claim, withdraw_fees)?
- **Q4** Which accounts are excluded from the lottery (escrow, treasury, LPs)? Is there a snapshot or min-hold rule?
- **Q5** What is the randomness source and the timeout/fallback policy?
- **Q6** Who holds the withdraw-withheld / fee config / upgrade authorities (PDA vs multisig)?

---

## 12. Coverage status

| Area | Section | Status |
|------|---------|--------|
| Global invariants helper (`assert_invariants`) | §2 | not started |
| Swap: initialize_config / create_recipe | §3 | not started |
| Swap: capture | §3 | not started |
| Swap: release | §3 | not started |
| Swap: fees / pause / withdraw / close | §3 | not started |
| Token-2022 fee math | §4.2 | not started |
| Token-2022 harvest / withdraw | §4.3 | not started |
| Treasury buy_nft | §4.3 | not started |
| Token-2022 extension matrix | §4.4 | not started |
| Lottery: init / rounds / snapshot | §5 | not started |
| Lottery: randomness & draw | §5 | not started |
| Lottery: claims / unclaimed | §5 | not started |
| Cross-component flows | §6 | not started |
| Fuzz: fuzz_swap | §7 | not started |
| Fuzz: fuzz_treasury | §7 | not started |
| Fuzz: fuzz_lottery | §7 | not started |
| Fuzz: fuzz_e2e | §7 | not started |
| Security regression tests | §8 | not started (no findings yet) |
| E2E desktop | §9 | not started |
| E2E mobile | §9 | not started |
| Usability / a11y | §9 | not started |
