# Launchpad QA Test Plan

Status: **DRAFT v0.2** (2026-09-24). Owner: QA.
Networks: **localnet and devnet only.** Nothing in this plan deploys to mainnet or uses real funds or keys.

**Changelog**
- v0.2 (2026-09-24): Q1 resolved by Barton. The product is split into two tracks that never share a mint. Track A is SPL-404 (classic SPL Token + MPL-Hybrid). Track B is a Token-2022 tax token + treasury + raffle for existing collections. Scope, invariants, matrices, fuzzing, E2E, layout and coverage are restructured per track, and track-separation tests are added (INV-13…16, SEP-xx).
- v0.1 (2026-09-24): initial draft.

---

## 1. Scope and assumptions

### 1.1 Product tracks (decision, Q1 resolved)

The launchpad has **two separate product tracks that never share a mint.**

| Track | Who it's for | Token | On-chain mechanics | Not included |
|-------|-------------|-------|--------------------|--------------|
| **Track A: SPL-404 launch** | **new** NFT collections | **classic SPL Token** mint (Token program), no transfer tax | Metaplex **MPL-Hybrid** capture/release. **Capture**: a user pays tokens (+ fees) and gets an NFT from escrow. **Release**: a user returns an NFT and gets tokens back (− fees). | no Token-2022, no transfer tax, no treasury buys, no raffle (see Q7) |
| **Track B: Token-2022 tax option** | **existing** NFT collections | **Token-2022** mint with the **TransferFee** extension ("tax"), launched on the launchpad | Withheld tax is harvested and withdrawn into a **treasury** that **buys NFTs from the existing collection**. Holders get a **ticket raffle**: `tickets = floor(balance / threshold)`. | **no MPL-Hybrid swap**, no capture/release, no escrow of NFTs for swapping |

Components by track:

| # | Component | Track | Short name |
|---|-----------|-------|------------|
| A1 | Hybrid swap wrapper around MPL-Hybrid (config, recipe, escrow, capture, release, fees) | A | `hybrid` |
| B1 | Token-2022 tax token launch + fee harvest/withdraw | B | `tax` |
| B2 | Treasury that buys NFTs from the configured existing collection | B | `treasury` |
| B3 | Holder ticket raffle | B | `raffle` |
| S1 | Launchpad registry / track routing (if it exists; see Q9) | shared | `registry` |
| S2 | Next.js site with a separate launch flow per track | shared | `app` |

Out of scope for now: mainnet deployment, audits themselves (those belong to the Security Auditors; QA only turns their findings into regression tests), and load testing beyond basic compute-unit budgets.

### 1.2 Assumptions (read first)

- **No code or spec exists yet.** `programs/`, `app/` and `docs/` are empty. **Every instruction, account and PDA name in this document is PROVISIONAL** and marked _(prov.)_. Rename the tests when the real IDL lands, but keep the test IDs.
- **ASSUMPTION (unconfirmed): the raffle/lottery exists only on Track B.** Track A has no raffle. This is **open question Q7**. If it turns out Track A also gets a raffle, §5 and INV-05/06/07 must be extended to classic SPL Token mints.
- The two tracks never share a mint. This plan also assumes they never share any token account, vault, escrow, treasury or PDA (INV-15). Whether both tracks live in one program or several is not decided yet _(prov.)_.
- Upstream facts checked against `metaplex-foundation/mpl-hybrid` @ `aacf1a5` (2026-05-27), which apply to **Track A only**:
  - Program ID `MPL4o4wMzndgh8T1NVDxELQCj5UQfYTYEkabX3wNKtb`.
  - Instructions: `init_recipe_v1`, `init_escrow_v1/v2`, `init_nft_data_v1`, `capture_v1/v2`, `release_v1/v2`, `update_recipe_v1`, `update_escrow_v1`, `update_new_data_v1`, `migrate_nft_v1`, `migrate_tokens_v1`.
  - `RecipeV1` fields: `amount`, `fee_amount_capture`, `fee_amount_release`, `sol_fee_amount_capture`, `sol_fee_amount_release`, `fee_location`, `min`/`max`, `count`, `path`.
  - Capture and release also charge a fixed Metaplex protocol SOL fee.
  - NFTs are **MPL Core** assets.
  - Optional re-roll randomness is derived from the **SlotHashes sysvar plus the timestamp**.
  - Upstream accounts use `Program<'info, Token>` (**classic SPL Token**), which matches the Track A decision.
- Track B's existing collections may use NFT standards other than MPL Core (Token Metadata NFTs, programmable NFTs, compressed NFTs). Which ones are supported is **Q8**.
- How the Track B treasury buys NFTs (marketplace, listings, bids, which currency) is **Q2**. The treasury tests in §4.4 are written generically.
- Randomness for the raffle is **not chosen yet**. Candidates are Switchboard On-Demand randomness, ORAO VRF, or commit-reveal. The tests below cover each option as _(prov.)_.
- Toolchain on the shared box:
  - Agave/Solana CLI 4.1.2. Anchor 1.2.0 pinned it; the installer's `stable` was 4.2.2.
  - Anchor CLI 1.2.0 via avm, Trident 0.12.0, yarn 1.22.22, Node 20.19.2, SBF platform-tools v1.57.
  - rustup `stable` is 1.98.1. The distro Rust 1.85.1 is still at `/usr/bin` and is available as rustup toolchain `system`.

### 1.3 Test levels and tooling

| Level | Tool | Where it runs |
|-------|------|---------------|
| Unit (pure math: Track A fees; Track B transfer fee, tickets, prize split) | `cargo test` on host | local |
| Program integration | Rust + LiteSVM (Anchor 1.2 `litesvm` test template), plus the Anchor TS test runner against `solana-test-validator` | local |
| Upstream programs | Track A: MPL-Hybrid + MPL Core. Track B: Token-2022, plus whatever NFT/marketplace programs Q2/Q8 require. Loaded with `[[test.genesis]]` or `solana program dump -u devnet` (read-only dump). | localnet |
| Fuzzing | Trident 0.12 (`trident fuzz run <target>`) | local |
| Security regression | one test per finding ID (§8) | CI + local |
| E2E / UI | Playwright + mocked wallet adapter (§9) | localnet, then devnet |

---

## 2. Invariants (per track)

Every integration test ends by calling the `assert_invariants()` helper for its track. The Trident fuzz targets check the same invariants after every flow. IDs from v0.1 are kept. INV-11…16 are new in v0.2.

### 2.1 Track A: SPL-404 / MPL-Hybrid

| ID | Track | Invariant | Check |
|----|-------|-----------|-------|
| **INV-01** Token supply conserved across capture/release | A | `tokens_in_swap_escrow + circulating_tokens + tokens_burned == mint.supply (+ minted − burned)`. No transfer fee on Track A, so no withheld term. | Read every token account for the mint plus `mint.supply` before and after each ix. The delta must be exactly `recipe.amount` and the token fees. |
| **INV-02** NFT count conserved | A | `nfts_in_escrow + nfts_held_by_users (+ nfts_burned if path=burn) == collection.num_minted` | Count MPL Core assets by owner (escrow vs others). |
| **INV-03** Escrow backing | A | `escrow_token_balance == recipe.amount × nfts_currently_out_of_escrow + initial_seed` (exact, because there's no transfer fee) | Compare derived vs actual balances. |

### 2.2 Track B: Token-2022 tax, treasury, raffle

| ID | Track | Invariant | Check |
|----|-------|-----------|-------|
| **INV-11** Tax token supply conserved _(new; was the withheld part of v0.1 INV-01)_ | B | `Σ holder account amounts + Σ withheld in token accounts + mint.withheld + treasury balance (+ burned) == mint.supply` | Read all token accounts' `amount` and `TransferFeeAmount.withheld_amount` plus the mint's withheld amount. |
| **INV-12** Treasury NFT custody _(new, prov.)_ | B | Every NFT the treasury buys belongs to the configured existing collection (verified membership). The number held by the treasury, or paid out as prizes, equals the recorded `nfts_bought`. | Collection verification of each acquired NFT, counted against treasury state. |
| **INV-05** Tickets exact | B | For every eligible holder in a snapshot: `tickets == floor(balance_at_snapshot / threshold)`, using the raw `amount` without withheld. `total_tickets == Σ tickets`. No tickets for a balance after the snapshot. | Recompute off-chain from snapshot data. |
| **INV-06** No prize paid twice | B | Each `(round, winner_slot)` pays at most once. `Σ prizes_paid ≤ prize_pool_funded`. | Check claim-receipt PDAs and pool balance. |
| **INV-07** No double draw | B | Each round goes through `Open → Snapshotted → RandomnessRequested → Drawn → Closed` exactly once. `draw` can't run twice or with stale or reused randomness. | Check the state machine and randomness account. |

### 2.3 Both tracks (each checked within its own track)

| ID | Track | Invariant | Check |
|----|-------|-----------|-------|
| **INV-04** Vault/treasury reconciliation | A + B | Every vault and treasury token/SOL balance equals the counters recorded in its state account. **A:** fee vaults. **B:** `treasury.total_withdrawn − total_spent == balance` _(prov. fields)_. | Compare the on-chain account with the state account after each ix. |
| **INV-08** Fees only to configured recipients | A + B | **A:** token fees go to `recipe.fee_location`, SOL fees to the project fee account and the Metaplex protocol fee wallet (+ `config.fee_recipient` if the wrapper adds one). **B:** transfer tax moves only token account → mint (harvest) → treasury (withdraw), and launchpad fees only to configured recipients. Nothing else changes balance except payer, escrow/treasury and user. | Diff all touched accounts' balances per tx. |
| **INV-09** Authority immutability | A + B | Only the configured admin/authority can change config, fees, pause or authorities. Authority changes need the current authority's signature. Track A and Track B authorities are configured separately. | Negative tests plus fuzz. |
| **INV-10** No stuck funds on close | A + B | Closing any PDA returns its rent and balances to the configured destination. Nothing can close while it holds user value. | Close tests. |

### 2.4 Track separation _(new in v0.2)_

| ID | Track | Invariant | Check |
|----|-------|-----------|-------|
| **INV-13** Track A accepts only classic SPL Token mints | A | Track A config/recipe creation accepts a mint only if its owner is the classic Token program. It rejects **every** Token-2022 mint, including one with **zero extensions**, and therefore any mint with extensions. | Owner check; SEP-01…04. |
| **INV-14** Track B rejects Track A mints | B | Track B tax/raffle/treasury config accepts only launchpad-created Token-2022 mints with TransferFee. It rejects any classic SPL mint, and any mint already registered to a Track A launch (if a registry exists, Q9). | SEP-05…08. |
| **INV-15** Nothing shared across tracks | A + B | The sets of mints, token accounts, vaults, escrows, treasuries, config accounts and PDAs used by Track A and Track B never overlap. PDA seeds are namespaced per track _(prov.)_, so a Track A PDA can never pass as a Track B account or the reverse. | SEP-09…12; fuzz cross-feeding. |
| **INV-16** UI keeps the tracks apart | app | The site shows Track A and Track B as distinct launch options and token types. It **never offers a swap/capture/release action for a Track B token** (UI, deep links or API), and never offers tax/treasury/raffle UI for a Track A token (unless Q7 changes that). | E2E-S01…S05. |

---

## 3. Track A: SPL-404 hybrid swap (classic SPL Token + MPL-Hybrid) _(prov.)_

### 3.1 Provisional instruction list

`initialize_config`, `create_recipe` (CPI `init_recipe_v1` / `init_escrow_v2`), `fund_escrow_nfts`, `fund_escrow_tokens`, `capture` (CPI `capture_v2`), `release` (CPI `release_v2`), `update_fees`, `set_paused`, `withdraw_fees`, `close_recipe`.

### 3.2 Test matrix

Columns: **HP** happy path · **AC** access control/signer · **PDA** wrong accounts/PDA spoofing · **MINT** wrong mint/token program · **MATH** overflow/rounding · **CPI** reentrancy/CPI ordering · **PAUSE** paused/closed state · **EDGE** zero/max amounts.

| Instruction | HP | AC | PDA | MINT | MATH | CPI | PAUSE | EDGE |
|-------------|----|----|-----|------|------|-----|-------|------|
| `initialize_config` | config created, admin set | non-payer or admin mismatch fails; can't re-init | config PDA with wrong seeds or bump rejected; a Track B config PDA passed in is rejected (INV-15) | n/a | fee bps > 10_000 rejected | n/a | n/a | fee = 0 allowed? (spec) |
| `create_recipe` | recipe and escrow created, CPI succeeds | only admin; collection update authority must match | fake collection, fake escrow, recipe for another collection | **token must be a classic SPL Token mint (INV-13)**: any Token-2022 mint is rejected, with or without extensions; token ≠ configured mint rejected | `min > max`, `max − min` overflow | CPI to MPL-Hybrid uses the expected program ID (reject spoofed program account) | fails when paused | amount = 0, amount = u64::MAX |
| `fund_escrow_nfts` / `_tokens` | escrow counters go up, INV-02/03 hold | only admin (or open, per spec) | NFT from another collection rejected; escrow must be the recipe's PDA | wrong mint rejected; Token-2022 token account rejected | counter overflow | n/a | fails when closed | 0 NFTs / 0 tokens |
| `capture` | user pays `amount + fee_amount_capture` (+ SOL fees) and receives exactly 1 NFT. `recipe.count` += 1. INV-01/02/03/08 hold. | user must sign; can't capture on another user's behalf with their tokens | spoofed escrow, recipe, fee_location or SlotHashes account rejected (upstream checks `SLOT_HASHES` address) | user ATA for wrong mint; Token-2022 program or token account passed in → rejected | `amount + fee` overflow; insufficient balance fails cleanly with no partial transfer | CPI order: token pull → NFT transfer → fees, all atomic; the failure of any leg reverts everything; no callback into the wrapper (MPL Core plugins/hooks) | fails when paused or recipe closed | escrow empty (0 NFTs) → clean error; balance exactly equal to the price; `fee = 0` |
| `release` | user returns their NFT and gets `amount − fee_amount_release` (per spec) tokens. The NFT goes to escrow, or is burned if `path` says burn. | only the NFT owner; delegate handling per spec | NFT from another collection; fake escrow token account | wrong mint; Token-2022 accounts rejected | fee > amount → reject, no underflow | same atomicity as capture | fails when paused (users may need release while paused → Q3) | escrow token balance < amount → clean failure; last NFT |
| `update_fees` | new fees apply to the next tx only | only admin | wrong config PDA | n/a | bps > max rejected | n/a | allowed when paused? | 0 and max caps |
| `set_paused` | toggles and emits an event | only admin / pauser | spoofed config | n/a | n/a | n/a | idempotent | n/a |
| `withdraw_fees` | only to the configured recipient, INV-08 | only admin | recipient ≠ configured rejected | wrong mint vault | withdraw > balance | n/a | allowed when paused (spec) | withdraw 0 / all |
| `close_recipe` | rent returned, INV-10 | only admin | n/a | n/a | n/a | n/a | only when escrow is empty or drained per spec | n/a |

Extra Track A cases:
- **Price/fee front-running:** admin changes the fee in the same slot as a user capture. The user should pass `max_price` / `expected_fee` _(prov.)_ and the tx must fail if it's exceeded.
- **Re-roll randomness** (if the `path` flag is used): SlotHashes + timestamp can be predicted or influenced by the leader. Test that a user can't pick their index by simulating first and retrying. Documented as a known-weak source; accepted only for cosmetic traits.
- **MPL Core plugin interaction:** an NFT with a freeze or transfer-delegate plugin gets returned. Release must fail, or must not let the user keep control.
- **Mint authorities** _(spec)_: whether the launch requires the SPL mint's mint/freeze authority to be revoked or held by a PDA. A frozen escrow token account would break INV-03, so test freeze behavior against whatever the spec decides.
- **Compute budget:** capture and release stay under the CU limit with max-length name/uri.

---

## 4. Track B: Token-2022 tax token + treasury _(prov.)_

### 4.1 Provisional instruction list

- `launch_tax_token`: creates a Token-2022 mint with TransferFee. `transfer_fee_config_authority` and `withdraw_withheld_authority` are set per Q6, ideally to a program PDA.
- `initialize_treasury`: binds the tax mint + the existing collection + the treasury PDA.
- `harvest_withheld`: wraps `HarvestWithheldTokensToMint` (permissionless) or `WithdrawWithheldTokensFromAccounts`.
- `withdraw_withheld_to_treasury`: wraps `WithdrawWithheldTokensFromMint`, which needs the withdraw-withheld authority.
- `buy_nft`: treasury buys an NFT from the existing collection. Venue and currency are Q2.
- `update_treasury_params`, `set_paused`.

### 4.2 Track B test matrix (same columns as §3.2)

| Instruction | HP | AC | PDA | MINT | MATH | CPI | PAUSE | EDGE |
|-------------|----|----|-----|------|------|-----|-------|------|
| `launch_tax_token` | mint created with the configured bps and max fee; authorities = expected PDAs | only the launch creator/admin | Track A config/PDA passed in → rejected (INV-15) | classic Token program passed → rejected (INV-14); disallowed extensions (§4.5) not added | bps > 10_000 rejected; max_fee = 0 behaviour defined | n/a | fails when paused | bps = 0 (allowed?), bps = 10_000, max_fee = u64::MAX |
| `initialize_treasury` | treasury bound to mint + collection | only admin | treasury PDA with wrong seeds; a Track A escrow as treasury → rejected | **classic SPL (Track A) mint rejected (INV-14)**; mint without TransferFee rejected; mint with a disallowed extension rejected | n/a | n/a | n/a | collection with 0 or 1 items |
| `harvest_withheld` | see §4.3 | permissionless (per Token-2022) | accounts of another mint in the list | Track A mint accounts rejected | Σ withheld overflow | n/a | allowed when paused? (Q3) | 0 accounts; max accounts per tx |
| `withdraw_withheld_to_treasury` | see §4.3 | only the withdraw-withheld authority | destination ≠ treasury ATA rejected | wrong mint | n/a | n/a | allowed when paused? (Q3) | withheld = 0 |
| `buy_nft` | see §4.4 | crank/admin per spec | NFT outside the configured collection; spoofed seller/listing/marketplace accounts | payment mint ≠ configured | price + fees overflow; tax on the payment leg (§4.4) | venue CPI ordering: pay only if the NFT arrives, all atomic | fails when paused | treasury balance exactly equal to the price; price = 0 |
| `update_treasury_params` / `set_paused` | applied, event emitted | only admin | spoofed config | n/a | bounds checks | n/a | idempotent | n/a |

### 4.3 Transfer-fee math, harvest and withdraw

Token-2022 computes `fee = min(ceil(amount × bps / 10_000), maximum_fee)`, and `fee = 0` when `bps == 0` or `amount == 0`. The recipient receives `amount − fee`, and the fee is **withheld in the recipient's token account**.

| ID | Case | Expected |
|----|------|----------|
| T22-FEE-01 | amount = 1, bps = 1 | fee = 1 (ceil), recipient gets 0 |
| T22-FEE-02 | amount where `amount × bps / 10_000` is exactly an integer | no rounding up |
| T22-FEE-03 | fee would exceed `maximum_fee` | fee == maximum_fee exactly |
| T22-FEE-04 | bps = 10_000 (max) | recipient gets 0 or `amount − max_fee` |
| T22-FEE-05 | amount = u64::MAX | no overflow (Token-2022 uses u128 internally); our off-chain/on-chain mirrors must too |
| T22-FEE-06 | `transfer_checked_with_fee` with the wrong expected fee | rejected |
| T22-FEE-07 | fee schedule change: new `TransferFeeConfig` only takes effect at `newer_transfer_fee.epoch` (≥ 2 epochs later) | program and UI read the **current epoch's** fee, not the newest config |
| T22-FEE-08 | our fee mirror (UI preview, treasury accounting) vs actual withheld amount | equal for 10k random (amount, bps, max_fee) triples |
| T22-HV-01 | harvest from N accounts to the mint (permissionless) | `mint.withheld` += Σ; each account's withheld amount goes to 0; INV-11 holds |
| T22-HV-02 | harvest with accounts of a **different mint** in the list | skipped or rejected; no cross-mint movement |
| T22-HV-03 | withdraw from the mint to the treasury by a non-authority | fails |
| T22-HV-04 | withdraw to a destination ≠ treasury PDA ATA | rejected by our wrapper (INV-08) |
| T22-HV-05 | treasury counters after harvest + withdraw | INV-04: `treasury.total_withdrawn == Δ treasury balance` |
| T22-HV-06 | the treasury ATA itself receives a taxed transfer (so it has withheld fees) | accounted for, not double counted |
| T22-HV-07 | closing a token account with withheld > 0 | Token-2022 blocks it; UI explains |
| T22-HV-08 | harvest with a very long account list | stays under tx size/CU limits; chunking works |

### 4.4 Treasury buys from the existing collection _(prov., depends on Q2/Q8)_

v0.1's "treasury buys via swap capture" is **removed**, because Track B has no MPL-Hybrid swap.

| ID | Case | Expected |
|----|------|----------|
| T22-BUY-01 | `buy_nft` happy path | treasury pays → NFT lands in the treasury-owned account; INV-04/08/11/12 hold |
| T22-BUY-02 | caller permissions | per spec (permissionless crank or admin); a caller can never redirect the NFT or the payment |
| T22-BUY-03 | treasury balance < price + fees | clean failure, no partial spend |
| T22-BUY-04 | if the treasury pays in the tax token: the transfer fee applies on the treasury → seller leg | seller receives `price − fee`; treasury accounting records gross vs net (INV-04); price shown to the seller is correct |
| T22-BUY-05 | if the treasury converts to SOL/other first (e.g. via a DEX, Q2) | slippage limit enforced; no sandwich-able unbounded swap; conversion proceeds reconcile (INV-04) |
| T22-BUY-06 | NFT not in the configured collection (unverified or fake collection, look-alike metadata) | rejected (INV-12) |
| T22-BUY-07 | collection standard specifics (Q8): programmable NFT rule sets/royalties, frozen/delegated NFTs, compressed NFT proofs | purchase succeeds with rules respected, or fails cleanly; treasury never ends up with a frozen/delegated NFT it can't control |
| T22-BUY-08 | the same listing bought twice / a stale listing | the second attempt fails; no double payment |
| T22-BUY-09 | price cap / max spend per period _(prov.)_ | a buy above the cap is rejected |

### 4.5 Token-2022 extensions that can break assumptions (Track B mint)

Each test creates a mint with the extension. The launch or `initialize_treasury` must **reject** it, or handle it safely:

| Extension | Risk | Test |
|-----------|------|------|
| TransferHook | arbitrary CPI during transfers → reentrancy, or a hook that blocks the treasury | the mint's hook program is in an allowlist, or transfers fail safely; reentrancy attempt via a malicious hook fixture |
| PermanentDelegate | delegate can drain treasury/holder balances at any time | reject mints with a permanent delegate (or document the trust assumption); test that a drain breaks INV-04/11 and is detected |
| DefaultAccountState = Frozen / FreezeAuthority | treasury or holder accounts can be frozen → funds stuck | detect at init; test behavior when frozen |
| MintCloseAuthority | mint closed after supply hits 0 | init rejects, or documents it |
| ConfidentialTransfer / ConfidentialTransferFee | balances/fees not visible → snapshot and ticket math wrong | reject for tax/raffle mints |
| InterestBearing / ScaledUiAmount | UI amount ≠ raw amount → ticket threshold confusion | tickets use **raw** amounts; UI shows both |
| NonTransferable / CpiGuard / MemoTransfer on user accounts | CPI transfers fail | clear error messages; CpiGuard'd user account can't be pulled via CPI |
| Pausable (if present in the deployed Token-2022) | mint paused → all flows stop | graceful failure |
| TransferFee config authority changes the fee | tax rate can change later | T22-FEE-07 plus a UI warning |

---

## 5. Track B: holder ticket raffle _(prov.; assumed Track B only, Q7)_

### 5.1 Provisional instruction list

`initialize_raffle`, `open_round`, `snapshot` / `register_tickets`, `fund_prize_pool`, `request_randomness`, `settle_randomness` / `draw_winners`, `claim_prize`, `sweep_unclaimed`, `close_round`, `set_paused`.

### 5.2 Test matrix (same columns as §3.2)

| Instruction | HP | AC | PDA | MINT | MATH | CPI | PAUSE | EDGE |
|-------------|----|----|-----|------|------|-----|-------|------|
| `initialize_raffle` | config created for a Track B tax mint | only admin | seeds/bump; Track A config/PDA rejected (INV-15) | **Track A (classic SPL) mint rejected (INV-14)**; non-TransferFee T22 mint rejected; extensions (§4.5) | threshold = 0 rejected (div by zero) | n/a | n/a | threshold = 1, u64::MAX |
| `open_round` | round N+1, state Open | only admin | round PDA with wrong index | n/a | round counter overflow | n/a | fails when paused | n/a |
| `snapshot`/`register_tickets` | INV-05 exact | holder signs own registration (or crank) | registering someone else's account; token account not owned by claimed holder | wrong mint's token account; Track A token account rejected | `floor` correctness; Σ tickets overflow (u64) | n/a | only while Open/before cutoff | balance = threshold−1 → 0 tickets; = threshold → 1; = 2·threshold−1 → 1; u64::MAX |
| `fund_prize_pool` | pool funded from the configured source (treasury? Q10) | only admin/treasury | pool PDA spoofing | prize mint per Q10 | n/a | n/a | fails when paused | 0 / max |
| `request_randomness` | state → RandomnessRequested | only admin/crank | spoofed oracle/randomness account | n/a | n/a | oracle CPI ordering | fails when paused | request twice → rejected |
| `draw_winners` | winners chosen from ticket ranges, state → Drawn, INV-07 | anyone/crank per spec | randomness account not the one committed at request | n/a | `rand % total_tickets` bias check; total_tickets = 0 → no draw / rollover | n/a | n/a | 1 ticket total; winners > ticket holders |
| `claim_prize` | winner paid once, receipt PDA created, INV-06 | only the winner (or their ATA as destination) | fake receipt, wrong round | prize mint mismatch | prize split rounding (dust goes to pool/treasury, not lost); tax on a tax-token prize transfer (winner receives `prize − fee`, shown in the UI) | pay after marking claimed (checks-effects-interactions) | allowed when paused? (Q3) | claim twice → second fails |
| `sweep_unclaimed` | after deadline, unclaimed → configured destination | only admin | destination ≠ config | n/a | n/a | n/a | n/a | sweep before deadline fails; claim after sweep fails |
| `close_round` | rent returned, INV-10 | only admin | n/a | n/a | n/a | n/a | only after Drawn + claims/sweep | n/a |

### 5.3 Raffle-specific risks

- **Randomness manipulation:**
  - SlotHashes / clock / recent blockhash must be **rejected** as the raffle source. A test shows that a leader or a simulate-then-send attacker could predict or retry them.
  - For VRF / Switchboard: the randomness account is bound at request time, can't be swapped at settle time, can't be reused across rounds, and the revealed value can't be seen before the commit.
  - For commit-reveal: a withheld reveal (admin refuses to reveal) → timeout path.
- **Snapshot timing / flash-balance gaming:**
  - A user borrows or buys tokens right before the snapshot and sells after. Mitigations: time-weighted or min-hold rule, or snapshot at an unannounced/random slot _(prov.)_. The transfer tax raises the cost of this attack but doesn't remove it; include tax cost in the attack test.
  - Test: the same tokens moved between wallets A→B during the registration window must not count twice. Register A, transfer to B, register B → total tickets must not exceed `floor(eligible_supply/threshold)`. Registration must be finalized at one slot, or re-verified at draw.
- **Threshold edges:** threshold−1, threshold, threshold+1, k·threshold−1, balance 0, threshold > supply.
- **Rounding:** `floor` only, never round up. Prize splits: `Σ payouts + dust == pool`.
- **Withheld fees vs balance:** tickets use the spendable `amount` only. Withheld amounts in a holder's account don't count (INV-05).
- **Excluded accounts:** treasury, prize pool, LP/AMM pools and program-owned accounts should not get tickets (Q4).
- **Duplicate claims:** double claim in the same tx (two ixs), in parallel txs, and after a round is closed and re-opened.
- **Unclaimed prizes:** deadline, sweep destination, and what happens to the next round's pool.

---

## 6. Track separation and cross-component flows

### 6.1 Track-separation tests _(new; `tests/cross-track/`)_

| ID | Case | Expected | Invariant |
|----|------|----------|-----------|
| SEP-01 | Track A `create_recipe` / `initialize_config` with a Token-2022 mint **with TransferFee** | rejected | INV-13 |
| SEP-02 | Same with a Token-2022 mint with **no extensions at all** | rejected (owner check, not extension check) | INV-13 |
| SEP-03 | Same with Token-2022 mints carrying other extensions (hook, permanent delegate, metadata pointer…) | rejected | INV-13 |
| SEP-04 | Track A capture/release with the Token-2022 program ID or a Token-2022 token account substituted | rejected | INV-13 |
| SEP-05 | Track B `launch_tax_token` / `initialize_treasury` / `initialize_raffle` with a classic SPL mint | rejected | INV-14 |
| SEP-06 | Track B init with the **exact mint of an existing Track A launch** | rejected | INV-14 |
| SEP-07 | Track B init with a Token-2022 mint not created by the launchpad (external mint) | rejected (or allowed per spec, Q11) | INV-14 |
| SEP-08 | Track B `register_tickets` / `harvest_withheld` with Track A token accounts | rejected | INV-14 |
| SEP-09 | Pass a Track A escrow / config / recipe PDA where a Track B treasury / config / round PDA is expected | rejected (discriminator/seed/owner mismatch) | INV-15 |
| SEP-10 | Pass a Track B treasury / prize-pool PDA where a Track A escrow / fee vault is expected | rejected | INV-15 |
| SEP-11 | Derive all PDAs for one Track A and one Track B launch with the same creator/collection inputs | no address collisions (track-namespaced seeds) | INV-15 |
| SEP-12 | Withdraw fees on one track targeting a vault of the other track | rejected; balances unchanged | INV-08, INV-15 |
| SEP-13 | Track B targets a collection that was launched through Track A | behaviour per Q9 (allowed with fully separate accounts, or rejected) | INV-15 |

### 6.2 Integration flows

Track A:
1. Launch a new collection → create recipe → fund escrow → user capture → transfer NFT → second user release. INV-01/02/03/04/08 hold after every step.
2. Pause mid-flow → only allowed ops succeed (Q3) → unpause → state consistent.

Track B:
3. Launch tax token for an existing collection → taxed transfers between holders → harvest → withdraw → treasury `buy_nft` → NFT in treasury. INV-04/08/11/12 hold after every step.
4. Raffle round → snapshot → prize pool funded (source per Q10) → randomness → draw → claims → sweep. INV-04/05/06/07 hold.
5. Pause mid-flow → only allowed ops succeed (Q3) → unpause → state consistent.

Both tracks side by side:
6. Run one Track A launch and one Track B launch at the same time on the same validator. Interleave their operations and run SEP-09…12 attacks between steps. Each track's invariants hold independently, and no account appears in both tracks (INV-15).

---

## 7. Fuzzing plan (Trident 0.12)

- Setup: `trident init` in the Anchor workspace once `programs/` exists. Targets are split per track. Run with `trident fuzz run <target> [seed] --with-exit-code` in CI. Reproduce with `trident fuzz debug <target> <seed>`.
- Upstream programs are loaded into the Trident SVM as program binaries dumped from devnet _(prov., verify loading support in 0.12 docs)_. Track A needs MPL-Hybrid + MPL Core. Track B needs Token-2022, plus the NFT/marketplace programs from Q2/Q8.
- Actors: admin, 3–5 users, attacker (random signer), cranker.
- Inputs: random amounts biased toward edges (0, 1, threshold±1, max_fee boundary, u64::MAX), random account substitution (swap in a wrong PDA or mint 5% of the time), random ordering.

| Target | Track | Instructions fuzzed | Invariant assertions after each flow |
|--------|-------|--------------------|--------------------------------------|
| `fuzz_track_a_hybrid` | A | capture, release, update_fees, set_paused, withdraw_fees, fund_escrow_* | INV-01, 02, 03, 04, 08, 09; no ix succeeds with a spoofed account |
| `fuzz_track_b_tax_treasury` | B | taxed transfers, harvest_withheld, withdraw_withheld_to_treasury, buy_nft, fee schedule change | INV-04, 08, 09, 11, 12; our fee mirror == Token-2022 fee |
| `fuzz_track_b_raffle` | B | open_round, register_tickets, taxed transfers during the window, request/settle/draw, claim_prize, sweep_unclaimed | INV-05, 06, 07, 10; Σ tickets ≤ floor(eligible_supply/threshold) |
| `fuzz_track_b_e2e` | B | all Track B instructions interleaved | all Track B invariants + INV-04/08/09/10 |
| `fuzz_cross_track` | A + B | both tracks initialized; every ix randomly fed mints, token accounts and PDAs from the other track (and Token-2022 mints with random extensions into Track A) | INV-13, 14, 15: any cross-track input must fail, and both tracks' invariants still hold |

Exit criteria: each target runs ≥ 1M iterations (or 1 h) with no invariant failures or panics. Every crash seed gets saved under `tests/fuzz/regressions/` and turned into a deterministic regression test.

---

## 8. Security-finding regression workflow

1. The Security Auditors write findings to `security/auditor-a/` and `security/auditor-b/`. The merged, de-duplicated list goes in `security/merged/` with stable IDs (e.g. `SEC-001`).
2. For each merged finding, QA adds a row to `qa/FINDINGS_TRACKER.md` and writes a test **named after the ID** in `tests/regression/`. Add a track tag in the slug (`a_`, `b_`, `x_` for cross-track, `app_`):
   - Rust: `tests/regression/sec_001_<track>_<slug>.rs`, `#[test] fn sec_001_<slug>()`
   - TS: `tests/regression/SEC-001-<track>-<slug>.test.ts`
   - Fuzz seed (if any): `tests/fuzz/regressions/SEC-001.seed`
3. The test must **fail on the pre-fix commit**. Record the commit hash and "FAIL" in "status before fix".
4. After the fix is merged, the test must **pass**. Record the commit and "PASS" in "status after fix". The finding is only closed when both columns are filled.
5. Regression tests run on every CI build forever and are never deleted. If the behavior changes on purpose, update the test and note it in the tracker.
6. Findings that can't be tested (docs or process issues) get the note "N/A – non-testable" plus a reviewer sign-off.

---

## 9. End-to-end plan: Next.js site _(prov.; `app/` is empty)_

- **Stack:** Playwright (`@playwright/test`) run with yarn.
- **Environments:**
  - (1) localnet: `solana-test-validator` with the programs and upstream clones, seeded with fixtures for both tracks (a new Track A collection, and an "existing" collection + Track B tax token).
  - (2) devnet: nightly/manual, using only devnet airdropped throwaway keys.
- **Wallet mocking:** inject a test wallet into `@solana/wallet-adapter` (a custom `MockWalletAdapter` implementing the `connect` and `signTransaction`/`signAllTransactions`/`signMessage` interface, backed by a localnet-only keypair from `tests/.keys/`). Wallet Standard mock as an alternative. Also cover: user rejects signing, wallet disconnects mid-flow, wrong network (mainnet-selected wallet must show a blocking warning and never send).
- **Viewports:** Desktop Chrome 1440×900, Desktop Firefox/WebKit smoke, Mobile Pixel 7 and iPhone 14 (Playwright device profiles).

### 9.1 Shared / track separation

| ID | Flow | Checks |
|----|------|--------|
| E2E-01 | Connect / disconnect wallet | address shown truncated; reconnect after reload |
| E2E-S01 | Landing / "Launch" entry point | Track A ("SPL-404 for a new collection") and Track B ("Tax token for an existing collection") are shown as two distinct options with a clear explanation of each (INV-16) |
| E2E-S02 | Track B token page | **no** swap/capture/release button, tab or link anywhere; tax rate, treasury and raffle shown instead |
| E2E-S03 | Deep link / URL tampering: open the swap route with a Track B mint | swap is refused with a clear message; no tx is built or sent |
| E2E-S04 | Track A token page | no tax/treasury/raffle UI (unless Q7 changes); swap shown |
| E2E-S05 | Token search / explore lists | every token is labeled with its track; filters per track work |
| E2E-09 | Network guard | app refuses mainnet RPC in test builds |
| E2E-08 | Tx failure / timeout / blockhash expired (both tracks) | retry UX, no duplicate submissions |

### 9.2 Track A flows

| ID | Flow | Checks |
|----|------|--------|
| E2E-A01 | **Track A launch wizard**: create a new collection + classic SPL mint + recipe (price, fees, NFT range) | inputs validated (fee bounds, min ≤ max); summary before signing; the created mint is classic SPL; the wizard never offers Token-2022/tax options |
| E2E-02 | Capture (buy NFT with tokens) | price + token fee + SOL fees (project + protocol) shown **before** signing, matching the on-chain result; NFT appears after confirmation |
| E2E-03 | Release (NFT → tokens) | expected tokens after fees shown; NFT disappears; balance updates |
| E2E-04 | Insufficient balance / escrow empty / paused | clear error, disabled button, no tx sent |

### 9.3 Track B flows

| ID | Flow | Checks |
|----|------|--------|
| E2E-B01 | **Track B launch wizard**: pick an existing collection (ownership/authority proof per spec) + configure tax bps, max fee, threshold | the created mint is Token-2022 with TransferFee; bps/max-fee bounds validated; a clear warning that tax applies to every transfer; the wizard never offers swap/MPL-Hybrid options |
| E2E-B02 | Transfer preview for a Track B token | shown fee == Token-2022 fee (T22-FEE-08); recipient amount shown |
| E2E-05 | Treasury dashboard | harvested, withdrawn, spent and NFT count match on-chain state (INV-04, INV-12) |
| E2E-06 | Raffle: tickets view | `floor(balance/threshold)` shown; tooltip explains threshold; updates after a transfer (before snapshot only) |
| E2E-07 | Raffle: claim prize | only winners see claim; double-click sends a single tx; already-claimed state; net-of-tax amount shown if the prize is the tax token |

### 9.4 Usability checks (all pages, both tracks)

- Axe accessibility scan (`@axe-core/playwright`) on each page.
- Keyboard-only navigation of both launch wizards and the swap and claim flows.
- Tap targets ≥ 44px on mobile.
- No horizontal scroll at 375px.
- Loading and pending states for every tx.
- Human-readable amounts (decimals) with raw amounts in details.
- The track is always visible on token pages.
- Visual snapshots per viewport.
- Lighthouse performance smoke.

---

## 10. Proposed `tests/` layout

```
tests/
├── .keys/                      # LOCALNET/DEVNET-ONLY throwaway keypairs (gitignored)
├── shared/                     # used by both tracks
│   ├── helpers/                # balance diff, tx helpers, assert_invariants() common parts
│   ├── fixtures/               # validator setup, dumped upstream .so files (from devnet)
│   │   └── programs/
│   └── registry/               # launchpad registry / track routing tests (if S1 exists)
├── track-a-hybrid/             # Track A: classic SPL Token + MPL-Hybrid
│   ├── unit/                   # fee math
│   ├── config/                 # initialize_config, create_recipe, mint admissibility
│   ├── swap/                   # capture, release
│   ├── admin/                  # fees, pause, withdraw, close
│   └── integration/            # §6.2 flows 1–2
├── track-b-tax-raffle/         # Track B: Token-2022 tax + treasury + raffle
│   ├── unit/                   # transfer-fee mirror, tickets, prize split
│   ├── tax/                    # launch, harvest, withdraw (T22-FEE, T22-HV)
│   ├── extensions/             # §4.5 extension matrix
│   ├── treasury/               # buy_nft (T22-BUY)
│   ├── raffle/                 # §5
│   └── integration/            # §6.2 flows 3–5
├── cross-track/                # SEP-01…13, §6.2 flow 6
├── regression/                 # SEC-xxx tests (one file per finding ID, track-tagged)
├── fuzz/                       # Trident targets (generated by `trident init`/`fuzz add`)
│   └── regressions/            # crash seeds
└── e2e/                        # Playwright
    ├── specs/
    │   ├── shared/             # E2E-01, E2E-S01…S05, E2E-08/09
    │   ├── track-a/            # E2E-A01, E2E-02…04
    │   └── track-b/            # E2E-B01/B02, E2E-05…07
    ├── wallet-mock/
    └── playwright.config.ts
```

---

## 11. Open questions

- **Q1: RESOLVED (2026-09-24, Barton).** The swap token and the tax token are never the same mint.
  - **Track A** (new collections): classic SPL Token mint + MPL-Hybrid capture/release, no transfer tax.
  - **Track B** (existing collections): Token-2022 transfer-fee token whose harvested/withdrawn tax funds a treasury that buys NFTs from the existing collection, plus the holder ticket raffle. No MPL-Hybrid swap on Track B.
  - Reflected in §1.1, §2, §3–§7, §9, §10.
- **Q2** How does the Track B treasury buy NFTs from the existing collection? Which venue (marketplace CPI, own listings/bids, OTC)? Which currency (tax token, or converted to SOL, and if so how)? Is there a price/spend cap?
- **Q3** Which operations are allowed while paused, per track (release; harvest/withdraw; claim; withdraw_fees)?
- **Q4** Which accounts are excluded from the raffle (treasury, prize pool, LPs, program-owned)? Is there a snapshot or min-hold rule?
- **Q5** What is the raffle randomness source and the timeout/fallback policy?
- **Q6** Who holds the authorities (PDA vs multisig), per track? Track A: config/fees/upgrade. Track B: `transfer_fee_config_authority`, `withdraw_withheld_authority`, treasury/raffle admin, upgrade.
- **Q7** **Is the raffle Track B only?** This plan assumes yes, but it's unconfirmed. If Track A also gets a raffle, extend §5, INV-05/06/07 and E2E-S04.
- **Q8** Which NFT standards must Track B support for existing collections (MPL Core, Token Metadata NFTs, programmable NFTs, compressed NFTs)? How is collection membership verified?
- **Q9** Is there a shared launchpad registry/program that knows about both tracks, or are the tracks fully separate programs? Can a Track B token target a collection that was originally launched through Track A (SEP-13)?
- **Q10** What are the raffle prizes (NFTs bought by the treasury, tax tokens, SOL) and where does the prize pool come from?
- **Q11** Must a Track B mint be created by the launchpad, or can an existing Token-2022 mint be onboarded? Same question for Track A classic SPL mints.

---

## 12. Coverage status

| Area | Track | Section | Status |
|------|-------|---------|--------|
| `assert_invariants` helpers (A, B, cross-track) | shared | §2 | not started |
| Config / create_recipe / mint admissibility | A | §3 | not started |
| Capture | A | §3 | not started |
| Release | A | §3 | not started |
| Fees / pause / withdraw / close | A | §3 | not started |
| Track A integration flows | A | §6.2 | not started |
| Tax token launch | B | §4.2 | not started |
| Token-2022 fee math | B | §4.3 | not started |
| Harvest / withdraw to treasury | B | §4.3 | not started |
| Treasury buy_nft (existing collection) | B | §4.4 | not started (blocked on Q2/Q8) |
| Token-2022 extension matrix | B | §4.5 | not started |
| Raffle: init / rounds / snapshot | B | §5 | not started |
| Raffle: randomness & draw | B | §5 | not started (blocked on Q5) |
| Raffle: claims / unclaimed | B | §5 | not started |
| Track B integration flows | B | §6.2 | not started |
| Track separation (SEP-01…13) | A + B | §6.1 | not started |
| Side-by-side flow (§6.2 flow 6) | A + B | §6.2 | not started |
| Fuzz: fuzz_track_a_hybrid | A | §7 | not started |
| Fuzz: fuzz_track_b_tax_treasury | B | §7 | not started |
| Fuzz: fuzz_track_b_raffle | B | §7 | not started |
| Fuzz: fuzz_track_b_e2e | B | §7 | not started |
| Fuzz: fuzz_cross_track | A + B | §7 | not started |
| Security regression tests | all | §8 | not started (no findings yet) |
| E2E shared / track separation (E2E-S01…S05) | shared | §9.1 | not started |
| E2E Track A launch + swap | A | §9.2 | not started |
| E2E Track B launch + treasury + raffle | B | §9.3 | not started |
| E2E mobile viewports | shared | §9 | not started |
| Usability / a11y | shared | §9.4 | not started |
