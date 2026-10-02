# Review A — Cross-review of Review B round 1 (merge round)

- **Author:** Review A. **Date:** 2026-09-25 (MT). This round MAY read `docs/security/review-b/` (it did).
- **Inputs:** `docs/security/review-a/round1.md` (A-01..A-13 confirmed, design risks written there as B-01..B-08),
  `docs/security/review-b/round1.md` (R1-B-01..R1-B-14) + `threat-model.md`, `design-requirements.md`,
  `code-review-no-commits.md`, `sim/`; docs BRIEF (incl. Decisions 2026-09-25 4:27 PM MT), DECISIONS, ARCHITECTURE,
  THREAT_MODEL, hybrid-rarity-and-assignment, stonkfun-lessons; upstream source
  `reference/mpl-hybrid-aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/` (all upstream file:line refs
  are that tree); our code `programs/hybrid_launch/src/` (commit c02be68, local repo).
- **Environment:** local only. No deployment, no mainnet, no real keys or funds. One **read-only** devnet RPC read (Anchor
  IDL of the Switchboard On-Demand program) and public package downloads (npm `@switchboard-xyz/on-demand` 3.10.6, crate
  `switchboard-on-demand` 0.13.0) were used to verify the VRF claims.
- **Merged output:** `docs/security/merged/round1-merged.md` (M-01..M-35).
- **New PoC this round:** `docs/security/review-a/poc/03_fee_model.py` → `03_fee_model.out` (8/8 checks PASS).

## ID convention (to avoid a clash)
- **A-01..A-13** = Review A confirmed-in-code findings (upstream MPL-Hybrid @ aacf1a53).
- **A-D01..A-D08** = Review A's design risks, printed as "B-01..B-08" in `review-a/round1.md` §B. Renamed here only.
- **B-01..B-14** = Review B's **R1-B-01..R1-B-14** (`review-b/round1.md`). B's older `threat-model.md` IDs are written
  "B/TM-xx".
- **F-01..F-12** = new fee-design findings in this file (§5). Merged IDs M-xx are in `docs/security/merged/round1-merged.md`.

## What changed since both round-1 reports (settled by Barton — not reopened here)
1. **One graduation.** Converting is closed during the curve; at graduation liquidity migrates to a DEX, the full
   collection is minted into escrow paid from graduation proceeds, then converting opens (BRIEF.md:65-68). LP custody and
   batching/crank remain OPEN (engineer).
2. **Fee (BRIEF.md:70-76, 4:27 PM MT 2026-09-25)** supersedes burn: 2% of ratio on capture and each re-roll, in the
   collection token, on top of N, directly to ONE fixed fee address Barton controls; rate hard-capped at 2% in code, never
   raisable; re-roll also has a small fixed SOL minimum; destination change never, or multisig + 7-day timelock (TBD).
3. Curve default = existing audited venue (Meteora or Raydium) (BRIEF.md:78). Engine = custom `hybrid_vault`
   (BRIEF.md:54, ADR-008 Accepted).

---

## 1. Verification of B's code claims against upstream @ aacf1a53

| B claim | Where B makes it | Verdict | Evidence |
|---|---|---|---|
| Capturer names the asset | B-01 | **Confirmed** | `capture_v2.rs:52-54` (`/// CHECK: We check the asset bellow` / `asset: UncheckedAccount`); only collection membership checked at `:177-181`; our PoC 11 executed it on-chain (`poc/11_cherrypick.out`: "POC 11 EXECUTED: PASS") |
| Reroll index = `(u64(SlotHashes[12..20]) − ts) × count mod (max−min) + min` | B-01 | **Confirmed** | `capture.rs:167-183` (V1, uses `escrow.count/min/max`), `capture_v2.rs:187-203` (V2, `recipe.*`). Our PoC 01 11/11 PASS |
| Capture is CPI-callable (predict-and-abort wrapper possible) | B-01 | **Confirmed** | `grep` for `get_stack_height`, instructions sysvar, `TRANSACTION_LEVEL_STACK_HEIGHT` in `src/` returns nothing: no CPI guard anywhere |
| Metaplex protocol fee 0.005 SOL per leg | B-01 cost | **Confirmed** | `constants.rs:10-18`: `Rent::minimum_balance(590) + 2_720` = (590+128)×3,480×2 + 2,720 = **5,000,000 lamports** at default rent; charged at `capture_v2.rs:302`, `capture.rs:267`, `release.rs:247`, `release_v2.rs:293` |
| Index `max` unreachable, draws with replacement | B-14 / B/TM-16 | **Confirmed** | `capture_v2.rs:200-203` `checked_rem(max-min)+min`; PoC 01 T7: 10,000 draws → only 52.8% distinct |
| `update_recipe_v1` overwrites `token` and `fee_location` unconditionally | B-10 item 7 (quoting hybrid-rarity §3.1) | **Confirmed + executed** | `update_recipe.rs:91-93`; PoC 12 (`poc/12_token_swap.out`: recipe.token replaced by junk mint on an update with every Option `None`) |
| Authority check skipped with `NoRerollMetadata` | B-01 (via ADR-008) | **Confirmed, with nuance** | `capture_v2.rs:183-185` checks the signer only `if authority_info.key == &recipe.authority`; it is not path-gated. With `NoRerollMetadata` nothing else needs the authority, so the caller just passes any other account. Same effect as B says |
| MPL-Hybrid upgradeable by `mp14o4AQ…`, "Audit Pending" | B-10 item 7 | **Confirmed** | A-13 read-only `program show` 2026-09-24; also the deployed mainnet binary is 630,752 B vs 521,696 B for a local build of the pinned commit (B didn't have this) |
| "No program code exists" / repo empty | B header, `code-review-no-commits.md` §1 | **Stale — challenge** | `programs/hybrid_launch/` exists at commit c02be68 (BRIEF.md:64, DECISIONS ADR-010), with 5 unit + 25 LiteSVM + QA tests. It is in scope now, and it contradicts the new fee decision (F-01). `hybrid_vault` itself still has no code (true) |

A correction to our own report: A-11 lists four TODO sites. There is a fifth, `release.rs:99`
(`//Need to add account checks for security`).

---

## 2. Finding-by-finding: agree / disagree

| B finding | B sev | A position | A proposed sev | Notes and evidence |
|---|---|---|---|---|
| B-01 cherry-pick + predict-and-abort on stock MPL-Hybrid | Critical (cond.) | **Agree on substance, concede severity when conditional.** We rated the same bugs A-03/A-04/A-05 **High** under our key ("reliably biased draw"). B's cost comparison (565–5,652× cheaper rares) and the fact that the same deployment also carries A-01/A-02 drains justify **Critical *if MPL-Hybrid is in the swap path***. It isn't (ADR-008 Accepted, BRIEF.md:54), so the finding is **moot-if-hybrid_vault** and becomes a negative requirement | Critical (conditional) / moot | Executed on-chain by A: PoC 11 (cherry-pick), PoC 01 (predictability model). The fix is the same in both reports |
| B-02 Switchboard early view + `expire` refund | High | **Agree, and it's worse than B says.** B called "anyone can reveal" *unverified*. We verified it's **false as designed**: Switchboard `randomness_reveal` requires the randomness account's `authority` as a **signer** (devnet on-chain IDL, §3). If the requester creates the randomness account (the SDK default: `authority = payer`), **only the requester can reveal**. Our crank cannot, so selective abort works 100% of the time, not just when the crank is down. Recommended rule in §3 | High | Engineer doc still says the fee is refunded on expire: hybrid-rarity-and-assignment.md:184-187, 206-210, 253, 257, 356; DECISIONS.md:310-312 (N7); qa-answers.md:28-29, 74-75; TEST_PLAN RR-08 |
| B-03 launch sniping / bundles on the curve | High (cond.) | **Agree it's open. Disagree on severity: A holds Critical** (A-D05, T-CURVE-05). The launch window is where Stonk.fun harmed holders most (stonkfun-lessons #5), and nothing is designed yet. The default is now an audited venue (BRIEF.md:78), so the fix becomes "pick a venue with a program-enforced activation point + fee scheduler/caps and prove it", not our own code | A: Critical / B: High (**open**) | Hybrid amplification (fixed NFT supply) is **reduced** by decision 1: no captures during the curve, so snipers can't take NFTs pre-graduation. They still hold cheap tokens at graduation (see B-04 residual) |
| B-04 launch-window rare farming | High | **Mostly resolved by decision 1.** Converting (capture *and* re-roll) is closed during the curve (BRIEF.md:65). The residual is real but smaller: at graduation the floor `F = R × FDV_grad / 1B` is still tiny for small ratios (R=10k at 400 SOL FDV → F = 0.004 SOL), so the 2% token fee is ~0.00008 SOL per attempt and the **SOL minimum is the whole cost floor** (PoC 03 Part 1). Folded into F-02 | Medium (residual) | Status: resolved-by-decision + residual F-02 |
| B-05 fee sizing / no minimum | Medium | **Agree; partly resolved by the fee decision.** 2% is now fixed by Barton. But `hybrid_launch` today accepts **0..1,000 bps** (constants.rs:44-46, validation.rs:31-33), so a creator can still set 0% (free token part) or 10%. The rate must be the constant 200 bps (F-01). B's "floor ≥ 100 bps" is superseded by "exactly 200" | Medium | B's break-even numbers used 200 bps + 0.003 SOL, consistent with our PoC 03 |
| B-06 escrow cornering + head-of-line DoS | Medium | **Agree.** Minimum collection size is now 100 (BRIEF.md:61; validation.rs:30). Cornering N=100 at R=10k = 1M tokens (0.1% of supply) is still cheap. HOL DoS becomes cheap if fees are refunded on expire; our §3 rule (fee never refunded, batch expire/re-request) closes the fee part | Medium | — |
| B-07 wrap/unwrap arbitrage vs stale listings | Medium | **Partly disagree on severity:** this is market behaviour against third-party listings, with no protocol loss and no invariant broken. A: **Low** plus a mandatory disclosure (live pool census, unwrap value shown). Keep it open for B | A: Low / B: Medium (**open**) | — |
| B-08 MEV / sandwich around converts | Low (Med cond.) | **Agree.** A covered the swap leg as T-CURVE-06 (on-chain `min_out`/`max_in`) | Low | — |
| B-09 graduation / migration | High (cond.) | **Agree** (A-D06 High; A's T-CURVE-07 was Critical for a discretionary trigger). B adds pool-creation front-running and the price gap, which A missed. Decision 1 fixes the event shape; LP custody + crank are OPEN (engineer) | High (Critical if any operator signer or wallet in the path) | New interplay: graduation proceeds also fund the NFT mint (BRIEF.md:65), so the carve-out must be a program constant, not operator-routed (M-series) |
| B-10 operator / authority powers | High (Crit if single key) | **Agree; A missed the platform-wide upgrade-key point.** One `hybrid_vault` program for all collections means a single upgrade = drain of every collection's backing. The 7-day upgrade delay is now stated by the engineer (BRIEF.md:62), so B's ≥7-day ask is met on paper and needs to be verified on-chain. **New tie-in to the fee:** "rate can never be raised, not even by multisig" (BRIEF.md:74) is only true after `--final`; until then an upgrade could change it (F-08) | High | — |
| B-11 commitment excludes image bytes | Medium | **Agree.** BRIEF.md:68 now records it as a security note for the graduation reveal | Medium | — |
| B-12 burn vs no-burn contradiction | Low | **Superseded by the fee decision.** The burn side effects (supply shrinks, last NFTs stranded) are **moot**: nothing is burned now, so supply stays exactly 1B. It becomes a **doc/code-bug** finding: every burn/split reference is now wrong (§6 list) | Low (doc bug) / resolved-by-decision | — |
| B-13 manifest withholding freezes the FIFO | Medium | **Agree; partly resolved by decision 1.** If graduation mints the *full* collection into escrow with metadata bound on-chain before converting opens, no settle ever needs an off-chain proof. The residual is the **graduation mint crank stalling** (large N needs many transactions): converting never opens and proceeds sit in limbo. Needs a permissionless, resumable crank | Medium | — |
| B-14 carry-overs | Info | Agree | Info | Add: the value from `get_value()` is only readable in the reveal slot (§3) |

### A found; B missed (or covered only by citing docs)
- **A-01 / A-02 executed on-chain** (PoCs 10, 12): B cites the `update_recipe` issue from docs (B-10 item 7) but doesn't
  rate it as its own Critical or test it. Both drains are Critical if MPL-Hybrid is ever used; moot for `hybrid_vault`
  but they define its "no update instruction" requirement.
- **A-06** V2 escrow `["escrow", authority]` is shared across recipes → cross-collection insolvency
  (`init_escrow_v2.rs:11-14`). Relevant to `hybrid_vault` as "one vault per collection, per-collection accounting".
- **A-07** Burn paths destroy backing (`capture_v2.rs:261-272`, `release_v2.rs:191-208`).
- **A-09** `migrate_nft_v1.rs:91-92` discards the CPI result. **A-10** raw init / `init_if_needed`. **A-11** TODOs (5
  sites, see §1). **A-12** authority-semantics footgun. **A-13** deployed binary ≠ pinned build.
- **Curve math:** T-CURVE-02 (rounding favours the pool; no free buy/sell round trip), T-CURVE-04 (boundary overflow),
  T-CURVE-08 (one-shot, atomic graduation flag). These still apply if an audited venue is used, as integration tests.
- **A-D03 / A-D04:** FIFO/seq merge invariants and Feistel+Merkle ordering (root committed before the VRF seed) need
  fuzzing. B assumes them correct.

### B found; A missed
- **B-02** the `expire`-refund selective-abort mechanism and pool-shifting by selective expiry (A-D02 only named
  "selective reveal" in general).
- **B-04** launch-window farming economics (now mostly resolved by decision 1).
- **B-06** cornering + HOL DoS. **B-07** stale-listing arbitrage and pool adverse selection. **B-11** image bytes.
  **B-13** manifest withholding.
- **B-09** pool-creation front-running and the curve→AMM price gap. **B-10** one upgrade key over every vault, pause
  scope, creator allocations.

---

## 3. VRF-expiry reconciliation (engineer "refund if randomness never arrives" vs B-02 "bad-draw refund exploit")

### 3.1 The two positions
- **Engineer docs:** the fee is escrowed at request and refunded in full on `expire` if the VRF is unfulfilled after
  `deadline_slot`; "a requester who withholds a Switchboard reveal gains nothing (anyone can reveal)"
  (hybrid-rarity-and-assignment.md:184-187, 206-210, 253, 257, 356; DECISIONS.md:310-312 N7; qa-answers.md:28-29,
  74-75; docs/qa/TEST_PLAN.md RR-08). Note that under the 2026-09-25 decision the fee goes **directly** to Barton's fee
  address, so an escrow-then-refund flow is no longer even the natural implementation.
- **B-02:** the requester can see the value early, withhold a bad draw, and `expire` for a refund. That's a free abort,
  plus pool-shifting by selective expiry.

### 3.2 Facts about Switchboard On-Demand (verified 2026-09-25)
1. **The reveal needs the randomness *authority's* signature.** Read-only fetch of the on-chain Anchor IDL for
   `SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv` (devnet): `randomness_reveal` accounts = randomness, oracle, queue,
   stats, **authority (signer)**, payer (signer), recent_slothashes, …; `randomness_commit` also requires
   **authority (signer)**; `randomness_init` requires authority + payer + the randomness keypair as signers. There's no
   set-authority instruction (only init / commit / reveal / close).
   → **"Anyone can reveal" is false** unless the authority is a key or PDA that a permissionless path can sign for. With
   the SDK default (`Randomness.create(... payer)` sets `authority = payer`, npm 3.10.6
   `dist/esm/accounts/randomness.js:100-121`), only the requester can put the value on-chain.
2. **The value is observable off-chain before it lands, and not only by the requester.** `revealIx()` gets the oracle's
   signed value from the oracle gateway with `fetchRandomnessReveal({randomnessAccount, slothash, slot, rpc})`
   (`randomness.js:180-222`). Every input is public on-chain data and no requester credential is sent, so **anyone**
   (requester, our crank, a bot) can learn the value as soon as the seed slot passes. The oracle knows it before anyone.
   Switchboard's own tutorial names the selective-revelation attack and requires "take collateral at commit"
   (randomness-tutorial.md, Security Best Practices §1).
3. **The value is readable only in the reveal slot.** In crate `switchboard-on-demand` 0.13.0,
   `src/on_demand/accounts/randomness.rs:51-57`, `get_value(clock_slot)` errors unless `clock_slot == reveal_slot`.
   A `settle` sent in a later slot than the reveal can't read the value through the SDK. So reveal and settle must
   happen in the same instruction, or the program must copy the value into the request at reveal time.
4. **Oracle withholding.** One oracle is assigned at commit. If it's down, or chooses not to sign, nobody can reveal
   that commitment. An oracle that colludes with a requester can know the value first and withhold bad draws. That's a
   trust assumption in the provider (Switchboard's TEE + slashing model), and our program can't remove it. It can only
   make withholding **unprofitable** (no refund) and **visible** (monitoring).
5. **Freshness at commit.** The tutorial requires `seed_slot == clock.slot − 1` and "not already revealed" at commit.
   Without these, a stale or already-known randomness account can be used (B-02 scenario C).

### 3.3 Analysis
- **Who can see the result before it's on-chain?** Anyone who queries the gateway after the seed slot, and the oracle
  even earlier. Treat the result as **public from the moment of commit + 1 slot**, not from reveal.
- **Who can submit the reveal?** Only a signer for the randomness account's `authority`. If that's the requester,
  B-02's abort works **whenever the requester wants**, not just when our crank is down. That's strictly worse than B
  assumed.
- **What does a refund buy an attacker?** Any refund after the value became observable is a free option on the draw.
  That includes fee refunds, and principal-plus-fee refunds on a capture (B-02: 3–9× cheaper legendaries,
  `sim/reroll_ev_r1_output.txt` Part E). Selective expiry also lets a holder of k queued requests choose among up to 2^k
  pool states for later requests (B-02 scenario B).
- **Is a refund ever fair?** Only when no one could have observed the outcome: randomness never committed, or committed
  but provably never revealable. With the fee charged in the same instruction that CPI-commits, "fee paid but never
  committed" can't happen. "Committed but the oracle never answers" is the only honest failure, and it's
  indistinguishable on-chain from "oracle withheld a bad draw on purpose".

### 3.4 Recommended rule (for `hybrid_vault`)
1. **Program owns the randomness.** `request_capture` / `request_reroll` create the Switchboard randomness account via
   CPI with `authority = PDA["rng_authority", request]` (or a vault-wide PDA) and CPI-commit in the same instruction.
   Reject any caller-supplied randomness account whose `authority` isn't that PDA. Enforce `seed_slot == clock.slot − 1`
   (or the SDK's current rule) and "not yet revealed". Pin the randomness pubkey, `seed_slot` and `oracle` in the
   request.
2. **Fee charged at request, never refunded.** In the same instruction, transfer the 2% token fee and the SOL minimum
   **directly** to the fixed fee destinations (no escrow in the request PDA). There's no instruction that returns them.
   This supersedes DECISIONS N7 ("refunded on expire").
3. **Permissionless reveal-and-settle.** One instruction, `reveal_and_settle(request, oracle_sig, recovery_id, value)`,
   callable by **anyone**. It CPIs `randomness_reveal` signing as the PDA authority, then reads the value in the same
   slot and executes the reassignment. The outcome depends only on `(value, pinned pool snapshot)`, **never on who
   calls**. Our crank calls it within a few slots. The requester has no veto and no cancel.
4. **Oracle non-delivery → re-request by anyone, not a refund.** If still unrevealed at `deadline_slot`, anyone may call
   `recommit(request)`: a new randomness account (same PDA authority), a **different oracle** than every oracle already
   tried, and the **same seq / candidate set** as the original request. The fee is **not** charged again and **not**
   refunded. Cap it at `MAX_RECOMMITS = 3`.
5. **Principal-only escape, after the retry cap.** Only after `MAX_RECOMMITS` unrevealed commitments and a long deadline
   (≥ 1 day) may anyone call `expire`. It returns the **principal only**: the N locked tokens for a capture, or the
   handed-in NFT for a re-roll. The fee stays forfeited. For a re-roll this leaves the user where an honest "bad" draw
   would, so withholding gains nothing. Batch-expire consecutive stale heads (bounded loop) so the FIFO can't be stalled
   (B-06).
6. **Monitoring.** Every recommit/expire is an incident: alert, and publish the per-oracle abandon rate. If one oracle
   withholds repeatedly, that's evidence of collusion. Consider ORAO as the provider (nodes fulfil; no authority-signed
   reveal), which removes items 1 and 3 but not item 4's trust assumption.
7. **Verify on devnet before choosing Switchboard** (B-02's ask, still open): a third-party wallet runs
   `reveal_and_settle` on a request it didn't make, using the PDA-authority flow, and it succeeds.

Why re-request is safe relative to refund: a recommit gives a fresh draw only when the **oracle** withheld, because
nobody else can block a PDA-authority reveal. An oracle colluding with a requester already breaks the scheme more
cheaply: it knows the value for a known seed slothash before anyone else, and in the SDK the **client** picks the oracle
at commit (`commitIx` → `queueAccount.selectRandomnessOracle()`, `randomness.js:136-172`). We have **not verified**
whether the Switchboard program constrains that choice on-chain. So recommit adds no attack beyond the provider trust
assumption we already accept. A refund, by contrast, gives the option to **every requester** under the current
requester-authority design. Each recommit also stalls the FIFO head for one more deadline, so keep `deadline_slot` short
(e.g. ≤ 150 slots) and `MAX_RECOMMITS` small.

Why this closes B-02: the requester can't reveal selectively (the PDA is the authority and anyone can reveal), can't
recover the fee (never refunded), and can't shift the pool (a recommit keeps the original candidate set, and expire
only returns principal after an external oracle failure). The residual risk is a colluding **oracle**, which costs the
colluder the full fee per aborted draw and shows up in monitoring.

---
## 4. New fee design (BRIEF.md:70-76): exploit review

Model: `poc/03_fee_model.py` → `poc/03_fee_model.out` (8/8 PASS). Market inputs are **examples**: graduation FDV
100 / 400 / 1,500 SOL, VRF cost 0.002 SOL. Floor `F = R × FDV_grad / 1B`. Attempt cost = `0.02·F + s`. Break-even
rarity premium `m* = (0.02 + s/F) / p`.

**Headline numbers (PoC 03 Parts 1–4, FDV 400 SOL):**

| Ratio R | Floor F | s | Cost / attempt | Share from SOL min | m* at p=0.1% | Expected cost of a 1-of-maxN |
|---|---|---|---|---|---|---|
| 10k | 0.004 | 0.003 | 0.00308 | 97% | 770× | 308 SOL |
| 100k | 0.04 | 0.003 | 0.0038 | 79% | 95× | 38 SOL |
| 1M | 0.4 | 0.003 | 0.011 | 27% | 27.5× | 11 SOL |
| 5M | 2.0 | 0.003 | 0.043 | 7% | 21.5× | 8.6 SOL |

- Without the SOL minimum, `m*` is **20× floor for a 0.1% tier at every price and every ratio** (the token fee is
  scale-invariant). So the 2% token part alone deters grinding only for traits priced below ~20× floor (0.1%) or ~2×
  (1%).
- The SOL minimum is the **only** meaningful floor for small ratios (97–99% of the cost at R=10k). It barely matters at
  R ≥ 1M.
- **Does it need to scale with rarity count?** No. Expected cost per hit already scales as 1/p, and farming a whole tier
  costs ~`N·H_c` attempts, with the last item costing ~N attempts (PoC 03 Part 2). A flat `s` per attempt is
  sufficient. B's `k / p_rarest` scaling (B-04 fix c) isn't needed once converting is closed during the curve. What
  matters is that `s` can't be bypassed (F-03) or recycled (F-04).
- **Is 0.003 SOL enough?** It's enough to make a 1-of-100,000 at R=10k cost ~308 SOL. For mid ratios (100k) at a low
  graduation FDV it gives legendary break-even ≈ 95×, which is fine. Recommend **`s` = a program constant ≥ max(rent-exempt
  minimum 890,880 lamports, VRF cost + margin)**, e.g. **0.005 SOL**, and Barton confirms the value (Needs Barton).

### F-01 — High — `hybrid_launch` contradicts the fee decision (rate not fixed at 2%, destination = BURN only, no fee owner)
- **Evidence:** `constants.rs:44-46` `MAX_TOKEN_FEE_BPS = 1_000` (10%); `validation.rs:31-33` accepts any creator
  `capture_fee_bps`/`reroll_fee_bps` in 0..=1,000 (so **0%** and **10%** are both launchable); `validation.rs:37` and
  `constants.rs:48-50` accept only `fee_destination == BURN`; `state.rs:41-42` stores `fee_destination: u8` and there's
  no fee-owner or fee-token-account field; `instructions/launch.rs:4,186-190`; tests
  `tests/track-a-hybrid/launch/launch.rs:281-283` (`attack_fee_destination_other_than_burn_is_rejected`) and
  `qa_launch.rs:186-187, 373-377` enforce the superseded rule.
- **Risk:** the code that fixes economics at launch encodes a burn model with a 10% cap and a creator-selectable rate,
  which breaks "hard-capped at 2%". If the fee owner were ever made a creator-supplied `LaunchParams` field, **any
  creator could route the fee to themselves**, because `launch` is creator-signed (`launch.rs:37`).
- **Fix:** `FEE_BPS: u16 = 200` as a compile-time constant (not a parameter); `MAX_FEE_BPS = 200` asserted too;
  `FEE_OWNER` = the fixed Barton-controlled address as a **program constant** (or a platform-config PDA, only if Barton
  picks the timelock option, F-09). `LaunchConfig` v2 stores `fee_bps = 200`, `fee_owner`, `fee_token_account =
  ATA(fee_owner, mint)`, `capture_fee_amount`, `reroll_fee_amount`, `sol_min_lamports`. Remove `fee_destination` from
  `LaunchParams`. Replace the burn tests.

### F-02 — Medium — Grinding cost floor at graduation depends entirely on the SOL minimum for small ratios
- See the table above. Residual of B-04/B-05 after decision 1.
- **Fix:** `REROLL_SOL_MIN` as a program constant (proposed 0.005 SOL; Barton picks), ≥ rent-exempt minimum.
  Publish per-tier expected cost `(0.02·F + s)/p` and live pool odds on the re-roll button (B-05/B-07). No fee
  discounts, ever.

### F-03 — High — Release + capture bypasses the re-roll SOL minimum
- `release` is free and exact. `request_capture` pays 2% of R, and the decision puts the SOL minimum on **re-rolls
  only**. The unwrap→rewrap route is a re-roll (the same pool, and the handed-back NFT can even be redrawn), and it's
  79–99% cheaper for R ≤ 100k (PoC 03 Part 4: R=10k 0.00308 → 0.00008 SOL). The existing rule `capture_fee_bps ≥
  reroll_fee_bps` (validation.rs:33) covers only the token part.
- **Fix:** `capture_sol_min ≥ reroll_sol_min`, enforced on-chain (simplest: the same constant on both). Test below.

### F-04 — Medium (Needs Barton) — Insider discount: the fee owner's own re-rolls pay the fee to itself
- If the fee address, or any wallet it funds, captures or re-rolls, the 2% token fee round-trips to the same owner. If
  the SOL minimum also goes to the fee owner, its marginal cost per attempt is **the VRF cost only**. PoC 03 Part 3,
  R=1M: public 11 SOL per 1-of-1,000 vs insider 2 SOL (SOL min to owner) or 3 SOL (SOL min to a sink). This can't be
  blocked on-chain: wallets are free, and checking `payer != fee_owner` is trivially bypassed.
- **Fix:** (a) the SOL minimum goes to a sink the fee owner doesn't control (the VRF cost, with any excess sent to the
  incinerator `1nc1nerator11111111111111111111111111111111`), not to Barton; (b) a published policy: the fee address
  and affiliated wallets never capture or re-roll; (c) a public monitor for fee-address outflows to wallets that later
  convert. Disclose it as a conflict of interest: the house earns from grinding.

### F-05 — Medium — Fee destination DoS (ATA missing or closed) blocks every capture and re-roll
- Freeze is impossible: the mint freeze authority is `None` (`launch.rs:96-99` init, post-check `:155`). But the fee ATA
  can be **absent** (never created) or **closed** by its owner whenever its balance is 0, for example after the owner
  sweeps it. Every capture and re-roll then fails with an account error: a platform-wide DoS on converting, triggered by
  the owner or by an attacker holding a compromised fee-owner key.
- **Fix:** `hybrid_launch` creates `ATA(FEE_OWNER, mint)` at launch (it already creates the launch-vault ATA). In
  `request_capture`/`request_reroll`, `hybrid_vault` CPIs `create_associated_token_account_idempotent` (payer = user)
  before the fee transfer, so a closed ATA is recreated permissionlessly. Keep the fee owner a multisig so closing needs
  a vote.

### F-06 — High — Fee account substitution (user pays the fee to themselves; wrong mint; owner reassigned)
- If `request_*` takes the fee destination as an account and checks less than address + mint + owner, a user passes
  **their own** token account (or a junk-mint account) and pays the "fee" to themselves: free captures and re-rolls, no
  grinding floor, Barton unpaid. The same applies to the SOL-minimum receiver. This is the A-02 class (upstream
  overwrote `fee_location`, `update_recipe.rs:91-93`).
- Classic SPL Token has **no ImmutableOwner** (that's a Token-2022 extension), so the owner of an ATA can
  `SetAuthority(AccountOwner)` it to another key while the address stays the same. An address-only check would then keep
  paying a stranger.
- **Fix:** `fee_token_account.key() == launch_config.fee_token_account == get_associated_token_address(FEE_OWNER, mint)`
  **and** `fee_token_account.mint == mint` **and** `fee_token_account.owner == FEE_OWNER`, with `token_program ==
  spl_token::ID`. The SOL receiver must equal `launch_config.sol_fee_receiver`. None of these comes from an instruction
  argument. If the owner check fails (owner reassigned), fail closed (`FeeAccountOwnerMismatch`); recovery is only via
  the F-09 path.

### F-07 — Medium — Fee must never be counted as backing, and release must carry no fee
- The design says it's on top of N (BRIEF.md:72). Enforce it: the capture debit is `N + fee` from the user's ATA, as two
  transfers: `N → vault ATA` and `fee → fee ATA`. The re-roll moves `fee → fee ATA` only. Release moves exactly `N`
  vault → user with no fee transfer at all. The fee never passes through the vault or a request PDA (supersedes N7
  escrow).
- **Invariants (fuzz):** `vault_balance ≥ N × nfts_outside` after every instruction; `Δfee_ata == fee` on capture and
  re-roll and `== 0` on release/settle/expire; `vault` unchanged by re-rolls.

### F-08 — High — "Rate can never be raised, not even by multisig" is false while `hybrid_vault` is upgradeable
- BRIEF.md:74 promises an absolute. BRIEF.md:62 says upgrades need 3-of-5 + 7 days until the post-audit freeze. An
  upgrade can change the constant, the transfer code or the destination for **every** collection (B-10 item 1). That's
  Stonk.fun S-3 ("retained power to change economics"), with a 7-day notice.
- **Fix:** (1) `FEE_BPS` a constant **and** stored per launch in `LaunchConfig` (immutable account). `hybrid_vault` uses
  `config.fee_bps` and asserts `≤ 200`, so a future upgrade raising the constant doesn't affect existing launches unless
  it also rewrites accounts (which a monitor detects). (2) Copy until `--final`: "Fee is 2% and can't be raised by any
  setting. The program can only be changed by a 3-of-5 multisig upgrade with a public 7-day delay, until it's frozen
  after audit." (3) Go `--final` after audit. (4) Monitor buffer writes and queued upgrades.

### F-09 — Medium (Needs Barton) — Fee address as a hot wallet; the destination-change policy
- A compromised hot-wallet fee owner can (a) sweep accrued fees, (b) close the ATA (DoS, F-05), and (c) reassign the ATA
  owner (fees go to the attacker, or fail closed under F-06). If the destination is **never** changeable, a compromise
  is permanent: every future fee is lost, or converting is DoSed until someone recreates the ATA.
- **Fix:** the fee owner is a **Squads multisig vault PDA** (off-curve owner; the ATA supports it), threshold ≥ 2-of-3
  (Barton + hardware keys). Barton picks either **(A) never**, where `FEE_OWNER` is a program constant and a change
  requires an upgrade, or **(B)** a platform `FeeConfig` PDA whose only mutable field is `fee_owner`, changed by
  `propose_fee_owner` (multisig) → ≥ 7-day wait → `execute_fee_owner`. The pending value is shown on every page. There's
  no rate field and no per-launch override. Existing launches read `FeeConfig` at request time. A recommends **(B)** for
  recoverability, and only if (1) the timelock is enforced in-program, and (2) the destination is still validated as
  `ATA(fee_owner, mint)`.

### F-10 — Medium (Needs Barton) — Sell pressure and disclosure: Stonk.fun's documented harm pattern
- Fees accrue as **collection tokens** at one address: 2% of `N×R` per full capture cycle. For a max-size collection
  that's **20,000,000 tokens (2% of supply)**, rising to 8% at 3 re-rolls/NFT and 22% at 10 (PoC 03 Part 6). Selling
  20M into a CP pool holding 200M tokens moves spot about −17%. "Fee wallet sells into each coin's own pool" is exactly
  what Bitquery documented for Stonk.fun (stonkfun-lessons.md; BRIEF.md:17).
- **Fix:** disclose on every page: fee rate, fee address, current balance, cumulative received, all outflows. Publish a
  reconciliation script (lesson #6). Barton adopts a **written sell policy** (e.g. no sales into the coin's own pool
  within X days of graduation, a max % per day, or vesting). Show the fee address in the top-holder view. No off-ledger
  transfers.

### F-11 — Low — Rounding and arithmetic of the fee
- With the allowed ratios (all multiples of 10,000) and decimals 0–9, 2% is exact and ≥ 200 whole tokens (PoC 03 Part
  5). But `floor` rounding would give **0** for `ratio_base < 50` if the ratio set ever changes (e.g. a test ratio).
  `R_base × bps` fits u64 at 200 bps but overflows above 3,689 bps at R=5M, d=9.
- **Fix:** `fee = ceil(u128(R_base) × 200 / 10_000)`, `require!(fee > 0)`, `u64::try_from`, `checked_add(N, fee)` for
  the debit. Compute once at launch and store it.

### F-12 — Low — SOL-minimum leg can fail on rent
- A lamport transfer below 890,880 lamports to an empty system account fails (`InsufficientFundsForRent`), so a tiny
  SOL minimum paid to a drained wallet DoSes re-rolls (PoC 03 Part 7).
- **Fix:** `REROLL_SOL_MIN ≥ 890,880` lamports, or pay it to a pre-funded or program-owned receiver.

### F-13 — Info — Fee timing
The fee is charged at request (with the commit), directly to the fee ATA, and never refunded (§3.4). That removes the
request-PDA fee custody in N7 and the refund exploit together.

---
## 5. Doc bugs (fee language, burn/split, VRF refund/reveal) — file:line

Every line below still says the capture/re-roll fee is **burned**, **split**, **≤ 10% / ≤ 1,000 bps**, **creator-set**,
**escrowed and refunded on expire**, or that "anyone can reveal". Each one conflicts with BRIEF.md:70-76 (2% fixed, paid to
one fixed Barton-controlled fee address, never raised) or with §3 of the cross-review. LP burns, anti-sniper fee burns,
and the `BurnOnCapture`/`BurnOnRelease` upstream paths are **not** doc bugs and aren't listed.

**Code and tests (highest priority: they enforce the superseded rule on-chain)**
- `programs/hybrid_launch/src/constants.rs:44-46` (`MAX_TOKEN_FEE_BPS = 1_000`), `:48-50` (`FEE_DESTINATION_BURN`, "Only BURN exists")
- `programs/hybrid_launch/src/validation.rs:15-16, 31-33` (creator-chosen bps 0..1000), `:37` (`FeeDestinationNotBurn`), `:84`
- `programs/hybrid_launch/src/state.rs:6, 41-42` (`fee_destination` "Always FEE_DESTINATION_BURN"; no fee owner/account)
- `programs/hybrid_launch/src/error.rs:17-18`; `lib.rs:14`; `instructions/launch.rs:4, 186-190`
- `tests/track-a-hybrid/launch/launch.rs:16, 39, 161, 281-283`; `tests/track-a-hybrid/launch/qa_launch.rs:25, 57, 186-187, 247, 373-377, 880-881`

**docs/**
- `docs/BRIEF.md:50, 52, 57, 62`: burn entries with no inline "SUPERSEDED by 2026-09-25" marker (`:71` supersedes them; add the marker at each line). `:39, 43`: "Fee destination TBD/OPEN" is now decided.
- `docs/DECISIONS.md:9, 117, 151, 163, 192, 195-198` (ADR-009 Decision D "re-roll fee = BURN"), `199-215` (burn safety check and `Σburned` invariant), `239, 242` (ADR-010 `fee_destination = BURN`, caps ≤ 1,000 bps), `288` (Q-H3 "Resolved: BURN"), `299-300` (N1 capture fee burned), `310-312` (N7: fee escrowed, burned at settle, **refunded on expire**). Mark ADR-009 D superseded; add an ADR for the 2% fee.
- `docs/ARCHITECTURE.md:22` ("Fees are burned"), `31` ("can only go down (burns)"), `38` ("≤ 10% … burned"), `39` (burn copy), `73, 77, 109` (`fee_destination = BURN`, ≤ 1000), `96, 99` (sequence diagram "token fee burned"), `121-122` ("Fee destination: burn; no account").
- `docs/THREAT_MODEL.md:21-22` ("No fee pile exists: re-roll fees are burned"), `58` (S-2 "fees are burned, never swept"), `60` (S-4 burns as supply reduction), `72` (T-TOK-04 `Σburned`), `83` (T-HL-06 "must equal BURN"), `90-96` (T-BURN-01..03 section), `144` (T-HV-10 "fee destination (burn)"). Replace with fee-address threats F-05/F-06/F-09/F-10.
- `docs/hybrid-rarity-and-assignment.md:9` ("fees = burn"), `22`, `184-187` (fee escrowed, burned at settle, refunded on expire), `206-210` (**"principal and the escrowed token fee are refunded in full"**, **"anyone can reveal"**, which is false per §3.2), `253, 255, 257` (instruction table: escrow/burn/refund), `276` ("fee destination (burn)"), `313, 315` ("hard cap 1,000 bps"), `322-343` (fee destination BURN section and burn safety), `356` ("fee is refunded minus VRF cost"), `357` ("≤ 10% of R token fee, ≤ 0.05 SOL"), `358` ("There is no destination account: the token fee is burned"), `371-372` (Q-H3 BURN; "capture fee burned too?"). Also `320` quotes B's superseded "~50 SOL at 0.05 SOL cycle".
- `docs/admin-multisig-timelock.md:19-20` ("fees are burned, there's no destination account"), `60` ("no economic parameter changes, so nothing to delay in-program"): if Barton picks the timelocked destination change, an in-program `propose → 7 days → execute` for `fee_owner` IS needed.
- `docs/qa-answers.md:28-29, 43, 72, 74-75, 80-82` (burn at settle, refunded on expire, ≤ 1,000 bps, "burned by re-roll fees" supply line).

**qa/**
- `docs/qa/TEST_PLAN.md:8, 18, 38, 64, 76-79` (supply after burns), `107` (INV-02 "only by burns"), `108` (INV-03 "fee handled per its policy"), `115` (INV-10 "Re-roll fee burned"), `116` (INV-11 `fee_destination = BURN`), `152` (SUP-08), `168` (CFG-08), `169` (CFG-09 cap 1000, `fee_destination` 1…255), `217` (RR-02 "Fee burned"), `223` (RR-08 refund on expire), `226-227` (RR-11/12 burned), `356` (E2E-C03 "burned"), `406-407, 413` (Q2/Q3/Q9).
- `qa/archive/TEST_PLAN.v0.3.md`: archived, so no action beyond the "archived" banner.

**design/ (user-facing copy)**
- `design/directions/README.md:35` ("Re-roll fees are paid in the collection's token and burned"); `design/directions/a-obsidian/NOTE.md:14, 16`.
- `design/directions/a-obsidian/token.html:217, 242` ("Re-roll fee: burned"), `265` ("Burned by re-rolls").
- `design/directions/a-obsidian/launch.html:244` ("Where the fee goes: Burned"), `252`/`324` (`"… ORBIT, burned (example)"`). The launch page also offers creator-chosen fee % (NOTE.md:16 "1% / 2% / 5%"); now fixed at 2%.
- The supply copy "No one can mint more; re-roll burns can only reduce it" (ARCHITECTURE.md:39, THREAT_MODEL.md:96, DECISIONS.md:196-197, NOTE.md:14, BRIEF.md:52) must become "Fixed at 1,000,000,000. No one can mint more." Nothing is burned now.

**security/ (our own reports; errata, don't rewrite history)**
- `docs/security/review-a/round1.md:11, 18, 61 (B-08), 63-65, 237-249, 318`: burn-based. Superseded by this round's F-series.
- `docs/security/review-b/round1.md:38, 177, 288, 309-323 (R1-B-12)`: burn-based. B's to annotate.

---

## 6. Open disagreements (left open for B)

| # | Topic | A position | B position | What would settle it |
|---|---|---|---|---|
| D1 | Severity of MPL-Hybrid cherry-pick / predictable reroll (A-03/A-04/A-05 vs B-01) | High under A's key. **Concede Critical if MPL-Hybrid is in the swap path.** Moot for hybrid_vault | Critical (conditional) | Nothing blocking: both agree it's moot under ADR-008. Merged uses Critical (conditional) |
| D2 | Curve launch sniping (A-D05/T-CURVE-05 vs B-03) | **Critical** until an audited venue with a program-enforced activation point / anti-snipe schedule is chosen and tested | High (conditional) | Engineer picks the venue (BRIEF.md:78) and shows the opening-window controls. Merged uses Critical, flagged |
| D3 | Wrap/unwrap arbitrage vs stale listings (B-07) | **Low**: market behaviour against third-party listings, no protocol loss, no invariant broken. Disclosure only | Medium | B to state a protocol-level loss path, if any. Merged uses Medium, flagged |
| D4 | Refund on VRF expiry | Fee never refunded; permissionless PDA-authority reveal; re-request by anyone (bounded) before a principal-only expire (§3.4) | No fee refund on expire; principal refund after a long deadline; or prefer ORAO | Mostly aligned. The only difference is **recommit before expire**. B to confirm, and to confirm the devnet third-party reveal test |
| D5 | "No program code exists" (B header, code-review-no-commits.md) | Stale: `hybrid_launch` exists at c02be68 and is in scope (it contradicts the fee decision, F-01) | Repo empty as of 2026-09-24 14:54 | B re-checks the repo / local tree |
| D6 | SOL-minimum scaling (B-04 fix c: `k / p_rarest`) | Flat constant is enough (cost already ∝ 1/p); what matters is no bypass (F-03) and no recycling to the fee owner (F-04) | Scale with collection size / rarest tier | B's view on PoC 03 Part 1–2 |

## 7. Bottom line
- B's report holds up. Every code claim I checked against aacf1a53 is accurate (§1), and B found real
  economic issues we missed (B-02, B-04, B-06, B-09 front-running, B-10 upgrade key, B-11, B-13).
- **B-02 is stronger than B stated:** Switchboard's reveal needs the randomness authority's signature, so "anyone can
  reveal" is false as designed. The fix is PDA authority + permissionless reveal-and-settle + a non-refundable fee.
- The **new fee design is sound in principle** (on top of N, no release fee, fixed rate) but has four
  must-fix issues. Our own `hybrid_launch` contradicts it (F-01). Release+capture bypasses the SOL minimum (F-03).
  Fee-account substitution would let users pay the fee to themselves (F-06). And "never raised" is untrue while the
  program is upgradeable (F-08). The insider discount (F-04) and sell pressure (F-10) need Barton's policy.
- **Not launch-ready.** A professional third-party audit is required before mainnet.
