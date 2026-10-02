# Mintmark: merged security findings, round 1 (Auditor A + Auditor B)

- **Prepared by:** Auditor A worker, merge round, 2026-09-25 (MT). Auditor B's positions are recorded as written in
  `security/auditor-b/round1.md`. Where A and B disagree, both are listed in "Disagreements" and **left open for B**.
- **Sources:** `security/auditor-a/round1.md` (A-01..A-13; A's design risks, renamed **A-D01..A-D08** to avoid clashing
  with B's IDs), `security/auditor-a/cross-review.md` (new fee findings **F-01..F-13**, VRF analysis),
  `security/auditor-b/round1.md` (**B-01..B-14** = R1-B-01..R1-B-14), docs as of BRIEF Decisions 2026-09-25 4:27 PM MT.
- **Settled by Barton (not reopened):** (1) one graduation; converting is closed during the curve; at graduation
  liquidity → DEX, the full collection is minted into escrow from graduation proceeds, then converting opens (LP
  custody and crank are OPEN, owner engineer). (2) Fee: 2% of ratio on capture and each re-roll, in collection tokens,
  on top of N, directly to ONE fixed fee address Barton controls; rate hard-capped at 2% and never raised; release
  returns exactly N; re-rolls also pay a small fixed SOL minimum; destination change either never or multisig + 7-day
  timelock (TBD).
- **Evidence base:** executed on-chain PoCs 10/11/12 against a local build of upstream MPL-Hybrid @ `aacf1a53`; models
  PoC 01/02/03 (`security/auditor-a/poc/`); B's sims (`security/auditor-b/sim/reroll_ev_r1_output.txt`); a read-only
  Switchboard IDL check (cross-review §3.2). Local only. Nothing deployed, no real keys or funds.
- **Status values:** `open` · `moot-if-hybrid_vault` (upstream MPL-Hybrid only; kept as a requirement hybrid_vault must
  meet) · `needs Barton` · `resolved-by-decision` (a Barton decision closes it; a regression test is still required).
- **Test style:** every test is an **attack test that must FAIL to exploit** (the transaction errors with the named
  error, or the invariant holds). The upstream PoCs 10/11/12 are ported against `hybrid_vault` as `regress_poc10_*`,
  `regress_poc11_*`, `regress_poc12_*` (LiteSVM, `tests/track-a-hybrid/`, ADR-007). Error names below are proposed.
  The engineer may rename them, but each test must assert a *specific* error, not just "any failure".
- **B: (Auditor B, 2026-09-25 ~16:45 MT)** Inline notes labeled "B:" were added by Auditor B. Nothing else was changed. D1 to D6 are
  updated in "Disagreements". Full evidence is in `security/auditor-b/cross-review.md` §6 (per-ID table) and §7 (D1 to D6, M-04 verification).

## Summary table

| ID | Sev | Title | Status | Owner | Sources |
|---|---|---|---|---|---|
| M-01 | Critical | Admin can change amount / mint / fee destination instantly → escrow drain (upstream `update_recipe`) | moot-if-hybrid_vault | engineer + QA | A-01, A-02, A-08, B-10(7) |
| M-02 | Critical | Capturer picks the NFT; predictable, abortable, biased reroll (upstream capture) | moot-if-hybrid_vault | engineer + QA | A-03, A-04, A-05, A-12, B-01, B-14 |
| M-03 | Critical (A) / High (B) | Launch-window sniping and bundles on the bonding curve | open | engineer | A-D05, T-CURVE-03/05, B-03 |
|  | B: | D2 still open: B holds High **as a blocking launch gate** (no drain; no curve code exists). The practical effect is the same as Critical | | | |
| M-04 | High | VRF selective reveal: requester-only reveal + refund on expire = free abort of bad draws | open | engineer + QA | B-02, A-D02, F-13 |
| M-05 | High | Fee account substitution: user pays the fee to themselves / wrong mint / reassigned ATA owner | open | engineer + QA | F-06 (A-02 class) |
| M-06 | High | Release + capture bypasses the re-roll SOL minimum | open | engineer + QA | F-03 |
| M-07 | High | `hybrid_launch` contradicts the fee decision (0–10% creator-set rate, BURN-only, no fee owner) | open | engineer + QA | F-01, B-05 |
| M-08 | High | "Fee can never be raised" is false while the program is upgradeable | needs Barton | engineer + Barton | F-08, B-10(1-2) |
| M-09 | High | One upgrade key over every collection's vault; operator powers | open | engineer + Barton | B-10, A-13 |
| M-10 | High | Graduation / migration: trigger, LP custody, pool front-running, price gap, proceeds carve-out | open | engineer | A-D06, T-CURVE-07/08, B-09 |
| M-11 | High | `hybrid_vault` doesn't exist yet; every safeguard depends on it | open | engineer | A-D01 |
|  | B: | Correction: `hybrid_vault` DOES exist (`wip/hybrid-vault` @ `977f8f2` plus a large uncommitted working tree). The requirement stands | | | |
| M-12 | High | Pooled/shared escrow and burn paths break per-collection backing (upstream) | moot-if-hybrid_vault | engineer + QA | A-06, A-07 |
| M-13 | High | Mint/freeze authority, supply and launch destination must be enforced on-chain | resolved-by-decision (implemented in `hybrid_launch`) | QA | T-CURVE-01/09, A-D07, B-10(8) |
| M-14 | Medium | Fee must never be backing; release must carry no fee (per-instruction deltas) | open | engineer + QA | F-07, T-BURN-01 |
| M-15 | Medium | Fee ATA missing/closed → every capture and re-roll fails (DoS) | open | engineer + QA | F-05 |
| M-16 | Medium | Insider discount: the fee owner's own re-rolls pay the fee to itself | needs Barton | Barton + engineer | F-04 |
| M-17 | Medium | Fee owner as a hot wallet; destination-change policy (never vs 7-day timelock) | needs Barton | Barton + engineer | F-09 |
| M-18 | Medium | Fee tokens accrue to one address → sell pressure; Stonk.fun disclosure pattern | needs Barton | Barton | F-10 |
| M-19 | Medium | Grinding cost floor after graduation depends on the SOL minimum (value TBD) | needs Barton | Barton + engineer | F-02, B-04, B-05 |
| M-20 | Medium | Escrow cornering and head-of-line queue DoS | open | engineer + QA | B-06 |
| M-21 | Medium | Metadata commitment excludes image bytes → art swap after reveal | open | engineer + QA | B-11 |
| M-22 | Medium | Graduation mint crank can stall → converting never opens / manifest withheld | open | engineer + QA | B-13, decision 1 |
| M-23 | Medium | FIFO/seq merge and Feistel+Merkle binding correctness (unfuzzed) | open | engineer + QA | A-D03, A-D04 |
| M-24 | Medium (B) / Low (A) | Wrap/unwrap arbitrage vs stale listings; pool adverse selection vs census | open | engineer (UI) | B-07 |
| M-25 | Medium | Curve/AMM integration: rounding, boundaries, slippage on composite flows | open | engineer + QA | T-CURVE-02/04/06 |
| M-26 | Medium | Pause scope: no admin power may block release/settle/expire | needs Barton | Barton + engineer | B-10(4), N4 |
| M-27 | Medium | Creator/team NFT allocations outside the VRF pool | needs Barton | Barton + engineer | B-10(5), Q-H6 |
| M-28 | Medium | Discarded CPI result; raw account init (upstream) | moot-if-hybrid_vault | engineer | A-09, A-10 |
| M-29 | Low | Fee rounding and arithmetic (ceil, nonzero, u128, checked) | open | engineer + QA | F-11 |
| M-30 | Low | SOL minimum below rent-exempt can DoS re-rolls | open | engineer | F-12 |
| M-31 | Low | MEV/sandwich around converts and composite swap flows | open | engineer | B-08, T-CURVE-06 |
| M-32 | Low | Shipped "Need to add account checks" TODOs; authority-semantics footgun (upstream) | moot-if-hybrid_vault | engineer | A-11, A-12 |
| M-33 | Low | Doc and code bugs: fee still described as burned/split/≤10%/refunded | open | engineer + QA + CD | B-12, A-D08, cross-review §5 |
| M-34 | Info | Deployed MPL-Hybrid binary ≠ pinned commit; externally upgradeable | moot-if-hybrid_vault | engineer | A-13 |
| M-35 | Info | VRF integration hygiene: SDK build, reveal-slot read, program IDs, CU | open | engineer | B-14, B/TM-15 |

---

## Findings: Critical

### M-01 — Critical — Admin can change amount / mint / fee destination instantly → escrow drain
- **Status:** moot-if-hybrid_vault (the requirement stands). **Owner:** engineer + QA. **Sources:** A-01, A-02, A-08, B-10(7).
- **Evidence:** upstream `update_recipe.rs:54-138` is single-signer with no timelock. `amount` is written at `:114-116`;
  `recipe.token` / `recipe.fee_location` are overwritten **unconditionally** at `:91-93`. Capture/release honour it at
  `capture_v2.rs:284`, `release_v2.rs:287`. **Executed:** PoC 10 drained 3,000,000 → 0; PoC 12 swapped the mint on an
  update whose every Option was `None`.
- **Remediation (hybrid_vault rule):** `hybrid_vault` has **no instruction that modifies `ratio`, `mint`,
  `collection_size`, `trait_root`, `vault` token account, fee rate, fee amounts, or the per-launch fee token account
  after init**, and no close instruction for the config/vault. These values are read from the immutable
  `hybrid_launch::LaunchConfig` (no copies). The only mutable fee field anywhere is the platform `fee_owner`, and only
  if Barton picks option (B) in M-17 (3-of-5 multisig + 7-day in-program timelock). No instruction takes a mint, fee
  destination or amount as an argument. At init, assert `collection_size × ratio_base ≤ supply_base` (`checked_mul`).
- **Tests:**
  - `regress_poc10_escrow_drain_must_fail`: setup a launched collection with 3 NFTs out and a vault holding 3×R. Action:
    enumerate every instruction in the `hybrid_vault` IDL and try each with arguments/accounts that would set the
    capture amount to 0 or change R. Expected: no such instruction exists (IDL assertion), and every attempted
    mutation fails. After 3 releases the vault = 0 and each user got exactly R, never more.
  - `regress_poc12_token_swap_must_fail`: action: `request_capture` passing a junk mint + junk-mint user ATA. Expected:
    `ConstraintAddress`/`MintMismatch` (mint must equal `LaunchConfig.mint`), and no NFT reserved.
  - `config_has_no_mutation_or_close_instruction_in_idl` (extend the existing hybrid_launch test to `hybrid_vault`).

### M-02 — Critical — Capturer picks the NFT; predictable, abortable, biased reroll
- **Status:** moot-if-hybrid_vault (the requirement stands). **Owner:** engineer + QA. **Sources:** A-03, A-04, A-05, A-12,
  B-01, B-14.
- **Evidence:** `capture_v2.rs:52-54` (caller-supplied `asset: UncheckedAccount`); `:183-185` (the signer check runs
  only if the passed authority equals `recipe.authority`); reroll seed from SlotHashes/Clock/count at `:187-203`
  (`capture.rs:167-183`); `checked_rem(max-min)+min` never selects `max`, draws with replacement (52.8% distinct over
  10k). No CPI guard anywhere in `src/`. **Executed:** PoC 11 (a bot cherry-picked a named rare); PoC 01 11/11. B's sim:
  rares 565–5,652× cheaper than honest VRF (Critical if MPL-Hybrid is the engine).
- **Remediation (hybrid_vault rule):** `request_capture` and `request_reroll` take **no asset / index argument**. The
  selection happens only in `reveal_and_settle` from the VRF value, using `uniform_below(value, pool_len)` with
  **rejection sampling** (no modulo), **without replacement**, over the candidate set pinned at request time (`seq <
  request.seq`). No SlotHashes, Clock or counters in any value-bearing selection. The re-roll hand-in is excluded from
  its own draw. MPL-Hybrid never holds or moves Mintmark backing.
- **Tests:**
  - `regress_poc11_cherrypick_must_fail`: setup: pool of 10 with one known "rare" index. Action: a bot sends
    `request_capture` with the rare asset appended as an extra account and as `remaining_accounts`. Expected: the extra
    accounts are ignored and the delivered NFT is whatever the VRF selects; over 10,000 simulated settles with mocked VRF
    values the rare frequency is within 3σ of 1/10.
  - `attack_predict_and_abort_success_equals_base_rate`: a wrapper program CPIs `request_*` and aborts unless it
    predicts a rare. Expected: the prediction is impossible (the value isn't known at request). Measured success = base
    rate ± 3σ.
  - `every_pool_index_is_reachable_and_uniform`: chi-square over mocked uniform VRF values for pool sizes 1, 2, 3, 2^k,
    2^k±1, 100,000. Expected: every index reachable, no index > 2× expected frequency.

### M-03 — Critical (A) / High (B) — Launch-window sniping and bundles on the bonding curve
- **Status:** open (severity disputed, see Disagreements D2). **Owner:** engineer. **Sources:** A-D05,
  T-CURVE-03/05, B-03.
- **Evidence:** Stonk.fun: 522 wallets traded 3.4× supply within a minute (Bitquery, via B-03). pump.fun: 36.5% of
  supply held by bundled accounts at migration (MELT, via B-03). The venue defaults to an existing audited curve (BRIEF.md:78).
  The anti-snipe mechanism isn't chosen. Decision 1 removes NFT sniping during the curve (converting is closed), but
  not token sniping.
- **Remediation:** choose a venue whose program enforces an **activation point** (trading opens at a stored slot/time,
  not at pool creation) plus either an on-chain **decaying anti-sniper fee schedule** or **per-tx/per-slot caps** in
  the opening window. The creator's buy goes through the same rules, capped (≤ 2% of supply) and disclosed. There's no
  privileged pre-allocation. Launch config and pool creation happen in the same program flow, so no one can pre-create
  the pool. Publish the exact opening rules. Any anti-sniper fee proceeds go to LP or are burned (admin-multisig-timelock
  §6), never to a wallet.
- **B:** Severity High, as a **blocking launch gate** (D2 open). `hybrid_launch` has no curve or distribution instruction. The 1B sits in the
  `launch_vault` PDA (main@658fb95 `launch.rs:84,109,162`). Add a test: `venue_accepts_preexisting_mint_from_launch_vault`, because
  Meteora DBC and Raydium LaunchLab may create their own mint (unverified).
- **Tests:**
  - `attack_bundle_create_and_buy_before_activation_fails`: in one bundle, create + buy at slot = creation. Expected: the
    buy fails with the venue's "not activated" error.
  - `attack_opening_window_cap_exceeded_fails`: in the first N slots, buy cap+1 in a single tx and cap/2+cap/2+1 across
    txs from one wallet. Expected: the last buy fails. Split across wallets is documented as a known speed bump.
  - `anti_snipe_fee_decays_to_base_and_proceeds_go_to_lp_or_burn`: check the fee at slots 0, N/2, N and assert the
    destination account.

## Findings: High

### M-04 — High — VRF selective reveal + refund on expire = free abort of bad draws
- **Status:** open. **Owner:** engineer + QA. **Sources:** B-02, A-D02, F-13. Full analysis: cross-review §3.
- **Evidence:** docs refund the escrowed fee on expire and claim "anyone can reveal" (hybrid-rarity-and-assignment.md:
  184-187, 206-210, 257, 356; DECISIONS.md:310-312). Verified: the Switchboard On-Demand `randomness_reveal` requires the
  randomness **authority as signer** (on-chain IDL). The SDK default authority is the requester's payer. The value can
  be fetched off-chain from the oracle gateway using only public inputs (`randomness.js:180-222`). `get_value()` works
  only in the reveal slot (`randomness.rs:51-57`). B's sim: legendaries 3–9× cheaper with the abort.
- **Remediation (exact rules):**
  1. `request_*` creates the randomness account via CPI with `authority = PDA["rng_authority", request]` and CPI-commits
     in the same instruction. It rejects any randomness account whose authority isn't that PDA
     (`RandomnessAuthorityMismatch`), requires `seed_slot == clock.slot − 1` (`RandomnessNotFresh`) and not-yet-revealed
     (`RandomnessAlreadyRevealed`), and pins pubkey, seed_slot and oracle in the request.
  2. The 2% token fee and the SOL minimum are transferred **directly** to the fixed fee destinations **in `request_*`**
     and are **never refunded**. There's no fee escrow in the request PDA (supersedes N7).
  3. `reveal_and_settle` is callable by **anyone**. It CPIs the reveal signing as the PDA, reads the value in the same
     slot, and executes the reassignment. The outcome is a pure function of `(value, pinned candidate set)`. No cancel
     after commit.
  4. After `deadline_slot` unrevealed, **anyone** may `recommit` (a new randomness account, a different oracle, the same
     seq/candidate set, no new fee, no refund), up to `MAX_RECOMMITS = 3`.
  5. Only after that and ≥ 1 day may anyone `expire`. It returns **principal only** (N tokens or the handed-in NFT) and
     can clear up to K consecutive stale heads in one call.
  6. Alert on every recommit/expire and publish the per-oracle abandon rate. A devnet test must prove a third party can
     reveal before Switchboard is chosen (otherwise use ORAO).
- **Tests:**
  - `attack_requester_withholds_reveal_gets_no_refund`: setup: user requests a re-roll (fee F, SOL s). Action: the user
    never reveals; warp past the deadline; call `expire` directly. Expected: `expire` fails with `RecommitsRemaining`.
    After 3 recommits + deadline, `expire` returns the handed-in NFT; the fee ATA keeps +F; the user's token balance
    didn't increase; no lamport refund of s.
  - `attack_foreign_randomness_account_rejected`: pass a randomness account the attacker created (authority = attacker)
    or one already revealed. Expected: `RandomnessAuthorityMismatch` / `RandomnessAlreadyRevealed`.
  - `third_party_can_reveal_and_settle`: a wallet unrelated to the requester calls `reveal_and_settle`. Expected: success,
    and the delivered NFT equals the one computed off-chain from the value.
  - `selective_expiry_cannot_shift_later_draw`: two queued requests s−1, s by one attacker. Expected: the candidate set
    and result for s are identical whether s−1 settles or goes through recommit/expire (assert the pinned pool hash).
- **B: M-04 verification (security/auditor-b/cross-review.md §7):** A's Switchboard claims are verified (IDL: authority signs init/commit/reveal; the value can be fetched
  from the gateway using public inputs; Switchboard expires unrevealed requests after ~1 h). A PDA can sign via `invoke_signed` (the WT does this;
  devnet proof still required). Amendments: **(H1)** don't make reveal+settle atomic. Reveal records the value, recommit is refused once
  revealed, and settle reads the stored value against the pinned seed_slot. Otherwise an attacker who can make settle fail gets up to 4
  draws per fee. **(H2)** The program picks the oracle (WT takes `sb_oracle` from the caller), a different one on each recommit. **(H3)** Deadline
  ≈ 9,000 slots (≈ Switchboard expiry), not 150: a short deadline lets a write-lock flood censor the reveal and force a re-draw (cost
  not measured). **(H4)** Cap at 3, then principal-only expire. The WT recommit is uncapped with no expire, so principals can be locked and FIFO stalled.
  **(H5)** A kept fee on an outage costs honest users 2%·R + s. Refund is only possible from a program-owned fee vault. Committed
  `977f8f2` refunds the fee on expire (`expire.rs`), and its mock reveal requires no authority (it can't work with real Switchboard).
  Added tests: `attack_settle_failure_cannot_trigger_recommit_after_reveal`, `attack_recommit_with_caller_chosen_dead_oracle_rejected`,
  `recommit_capped_then_principal_only_expire`.

### M-05 — High — Fee account substitution (user pays the fee to themselves; wrong mint; reassigned owner)
- **Status:** open. **Owner:** engineer + QA. **Source:** F-06 (the A-02 class: upstream overwrote `fee_location`,
  `update_recipe.rs:91-93`).
- **Risk:** if `request_*` accepts any token account as the fee destination, a user passes their own ATA and gets free
  captures/re-rolls (no grinding floor, fee never paid). The same applies to the SOL-minimum receiver. Classic SPL Token
  has **no ImmutableOwner**, so an ATA's owner can be reassigned with `SetAuthority(AccountOwner)` while the address
  stays the same.
- **Remediation (exact rule):** in `request_capture` and `request_reroll`: `fee_token_account.key() ==
  LaunchConfig.fee_token_account == get_associated_token_address(FEE_OWNER, LaunchConfig.mint)`, **and**
  `fee_token_account.mint == LaunchConfig.mint`, **and** `fee_token_account.owner == FEE_OWNER`, **and** `token_program
  == spl_token::ID`; `sol_fee_receiver.key() == LaunchConfig.sol_fee_receiver`. `FEE_OWNER` is a program constant (or
  the M-17 `FeeConfig` value), **never a creator-supplied launch parameter or instruction argument**. An owner mismatch
  fails closed (`FeeAccountOwnerMismatch`).
- **Tests:**
  - `attack_fee_to_own_ata_rejected`: action: `request_reroll` with the user's own ATA as the fee account. Expected:
    `FeeAccountMismatch`; user balance, fee ATA and pool are unchanged.
  - `attack_fee_to_junk_mint_account_rejected`: action: pass `ATA(FEE_OWNER, junk_mint)`. Expected: `FeeAccountMismatch`
    / `MintMismatch`.
  - `attack_fee_ata_owner_reassigned_fails_closed`: setup: the fee owner `SetAuthority(AccountOwner → attacker)` on the
    fee ATA. Action: `request_capture`. Expected: `FeeAccountOwnerMismatch`; no tokens reach the attacker.
  - `attack_sol_min_to_own_wallet_rejected`: action: pass the user's wallet as the SOL receiver. Expected:
    `SolFeeReceiverMismatch`.
  - `attack_creator_cannot_set_fee_owner_at_launch`: `LaunchParams` has no fee-owner field (IDL assertion);
    `LaunchConfig.fee_owner == FEE_OWNER` for every launch.

### M-06 — High — Release + capture bypasses the re-roll SOL minimum
- **Status:** open. **Owner:** engineer + QA. **Source:** F-03.
- **Evidence:** release is free and capture pays only 2%, while the SOL minimum applies to re-rolls only
  (BRIEF.md:75). PoC 03 Part 4: at R=10k, re-roll 0.00308 SOL vs release+capture 0.00008 SOL (−97%). At R=100k, −79%.
  `validation.rs:33` covers only the token bps.
- **Remediation (exact rule):** `capture_sol_min_lamports ≥ reroll_sol_min_lamports`, both program constants (simplest:
  one `SOL_MIN_LAMPORTS` charged on both). Token fee: capture = re-roll = 200 bps.
- **Tests:**
  - `unwrap_rewrap_never_cheaper_than_reroll`: for every allowed ratio, compare the lamports and tokens debited by
    `release`+`request_capture` vs `request_reroll`. Expected: rewrap ≥ reroll in both tokens and lamports.
  - `attack_capture_without_sol_min_rejected`: a capture transaction with the user holding exactly `N + fee` tokens and
    lamports < SOL_MIN + tx fee. Expected: fails (`InsufficientSolFee` / system transfer error); no reservation.

### M-07 — High — `hybrid_launch` contradicts the fee decision
- **Status:** open. **Owner:** engineer + QA. **Sources:** F-01, B-05.
- **Evidence:** `constants.rs:44-46` (`MAX_TOKEN_FEE_BPS = 1_000`), `validation.rs:31-33` (creator-chosen 0..=1,000
  bps, so 0% and 10% both launch), `validation.rs:37` + `constants.rs:48-50` (BURN only), `state.rs:41-42` (no fee
  owner / fee account), `launch.rs:186-190`; tests `launch.rs:281-283`, `qa_launch.rs:186-187, 373-377` enforce BURN.
- **Remediation:** `pub const FEE_BPS: u16 = 200;` (no parameter). Remove `capture_fee_bps`, `reroll_fee_bps` and
  `fee_destination` from `LaunchParams`. `LaunchConfig` v2 (`LAUNCH_CONFIG_VERSION = 2`) stores `fee_bps = 200`,
  `fee_owner = FEE_OWNER`, `fee_token_account = ATA(FEE_OWNER, mint)` (created by `launch`), `capture_fee_amount =
  reroll_fee_amount = ceil(ratio_base × 200 / 10_000)`, and `sol_min_lamports`. Post-launch check re-reads the fee ATA
  (owner, mint, balance 0).
- **Tests:**
  - `fee_rate_is_fixed_at_200_bps`: launch every allowed ratio × decimals {0, 6, 9}. Expected: `cfg.fee_bps == 200`,
    `cfg.capture_fee_amount == ratio_base / 50`, and `cfg.fee_token_account` exists with owner `FEE_OWNER`.
  - `attack_launch_params_cannot_carry_fee_fields`: IDL has no fee bps/destination/owner arg. Sending the old v1 layout
    fails to deserialize.
  - Replace `attack_fee_destination_other_than_burn_is_rejected` and `qa_every_non_burn_fee_destination_value_rejected`
    with the two tests above.

### M-08 — High — "Fee can never be raised, not even by multisig" is false while the program is upgradeable
- **Status:** needs Barton (copy + freeze timing). **Owner:** engineer + Barton. **Sources:** F-08, B-10(1-2).
- **Evidence:** BRIEF.md:74 promises an absolute. BRIEF.md:62 says upgrades need 3-of-5 + 7-day delay until the
  post-audit freeze. An upgrade can change any constant or transfer target for every collection (Stonk.fun S-3,
  "retained power").
- **Remediation:** (1) the rate is a constant **and** stored per launch in the immutable `LaunchConfig`. `hybrid_vault`
  charges `cfg.capture_fee_amount`/`cfg.reroll_fee_amount` and asserts `cfg.fee_bps ≤ 200` (`FeeAboveHardCap`). (2) Until
  `--final`, the copy says: "The fee is 2% and no setting can raise it. The program itself can only be changed by a
  3-of-5 multisig upgrade with a public 7-day delay, until it's frozen after the audit." (3) Upgrade authority → `None`
  (`--final`) after audit + stabilization (N5: Barton sets the date). (4) A monitor alerts on buffer writes and queued
  upgrades.
- **Tests:** `attack_fee_above_hard_cap_rejected_even_if_config_forged`: inject a `LaunchConfig` with `fee_bps = 201` into
  LiteSVM. Action: `request_reroll`. Expected: `FeeAboveHardCap`. Plus `upgrade_authority_is_multisig_with_7day_timelock`
  (a deploy-script check that reads the program data account and the Squads config; fails otherwise).

### M-09 — High (Critical if any single key) — One upgrade key over every collection's vault; operator powers
- **Status:** open. **Owner:** engineer + Barton. **Sources:** B-10, A-13.
- **Evidence:** a single `hybrid_vault` program holds all collections' backing. Whoever controls the upgrade can ship a
  `sweep` (B-10 scenario). The 3-of-5 + 7-day policy is stated (BRIEF.md:62) but not yet verified on-chain; signers are
  open (DECISIONS Q8).
- **Remediation:** upgrade authority = Squads v4 vault PDA, ≥ 3-of-5 independent signers, `configAuthority = null`,
  timelock ≥ 7 days, signers published. `--final` after audit. `assert_launch_ready` (or the launch script) refuses to
  launch unless those hold. There's no instruction that moves vault tokens or pooled NFTs except `release` (to the NFT
  owner) and `reveal_and_settle` (to the requester). There's no admin withdraw, and no instruction takes a free
  destination.
- **Tests:** `launch_refused_unless_upgrade_authority_is_timelocked_multisig` (script test); `vault_has_no_outflow_except_release_and_settle`
  (enumerate IDL instructions; for each, assert the vault ATA balance can't decrease except by exactly R to the NFT
  owner in `release`).

### M-10 — High (Critical if any operator signer or wallet in the path) — Graduation / migration
- **Status:** open (LP custody + crank are OPEN per Barton; owner engineer). **Sources:** A-D06, T-CURVE-07/08, B-09.
- **Evidence:** the event shape is settled (BRIEF.md:65). Who triggers it, where the LP goes, pool pre-creation and the
  price gap are unspecified. Precedent: pump.fun's privileged migrate key, ~12,300 SOL (B-09).
- **Remediation:** graduation is **permissionless and deterministic** (it fires at a stored threshold) and **one-shot**
  (`graduated = true` set before any transfer). Migration seeds the DEX pool at the curve's final price from program
  reserves in the same flow, and fails closed if a pool already exists at another price. **LP tokens are burned or
  locked in a PDA with no withdraw instruction.** The NFT-mint carve-out from proceeds is a program-computed amount
  (per-NFT mint cost × N, constant formula) paid to a program PDA that only the mint crank can spend. It never goes to
  an operator wallet. `hybrid_vault` backing is never counted as curve reserves.
- **Tests:** `attack_graduate_twice_fails` (`AlreadyGraduated`); `flash_buy_to_threshold_then_migrate_price_continuity`
  (the AMM spot after migration equals the final curve price within 1 bp); `attack_precreated_pool_at_skewed_price_fails_closed`;
  `lp_tokens_burned_or_pda_locked` (assert the LP mint supply held by any non-PDA = 0); `graduation_needs_no_privileged_signer`
  (a random keypair triggers it successfully).

### M-11 — High — `hybrid_vault` doesn't exist yet; every safeguard depends on it
- **Status:** open. **Owner:** engineer. **Source:** A-D01.
- **B:** Correction: code exists. `wip/hybrid-vault` @ `977f8f2` (guardian single-key pause up to ~7 days, fee refunded on expire, 1,500-slot
  timeout, mock reveal, no image hash) plus an uncommitted working tree (ADR-012: PDA-authority randomness, no expire, fee at
  request to a pinned `fee_account`). Neither auditor has reviewed `graduation.rs`, `mint_assets.rs` or `open_vault.rs`. Pin a commit per audit round.
- **Remediation:** build `hybrid_vault` to the checklist at the end of this file. It becomes the top-priority audit
  target. No converting on devnet until the checklist tests pass.
- **Test:** all M-01, M-02, M-04..M-07, M-12, M-14, M-15 tests green in CI with `--locked`, plus a fuzz campaign (Trident)
  of `vault ≥ R × NFTs_outside` over ≥ 10^6 random op sequences.

### M-12 — High — Pooled/shared escrow and burn paths break per-collection backing (upstream)
- **Status:** moot-if-hybrid_vault (the requirement stands). **Owner:** engineer + QA. **Sources:** A-06, A-07.
- **Evidence:** V2 escrow seeds `["escrow", authority]`, shared across recipes (`init_escrow_v2.rs:11-14`,
  `capture_v2.rs:44-48`). Burn paths at `capture_v2.rs:261-272`, `release_v2.rs:191-208`. PoC 02 5/5.
- **Remediation:** one vault PDA per collection, `["vault", launch_config]`, with its own ATA. No instruction burns
  from the vault or burns NFTs on release. Invariant `vault_balance ≥ R × nfts_outside` per collection.
- **Tests:** `cross_collection_release_cannot_draw_other_vault` (two launches with the same creator: releasing an NFT of
  A with B's vault account fails with `ConstraintSeeds`); `fuzz_backing_invariant_per_collection`.

### M-13 — High — Mint/freeze authority, supply and launch destination enforced on-chain
- **Status:** resolved-by-decision (implemented in `hybrid_launch`, ADR-010; regression tests must stay green).
  **Owner:** QA. **Sources:** T-CURVE-01/09, A-D07, B-10(8).
- **Evidence:** `launch.rs:96-99` (mint authority PDA, freeze `None`), post-checks at `:155`; the 1B goes to the
  launch-vault PDA ATA with no signing path (DECISIONS ADR-010 step 3).
- **Remediation (remaining):** the curve distribution out of the launch vault must arrive via the multisig+timelock
  upgrade path and move tokens only to the venue's program-owned pool/curve accounts.
- **Test:** existing `hybrid_launch` suite + `launch_vault_tokens_only_move_to_curve_program_accounts` once the curve exists.

## Findings: Medium

### M-14 — Medium — Fee must never be backing; release carries no fee
- **Status:** open. **Owner:** engineer + QA. **Sources:** F-07, T-BURN-01 (restated for the fee address).
- **Remediation (exact deltas):** `request_capture`: user −(N + fee), vault +N, fee ATA +fee (two separate transfers;
  the fee never touches the vault or a request PDA). `request_reroll`: user −fee, fee ATA +fee, vault unchanged, NFT →
  vault PDA. `release`: vault −N, user +N, fee ATA unchanged, no SOL fee beyond the network fee. `reveal_and_settle` /
  `recommit` / `expire`: fee ATA unchanged. Mint supply stays exactly 1B × 10^d forever (nothing is burned).
- **Tests:** `per_instruction_token_deltas_exact` (assert the deltas above for every instruction);
  `release_pays_exactly_n_with_zero_fee` (after 1k captures/re-rolls/releases: every release credits exactly `ratio_base`,
  and `Δfee_ata == 0`); `fuzz_vault_ge_ratio_times_outside_and_supply_constant`.

### M-15 — Medium — Fee ATA missing or closed → every capture/re-roll fails
- **Status:** open. **Owner:** engineer + QA. **Source:** F-05.
- **Evidence:** freeze is impossible (freeze authority `None`, `launch.rs:155`). But the owner can close a zero-balance
  ATA, or it may never have been created.
- **Remediation:** `hybrid_launch::launch` creates `ATA(FEE_OWNER, mint)`. `request_capture`/`request_reroll` CPI
  `create_associated_token_account_idempotent` (payer = user, owner = FEE_OWNER, mint = cfg.mint) before the fee
  transfer. The fee owner is a multisig (M-17).
- **Tests:** `fee_ata_closed_is_recreated_permissionlessly`: setup: the fee owner sweeps to 0 and closes the ATA.
  Action: an ordinary user calls `request_capture`. Expected: success; the ATA is recreated at the same address with
  owner FEE_OWNER and balance = fee.

### M-16 — Medium — Insider discount: the fee owner's own re-rolls pay the fee to itself
- **B:** B rates this High (insider). The WT sends the SOL minimum to `fee_owner` too (`hybrid_vault/.../request.rs:224-226, 272-274`).
  Route it to a sink.
- **Status:** needs Barton. **Owner:** Barton + engineer. **Source:** F-04.
- **Evidence:** PoC 03 Part 3. At R=1M, the expected cost of a 1-of-1,000 is 11 SOL for the public, 2 SOL for the fee
  owner if the SOL minimum also goes to it, and 3 SOL if the SOL minimum goes to a sink. This can't be blocked on-chain.
- **Remediation:** the SOL minimum pays the VRF cost, and any excess goes to a sink the fee owner doesn't control
  (incinerator `1nc1nerator11111111111111111111111111111111`), **not** to Barton. Barton publishes a policy that the fee
  address and affiliated wallets never capture or re-roll. A public monitor flags fee-address outflows that reach
  converting wallets. Disclose "the platform earns from re-rolls".
- **Test:** `sol_min_never_reaches_fee_owner` (assert the lamport destination on re-roll ≠ FEE_OWNER and equals the configured sink/VRF).

### M-17 — Medium — Fee owner as a hot wallet; destination-change policy
- **Status:** needs Barton. **Owner:** Barton + engineer. **Source:** F-09.
- **Risk:** a compromised hot key sweeps fees, closes the ATA (M-15) or reassigns its owner (M-05). With "never
  changeable", the compromise is permanent.
- **Remediation:** the fee owner is a **Squads vault PDA** (≥ 2-of-3, hardware keys). Barton picks **(A) never** (a
  `FEE_OWNER` program constant; changing it means an upgrade) or **(B, A recommends)** a platform `FeeConfig` PDA whose
  **only** mutable field is `fee_owner`: `propose_fee_owner` (3-of-5 multisig) → `execute_fee_owner` after ≥ 7 days
  (604,800 s, checked against `Clock`), the pending value shown on every page, cancel allowed, **no rate field**. The
  per-launch fee ATA is re-derived from the current `fee_owner` at request time and still validated as
  `ATA(fee_owner, mint)`.
- **Tests (option B):** `attack_execute_fee_owner_before_7_days_fails` (`TimelockNotElapsed`);
  `attack_fee_owner_change_by_non_multisig_fails` (`Unauthorized`); `fee_config_has_no_rate_field` (IDL/struct assertion);
  `pending_fee_owner_visible_and_cancellable`.

### M-18 — Medium — Fee tokens accrue to one address: sell pressure and disclosure
- **Status:** needs Barton. **Owner:** Barton. **Source:** F-10.
- **Evidence:** PoC 03 Part 6. A full capture of a max-size collection routes 20,000,000 tokens (2% of supply) to one
  address; 3 re-rolls/NFT makes it 8%. Selling 20M into a 200M-token CP pool: −17%. "Fee wallet sells into the coin's
  pool" is the documented Stonk.fun harm.
- **Remediation:** every token page shows the fee rate, fee address, balance, cumulative fees and all outflows.
  Publish a reconciliation script. Barton publishes a written sell policy (e.g. no sales into the coin's own pool for X
  days after graduation and ≤ Y% of pool depth per day, or vesting). The fee address appears in the top-holder view.
  No off-ledger transfers.
- **Test:** `ledger_reconciles_fee_address` (QA: for a devnet run, Σ on-chain fee transfers == the site's cumulative
  figure, and every outflow is listed).

### M-19 — Medium — Grinding cost floor after graduation depends on the SOL minimum
- **Status:** needs Barton (value). **Owner:** Barton + engineer. **Sources:** F-02, B-04 (mostly resolved by decision
  1), B-05.
- **Evidence:** PoC 03 Part 1. Without a SOL minimum, break-even is 20× floor for a 0.1% tier at any price. At R=10k
  (graduation FDV 400 SOL) the SOL minimum is 97% of the attempt cost. A flat per-attempt minimum scales correctly
  with rarity (cost ∝ 1/p), so no rarity scaling is needed.
- **Remediation:** `SOL_MIN_LAMPORTS` as a program constant ≥ max(890,880, VRF cost + margin). A proposes 5,000,000
  (0.005 SOL); Barton confirms. The same value applies to capture (M-06). The UI shows live pool odds and the
  expected cost per tier `(fee_in_SOL + s)/p`.
- **Test:** `sol_min_charged_on_every_request` (capture and re-roll each debit ≥ SOL_MIN_LAMPORTS to the configured
  receiver; there's no path with 0).
- **B:** D6 resolved in A's favour. `sim/fee_stress.py`: 1/1 break-even at s = 0.005 is 126,390× floor (R10k), 1,444× (100k), 411× (200k),
  90× (500k), 32× (1M), 10× (2.5M), 4.5× (5M). A flat s is enough for the top tier. The WT constant is 0.001 SOL
  (`hybrid_launch/src/constants.rs:56`). Raise it to 0.005. Mid tiers with p·m > 2% aren't bounded by any fixed s, so disclose them.

### M-20 — Medium — Escrow cornering and head-of-line queue DoS
- **Status:** open. **Owner:** engineer + QA. **Source:** B-06.
- **Remediation:** a non-refundable fee (M-04) makes stall attempts cost the full fee. Short `deadline_slot`
  (≤ 150 slots). `expire`/`recommit` handle up to K consecutive stale heads per call. Token or foreign-asset donations
  never affect accounting (use `≥` invariants, never balance-derived counts). Minimum collection size 100 is already
  enforced (`validation.rs:30`). Show live pool size and top-holder concentration.
- **Tests:** `queue_of_100_unrevealed_requests_clears_in_bounded_calls`; `donation_to_vault_changes_no_outcome`.
- **B:** Don't use a 150-slot deadline. It conflicts with M-04 H3. Use ~9,000 slots with batch-clear. Also: the WT solvency invariant ignores pending
  re-roll hand-ins (`hybrid_vault/src/invariants.rs:3,15-19`).

### M-21 — Medium — Metadata commitment excludes image bytes
- **Status:** open. **Owner:** engineer + QA. **Source:** B-11 (BRIEF.md:68 security note).
- **Remediation:** the launch gate rejects any `image`/`animation_url`/`uri` that isn't content-addressed (`ar://`,
  `ipfs://<CID>`). The JSON includes `image_sha256`. The leaf is `sha256("mintmark-trait-v1" ‖ j ‖
  sha256(metadata_json_j))`, and the on-chain URI points to the same immutable JSON. The verify script downloads and
  hashes every image. The art revealed at graduation must match the committed root.
- **Tests:** `launch_rejects_mutable_http_image_uri`; `verify_script_detects_swapped_image` (swap one image → the script fails).

### M-22 — Medium — Graduation mint crank can stall, so converting never opens
- **Status:** open. **Owner:** engineer + QA. **Sources:** B-13 (partly resolved by decision 1), BRIEF.md:66.
- **Remediation:** the mint crank is **permissionless and resumable** (a stored cursor, idempotent per index). Anyone
  can run it, funded from the program-held carve-out (M-10). `converting_open` is set only when `minted == N` and every
  leaf is bound on-chain (no off-chain proof needed later). If the carve-out runs short, graduation doesn't complete
  (fail closed before migration), or the shortfall is a documented constant formula. Publish the max safe N per ratio
  from a CU/tx benchmark.
- **Tests:** `converting_closed_until_fully_minted`; `mint_crank_resumes_after_interruption_by_any_caller`;
  `mint_crank_idempotent_no_double_mint`.

### M-23 — Medium — FIFO/seq merge and Feistel+Merkle binding correctness
- **Status:** open. **Owner:** engineer + QA. **Sources:** A-D03, A-D04.
- **Remediation:** the trait root is committed before the VRF permutation seed is requested (enforced by state
  machine). Feistel permutation cycle-walking is proven bijective for every N. Merge only `seq < request.seq`.
- **Tests:** `feistel_is_bijection_for_all_n_up_to_100k`; `seed_request_rejected_before_root_committed`;
  `fuzz_fifo_candidate_set_matches_reference_model`.

### M-24 — Medium (B) / Low (A) — Wrap/unwrap arbitrage and pool adverse selection
- **Status:** open (severity disputed, D3). **Owner:** engineer (UI). **Source:** B-07.
- **Remediation:** show the live "unwrap value = R × price" next to listings and offers, the live pool census and odds
  (not just the launch census), and document the release/capture asymmetry.
- **Test:** `ui_pool_census_matches_chain` (QA E2E).
- **B:** D3 resolved: B concedes **Low**, conditional on the UI showing live pool odds and unwrap value. Medium if the launch census is shown as odds.

### M-25 — Medium (High if we build our own curve) — Curve/AMM integration math and slippage
- **Status:** open. **Owner:** engineer + QA. **Sources:** T-CURVE-02/04/06.
- **Remediation:** every composite flow ("buy NFT with SOL", "sell NFT for SOL") carries on-chain `min_out`/`max_in`.
  `request_capture` takes `expected_ratio` and `expected_fee` and fails on mismatch. If we build our own curve: rounding
  favours the pool, there's no free round trip, and `checked_*` with `overflow-checks = true`.
- **Tests:** `composite_buy_nft_respects_max_in`; `curve_round_trip_never_profits` (fuzz; own curve only);
  `curve_boundaries_no_overflow`.

### M-26 — Medium — Pause scope
- **Status:** needs Barton (N4). **Owner:** Barton + engineer. **Source:** B-10(4).
- **Remediation:** either no pause at all (A recommends), or `pause_new_requests` only: multisig + timelock, auto-expires,
  and **can never block `release`, `reveal_and_settle`, `recommit` or `expire`**.
- **Test:** `release_and_settle_succeed_while_paused`.

### M-27 — Medium — Creator/team NFT allocations outside the VRF pool
- **Status:** needs Barton (Q-H6). **Owner:** Barton + engineer. **Source:** B-10(5).
- **Remediation (recommended):** forbid them on-chain. The graduation mint crank mints only into the vault PDA, and
  there's no instruction that delivers an NFT except `reveal_and_settle`.
- **Test:** `mint_crank_destination_is_vault_only`.

### M-28 — Medium — Discarded CPI result; raw account init (upstream)
- **Status:** moot-if-hybrid_vault. **Owner:** engineer. **Sources:** A-09 (`migrate_nft_v1.rs:91-92`), A-10
  (`init_escrow_v2.rs:25-53`, `init_recipe.rs:53-79`).
- **Requirement:** every CPI result is propagated with `?`. Anchor `init` (not `init_if_needed`) for all program state.
- **Test:** `attack_reinitialize_vault_fails` (`AccountAlreadyInUse`).

## Findings: Low

### M-29 — Low — Fee rounding and arithmetic
- **Status:** open. **Owner:** engineer + QA. **Source:** F-11.
- **Remediation:** `fee = ceil(u128(ratio_base) × 200 / 10_000)`, `require!(fee > 0, FeeZero)`, `u64::try_from`,
  `checked_add(ratio_base, fee)` for the capture debit. Compute once at launch and store it.
- **Test:** `fee_nonzero_and_ceiled_for_all_ratios_and_decimals` (plus a unit test that a hypothetical `ratio_base = 49`
  gives fee 1, not 0).

### M-30 — Low — SOL minimum below rent-exempt can DoS re-rolls
- **Status:** open. **Owner:** engineer. **Source:** F-12.
- **Remediation:** `SOL_MIN_LAMPORTS ≥ 890,880`, or a pre-funded / program-owned receiver.
- **Test:** `sol_min_transfer_succeeds_to_empty_receiver`.

### M-31 — Low — MEV around converts
- **Status:** open. **Owner:** engineer. **Sources:** B-08, T-CURVE-06. **Remediation / test:** as M-25 (slippage
  bounds; `jitodontfront` on frontend swaps).

### M-32 — Low — Shipped TODOs; authority-semantics footgun (upstream)
- **Status:** moot-if-hybrid_vault. **Sources:** A-11 (`release_v2.rs:110`, `release.rs:99`, `update_recipe.rs:58`,
  `update_escrow.rs:56`, `migrate_tokens_v1.rs:74`), A-12. **Requirement:** no `UncheckedAccount` without a `/// CHECK`
  that's enforced in code. CI greps for "TODO"/"Need to add" in `programs/` and fails.

### M-33 — Low — Doc and code bugs: fee still described as burned/split/≤10%/refunded
- **Status:** open. **Owner:** engineer (code, docs) + QA (TEST_PLAN) + Creative Director (design copy). **Sources:** B-12,
  A-D08, cross-review §5. Full list in "Doc bugs" below.
- **Test:** `rg -n -i "burn" docs qa/TEST_PLAN.md design/directions programs tests`: every remaining hit is an LP/anti-snipe
  burn or a history note marked SUPERSEDED.

## Findings: Info

### M-34 — Info — Deployed MPL-Hybrid ≠ pinned commit; externally upgradeable
- moot-if-hybrid_vault. A-13: mainnet 630,752 B vs local build 521,696 B; upgrade authority `mp14o4AQ…`.
  **Requirement:** no Mintmark value is ever held by MPL-Hybrid.

### M-35 — Info — VRF integration hygiene
- open, engineer. Pin the VRF program IDs. Parse with the provider SDK or an audited parser (both SDKs currently fail
  to build with Anchor 1.2, DECISIONS Q1). `get_value()` only works in the reveal slot (`randomness.rs:51-57`), so
  reveal and settle go in one instruction. Benchmark settle CU (Core mint + Merkle proof) at max N.
- **B:** One instruction isn't required, and it's riskier (M-04 H1). Pin `seed_slot`, record the value at reveal, and read the stored value at settle.

---
## Disagreements between A and B (left open for B)

B: updated 2026-09-25 ~16:45 MT. D1 and D3 to D6 resolved; D2 open on the label only. Evidence: `security/auditor-b/cross-review.md` §7.

| # | Topic | Auditor A | Auditor B | Used in this merge |
|---|---|---|---|---|
| D1 | Severity of upstream cherry-pick / predictable reroll (M-02) | High under A's key; **concedes Critical if MPL-Hybrid is in the swap path** | Critical (conditional) | **RESOLVED: Critical, moot-if-hybrid_vault.** B: confirmed; B13 PoC shows the full escrow drain (1,000,000 → 0) |
| D2 | Curve launch sniping (M-03) | **Critical** until a venue with a program-enforced activation point and anti-snipe schedule is chosen and tested | B: **High, as a blocking launch gate.** No protocol drain; converting closed on the curve; no curve code; audited venue by default | **OPEN on the label only.** Both agree it blocks launch until the venue's activation point and anti-snipe rules are tested |
| D3 | Wrap/unwrap arbitrage vs stale listings (M-24) | **Low**: market behaviour against third-party listings; no protocol loss, no invariant broken | B: concedes Low (Medium only if the UI shows the launch census as odds) | **RESOLVED: Low** |
| D4 | VRF failure handling (M-04) | Fee never refunded; PDA-authority permissionless reveal; **bounded re-request by anyone (new oracle, same candidate set) before a principal-only expire** | B: accepts A's rule, with amendments: reveal separate from settle (records the value); program-chosen oracle; deadline ≈ 9,000 slots; cap 3 then principal-only expire | **RESOLVED in principle (A's rule plus B's H1 to H4).** Devnet third-party reveal proof still required |
| D5 | Code in scope | `hybrid_launch` exists (c02be68) and contradicts the fee decision (M-07) | B: **concedes** (the claim came from an empty GitHub repo; local `git log`: c02be68 2026-09-24 15:03 MT, main 658fb95). Also notes `hybrid_vault` exists too (wip/hybrid-vault 977f8f2) | **RESOLVED: A's position**, plus B's correction to M-11 |
| D6 | SOL-minimum scaling (M-19) | Flat constant is enough: expected cost already scales 1/p. The risks are bypass (M-06) and recycling (M-16) | B: **concedes** (sim: 1/1 break-even ≥ 4.5× floor even at R5M, 32× at R1M, with s = 0.005); `k/p_rarest` points the wrong way | **RESOLVED: flat constant** ≥ 0.005 SOL, on capture and re-roll, never to the fee owner |

## Needs Barton

1. **Fee destination change (M-17):** (A) never, or (B) multisig + 7-day in-program timelock on `fee_owner` only. A
   recommends (B) for recoverability after a key compromise.
2. **Fee owner custody (M-17):** confirm it's a Squads multisig vault (A: ≥ 2-of-3 hardware keys), not a hot wallet. Name
   the signers.
3. **SOL minimum value (M-19/M-06/M-30):** A proposes 0.005 SOL, charged on **both** capture and re-roll.
4. **Where the SOL minimum goes (M-16):** VRF cost + sink (A's recommendation), not the fee owner.
5. **Insider policy (M-16):** the fee address and affiliates never capture or re-roll. Publish it.
6. **Sell policy and disclosure for fee tokens (M-18).**
7. **Copy for "never raised" until the freeze, and when to go `--final` (M-08, N5).**
8. **VRF provider (M-04, Q1):** Switchboard (needs the PDA-authority design and a devnet third-party-reveal proof) or ORAO.
9. **Pause at all? (M-26, N4).** A recommends none.
10. **Creator allocations outside the pool (M-27, Q-H6).** A recommends forbidding them.
11. **Ratio set / max collection size (BRIEF.md:67)**, because it drives the graduation mint carve-out (M-10/M-22).
12. **Multisig signers, threshold and timelock for upgrades (M-09, Q8).**

## Doc bugs (burn/split/fee language) — file:line

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
- `qa/TEST_PLAN.md:8, 18, 38, 64, 76-79` (supply after burns), `107` (INV-02 "only by burns"), `108` (INV-03 "fee handled per its policy"), `115` (INV-10 "Re-roll fee burned"), `116` (INV-11 `fee_destination = BURN`), `152` (SUP-08), `168` (CFG-08), `169` (CFG-09 cap 1000, `fee_destination` 1…255), `217` (RR-02 "Fee burned"), `223` (RR-08 refund on expire), `226-227` (RR-11/12 burned), `356` (E2E-C03 "burned"), `406-407, 413` (Q2/Q3/Q9).
- `qa/archive/TEST_PLAN.v0.3.md`: archived, so no action beyond the "archived" banner.

**design/ (user-facing copy)**
- `design/directions/README.md:35` ("Re-roll fees are paid in the collection's token and burned"); `design/directions/a-obsidian/NOTE.md:14, 16`.
- `design/directions/a-obsidian/token.html:217, 242` ("Re-roll fee: burned"), `265` ("Burned by re-rolls").
- `design/directions/a-obsidian/launch.html:244` ("Where the fee goes: Burned"), `252`/`324` (`"… ORBIT, burned (example)"`). The launch page also offers creator-chosen fee % (NOTE.md:16 "1% / 2% / 5%"); now fixed at 2%.
- The supply copy "No one can mint more; re-roll burns can only reduce it" (ARCHITECTURE.md:39, THREAT_MODEL.md:96, DECISIONS.md:196-197, NOTE.md:14, BRIEF.md:52) must become "Fixed at 1,000,000,000. No one can mint more." Nothing is burned now.

**security/ (our own reports; errata, don't rewrite history)**
- `security/auditor-a/round1.md:11, 18, 61 (B-08), 63-65, 237-249, 318`: burn-based. Superseded by this round's F-series.
- `security/auditor-b/round1.md:38, 177, 288, 309-323 (R1-B-12)`: burn-based. B's to annotate.

## `hybrid_vault` requirements checklist (all must be true, each with a passing test)

**Economics and config**
- [ ] Reads ratio, mint, N, fee amounts, fee token account and SOL receiver from the immutable `LaunchConfig` v2. No local copies. (M-01, M-07)
- [ ] No instruction modifies ratio, mint, N, trait root, vault, fee rate, fee amounts or fee token account. No config/vault close. (M-01)
- [ ] Fee = constant 200 bps on capture and re-roll, `ceil`, u128, `> 0`, checked; `cfg.fee_bps ≤ 200` asserted at use. (M-07, M-08, M-29)
- [ ] `FEE_OWNER` is a program constant, or `FeeConfig.fee_owner` changeable only via multisig + ≥ 7-day in-program timelock (Barton's choice). Never a creator parameter or an instruction argument. (M-05, M-17)
- [ ] Fee account validated by address == `ATA(fee_owner, mint)` and == `cfg.fee_token_account`, mint == cfg.mint, owner == fee_owner, token program == SPL Token. SOL receiver validated by address. (M-05)
- [ ] Fee ATA created at launch. Idempotent ATA creation inside `request_*`. (M-15)
- [ ] SOL minimum charged on capture **and** re-roll (capture ≥ re-roll), ≥ 890,880 lamports, never to the fee owner. (M-06, M-16, M-19, M-30)
- [ ] Capture: user −(N+fee), vault +N, fee ATA +fee. Re-roll: user −fee, vault unchanged. Release: exactly N back, zero fee. Supply constant at 1B. (M-14)
- [ ] One vault PDA per collection. Nothing burns from the vault. `vault ≥ R × NFTs_outside` fuzzed. (M-12, M-14)

**Selection and VRF**
- [ ] `request_*` takes no asset/index argument. Selection only in `reveal_and_settle`. Rejection sampling, without replacement, candidate set pinned by `seq`. Hand-in excluded. (M-02, M-23)
- [ ] Randomness account created and committed by CPI with PDA authority. Freshness + not-revealed checked. Pubkey/slot/oracle pinned. (M-04)
- [ ] Fee charged at request, directly to the fee destination, **never refunded**. No request-PDA fee escrow. (M-04)
- [ ] `reveal_and_settle` permissionless. Outcome independent of caller. Value read in the reveal slot. No cancel after commit. (M-04, M-35)
- [ ] `recommit` by anyone after the deadline (new oracle, same candidate set, ≤ 3). Then `expire` returns principal only, batch-capable. (M-04, M-20)
- [ ] No SlotHashes/Clock/counter in any value-bearing selection. (M-02)

**Lifecycle and authority**
- [ ] Converting opens only after graduation **and** full mint of N into the vault with all leaves bound. Mint crank permissionless, resumable, idempotent, funded from a program-held carve-out. (M-22, M-10)
- [ ] Mint crank mints only to the vault. No out-of-pool allocations. (M-27)
- [ ] Content-addressed image + `image_sha256` in committed JSON. (M-21)
- [ ] No pause, or pause that can never block release/settle/recommit/expire. (M-26)
- [ ] No vault outflow except `release` (exactly N to the NFT owner) and `reveal_and_settle` (NFT to the requester). (M-09)
- [ ] Upgrade authority = Squads ≥ 3-of-5, ≥ 7-day timelock, `configAuthority = null`, then `--final` after audit. (M-08, M-09)
- [ ] Every CPI result propagated. Anchor `init` only. No unenforced `/// CHECK`. No TODOs in shipped code. (M-28, M-32)
- [ ] PoCs 10, 11 and 12 ported as `regress_poc10_escrow_drain_must_fail`, `regress_poc11_cherrypick_must_fail`, `regress_poc12_token_swap_must_fail` and green in CI. (M-01, M-02)

## Launch readiness

**Mintmark is NOT launch-ready.** The converter (`hybrid_vault`) doesn't exist. The only on-chain code we have
(`hybrid_launch`) still encodes the superseded burn / 0–10% fee model. The VRF design as documented lets requesters
abort bad draws for a refund. The new fee design has three exploitable gaps: fee-account substitution, the
release+capture SOL-minimum bypass, and the insider discount. The curve's anti-sniping, graduation trigger and LP
custody are undesigned. Fix the Critical and High items and pass every checklist test on localnet/devnet before any
public devnet launch.

**A professional third-party audit of `hybrid_vault`, `hybrid_launch`, the graduation/migration and mint-crank flow,
the VRF integration, the chosen curve venue integration, and the authority/deployment configuration is REQUIRED before
mainnet. This internal A+B review is not a substitute for it.**
