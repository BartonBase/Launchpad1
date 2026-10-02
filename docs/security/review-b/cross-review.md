# Cross-review: Review B on Review A's round 1 (Mintmark SPL-404 hybrid)

**Author:** Review B. **Date:** 2026-09-25, written 16:35 to 17:30 MT.
**Comparison round:** OPEN. I read `docs/security/review-a/round1.md` and every file under `docs/security/review-a/poc/`
(harness JS, Python models, `.out` files, build log). **Nothing in `docs/security/review-a/` or `docs/security/merged/` was edited.**
**Scope:** SPL-404 hybrid launches only (Token-2022 deferred).
**Safety:** Only a local `solana-test-validator` (Agave 4.1.2) on my own ports (28899, ledger `/tmp/b-ledger`) was used.
Another validator was already running on A's port 18899, so I left it alone, copied A's harness to `/tmp/b-rerun`,
and changed only the RPC port in the copy. No devnet or mainnet, no real keys, no real funds.

**Inputs re-read for this review:** my `round1.md`, `sim/reroll_ev_r1.py` + output, `research-sources.md`, `threat-model.md`.
Also `docs/BRIEF.md` (mtime 2026-09-25 16:28 MT, which includes the 2026-09-25 FEE CHANGE and the 2026-09-24 3:22 PM GRADUATION MODEL),
`DECISIONS.md`, `hybrid-rarity-and-assignment.md`, `ARCHITECTURE.md`, `stonkfun-lessons.md`, plus the code on
`main` (with uncommitted `hybrid_launch` changes) and on branch `wip/hybrid-vault`. That branch was `f4761af` when I first
read it and `977f8f2` at 16:34 MT. Both are dated 2026-09-24 15:18 MT, so someone is amending it right now, and line
references to it may drift.

**Evidence I produced:** `docs/security/review-b/poc-crosscheck/`
- `a_pocs_10_11_12_rerun.out`: A's on-chain PoCs 10, 11 and 12 re-run unmodified (apart from the RPC port) on my validator. **All three PASS.**
- `b13_token_swap_full_drain.js` + `.out`: my extension of A's PoC 12. It completes the drain loop A describes but didn't
  execute, and it shows the fee-destination hijack against a third-party user. **PASS.** (It needs `lib.js` from A's harness.
  To run it, copy it into a copy of that harness and point RPC at a local validator.)
- I also re-ran A's `01_reroll_predictability.py` (11/11) and `02_escrow_accounting_model.py` (5/5), then checked what each assertion actually tests (§3).

---

## 1. Mapping table: every A finding → matching B finding

Severities are as each review wrote them in round 1. "—" means the other review has no matching item.

| A ID | A sev | A title (short) | B match | B sev | Match quality |
|---|---|---|---|---|---|
| A-01 | Critical | `update_recipe_v1` changes `amount`/fees instantly, single signer → escrow drain (PoC 10, on-chain) | R1-B-10 item 7 (and the R1-B-01 condition) | High (Critical if single key), CONDITIONAL | Same root cause. A executed it, B cited source only |
| A-02 | Critical | `update_recipe_v1` always overwrites `recipe.token` + `fee_location` (PoC 12, on-chain) | R1-B-10 item 7 | High / Critical if single key, CONDITIONAL | Same. B had no PoC and missed the "innocent update re-points" angle |
| A-03 | High | Capturer names the asset; permissionless capture with `NoRerollMetadata` (PoC 11, on-chain) | R1-B-01 (choosing mode) | **Critical**, CONDITIONAL | Same. Severity differs (§3 D2) |
| A-04 | High | Reroll index from SlotHashes/ts/count, revert-until-rare | R1-B-01 (predict-and-abort) | **Critical**, CONDITIONAL | Same. B has cost numbers (sim Part D), A has a model |
| A-05 | High | Index biased: `max` unreachable, with replacement, power-of-two collapse | R1-B-14 (carry-over B-16) + R1-B-01 text | Info | Same facts, far apart on severity (§3 D3) |
| A-06 | High | V2 escrow seeded per authority → cross-recipe backing pool | — | — | B missed it. Severity disputed (§3 D4) |
| A-07 | High | `BurnOnCapture`/`BurnOnRelease` burn backing → insolvency | R1-B-12 (burn side-effects, "never burn from vault") | Low | Partial overlap. Severity disputed (§3 D5) |
| A-08 | Medium | No ratio/supply/solvency binding upstream | R1-B-10 item 7 (implicit); design-requirements R-03 | (requirement) | Partial |
| A-09 | Medium | `migrate_nft_v1` drops the CPI result | — | — | B missed it (§5) |
| A-10 | Medium | Raw account init / `init_if_needed` | — | — | B missed it (§5). Severity disputed (§3 D8) |
| A-11 | Low | "Need to add account checks" TODOs in shipped handlers | R1-B-10 item 7 ("Audit Pending") | — | Partial |
| A-12 | Low | `authority` is a bare `AccountInfo`, meaning flips with config | R1-B-01 (why choosing mode is permissionless) | Critical (as part of R1-B-01) | Same mechanism |
| A-13 | Info | Mainnet binary length ≠ pinned build; upgradeable by `mp14o4AQ…` | R1-B-10 item 7 (upgradeable by Metaplex key) | (part of High) | B missed the length mismatch |
| A B-01 | High (until built) | Everything depends on the unwritten `hybrid_vault` | R1-B-01 fix; "Not safe until" #1, #11 | — | Same. Both stale: WIP code exists (§4 N-01) |
| A B-02 | Medium | VRF integration risk (foreign/pre-revealed account, selective reveal, modulo bias) | **R1-B-02** | **High** | Overlap. B's `expire`-refund abort isn't in A (§3 D7) |
| A B-03 | Medium | FIFO/`seq` pool subtle, unfuzzed | R1-B-02 scenario B, R1-B-06 item 3 | High / Medium | Overlap |
| A B-04 | Medium | Feistel + Merkle binding: root before seed, FPE correctness | R1-B-11, R1-B-13 | Medium / Medium | Partial (B: image bytes, manifest withholding; A: ordering, FPE bugs) |
| A B-05 | **Critical** (curve) | Launch-window sniping / Jito bundles / sandwiching | R1-B-03 (+ R1-B-08 for sandwiching) | High (+ Low/Med) | Same. Severity differs (§3 D6) |
| A B-06 | High (curve) | Graduation/migration: trigger, LP custody, drain path | R1-B-09 | High | Same |
| A B-07 | Medium | Launch supply goes to a person by default (T-HL-11) | R1-B-10 item 5 (creator allocations, NFTs only) | High (as part of R1-B-10) | A is more precise (§5) |
| A B-08 | Medium | Re-roll burn must come from the user's balance, never escrow | R1-B-12 item 2 | Low | Same. Now superseded by the fee-to-address design; the invariant carries over |
| T-CURVE-01 | Critical if missed | Mint/freeze `None`, supply check before sale | R1-B-10 item 8 | (good, keep) | Same |
| T-CURVE-02/04 | High | Curve rounding / overflow at extremes | — | — | B missed it. Conditional on building our own curve |
| T-CURVE-03/05 | High / Critical | First-buyer advantage, fair-launch opening | R1-B-03 | High | Same as A B-05 |
| T-CURVE-06 | High | On-chain `min_out`/`max_in` | R1-B-08 | Low / Medium | Same fix (R-11.1) |
| T-CURVE-07/08 | Critical / High | Permissionless deterministic graduation; one-shot migration | R1-B-09 | High | Same. A adds the re-entrancy / double-run angle |
| T-CURVE-09 | High | Supply to an EOA → creator rug | R1-B-10 item 5 | High | Same as A B-07 |
| T-CURVE-10 | High | No discretionary platform wallet; fee destinations immutable | R1-B-10 items 3, 6 | High | Same. Newly relevant to the 2026-09-25 fee design |
| T-CURVE-11 | Medium | Don't spin burns as buyback; publish a ledger | R1-B-10 item 6 | High | Same. Burn is now superseded |

B findings with **no A equivalent**: R1-B-02 (the `expire` refund part), R1-B-04, R1-B-05, R1-B-06 (cornering), R1-B-07, R1-B-11 (image bytes),
R1-B-13, and R1-B-10 items 1, 2 and 4 (a shared upgrade key over every vault, fee timelock length, pause). See §4.

## 2. Agreements (and where A's evidence is stronger)

| # | Topic | A | B | Stronger evidence |
|---|---|---|---|---|
| G1 | MPL-Hybrid must not be the swap engine | Launch-readiness statement | R1-B-01 fix, "Not safe until" #1 | **A**: three on-chain PoCs against a build of the pinned commit (sha256 `9e99822…c830`). I confirmed the `build-mplh/programs/mpl-hybrid/src` tree is byte-identical to `reference/mpl-hybrid-aacf1a53…` (`diff -rq` returned nothing) and the `.so` hash matches |
| G2 | Untimelocked `update_recipe_v1` = authority drain | A-01 | R1-B-10 item 7 | **A** (PoC 10 on-chain: 3,000,000 → 0). I reproduced it: `poc-crosscheck/a_pocs_10_11_12_rerun.out` |
| G3 | `token`/`fee_location` overwrite | A-02 | R1-B-10 item 7 | **A**, and my B13 now completes the loop (§3 D1) |
| G4 | Caller-chosen asset | A-03 | R1-B-01 | **A** (PoC 11 on-chain, reproduced). B has the cost-per-rare numbers (0.011 SOL per rare in choosing mode) |
| G5 | Predictable reroll, revert-until-rare | A-04 | R1-B-01 | Roughly even. A's source reading is exact (the SlotHashes byte layout and the power-of-two collapse match my `reroll_ev.py`). B has cost and latency numbers. Neither of us ran an on-chain abort PoC (§3 D2) |
| G6 | Safeguards all need `hybrid_vault`; the multisig must sit behind program-enforced, field-restricted changes | Safeguards §3 | R1-B-10 fix | **A** states it more sharply: a multisig over an atomic, unbounded upstream instruction doesn't help |
| G7 | Rejection sampling, no modulo bias; pool fixed at request time; no cancel after fulfilment | Safeguards §1 | R1-B-02, R-01.4 | Even |
| G8 | Anti-sniping on the curve's first minutes; dev buy capped and disclosed | T-CURVE-03/05, B-05 | R1-B-03 | B has sourced magnitudes (MELT 36.5% bundled, Bitquery 522 wallets/3.4× supply) |
| G9 | Graduation permissionless, deterministic, LP burned or PDA-locked, no operator signer | T-CURVE-07/08, B-06 | R1-B-09 | Even. A adds one-shot/re-entrancy, B adds pool-squatting and the pump.fun precedent |
| G10 | Mint/freeze revoked and verified on-chain before sale | T-CURVE-01 | R1-B-10 item 8 | Even. Now implemented in `hybrid_launch` (post-launch re-read asserts supply and authorities) |
| G11 | Fee must come from the user's own balance, never the vault; fuzz `vault ≥ R × NFTs_outside` | Safeguards §2, B-08 | R1-B-12 item 2 | Even. **Carries over unchanged to the 2026-09-25 fee-to-address design** |
| G12 | Charge the fee in force at request; `capture_fee ≥ reroll_fee` | Safeguards §2 | R1-B-05, R1-B-08 | Even |
| G13 | Charge the fee **at request** and make it non-refundable (A: "inline `token::burn` … in `request_reroll`") | Safeguards §2 | R1-B-02 fix ("no refund of the token fee on `expire`, ever") | Even. **A and I independently reached the same fix, and the docs still do the opposite** (N7: escrow at request, refund on expire; WIP `expire.rs` refunds the fee). Under the new design this means: transfer to the fee address at request, never refund |
| G14 | Pinned binaries, don't trust the deployed MPL-Hybrid program | A-13 | R1-B-10 item 7, B-15 | **A** (the binary length mismatch is new evidence) |
| G15 | Professional third-party audit before mainnet | closing statement | closing statement | Even |

## 3. Disagreements, with evidence (A's PoCs checked line by line)

### PoC verification summary

| PoC | What A claims | What the code actually does | Holds up? |
|---|---|---|---|
| `harness/10_escrow_drain.js` | Authority sets `amount=0` with no timelock, captures 2 NFTs for 0, raises `amount` to 3R, and one release drains 3R | Exactly that. Setup mints backing into the escrow ATA with the test mint authority and creates the NFTs directly in escrow. Both are legitimate simulation of prior state. `release_v2` reuses `captureKeys` (same account order), and the run succeeded. **Re-run: PASS, identical numbers** | **Yes.** Two nits: (1) the `amount=0` capture step isn't needed for the drain. Raising `amount` and releasing any NFT the authority holds is enough, so the attack is even simpler. (2) The attacker is the authority, so this is a privileged-key attack. It fits A's own Critical definition ("a single non-timelocked key") |
| `harness/11_cherrypick.js` | A non-authority bot captures a named NFT with no authority signature | The bot passes itself as `authority` (≠ `recipe.authority`, so `assert_signer` is skipped at `capture_v2.rs:183-185`), and `NoRerollMetadata` skips the metadata branch. The capture lands. **Re-run: PASS** | **Yes.** No timing assumption is needed. The token fee was 0 and the Metaplex SOL protocol fee still applied. The attack costs one capture |
| `harness/12_token_swap.js` | An update with every option `None` still re-points `recipe.token`. The NFT is captured with junk, the real mint restored, "and the real backing remains claimable by releases" | Proves the overwrite and the junk capture. **Does not** execute the release against real backing. The escrow held no real backing in this PoC | **Partly.** The overwrite claim is proven. The "drain" half was asserted, not run. **I closed the gap:** `poc-crosscheck/b13_token_swap_full_drain.js` escrows 1R of real backing, captures with junk, restores the real mint, and releases. Result: real escrow 1,000,000 → 0 and attacker +1,000,000 real tokens after paying only junk. Part 2 shows that after the same "empty" update, an **unrelated user's** 2% capture fee (20,000 tokens) goes to the attacker's ATA and the disclosed fee address gets +0. **PASS** |
| `harness/12_reroll_peek.js` | (Listed as an on-chain peek PoC in its header comment) | A **10-line truncated stub**. `node --check` fails with `SyntaxError: Unexpected end of input`. It isn't in A's PoC table, but the file name collides with PoC 12 | **No on-chain predict-and-abort was ever executed** by A (or by me) |
| `01_reroll_predictability.py` | "Executed (model) 11/11"; "revert-until-rare always wins" | T1, T3 to T8 are genuine checks of the formula's structure, and I agree with all of them. **T2 is tautological:** `predicted` and `actual` come from the same function with the same inputs, and the check is hard-coded `check(..., True, ...)`. It measures only the base-rate number of tries (~100 for 1%), not that an attacker can observe the inputs or abort for free | **The structure findings hold. The "attack works" part rests on source reading, not the PoC.** The source reading is correct: SlotHashes, Clock and `count` are readable by an attacker program in the same transaction |
| `02_escrow_accounting_model.py` | Shared-escrow cross-drain (A-06) and BurnOnCapture insolvency (A-07) | A Python counter with `deposit`/`withdraw`. Hazard A **assumes** collection-B NFTs exist outside the escrow with no backing and never shows how they arise. `B_burn_paths_violate_fixed_supply` is hard-coded `True`. The control test (C) is fine | **Weak.** It illustrates the arithmetic, not the exploitability (D4, D5) |

**Tested version vs design.** Every executed PoC targets **upstream MPL-Hybrid @ `aacf1a53`**. The design A reviewed says
MPL-Hybrid is *not* in the swap path (ADR-008 **Accepted** 2026-09-24; BRIEF SCOPE CHANGE "custom hybrid_vault program …
Upstream MPL-Hybrid is NOT in the swap path"). So the PoCs prove "don't use MPL-Hybrid". They don't test the Mintmark design.
None of A's PoCs touch `hybrid_vault` (WIP branch) or `hybrid_launch` (main).

### D1. A-01 / A-02 are listed as unconditional Critical, but they apply only if MPL-Hybrid is in the swap path
- **Evidence:** ADR-008 Accepted; BRIEF 2026-09-24 decisions; `hybrid_vault` WIP has no `update_*` instruction for ratio, mint or fees (`lib.rs`: init_vault, deposit_asset, request_capture, settle_capture, request_reroll, settle_reroll, unwrap, expire_request, pause, unpause, merge_incoming). A's own table title says "Confirmed in code (upstream MPL-Hybrid)", but the summary and readiness statement read as if the product has two live Critical drains.
- **Resolution I'd accept:** **Critical, CONDITIONAL (MPL-Hybrid in swap path); currently Closed-by-design (ADR-008).** Keep it as a regression guard, with a CI check that no Mintmark program or frontend ever builds an instruction to `MPL4o4wMzndgh8T1NVDxELQCj5UQfYTYEkabX3wNKtb`. A-02 gains a **live** analogue under the new fee design, since the fee destination must never be changeable (fee-design F-05). That analogue is Critical if a change path exists.

### D2. A-03 / A-04 at High vs my R1-B-01 at Critical, and the A-04 evidence label
- **Evidence (severity):** A's own key says Critical = "an unprivileged attacker … can drain pooled value". In choosing mode, an unprivileged bot removes every rare from the pool for one capture each (PoC 11 + my sim Part D: 0.011 SOL per rare vs 9.10 SOL per legendary under honest VRF at F = 0.3). That drains the rarity value of the whole collection.
- **Evidence (timing / abort assumptions for A-04):** Within one slot, SlotHashes, `unix_timestamp` and `recipe.count` are all fixed, so an attacker gets **one distinct draw per slot**, unless they shift `count` by landing real, paid captures ahead in the same slot. So: ~100 slots (~40 s) per top-1% hit, and ~950 slots (~6.3 min, my sim Part D) per 0.1% hit. Aborts are free **only inside Jito bundles**. Without Jito, each failed attempt lands and pays ≥ 5,000 lamports (+0.005 SOL per 1,000 misses). A's "free retries" is right for Jito users, but the rate limit is missing. It doesn't change the verdict.
- **Resolution:** merged **Critical, CONDITIONAL; N/A under `hybrid_vault`**. Relabel A-04's evidence from "Executed (model) 11/11" to "source-confirmed; structural model (T1, T3 to T8); abort not executed". If A wants on-chain proof, finish `12_reroll_peek.js` as a wrapper program that CPIs `capture_v2` and reverts on non-rare. I'd also accept **High** if the merged doc keeps a single Critical line for "MPL-Hybrid in swap path" covering A-01 to A-04.

### D3. A-05 (index bias) High vs my Info
- **Evidence:** `max` unreachable and the power-of-two collapse are real (A T3, T6; my B-16). The collapse is really a predictability issue (already A-04). Drawing with replacement (A T7: 52.8% distinct over 10k draws) falsifies a fixed census. That's a misrepresentation to buyers, not a loss of funds.
- **Resolution:** **Medium, CONDITIONAL** (MPL-Hybrid reroll mode). I concede that Info was too low for the census break (§5 C10).

### D4. A-06 (shared V2 escrow) High → Low (conditional)
- **Evidence:** (1) The escrow token account is `ATA(["escrow", authority], token)`, so it's shared only by recipes with the **same authority and the same mint**. Every Mintmark launch mints a fresh token (`hybrid_launch`), so two collections share an ATA only if one creator makes two collections over one token. (2) A cross-drain needs collection-B NFTs **outside** the escrow that were never paid for. Under MPL-Hybrid those arise only if the collection update authority mints B NFTs to someone, or changes `amount` (A-01). Either way it's an authority action already covered by A-01. PoC 02 assumes the unbacked NFTs exist and doesn't call `release_v2`.
- **Resolution:** **Low, CONDITIONAL** (MPL-Hybrid + ≥ 2 recipes per mint under one authority). Keep A's fix ("one authority/escrow per collection") as a launch-script assertion.

### D5. A-07 (Burn paths) High → Low (conditional)
- **Evidence:** `BurnOnCapture`/`BurnOnRelease` are `path` bits the authority sets (`state/path.rs`). No unprivileged user can turn them on. It's a misconfiguration hazard, and a malicious authority has A-01 anyway. PoC 02 part B is a counter model with one hard-coded `True`. `hybrid_vault` has no such path, and the burn decision it guarded against is now superseded by the fee-to-address design.
- **Resolution:** **Low, CONDITIONAL**, merged with G11 ("fee from user balance only; fuzz the invariant"), which is the part that still matters.

### D6. A B-05 (curve sniping) Critical vs my R1-B-03 High
- **Evidence:** A's key defines Critical as draining pooled value or taking an authority. Sniping harms retail through price, not by draining. Two later decisions also shrink my own amplification argument: the **GRADUATION MODEL** (BRIEF 3:22 PM, 2026-09-24: "During the bonding curve … converting is closed"), so snipers can't pre-empt NFTs or farm rares at curve prices, and "default is an existing audited curve (Meteora or Raydium)".
- **Resolution:** **High**. Anti-sniping is a hard launch gate (stonkfun lesson #5) regardless of the label.

### D7. A B-02 (VRF integration) Medium vs my R1-B-02 High
- **Evidence:** A writes "no cancel after fulfilment" and treats selective reveal as a design-spec risk. But the documented design (hybrid-rarity §3.3, DECISIONS N7) **and the WIP code** (`expire.rs`: "Refunds exactly what was locked … plus the escrowed fee"; `REQUEST_TIMEOUT_SLOTS = 1_500`, ~10 min) refund the fee when an unrevealed request expires. A Switchboard requester can learn the value off-chain before reveal (Switchboard docs, quoted in R1-B-02), so every bad draw is free whenever our crank doesn't reveal within ~10 minutes. Sim Part E: legendary cost 9.10 → 1.06 to 3.06 SOL. The WIP does close Scenario C (stale randomness): it commits via CPI inside `request_*`, checks `seed_slot == slot − 1`, and adds a `RandLock`. That's good, and I credit it.
- **Resolution:** **High until the fee is non-refundable, then Medium** (only crank-liveness DoS remains). Under the new fee design, "transfer the fee to the fee address at request" makes it structurally non-refundable, but only if engineering drops the escrow-then-pay-at-settle pattern.

### D8. A-09 / A-10 at Medium
- **Evidence:** A-09 (`migrate_nft_v1` ignores its result) is reachable only if MPL-Hybrid migration is used. A-10: A itself writes "Re-invocation is blocked in practice".
- **Resolution:** **A-09 Low, CONDITIONAL; A-10 Low/Info, CONDITIONAL.** Keep them in the upstream-quality appendix.

### D9. A B-01 "no code exists yet": stale at the time A wrote it
- **Evidence:** `wip/hybrid-vault` (commit dated 2026-09-24 15:18 MT: "program (11 ix) … builds … capture_then_unwrap_returns_exactly_ratio passes"). A's round1.md mtime is also 2026-09-24 15:18 MT. My round 1 made the same mistake (§4 N-01).
- **Resolution:** keep **High until audited**, but re-scope it to "WIP `hybrid_vault` exists and has concrete gaps (N-01)".

## 4. Findings A missed, and findings both of us missed

### 4a. B findings with no A equivalent (for the merge)
| B ID | Sev (R1) | Why it still matters after the 2026-09-24/25 decisions |
|---|---|---|
| R1-B-02 (`expire` refund = free abort) | High | Live in the WIP code (`expire.rs` refunds the fee; 10-min deadline). A's B-02 doesn't cover it (D7) |
| R1-B-04 (launch-window rare farming) | High | Converting is now closed during the curve (GRADUATION MODEL), so the window moves to **graduation**. The floor at graduation is still the lowest price at which draws exist. Re-quantified in `fee-design-stress-test.md` (a) |
| R1-B-05 (fee sizing, no minimum) | Medium | The fee is now fixed at 2% (a floor and a cap). The "no minimum" half is fixed. The SOL minimum is new and has its own issues (fee doc) |
| R1-B-06 (cornering, head-of-line DoS) | Medium | Cornering is now **Low**: min collection is 100, and converting is closed during the curve, so cornering needs post-graduation AMM prices. HOL DoS is unchanged in WIP (`expire_request` handles one head per call) |
| R1-B-07 (stale listings, pool adverse selection) | Medium | Unchanged. Frontend "unwrap value" display |
| R1-B-10 items 1, 2, 4 (one upgrade key over every vault; timelock length; pause) | High | Item 1 is unchanged (one `hybrid_vault` program for all collections). Item 4 is now concrete in WIP (N-01c) |
| R1-B-11 (image bytes not committed) | Medium | **Confirmed in WIP code:** `deposit_asset` leaf = `leaf_hash(index, name, uri)`, with no image hash |
| R1-B-13 (creator withholds manifest → stuck) | Medium | Changed shape in WIP. The creator pre-deposits every index and the vault "seals" only when all are deposited, so withholding means **converting never opens**. That's liveness, not trapped funds. Under the graduation model it's the graduation mint crank instead |

### 4b. Missed by both of us (new)
- **N-01 WIP `hybrid_vault` exists, and neither round reviewed it.** Branch `wip/hybrid-vault` (commit dated 2026-09-24 15:18 MT, amended during this review). Concrete gaps:
  - (a) `expire_request` refunds the escrowed fee (R1-B-02 live). **High**.
  - (b) `init_vault` hard-requires `fee_destination == BURN`, and `hybrid_launch` still caps fees at `MAX_TOKEN_FEE_BPS = 1_000` with `FeeDestinationNotBurn`. The 2026-09-25 fee design (2% hard cap, fixed fee address) is **not implemented anywhere**, so the WIP doesn't match the current design. **Info/tracking** (fee doc F-08).
  - (c) Guardian pause: `guardian` is **any pubkey the creator passes at `init_vault`** (it can be the creator's own hot wallet). It can pause captures and re-rolls for up to `MAX_PAUSE_SLOTS = 1_512_000` (~7 d) with a `216_000`-slot (~1 d) cooldown. Repeated back to back, that blocks new draws **~87.5% of the time indefinitely** with one key. Unwrap, settle and expire are correctly never blocked. **Medium** (R1-B-10 item 4 made concrete). Fix: guardian = platform Squads multisig, or none. Cap the total paused slots per 30 days (e.g. ≤ 7 d).
  - (d) No SOL cost fee or SOL minimum exists in WIP (`request_*` moves tokens only). **Tracking.**
  - (e) No Feistel permutation. Leaves are bound by raw index. That's acceptable because selection is VRF from a request-time pool, but hybrid-rarity §2 still promises a permutation. Docs vs code drift: **Info**.
- **N-02 The GRADUATION MODEL (post-dates both rounds) adds its own risks.** "The full NFT collection … is minted into the vault escrow, funded from a slice of graduation proceeds", with a crank, and "converting opens only when fully minted":
  - (a) **Who holds the proceeds slice between graduation and the mint crank?** If it's an operator wallet, that's stonkfun lesson #2. It must be a program PDA that only pays mint rent.
  - (b) **Liveness:** a stalled crank keeps converting closed forever.
  - (c) **Graduation-window farming** (fee doc (a)).
  - **Medium to High** depending on (a).
- **N-03 Release → capture bypasses any re-roll-only surcharge.** The new SOL minimum is specified for re-rolls only. `unwrap` is free, so `unwrap` + `request_capture` is a re-roll that skips the SOL minimum. **High** once the SOL minimum is the anti-grinding control. Fee doc F-02.
- **N-04 The fee recipient grinds for free.** Tokens paid to Barton's fee address come back to Barton. Anyone controlling that address (or colluding with it) re-rolls at net cost ≈ SOL only. **High (insider).** Fee doc F-03.
- **N-05 The fee account can be closed or be missing → all captures and re-rolls halt.** Fee doc F-06. **Medium.**
- **N-06 "Hard-capped at 2% … can never be raised, not even by the multisig" is false while the program is upgradeable.** The 3-of-5 multisig with a 7-day delay can ship code that raises it. **Medium (disclosure).** Fee doc F-07.
- **N-07 A-02's fee-destination hijack is the upstream twin of the new design's biggest trust risk.** My B13 run shows a third party's fee re-routed after an "empty" update. Any post-launch destination-change path in the new design ("or only via multisig + 7-day timelock, TBD") recreates it. **Critical if a change path exists; closed if immutable.** Fee doc F-05.
- **N-08 Audit baseline drift.** The WIP branch was re-pointed (`f4761af` → `977f8f2`) during this review, and `main` has large uncommitted changes to `hybrid_launch`. **Info.** Pin a commit per audit round.

## 5. Concessions: where A is right and I was wrong or incomplete

- **C1 (A-01/A-02).** A executed the authority drain on-chain. I had it only as a sub-bullet (R1-B-10 item 7) with no PoC, and I under-weighted it by folding it into a general operator-power finding. It deserves its own Critical (conditional) entry.
- **C2 (A-02 tooling hazard).** "The overwrite happens even on an 'innocent' update … tooling that passes the wrong account silently re-points the recipe." I missed this. It's also the direct precedent for the new fee destination (N-07).
- **C3 (A-06).** I missed the per-authority seeding of V2 escrows. I dispute the severity (D4), but the fact is correct and worth a guard.
- **C4 (A-09, A-10, A-11).** Upstream code-quality issues I didn't list: the dropped CPI result, raw init, and the TODO account checks. They are correct as static findings.
- **C5 (A-13).** The mainnet binary (630,752 B) ≠ the pinned build (521,696 B). New evidence I didn't have. It strengthens "never assume deployed == reviewed".
- **C6 (T-CURVE-02/04).** Curve rounding and overflow at the extremes. I missed these. They matter only if Mintmark builds its own curve (the current default is an audited third-party curve).
- **C7 (A B-07 / T-CURVE-09).** Launch supply defaulting to a person. I only covered NFT allocations. A was right, and engineering has since fixed it in the uncommitted `hybrid_launch` diff: the supply goes to ATA(`launch_vault` PDA), no instruction signs for it, and the post-launch re-read asserts owner, amount, no delegate and no close authority.
- **C8 (Safeguards §3).** A's point that a multisig over atomic, unbounded upstream instructions buys nothing, and that the timelock must be enforced in the program, is sharper than my R1-B-10 wording.
- **C9 (Safeguards §2).** A's "burn inline at request" is the right fix for my own R1-B-02 finding. The docs went the other way (burn at settle, refund on expire), and A's version is better.
- **C10 (A-05).** Rating the with-replacement census break "Info" (my B-16/R1-B-14) was too low. It falsifies "1 of 1" copy. I accept Medium (conditional).


## 6. Merged severity table (B's position per merged ID in `docs/security/merged/round1-merged.md`)

A's merge came out first, so I key everything to A's **M-01 to M-35** and don't add a second numbering. "=" means I agree
with A's severity, fix and test as written. Deltas are what I'd add or change. B-only items that A's merge lacks are at the end (M-B1 to M-B6).
Code refs are working tree (uncommitted, on `wip/hybrid-vault` @ `977f8f2`, read 16:39 MT) unless marked `main@658fb95`.

| Merged | A sev | B sev | B delta to fix | Owner | Proving test (must FAIL to exploit) |
|---|---|---|---|---|---|
| M-01 authority drain | Critical | = (moot-if-hybrid_vault) | — | eng+QA | `regress_poc10_*`, plus port of my `b13_token_swap_full_drain.js` as `regress_b13_token_swap_full_drain_must_fail` (escrow 1,000,000 → 0 upstream) |
| M-02 cherry-pick/reroll | Critical | = (D1 resolved) | — | eng+QA | `regress_poc11_*`, `regress_poc12_*` |
| M-03 curve sniping | Critical (A) | **High, blocking launch gate** (D2) | Also verify that the chosen venue (Meteora DBC / Raydium LaunchLab) accepts a mint created by `hybrid_launch` (its 1B sits in `launch_vault`, main@658fb95 launch.rs:84,109,162). Unverified; these venues may create their own mint | eng | A's 3 tests, plus `venue_accepts_preexisting_mint_from_launch_vault` |
| M-04 VRF | High | = High, open until the §7 amendments land | Separate reveal (records value) from settle; program-pinned oracle, different on each recommit; recommit cap 3; deadline ≥ Switchboard's ~1 h expiry (not 150 slots); principal-only escape after the cap. See §7 | eng+QA | A's 4 tests, plus `attack_settle_failure_cannot_trigger_recommit_after_reveal`, `attack_recommit_with_caller_chosen_dead_oracle_rejected`, `recommit_capped_then_principal_only_expire` |
| M-05 fee-account substitution | High | = | WT pins the address (`request.rs` fee_account) but not owner/mint at use (F-B8) | eng+QA | A's 5 tests |
| M-06 release+capture bypass | High | = (closed in WT: `validation.rs:50`) | Keep `capture_sol ≥ reroll_sol` | eng+QA | A's 2 tests |
| M-07 hybrid_launch fee | High | = | WT has moved to 100–200 bps (`constants.rs:47,50`). Must be a fixed 200 bps, not creator-chosen 1–2% (F-B11) | eng+QA | `fee_rate_is_fixed_at_200_bps` |
| M-08 "never raised" | High | = | — | eng+Barton | = |
| M-09 upgrade key/operator | High | = | Add the **guardian** in committed `977f8f2`: a single creator-set pubkey that can pause up to ~7 days with a 1-day cooldown, blocking new draws ~87.5% of the time. The WT keeps release, reveal and recommit unpausable (`randomness_ix.rs:1-2`). Verify request-pause limits too | eng+Barton | `guardian_cannot_block_release_reveal_recommit_settle`, `guardian_pause_duty_cycle_bounded` |
| M-10 graduation | High | = | New WT `graduation.rs`, `mint_assets.rs`, `open_vault.rs` not reviewed by either of us | eng | = |
| M-11 "hybrid_vault doesn't exist" | High | = (as a requirement) | **Title factually wrong; B correction.** It exists on `wip/hybrid-vault` @ `977f8f2` plus a large uncommitted WT. The requirement (build to checklist, audit it) stands | eng | = |
| M-12 shared escrow | High | Low (conditional); I don't contest keeping it as a requirement | — | eng+QA | = |
| M-13 mint/freeze/supply | High | = (resolved; verified main@658fb95 launch.rs:132-155, constants.rs:15-21) | — | QA | = |
| M-14 fee ≠ backing | Medium | = | WT invariant ignores pending re-roll hand-ins (`invariants.rs:3,15-19`, F-B13) → see M-B3 | eng+QA | = |
| M-15 fee ATA DoS | Medium | = (WT uses create_idempotent at request) | Owner check still missing (F-B8) | eng+QA | = |
| M-16 insider discount | Medium | **High (insider)** | WT sends the SOL fee to `fee_owner` too (`request.rs:224-226, 272-274`). It must go to a sink/incinerator | Barton+eng | `sol_min_never_reaches_fee_owner` |
| M-17 fee owner custody | Medium | = | — | Barton+eng | = |
| M-18 sell pressure | Medium | = (sim: 0.40%/day of supply normal; farming all legendaries at R10k = 5.86% of supply) | — | Barton | = |
| M-19 SOL minimum | Medium | = (D6 conceded) | s = max(0.005 SOL, VRF cost + margin), **raise WT `MIN_REROLL_SOL_FEE_LAMPORTS` from 1,000,000 to 5,000,000** (`constants.rs:56`). Optional graduation surcharge (fee doc) | Barton+eng | `sol_min_charged_on_every_request` |
| M-20 cornering/HOL | Medium | = | A's "deadline ≤ 150 slots" conflicts with M-04 safety (§7 H3). Use ~9,000 slots and batch-clear | eng+QA | = |
| M-21 image bytes | Medium | = (committed leaf has no image hash) | — | eng+QA | = |
| M-22 mint crank stall | Medium | = | — | eng+QA | = |
| M-23 FIFO/Feistel | Medium | = | — | eng+QA | = |
| M-24 stale listings | Medium (B) | **Low (D3 conceded)**, if the UI shows live pool odds and unwrap value | — | eng (UI) | `ui_pool_census_matches_chain` |
| M-25 curve math | Medium | = | — | eng+QA | = |
| M-26 pause scope | Medium | = | See M-09 guardian | Barton+eng | = |
| M-27 creator allocations | Medium | = | — | Barton+eng | = |
| M-28 CPI/raw init | Medium | Low (upstream, moot) | — | eng | = |
| M-29 rounding | Low | = (sim exact for all ratios; WT `validation.rs:31` ceil) | — | eng+QA | = |
| M-30 SOL min vs rent | Low | = | — | eng | = |
| M-31 MEV | Low | = | — | eng | = |
| M-32 TODOs | Low | = | — | eng | = |
| M-33 doc bugs | Low | = | Add: ADR-012 docs vs committed `expire.rs` refund | eng+QA+CD | = |
| M-34 deployed ≠ pinned | Info | = | — | eng | = |
| M-35 VRF hygiene | Info | = | "Reveal and settle go in one instruction" is **not** required: pin `seed_slot`, record the value at reveal, and read the stored value at settle (the WT does this). That's safer (§7 H1) | eng | = |
| **M-B1** committed WIP has no real Switchboard init/reveal CPI; its mock reveal needs no authority | — | **High (functional blocker)** | Real Switchboard requires the authority's signature on reveal/commit/init, so committed `977f8f2` can't settle. Fixed in WT `randomness_ix.rs` | eng | Devnet: `third_party_can_reveal_with_pda_authority` |
| **M-B2** committed WIP refunds the fee on expire (`expire.rs`, 1,500-slot timeout) | — | **High** in `977f8f2`; closed in WT (no expire) | Don't reintroduce a fee refund | eng+QA | A's `attack_requester_withholds_reveal_gets_no_refund` |
| **M-B3** solvency invariant ignores pending re-roll hand-ins | — | Low | owed = R × (outside + pending_captures + pending_rerolls) or asserted separately | eng+QA | `invariant_counts_pending_reroll_handins` |
| **M-B4** WT recommit is uncapped, uses a caller-supplied oracle, and has no principal escape | — | **Medium** (HOL lock + oracle griefing) | See §7 H2/H4 | eng+QA | see M-04 row |
| **M-B5** audit-baseline drift (branch amended, uncommitted WT changing during review) | — | Info | Pin a commit per audit round | eng | CI check that the audit tag == deployed hash |
| **M-B6** fee-owner / insider SOL recycling | (in M-16) | High (insider) | as M-16 | Barton+eng | as M-16 |

## 7. Addendum: response to A's merged draft (D1 to D6) and verification of A's M-04 rule (2026-09-25, 16:40–16:45 MT)

### D1 to D6
- **D1 (cherry-pick severity): RESOLVED, Critical, moot-if-hybrid_vault.** A concedes Critical when MPL-Hybrid is in the swap path. My B13 run adds the full-drain evidence PoC 12 lacked.
- **D2 (curve sniping): OPEN. B holds High, as a blocking launch gate.** Evidence: (1) No curve or distribution code exists. `hybrid_launch`
  mints 1B to a `launch_vault` PDA with no signing path (main@658fb95 `constants.rs:15-21`, `launch.rs:84,109,162`), and no
  instruction moves it. (2) Converting is closed on the curve (Barton decision 1), so sniping can't take NFTs or backing.
  (3) The default is an existing audited venue (BRIEF.md:78) with its own anti-snipe features. It's a fairness and distribution
  harm to buyers. Nothing drains the protocol, which doesn't meet A's own Critical definition. I agree it **blocks launch** until a venue with an
  activation point plus anti-snipe schedule is chosen and tested. So the practical effect is the same as A's Critical. Only the label differs.
- **D3 (stale listings): RESOLVED, Low (B concedes).** It's third-party market behaviour, with no protocol loss and no broken invariant. It goes back up to Medium only if
  the UI shows the launch census as the current odds.
- **D4 (VRF failure): RESOLVED in principle.** I accept A's rule (fee at request, never refunded; PDA authority;
  permissionless reveal; bounded recommit by anyone; principal-only expire), with the amendments in H1 to H5 below.
- **D5 (code in scope): RESOLVED, B concedes.** `programs/hybrid_launch` exists locally: `git log` shows `c02be68` (2026-09-24
  15:03 MT) introduced it, and `main` is at `658fb95` (2026-09-25 16:30 MT). My "no code" claim came from an empty GitHub repo.
  Correction back to A: `hybrid_vault` also exists (`wip/hybrid-vault` @ `977f8f2` plus the uncommitted WT), so merged M-11's
  title is wrong. Review of `hybrid_launch` (main@658fb95) against round-1:
  - Fixed: mint and freeze authority → None (`launch.rs:132-155`); 1B to the launch_vault PDA (`constants.rs:15-21`, `launch.rs:84,109,162`); ratio set, min 100,
    checked_mul (`validation.rs:30-47`); pre-funded mint handled (`launch.rs:223-224`).
  - Still stale in committed main: `MAX_TOKEN_FEE_BPS = 1000` (`constants.rs:46`); BURN-only (`constants.rs:50`,
    `validation.rs:37`, `state.rs:41-42`, `launch.rs:190`); creator-chosen bps (`validation.rs:34-36`) = merged M-07.
    The WT moves to 100–200 bps with a `PLATFORM_FEE_OWNER` constant (`constants.rs:47,50,56,58,67-71`). It must be a fixed 200.
  - No curve or distribution instruction exists, so R1-B-03 and R1-B-09 stay design-level.
- **D6 (SOL-minimum scaling): RESOLVED, B concedes to A.** From `sim/fee_stress.py` (FDV 410 SOL at graduation, s = 0.005):
  the top-tier (1/1) break-even multiple m* = N·(0.02 + (s+tx)/F) is 126,390× at R10k, 1,444× at R100k, 411× at 200k, 90× at 500k, 32× at 1M,
  10× at 2.5M and 4.5× at 5M. Farming the top tier is unprofitable over the realistic range of floors and premiums with a flat s, because
  cost ∝ 1/p already. My `k/p_rarest` scaling points the wrong way. Caveats that stay: s ≥ 0.005 (the WT has 0.001), no
  bypass (closed in WT), never recycled to the fee owner (open in WT), and mid tiers with p·m > 2% are only bounded by an s ∝ F,
  so disclose them as a market-priced lottery.

### M-04 verification (Switchboard on-demand: mainnet IDL read-only, `@switchboard-xyz/on-demand` 3.10.6, Rust crate 0.13.0, docs)
- **A's factual claims hold.** `randomness_reveal`, `randomness_commit` and `randomness_init` all require `authority` as signer
  (IDL). The SDK's `Randomness.create` defaults the authority to the payer. `revealIx` gets the signed value from the oracle gateway using
  only public inputs (randomness pubkey, seed slothash, seed slot, RPC). There's no credential, so **anyone can learn the value
  off-chain once the seed slot passes, before any on-chain reveal**. The crate's `get_value()` succeeds only when `clock.slot ==
  reveal_slot`. Switchboard's docs say an unrevealed request **expires about 1 hour after commit**, and that oracles within about 1 h of an enclave key
  rotation won't commit.
- **Can a program-owned authority sign the reveal? Yes.** `invoke_signed` with the PDA seeds marks the PDA as signer for
  the CPI. The WT implements exactly this for init, commit and reveal (`randomness_ix.rs:1-11` and the `seeds` in each handler,
  `RANDOMNESS_AUTHORITY_SEED`). Not yet proven against the real program: this needs the devnet `third_party_can_reveal` test
  (A's step 6). I ran nothing on devnet.
- **Residual holes:**
  - **H1. An atomic `reveal_and_settle` plus recommit can be gamed if settle can be made to fail.** A's step 3 puts reveal and
    settle in one instruction. The attacker learns the value off-chain first. If he can make *settle* fail for a bad value (anything
    requester-influenced in settle: a destination account state, CU pressure from a queue he fills, a token account he controls),
    the whole tx reverts, the value is never recorded, and after the deadline anyone (him included) recommits for a new value. My
    sim: up to 4 draws per fee with cap 3 (3.32 SOL vs 13.30 honest for a legendary at R1M). **Fix:** keep reveal
    separate. Reveal only records `value` on-chain (permissionless, touches no requester accounts), and recommit is refused once
    revealed. Settle reads the stored value against the pinned `seed_slot`. The WT already does this (`randomness_ix.rs:8-11`,
    settle reads the raw `value`). This also makes M-35's "must be one instruction" unnecessary.
  - **H2. Caller-chosen oracle at request and recommit → deliberate non-reveal and an asset lock.** The WT takes `sb_oracle` from
    the caller on commit and recommit. An attacker requests with an oracle he expects to be offline or near rotation. Nobody can reveal. After the
    deadline he races the crank to recommit with another bad oracle. Because settle is FIFO, **his stuck head blocks every later
    request in the vault** (HOL). With an uncapped recommit (WT: `commits` is a u16 with no cap, `randomness_ix.rs:209`) and no expire
    (ADR-012), this is an indefinite collection-wide stall, and honest principals are locked. **Fix:** the program picks the oracle
    (for example by `hash(seq, commits) mod queue.oracles`, filtering for a recent heartbeat), requires a **different** oracle on each recommit, caps
    recommits at 3, then allows a principal-only expire that can batch-clear heads. It doesn't give him a re-draw after seeing a value
    (recommit is refused once revealed), but it is a cheap griefing and liveness hole.
  - **H3. A short deadline turns "censor the reveal" into a re-draw.** A's M-20 proposes a deadline ≤ 150 slots. Any tx can
    write-lock the randomness/request accounts without owning them. An attacker who dislikes the value (known off-chain) can
    fill the per-account write-CU budget with high-priority txs for ~150 slots (rough cost ~1–2 SOL at aggressive priority fees,
    **not measured**) so the crank's reveal doesn't land, then recommit for a new value. At ~9,000 slots (≈ 1 h, matching
    Switchboard's own expiry, as in the WT `REVEAL_TIMEOUT_SLOTS`) this costs ~60× more and is impractical. **Fix:** deadline ≈
    9,000 slots, recommit refused after reveal, and a reveal crank with priority-fee escalation. This is theoretical and needs a cost
    measurement before anyone rates it above Low.
  - **H4. Liveness.** A's rule (cap 3, then principal-only expire after ≥ 1 day) bounds the lock to about 4 h plus 1 day. The WT has **no
    expire at all**, so a sustained Switchboard queue outage locks principals indefinitely. Adopt A's expire. Pause must never block
    reveal, recommit or expire (the WT's pause scope already exempts reveal and recommit).
  - **H5. Honest users and the fee on an outage.** Keeping the fee costs an honest user 2%·R + s per stuck request (at R1M:
    20k tokens + 0.005 SOL). That's small, and a refund reopens the abort. Acceptable if disclosed. If Barton wants goodwill,
    fees must sit in a **program-owned fee vault** (not paid straight to the fee owner) so a governance-gated,
    outage-wide refund is possible. Otherwise no refund is technically possible.
  - **H6. The reveal must not depend on requester-controlled balances.** Reveal needs `sb_reward_escrow` (WSOL) and a payer. The
    escrow is created at init by the program flow, and the payer is whoever cranks, so it's fine as long as the requester can't close
    or drain the escrow. It can't, because only the PDA authority can. Keep it that way.
- **Verdict:** A's rule is **sound in principle and is the right direction**. Adopt it with H1 (separate reveal that records the value),
  H2 (program-pinned, rotating oracle), H3 (deadline ≈ 1 h, not 150 slots), and H4 (cap plus principal-only expire). The WT already has H1,
  PDA authority, no refund and a ~1 h deadline. It lacks the recommit cap, program-chosen oracle and principal escape. Committed `977f8f2`
  has neither: it refunds on expire, and its mock reveal requires no authority (M-B1, M-B2).

### Merged file edits
Per the parent's steering, I annotated `docs/security/merged/round1-merged.md` inline with lines labeled "B:" (D1 to D6 marked
resolved or open with both positions; M-03, M-04, M-11, M-16, M-19, M-20, M-24, M-35 notes). Nothing was restructured. `docs/security/review-a/`
was not touched.
