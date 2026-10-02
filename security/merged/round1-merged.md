# Mintmark: merged security findings, round 1 — v2.1 (Auditor A + Auditor B)

## Changelog
- **v2.1, 2026-09-25 ~17:00–17:10 MT.** Folded in Auditor B's flat-fee notes (`auditor-b/flat-fee-stress-test.md`, `sim/flat_fee.py`,
  `flat_fee_output.txt`) and BRIEF 4:55 PM (Barton: Meteora DBC, 10k dropped, collections ≤ 10,000).
  - Every code claim was re-checked against the WT: HEAD `977f8f2` plus 53 uncommitted paths, read 16:58–17:03 MT. Key mtimes:
    - `unwrap.rs` / `constants.rs` 16:57:55;
    - `request.rs` / `randomness_ix.rs` 16:58:13;
    - `randomness.rs` 16:58:22;
    - `config.rs` 16:53:11;
    - `invariants.rs` 16:49:39;
    - `hybrid_launch` `needs_barton.rs` 16:56:23, `validation.rs` 16:56:54.
  - **New M-41 (High; B FF-14 rated Medium):** fee/config validation on the release path freezes release after a tier, wallet or
    version upgrade.
  - **M-16 → High** (B FF-03 numbers). Still tabled by Barton; mitigations listed.
  - **M-08** absorbs FF-07: charge `min(stored, tier, MAX)` on requests; release never reverts on a mismatch.
  - **M-14/M-36:** release fee = 0 is now the joint recommendation (needs Barton).
  - **M-37** absorbs FF-13; the VRF price source is cited.
  - **M-19** absorbs FF-04 and FF-05.
  - **M-33 → Medium** (FF-11: ADR-013/015 missing; doc list refreshed).
  - **Closed in WT (tests still required):** M-06, M-07 (code), M-15, M-29, M-30, M-40, M-26 (guardian removed).
  - **M-04:** the program-chosen oracle, reveal-once, cap 3, 9,000-slot deadline and principal-only expire are verified implemented;
    heartbeat filter, batch expire and the devnet proof stay open.
  - **Needs Barton** updated.
- **v2, merged with Auditor B review and the 4:49 PM flat-SOL fee change, 2026-09-25** (Auditor A worker, ~16:45–17:10 MT).
  - Folded in Auditor B's review: inline "B:" notes and D-table, `auditor-b/cross-review.md` §6/§7, the fee stress test and B13 PoC.
    The raw "B:" lines are gone. Attribution now sits in each finding as "(B)". The v1 file with B's inline notes is kept at
    `round1-merged.v1-with-B-notes.md`.
  - **Fee model replaced**, in two steps:
    - BRIEF 4:49 PM MT: no token fee. One flat SOL fee on capture, release and re-roll, paid to one fixed fee wallet.
    - 4:52 PM MT refinement: the fee is tiered by ratio (10k/50k = 0.002 SOL, 100k/200k = 0.005, 500k–5M = 0.01). It's fixed per
      launch in `LaunchConfig`, with a 0.01 SOL hard cap in code.
    - Token-fee-only findings are marked "superseded by fee change". Requirements that still apply are kept.
  - **The engineer's working tree was reviewed** (read-only).
    - Repo `/workspace/launchpad`, branch `wip/hybrid-vault` @ `977f8f2`, plus ~50 uncommitted paths still being edited during this
      review (latest mtime ~16:53 MT).
    - `hybrid_vault` exists. M-11 is retitled.
    - New findings: M-36 release-fee trap, M-37 VRF cost funding, M-38 test-only mocks and ship guards, M-39 audit-baseline drift,
      M-40 solvency invariant omits pending re-rolls.
  - **D1–D6 resolved.** D2 settled as "High, launch-blocking gate", with A's Critical rationale noted.
  - **New evidence:**
    - B13 full drain (B).
    - New on-chain PoC 13: predict-and-abort against the upstream reroll (A-04, executed).
    - PoC 01 T2 and PoC 02 now compute their results.
    - Stub `12_reroll_peek.js` deleted (errata in `auditor-a/round1.md`).
    - PoC 04 models the flat and tiered SOL fee.
  - **Venue research:** Meteora DBC and Raydium LaunchLab **cannot** take a pre-existing mint (M-03/M-10). See the source lines cited there.
  - **Status changes (Barton):** M-16 tabled by Barton (2026-09-25). M-06 and M-08: product copy needs Barton (with Creative Director).
- v1, 2026-09-25 (A merge of A round 1 + B round 1).

## Scope, sources, conventions
- **Sources:**
  - `security/auditor-a/round1.md` (A-01..A-13, A-D01..A-D08) and `security/auditor-a/cross-review.md` (F-01..F-13, VRF §3).
  - `security/auditor-b/round1.md` (B-01..B-14) and `security/auditor-b/cross-review.md` (§6 per-ID, §7 D1–D6, H1–H6, N-01..N-08, M-B1..M-B6).
  - `security/auditor-b/fee-design-stress-test.md` and `sim/fee_stress.py`.
  - BRIEF Decisions through **2026-09-25 4:52 PM MT**.
- **Code baselines:**
  - `main@658fb95` (committed `hybrid_launch`).
  - `wip/hybrid-vault@977f8f2` (committed WIP `hybrid_vault`).
  - **WT** = the uncommitted working tree as read between 16:31 and 16:55 MT. It changed while we read it, so WT line references
    are indicative only. See M-39.
- **Settled by Barton (not reopened):**
  1. One graduation event. Converting is closed during the curve. At graduation, liquidity moves to a DEX, the full collection is
     minted into escrow from graduation proceeds, and then converting opens.
  2. **Fee (4:49 PM + 4:52 PM):** no token fee. A flat SOL fee on capture, release and re-roll, tiered by ratio and fixed per
     launch in `LaunchConfig`. Hard cap 0.01 SOL in code. Paid to Barton's fee wallet. Release returns exactly N tokens.
     "Can't be raised" copy is allowed only with the beta caveat (3-of-5 multisig + 7-day delay until the post-audit freeze).
  3. Curve: an existing audited venue (engineer recommends Meteora DBC; still pending Barton).
- **Evidence base:**
  - Executed on-chain PoCs 10/11/12/13 against a local build of upstream MPL-Hybrid @ `aacf1a53`.
  - B's independent re-run of 10/11/12, and B13.
  - Models: PoC 01/02/03/04 and B's sims.
  - Read-only Switchboard IDL/SDK checks.
  - Local only. Nothing deployed; no real keys or funds.
- **Status values:**
  - `open`
  - `open (partly fixed in WT)`
  - `moot-if-hybrid_vault` (upstream only; kept as a requirement)
  - `needs Barton`
  - `tabled by Barton`
  - `resolved-by-decision`
  - `superseded by fee change`
  - A regression test is always still required.
- **Tests:** attack tests that must **fail to exploit**, each asserting a specific error. Upstream PoCs are ported as `regress_poc1x_*`
  (LiteSVM, `tests/track-a-hybrid/`). Error names are proposals.

## Summary table (Critical first)

| ID | Sev | Title | Status | Owner |
|---|---|---|---|---|
| M-01 | Critical | Admin can change amount / mint / fee destination instantly → escrow drain (upstream) | moot-if-hybrid_vault | engineer + QA |
| M-02 | Critical | Capturer picks the NFT; predictable, abortable, biased reroll (upstream) | moot-if-hybrid_vault | engineer + QA |
| M-03 | High, launch-blocking gate (A: Critical) | Launch sniping; DBC (chosen 4:55 PM) creates its own mint, so launch design must change | open (blocks launch) | engineer |
| M-04 | High | VRF selective reveal / oracle griefing / recommit liveness | open: core rules implemented in WT; heartbeat filter, batch expire, devnet proof remain | engineer + QA |
| M-05 | High | Fee recipient must be the LaunchConfig wallet only | fixed in WT; tests + recipient launch check | engineer + QA |
| M-06 | High | Release + capture cheaper than re-roll | closed in WT (single tier fee); copy needs Barton (with Creative Director) | engineer + QA |
| M-07 | High | `hybrid_launch` fee model contradicts the decision | closed in WT code (commit + tests); docs → M-33 | engineer + QA |
| M-08 | High (disclosure) | "No setting can raise the fee" vs upgradeable; charge min(stored, tier) (FF-07) | needs Barton (with Creative Director); code rule open | engineer + Barton |
| M-09 | High | One upgrade key over every collection's vault | open | engineer + Barton |
| M-10 | High | Graduation/migration on DBC; buffer lock; funding refusal | open | engineer |
| M-11 | High | `hybrid_vault` in progress, uncommitted, unaudited | open | engineer |
| M-12 | High | Shared escrow / burn paths (upstream) | moot-if-hybrid_vault | engineer + QA |
| M-13 | High | Mint/freeze/supply on-chain (reopens under DBC-created mint) | resolved-by-decision for current code | QA |
| M-16 | High (B FF-03; was Medium) | Insider discount: fee wallet converts at tx + VRF (21% of public cost at 0.01) | tabled by Barton | Barton + engineer |
| M-36 | High | Release fee must never trap holders | open; release fee = 0 recommended (needs Barton) | engineer + Barton |
| M-41 | High (A) / Medium (B) | Fee/config validation on the release path: upgrade freezes release in every collection (FF-14) | open | engineer + QA |
| M-38 | High (ship blocker) | Test-only mocks/features must never ship; real Switchboard reveal unproven | open | engineer + QA |
| M-14 | Medium | Fee never backing; exact deltas; release fee value | open; release fee = 0 recommended (needs Barton) | engineer + QA + Barton |
| M-15 | Medium | Fee recipient state must not DoS converts | closed in WT | engineer + QA |
| M-17 | Medium | Fee wallet custody; changeability | needs Barton | Barton + engineer |
| M-19 | Medium | Mid tiers and 2.5M/5M are market-priced lotteries; pool depletion (FF-04/05) | needs Barton (disclosure; keep 5M?) | Barton + engineer (UI) |
| M-20 | Medium | Escrow cornering / head-of-line DoS | open (partly fixed in WT) | engineer + QA |
| M-21 | Medium | Metadata commitment excludes image bytes | open (partly fixed in WT) | engineer + QA |
| M-22 | Medium | Graduation mint crank stall and funding; vault-side size cap | open | engineer + QA |
| M-23 | Medium | FIFO/seq merge and leaf-binding correctness | open | engineer + QA |
| M-25 | Medium | Curve/AMM integration math and slippage | open | engineer + QA |
| M-26 | Medium | Pause scope | resolved in WT (guardian removed, ADR-015 cited); ADR not written | Barton (confirm) + engineer |
| M-27 | Medium | Creator/team NFT allocations outside the pool | needs Barton | Barton + engineer |
| M-28 | Medium (B: Low) | Discarded CPI result; raw init (upstream) | moot-if-hybrid_vault | engineer |
| M-33 | Medium (was Low; FF-11) | Docs still describe 2% token fee, burn, refunds; ADR-013/015 missing | open | engineer + QA + CD |
| M-37 | Medium | VRF cost funding; the 0.002 tier doesn't cover VRF + crank (FF-13) | open | engineer + Barton |
| M-18 | Low | Fee accrual sell pressure (SOL now) | superseded (disclosure kept) | Barton |
| M-24 | Low | Stale-listing arbitrage / pool adverse selection | resolved (Low; Medium if UI shows launch census as odds) | engineer (UI) |
| M-29 | Low | Fee arithmetic | closed/superseded (compile-time bounds) | engineer + QA |
| M-30 | Low | SOL fee below rent-exempt | closed in WT | engineer |
| M-31 | Low | MEV around converts | open | engineer |
| M-32 | Low | Shipped TODOs (5); authority footgun (upstream) | moot-if-hybrid_vault | engineer |
| M-40 | Low | Solvency invariant omitted pending re-rolls | closed in WT | engineer + QA |
| M-34 | Info | Deployed MPL-Hybrid ≠ pinned commit | moot-if-hybrid_vault | engineer |
| M-35 | Info | VRF integration hygiene | open | engineer |
| M-39 | Info | Audit-baseline drift | open | engineer |

---

## Findings: Critical

### M-01 — Critical — Admin can change amount / mint / fee destination instantly → escrow drain
- **Status:** moot-if-hybrid_vault (the requirement stands). **Owner:** engineer + QA. **Sources:** A-01, A-02, A-08, B-10(7), B N-07.
- **Evidence:**
  - Upstream `update_recipe.rs:54-138` is single-signer with no timelock.
    - `amount` is written at `:114-116`.
    - `recipe.token` and `recipe.fee_location` are overwritten **unconditionally** at `:91-93`.
  - **Executed:**
    - PoC 10 drained 3,000,000 → 0.
    - PoC 12 swapped the mint on an update with every Option set to `None`.
    - **B13 (B, `auditor-b/poc-crosscheck/b13_token_swap_full_drain.js/.out`):** full drain of the real escrow 1,000,000 → 0 for junk
      tokens, **plus a third party's capture fee rerouted** to the attacker's fee ATA (+20,000; the disclosed fee address got +0).
    - B re-ran 10/11/12 PASS on an independent validator.
- **Remediation (hybrid_vault rule):**
  - No instruction modifies ratio, mint, collection size, trait root, vault account, fee amount or fee recipient after init.
  - No close instruction for the config or vault.
  - All values are read from the immutable `hybrid_launch::LaunchConfig` (no copies).
  - No instruction takes a mint, fee destination or amount as an argument.
  - WT status: `config::econ` re-reads `LaunchConfig` on every use and fails closed. No mutation instruction was found.
- **Tests:**
  - `regress_poc10_escrow_drain_must_fail`.
  - `regress_poc12_token_swap_must_fail`.
  - `regress_b13_token_swap_full_drain_must_fail` (B): escrow unchanged and no fee reaches any account but `LaunchConfig.fee_recipient`.
  - `config_has_no_mutation_or_close_instruction_in_idl`.

### M-02 — Critical — Capturer picks the NFT; predictable, abortable, biased reroll
- **Status:** moot-if-hybrid_vault (requirement stands). **Owner:** engineer + QA. **Sources:** A-03, A-04, A-05, A-12, B-01, B-14.
  D1 resolved.
- **Evidence:**
  - Upstream code:
    - `capture_v2.rs:52-54`: caller-supplied asset.
    - `:183-185`: the signer check runs only if authority == `recipe.authority`.
    - `:187-203`: reroll seed from SlotHashes/Clock/count.
    - `checked_rem(max-min)+min` never selects `max` and draws with replacement.
  - **Executed on-chain:**
    - PoC 11: a bot cherry-picked a named rare.
    - **PoC 13 (new, 2026-09-25):** a non-authority bot's program CPIs `capture_v2` in reroll mode and reads the new URI in the same tx.
      It reverts unless the index is < 10 of 100. Result: 18 attempts, 15 reverted on-chain with tokens and NFT unchanged,
      3 committed, **all rare** (vs 10% honest odds). Cost of aborts: tx fees only.
  - **Models:** PoC 01 11/11 (T2 now computed).
  - **B's sim:** rares are 565–5,652× cheaper than honest VRF.
- **Remediation (hybrid_vault rule):**
  - `request_*` takes no asset or index argument.
  - Selection happens only at settle, from the recorded VRF value: `uniform_below` with rejection sampling, without replacement, over
    the pool pinned at request time.
  - The re-roll hand-in is excluded from its own draw.
  - No SlotHashes, Clock or counters in selection.
  - WT: `settle.rs` picks with `uniform_below` + `swap_remove`, and the hand-in is pushed after the pick.
- **Tests:**
  - `regress_poc11_cherrypick_must_fail`.
  - `regress_poc13_predict_and_abort_must_fail`: port PoC 13's guard as a LiteSVM program wrapping `request_*`. Expected: nothing
    to predict at request, so success = base rate ± 3σ.
  - `every_pool_index_is_reachable_and_uniform`.

---

## Findings: High

### M-03 — High, launch-blocking gate (A's Critical rationale noted) — Launch-window sniping/bundles; the venue must accept our mint
- **Status:** open, and **blocks launch** until a venue with a program-enforced activation point and anti-snipe schedule is chosen,
  integrated and tested. **Owner:** engineer. **Sources:** A-D05, T-CURVE-03/05, B-03. **D2 resolved.**
- **Severity rationale:**
  - B's facts verified:
    - No curve or distribution code exists anywhere.
    - `hybrid_launch` on `main@658fb95` has a single `launch` instruction. It mints 1B into `ATA(launch_vault PDA)` with no signing
      path (`launch.rs:84-90, 105-112, 158-166`; `constants.rs:18` "Distribution (bonding curve) is TBD").
    - Converting is closed on the curve.
    - The default is an audited venue.
  - So there is no protocol drain today.
  - A's Critical rationale, recorded: the Stonk.fun and pump.fun harm is buyer-side value extraction at launch; the venue isn't chosen;
    and a mis-integration could route the 1B.
  - Both auditors agree the practical effect is the same: **no launch until closed**.
- **Venue research (new, answers B's open question):**
  - **Neither Meteora Dynamic Bonding Curve nor Raydium LaunchLab accepts a pre-existing mint. Each creates the base mint inside its
    own pool-initialize instruction.**
    - **Meteora DBC** (source `github.com/MeteoraAg/dynamic-bonding-curve` @ `f552f20`, 2026-09-09).
      `ix_initialize_virtual_pool_with_spl_token.rs:62-70` declares
      `base_mint: #[account(init, signer, payer = payer, mint::decimals = config.token_decimal, mint::authority = pool_authority)]`.
      Lines 217-240 `mint_to` the initial supply and then `set_authority(MintTokens)`. The Token-2022 and transfer-hook variants are
      `init, signer` too. Meteora's docs say: "Creating a DBC token pool will automatically mint the token within the same
      `initialize_virtual_pool` instruction" (docs.meteora.ag/invent/launch-pools/dbc-token-launch-pool). The SDK example passes a
      fresh `Keypair.generate()` as `baseMint` (docs.meteora.ag/developer-guides/dbc/typescript-sdk/examples).
    - **Raydium LaunchLab** (program source not public: docs.raydium.io/reference/program-addresses). The official CPI crate
      `github.com/raydium-io/raydium-cpi` @ `115df27`, `programs/launch-cpi/src/context.rs:69-78` (`Initialize`), `:204-213`
      (`InitializeV2`) and `:339-342` (`InitializeWithToken2022`) declares `base_mint` as `init` with `mint::authority = authority`
      ("Created in this instruction"). The docs' `Initialize` table says `base_mint`: "Fresh Keypair (or PDA) — this instruction
      initializes it" (docs.raydium.io/products/launchlab/instructions). The mint authority is held by LaunchLab until graduation and
      then revoked (docs.raydium.io/products/launchlab/overview).
  - **Consequence:** the current `hybrid_launch` design, which creates the mint and puts 1B in `launch_vault`, **cannot feed either
    venue**. The engineer must pick one:
    - (a) `hybrid_launch` CPIs the venue's initialize with a PDA-signed base mint, so the venue creates it and our program records it.
      DBC's `init, signer` and LaunchLab's "or PDA" allow `invoke_signed`. The venue then mints the supply and holds mint authority
      until migration, so M-13's "we mint exactly 1B, authorities None" must be re-derived from the venue config (fixed supply, mint
      authority revoked at migration, freeze None).
    - (b) The venue launches the token and `hybrid_launch` becomes a registration step that validates the venue-created mint.
    - (c) A different venue that takes an existing mint (none verified).
  - Either way, the NFT backing tokens (N × R) must come from somewhere the venue allows, such as DBC's fixed-supply/leftover or
    LaunchLab's vesting/locked allocation. That must be designed and tested. Ties to M-10 and M-13.
- **Remediation:**
  - Choose a venue whose program enforces an activation point plus a decaying anti-snipe fee or opening-window caps.
  - The creator buy goes through the same rules, capped (≤ 2%) and disclosed.
  - Pool creation happens in the same program flow as launch config.
  - Publish the rules.
  - Any anti-snipe proceeds go to LP or are burned.
- **Tests:**
  - `attack_bundle_create_and_buy_before_activation_fails`.
  - `attack_opening_window_cap_exceeded_fails`.
  - `anti_snipe_fee_decays_to_base_and_proceeds_go_to_lp_or_burn`.
  - `venue_mint_integration_matches_launch_config` (B, restated): the mint `hybrid_vault` binds to is exactly the venue-created
    mint; supply, mint authority and freeze authority are as disclosed; the N × R backing allocation is reachable only by the vault.
- **v2.1 update (BRIEF 4:55 PM MT, Barton approved):**
  - Bonding curve = **Meteora DBC**.
  - DBC's 25% unsold buffer is locked in a program-owned account.
  - 10k ratio dropped. Collections capped at 10,000.
  - Launches that can't fund the full mint (with a 25% margin) are refused.
  - Because DBC creates the base mint itself (`init, signer`, cited above), option (a) or (b) above is now **mandatory design work**:
    - The WT `hybrid_launch` still creates its own mint and the 1B supply.
    - `graduation.rs` still fails closed pending a DBC verifier.
  - Also add a test that the locked 25% buffer account has no withdraw path.

### M-04 — High — VRF selective reveal, oracle griefing and recommit liveness
- **Status:** open, with most parts implemented in the WT (verified 16:58–17:02 MT; `randomness.rs` mtime 16:58:22, `randomness_ix.rs`/
  `request.rs` 16:58:13, `expire.rs` 16:52:31). **Owner:** engineer + QA. **Sources:** B-02, A-D02, F-13, B §7 H1–H6, B FF-09.
  **D4 resolved.**
- **Implemented in the WT (code-verified; each still needs its regression test):**
  - PDA randomness authority, via CPI init/commit/reveal.
  - Fee at request, never refunded.
  - **Program-chosen oracle (H2):**
    - `randomness::commit_for_request` calls `select_oracle(queue, vault, seq, used)` and requires the passed oracle == the chosen one
      (`WrongOracle`, `randomness.rs:108-109`).
    - Selection is `(queue.curr_idx + H(vault, seq) + used.len()) mod len`, skipping default and already-used oracles
      (`randomness.rs:198-218`).
    - Used on commit and on recommit (`randomness_ix.rs:196-224` also rejects `OracleReused`).
  - **Reveal-once (H1):**
    - Separate permissionless `reveal_randomness` sets `Request.revealed` and `Request.value`, recorded once (`state.rs:78-80`).
    - Settle reads `req.value` (`settle.rs:80`).
    - Recommit is refused once revealed.
  - **Cap 3 (H4):** `MAX_RECOMMITS = 3` (`constants.rs:57`); `randomness_ix.rs:195` `RecommitsExhausted`.
  - **Deadline (H3):** `REVEAL_TIMEOUT_SLOTS = 9_000` (`constants.rs:38`).
  - **Principal-only expire (H4):**
    - Head only.
    - Requires `commits > MAX_RECOMMITS` and `deadline + EXPIRE_GRACE_SLOTS (216,000)` (`expire.rs:78-79`).
    - Returns N tokens or the handed-in NFT. No fee refund.
  - No pause of any kind can block reveal, recommit, settle or expire (guardian removed, M-26).
- **Still open:**
  - (a) **Oracle liveness residual:** `select_oracle` skips only empty and used slots. It doesn't filter on heartbeat/staleness, so a
    dead oracle in the queue list can still be chosen, costing one deadline (~1 h) per dead pick. Add a freshness filter, or accept
    and disclose.
  - (b) **Batch expire:** K stale heads per call isn't implemented. Expire clears one head per call.
  - (c) **Devnet proof** with the real Switchboard program (third-party reveal with a PDA authority, M-38).
  - (d) H5: honest users lose one fee per outage-stuck request. Accept and disclose (B FF-09). Don't reintroduce a refund.
  - (e) Regression tests for everything above.
- **Tests:**
  - `attack_requester_withholds_reveal_gets_no_refund`.
  - `withheld_reveal_past_all_recommits_returns_principal_only` (B).
  - `attack_foreign_randomness_account_rejected`.
  - `attack_caller_supplied_oracle_rejected` (`WrongOracle`).
  - `recommit_with_reused_oracle_rejected`.
  - `attack_settle_failure_cannot_trigger_recommit_after_reveal`.
  - `recommit_capped_then_principal_only_expire`.
  - `expire_clears_k_stale_heads_in_one_call`.
  - `selective_expiry_cannot_shift_later_draw`.
  - `third_party_can_reveal_with_pda_authority` (devnet).

### M-05 — High — Fee recipient substitution: the fee goes only to the LaunchConfig wallet
- **Status:** open (fixed in WT; tests required). **Owner:** engineer + QA. **Sources:** F-06 (A-02 class), B N-07.
  **Fee change:** the token-account parts (ATA mint and owner reassignment) are superseded, because there is no token fee.
- **Requirement (still applies):**
  - The SOL fee recipient is `LaunchConfig.fee_recipient == PLATFORM_FEE_RECIPIENT` (program constant). It is never a creator launch
    parameter and never a caller-supplied arbitrary account.
  - The recipient should be a system-owned, non-executable wallet (e.g. a Squads vault PDA, which is system-owned).
  - The rate can't be raised.
- **WT status:**
  - `request.rs:52-53, 105-106` pin `address = launch_config.fee_recipient`.
  - `config.rs:27` asserts `fee_recipient == PLATFORM_FEE_RECIPIENT` (`FeeRecipientNotPlatform`).
  - `LaunchParams` has no recipient field.
  - The release fee goes to a per-vault `FeeVault` PDA, which only a permissionless `sweep_fees` can move, and only to
    `LaunchConfig.fee_recipient`.
- **Remaining:**
  - Add a launch-time or deploy-script check that `PLATFORM_FEE_RECIPIENT` is system-owned and not executable. Otherwise
    capture and re-roll revert for every collection (release is safe via `FeeVault`).
  - Optional: route capture/re-roll fees through `FeeVault` too, for uniform failure isolation.
- **Tests:**
  - `attack_fee_to_own_wallet_rejected` (`FeeRecipientMismatch`).
  - `attack_forged_launch_config_other_recipient_rejected` (`FeeRecipientNotPlatform`).
  - `sweep_fees_only_to_launch_config_recipient`.
  - `attack_creator_cannot_set_fee_recipient_at_launch` (IDL assertion).

### M-06 — High — Release + capture cheaper than a re-roll (bypass of the per-draw fee)
- **Status:** **closed in WT** (verified: `config.rs:22-37`, mtime 16:53:11; `econ()` returns one `fee_lamports` for capture, release and
  re-roll; `LaunchParams` has no fee field, `validation.rs:13`).
  - B's FF-06: at ~16:50 the WT briefly had creator-set SOL fees with capture allowed to be 0, which reopened M-06. The 16:53–16:54
    tier table fixed it.
  - Regression test required.
  - Product copy: **needs Barton (with Creative Director)**.
- **Owner:** engineer + QA. **Sources:** F-03, B N-03, B FF-06.
- **Rule to keep:** `capture_fee ≥ reroll_fee`. That holds even with release = 0 (M-14/M-36 joint recommendation).
- **Tests:** `unwrap_rewrap_never_cheaper_than_reroll` (every tier); `launch_params_have_no_fee_field`;
  `vault_rejects_config_with_fee_not_equal_tier` (requests only; see M-41 for release).

### M-07 — High — `hybrid_launch` fee model contradicts the decision
- **Status:** **closed in WT code (v2.1, verified: `validation.rs:13,45-59` no fee field, `checked_fee_for_ratio`; `needs_barton.rs` FEE_TIERS without 10k, mtime 16:56)**; must be committed and tested. The docs half is reopened as M-33 (B FF-11). **Owner:** engineer + QA. **Sources:** F-01, B-05.
- **Committed `main@658fb95`:** `MAX_TOKEN_FEE_BPS = 1_000`, creator-chosen bps, BURN-only (`constants.rs:44-50`, `validation.rs:31-37`,
  `state.rs:41-42`).
- **WT (≈16:53 MT):**
  - `LaunchConfig` v3.
  - The flat SOL fee comes from the `FEE_TIERS` const table by ratio (`needs_barton.rs`: 0.002 / 0.005 / 0.01).
  - `MIN_FEE_LAMPORTS = 2,000,000`, `MAX_FEE_LAMPORTS = 10,000,000`, with compile-time asserts.
  - Never creator-supplied.
  - The vault re-asserts it.
  - F-01's intermediate issues (0.001 floor; capture ≥ re-roll not enforced; "capture ≥ re-roll ≥ MIN" comment/code mismatch) were
    fixed by the rewrite.
- **Remaining:**
  - Commit.
  - Remove the dead v2 token-fee code, constants and tests.
  - Update `launch.rs`/`qa_launch.rs` tests.
- **Tests:**
  - `fee_is_tier_value_for_every_ratio`.
  - `attack_launch_params_cannot_carry_fee_fields`.
  - `attack_forged_config_fee_above_cap_rejected`.

### M-08 — High (disclosure) — "No setting can raise the fee" vs an upgradeable program (incl. B FF-07)
- **Status:** needs Barton (with Creative Director): copy and freeze timing. Code rule open. **Owner:** engineer + Barton. **Sources:**
  F-08, B N-06, **B FF-07 (High, disclosure)**.
- **Evidence (WT, `config.rs:24-26`, 16:53):**
  - `econ()` requires `fee == fee_for_ratio(ratio)` and `fee ≤ MAX_FEE_LAMPORTS`, where both are **current code constants**.
  - An upgrade that raises `FEE_TIERS`/`MAX_FEE_LAMPORTS` makes the code demand the new value, so old configs are rejected, or a
    migration rewrites them.
  - The stored per-launch `LaunchConfig.fee_lamports` is immutable but not authoritative.
  - The 0.01 "hard cap in code" can be changed by a 3-of-5 + 7-day upgrade until `--final`.
- **Remediation (must not conflict with M-41):**
  - For **requests**, charge `min(cfg.fee_lamports, current tier (if the ratio still has one), MAX_FEE_LAMPORTS)`. Don't require
    equality. An upgrade can then only lower what an existing collection pays. A forged config with fee > MAX is simply capped.
  - **Release never reverts on any fee/tier/version/recipient mismatch** (M-41).
  - Copy: "Fee set per collection at launch; no setting can raise it. Until the program is frozen after the final audit, a 3-of-5
    multisig upgrade with a public 7-day delay could change the program."
  - `--final` after audit (Barton sets the date).
  - Monitor buffer writes and queued upgrades.
- **Tests:**
  - `upgrade_cannot_raise_fee_for_existing_launch` (B: `econ` with a raised tier table charges the stored, lower fee).
  - `lowered_tier_charges_lower_fee_for_existing_launch`.
  - `forged_config_fee_above_cap_is_capped_or_rejected_for_requests`.
  - `upgrade_authority_is_multisig_with_7day_timelock`.

### M-09 — High (Critical if any single key) — One upgrade key over every collection's vault; operator powers
- **Status:** open. **Owner:** engineer + Barton. **Sources:** B-10, A-13.
- **Remediation:**
  - Upgrade authority is a Squads v4 vault: ≥ 3-of-5 independent signers, `configAuthority = null`, ≥ 7-day timelock, signers published.
  - `--final` after audit.
  - No vault outflow except `unwrap` (exactly N to the NFT owner), settle (NFT to the requester) and principal-only `expire`.
  - `sweep_fees` moves only `FeeVault` lamports to the fixed recipient.
  - No pause/guardian exists in the WT (M-26).
- **Tests:**
  - `launch_refused_unless_upgrade_authority_is_timelocked_multisig`.
  - `vault_has_no_outflow_except_unwrap_settle_expire`.
  - `no_key_can_block_release_reveal_recommit_settle_expire` (B).

### M-10 — High (Critical if any operator signer or wallet in the path) — Graduation/migration and the venue mint model
- **Status:** open. **Owner:** engineer. **Sources:** A-D06, T-CURVE-07/08, B-09, B N-02.
- **New:**
  - The venue creates the mint (M-03 research), so graduation, the backing allocation and "converting opens" must be built around the
    venue's migration (DBC migrates to DAMM v1/v2; LaunchLab graduates to AMM v4/CPMM).
  - WT: `open_vault` is permissionless and one-way. It requires `minted_count == collection_size` plus `graduation::verify`, which
    **fails closed by default** (`GraduationCheckUnavailable`) until a venue verifier exists.
  - `graduation.rs`, `mint_assets.rs` and `open_vault.rs` have **not been reviewed** by either auditor (B). They're the next audit target.
- **Remediation:**
  - Graduation is permissionless, deterministic and one-shot.
  - LP is burned or PDA-locked.
  - The graduation verifier checks the venue's on-chain migration state (program ID, pool, mint, status), not a marker account.
  - The NFT-mint carve-out is program-computed and held by a PDA that only the mint crank can spend.
    - WT gap: `mint_assets` has the caller pay rent (`mint_assets.rs:1-5`). The carve-out isn't implemented.
- **Tests:**
  - `attack_graduate_twice_fails`.
  - `flash_buy_to_threshold_then_migrate_price_continuity`.
  - `attack_precreated_pool_at_skewed_price_fails_closed`.
  - `lp_tokens_burned_or_pda_locked`.
  - `graduation_needs_no_privileged_signer`.
  - `open_vault_rejects_non_venue_graduation_account`.
- **v2.1 (BRIEF 4:55 PM):** DBC chosen; DBC 25% unsold buffer locked in a program-owned account (add `dbc_buffer_account_has_no_withdraw`); launches that cannot fund the full mint with 25% margin are refused at launch (WT `needs_barton.rs:92-107` `PER_NFT_MINT_COST_LAMPORTS = 5,090,000`, funding formula; verify it is enforced in `launch` and add `underfunded_graduation_refused_at_launch`).

### M-11 — High — `hybrid_vault` is in progress and unaudited; every safeguard depends on it
- **Status:** open. **Owner:** engineer. **Sources:** A-D01. B's correction accepted: v1's "doesn't exist" title was wrong.
- **State:**
  - `wip/hybrid-vault@977f8f2` (2026-09-24 15:18 MT; amended from `f4761af` during review, B).
  - Plus a large, actively changing uncommitted WT (ADR-012/013): PDA-authority randomness, separate reveal, FIFO settle from the
    recorded value, no fee refund, flat SOL fee, `FeeVault`, recommit cap, principal-only expire, `mint_assets`/`open_vault`/
    `graduation`, and (at v2) a guardian pause, since removed (ADR-015, v2.1).
  - Committed `977f8f2` itself is **not shippable**: fee refund on expire (M-04) and a mock reveal with no authority (M-38).
- **Remediation:**
  - Commit the WT.
  - Tag an audit baseline (M-39).
  - Build to the checklist below.
  - It becomes the top-priority target of the third-party audit.
  - No converting on public devnet until the checklist tests pass.
- **Test:** all M-01, M-02, M-04–M-06, M-12, M-14, M-15, M-36 tests green in CI (`--locked`), plus a Trident fuzz campaign of
  INV-1/INV-2 (M-40) over ≥ 10^6 op sequences.

### M-12 — High — Pooled/shared escrow and burn paths (upstream)
- **Status:** moot-if-hybrid_vault. **Owner:** engineer + QA. **Sources:** A-06, A-07 (PoC 02 5/5, now fully computed).
  B rates this Low (conditional) and doesn't contest keeping it as a requirement.
- **Requirement:**
  - One vault PDA per collection.
  - Nothing burns from the vault.
  - `vault ≥ R × owed` per collection.
- **Tests:** `cross_collection_release_cannot_draw_other_vault`; `fuzz_backing_invariant_per_collection`.

### M-13 — High — Mint/freeze authority, supply and launch destination enforced on-chain
- **Status:** resolved-by-decision for `hybrid_launch` as written (`main@658fb95 launch.rs:132-166`). **Reopens if the venue creates
  the mint (M-03):** the guarantees must then be re-proven against the venue config. **Owner:** QA.
- **Test:** existing suite, plus `venue_mint_supply_and_authorities_as_disclosed` once the venue is integrated.

### M-16 — High (B FF-03, insider; A concurs on the new numbers; was Medium) — Insider discount: the fee wallet's own converts cost it only tx + VRF
- **Status:** **tabled by Barton (2026-09-25)**. He wants the fee income from pre-launch onward. Recorded for his information. **Not a blocker.**
  **Owner:** Barton + engineer. **Sources:** F-04, B N-04/M-B6, **B FF-03**.
- **Numbers:** B's `sim/flat_fee.py` Parts E/G, VRF ≈ 0.002 SOL (third-party estimate, see M-37).
  - The fee wallet's true cost per draw is tx + VRF ≈ 0.0021 SOL vs the public's fee + tx.
  - That's **21%** of the public's cost in the 0.01 tier (4.8× cheaper) and **41%** in the 0.005 tier. No discount in the 0.002 tier.
  - 1/1 at C = 10k, F = 1 SOL: public m* **102×**, insider **22×**.
  - Epic at F = 0.25: public 5.0×, insider 1.8×.
  - The insider is also the only zero-friction arbitrageur on both edges of the NFT band (exclusive arbitrage edge).
  - A's PoC 04 Part C agrees in form: insider m* 2.7× vs 24.6× public at R=1M with an example 0.001 VRF cost.
  - The published "fee wallet never converts" policy can't be verified, because funding can come from off-chain or a CEX.
- **Mitigations that keep Barton's income (options, none blocking):**
  - (a) **Disclose** on every token page: "Mintmark earns every capture, release and re-roll fee. Wallets we control could re-roll at
    cost; our policy is that they never do." Publish the fee wallet and FeeVault addresses and a monitor of fee-wallet outflows to
    converting wallets.
  - (b) **Exclude the fee wallet on-chain:** `request_*` rejects `user == PLATFORM_FEE_RECIPIENT` (and any published affiliate list).
    It's cheap, but only stops the naive case, since off-chain-funded wallets bypass it.
  - (c) **Sink for re-roll fees only:** capture (and release) fees stay with Barton; re-roll fees, or a share of them, go to the
    incinerator. This closes the grinding discount while keeping capture income.
  - (d) Release fee 0 (M-14) removes the exit-side arbitrage half.
- **Tests (if adopted):** `fee_wallet_cannot_request` (b); `reroll_fee_reaches_sink` (c); a QA check that the disclosure string appears (a).

### M-36 — High (fee change) — The release fee must never trap holders
- **Status:** open (mostly handled in WT; release-fee value needs Barton, joint recommendation 0 (M-14); the upgrade-induced freeze is
  M-41). **Owner:** engineer + Barton. **Sources:** A, B FF-01 (Low; Medium at 10k), B FF-02 (closed in WT).
- **Checked (WT, `unwrap.rs` mtime 16:57:55):**
  - No pause and no open gate (`unwrap.rs:68`).
  - The release fee is paid into the per-vault `FeeVault` PDA, so recipient state can't block release. Only `sweep_fees` can fail.
    B's FF-02 is closed.
  - Every tier ≥ 890,880 lamports.
  - The remaining fee-side failure is the holder's own SOL balance.
- **Remaining:**
  - (a) Soft trap: at low ratio and low FDV the fee exceeds the value returned. 10k is now dropped. At 50k (0.002) release is underwater
    below a ~42 SOL market cap (A and B agree).
  - (b) Release fee = 0 (joint recommendation, M-14).
  - (c) **M-41:** fee/config validation on the release path.
  - (d) Never add a token fee or an oracle-priced fee to release.
- **Tests:**
  - `release_succeeds_when_fee_recipient_closed_or_executable_or_zero_balance`.
  - `sweep_failure_does_not_affect_release`.
  - `release_fee_le_capture_fee_every_tier`, or `release_charges_zero_lamports`.

### M-41 — High (A; B rates Medium) — Fee/config validation on the release path: an upgrade that changes a tier, the fee wallet or the config version freezes release in every collection (B FF-14)
- **Status:** open. **Confirmed in code:** WT `unwrap.rs` mtime 16:57:55, `config.rs` mtime 16:53:11, HEAD `977f8f2` plus uncommitted
  changes, checked 16:58:49 MT. **Owner:** engineer + QA. **Source:** B FF-14 (new).
- **Evidence:**
  - `unwrap.rs:70` runs `let econ = config::econ(&ctx.accounts.launch_config)?;` before paying out, and `unwrap.rs:110` runs
    `invariants::check(..., &econ, ...)` at the end.
  - `econ()` fails unless all of these hold:
    - `config.rs:23`: `cfg.version == SUPPORTED_LAUNCH_CONFIG_VERSION (3)`, else `UnsupportedLaunchConfig`;
    - `config.rs:25`: `MIN_FEE_LAMPORTS ≤ fee ≤ MAX_FEE_LAMPORTS`, else `FeeAboveHardCap`;
    - `config.rs:26`: `fee_for_ratio(cfg.ratio_whole_tokens) == Some(fee)`, else `FeeAboveHardCap`. It also fails if a future
      upgrade removes the ratio from `FEE_TIERS` (the 4:55 PM 10k drop is exactly such a table change; harmless now only because no
      10k collection exists);
    - `config.rs:27`: `cfg.fee_recipient == PLATFORM_FEE_RECIPIENT`, else `FeeRecipientNotPlatform`.
  - Separately, `launch_config` is a typed `Account<LaunchConfig>`, so a `hybrid_launch` upgrade that changes the struct layout would
    fail deserialization on release too.
  - So any routine upgrade that lowers or raises a tier, removes a ratio, changes MIN/MAX, rotates the recipient (M-17's only recovery
    path), or bumps the config version makes **every live collection's release revert** until another upgrade ships (≥ 7 days under
    the timelock).
  - BRIEF.md:84: a SOL fee on release "must never be able to block release".
- **Severity: High (A).**
  - It's an upgrade-induced holder lockout across all collections at once, directly on the exit path M-36 protects. The trigger is a
    well-meant, plausible change (tiers are explicitly expected to change), and recovery takes at least another timelocked upgrade.
  - B rates it Medium, because it needs an upgrade and is recoverable. Recorded.
- **Fix:**
  - `unwrap` uses a **release-only reader** that checks only what release needs:
    - the mint (vault token account and mint pinned);
    - `ratio_base > 0`;
    - `collection_size` for the index bound.
  - Fee = `min(cfg.fee_lamports, MAX_FEE_LAMPORTS)`, or 0 if Barton adopts release = 0.
  - No version equality (accept any version whose prefix layout is known, or read the fields raw). No tier-equality and no recipient
    check (the fee goes to the `FeeVault`).
  - `sweep_fees` alone checks the recipient.
  - Keep strict validation for `request_*`, using M-08's `min(stored, tier, MAX)` rule instead of equality.
  - Invariant checks on release must use the release-only values.
- **Tests (regression, required):**
  - `release_survives_fee_tier_change`: after launch, rebuild with a modified `FEE_TIERS` (raised, lowered, and ratio removed). Release
    against the old config succeeds, returns exactly N, and charges ≤ the stored fee.
  - `release_survives_recipient_rotation`.
  - `release_survives_config_version_bump` (hybrid_launch v4 with an appended field; the old v3 config releases).
  - `release_survives_max_min_fee_change`.

### M-38 — High (ship blocker, new from the WIP) — Test-only mocks/features must never ship; the real Switchboard reveal is unproven
- **Status:** open. **Owner:** engineer + QA. **Sources:** B M-B1, A (WT review).
- **Evidence:**
  - Committed `977f8f2` mock-switchboard `MOCKREVL` reveal requires **no authority signature and no oracle signature**, so any caller
    can reveal any value.
  - The WT mock now requires the authority and a matching oracle, but still does **not** verify the oracle's secp256k1 signature
    (`tests/track-a-hybrid/mock-switchboard/src/lib.rs:10-13, 102-112`).
  - The mock is a separate LiteSVM program loaded at the Switchboard devnet ID. `hybrid_vault` contains no mock code. The risk is:
    - (1) the mock being deployed or loaded anywhere real;
    - (2) tests passing that can't prove reveal authentication.
  - The `test-mock-graduation` feature accepts marker accounts owned by `grADWKnwMo64gj6DYGQEwuiEv9g5KkofdUT64WtWP2W` ("MOCKGRAD").
    It has a `compile_error!` with `mainnet`, and `scripts/build-test-sbf.sh:19-24` guards it.
  - But `graduation.rs:12-13` says `scripts/deploy-devnet.sh` refuses the marker, and **that script doesn't exist** (`scripts/`:
    `build-test-sbf.sh`, `build.sh`, `devnet-airdrop-once.sh`, `env.sh`, `test.sh`). A devnet deploy of a mock-graduation build isn't
    guarded.
- **Remediation:**
  - The mock reveal must never ship and never be deployed.
  - Add the missing deploy guard: refuse any `.so` containing the `MOCKGRAD`/`MOCKREVL` markers or built with `test-*` features.
  - Assert `SWITCHBOARD_PROGRAM_ID` equals the real devnet/mainnet IDs (`Aio4gaXj…` / `SBondMDr…`) per cluster feature.
  - Prove reveal with the real Switchboard on devnet (third-party caller, PDA authority).
- **Tests:**
  - `release_build_contains_no_mock_markers` (CI greps the release `.so` for `MOCKREVL`/`MOCKGRAD` and fails on a hit).
  - `mock_crate_not_in_program_dependency_tree` (`cargo tree -p hybrid_vault`).
  - `switchboard_program_id_matches_cluster`.
  - `third_party_can_reveal_with_pda_authority` (devnet, real Switchboard).
  - `mock_reveal_rejects_bad_oracle_signature` (so the mock can't hide auth bugs).

---

## Findings: Medium

### M-14 — Medium — Fee is never counted as backing; exact per-instruction deltas; release fee
- **Status:** open. **Release fee value: needs Barton**, with the **joint A+B recommendation of release fee = 0**. **Owner:** engineer + QA
  (deltas), Barton (release fee). **Sources:** F-07, T-BURN-01, B FF-01.
- **Required deltas:**
  - `request_capture`: user −N tokens, vault +N; user −f lamports, recipient +f.
  - `request_reroll`: tokens unchanged; NFT → vault; user −f, recipient +f.
  - `unwrap`: vault −N, user +N exactly; user −f_release, `FeeVault` +f_release.
  - settle/recommit/reveal: no fee movement. `expire`: principal only.
  - The fee never touches `vault_tokens`. Supply stays exactly 1B × 10^d.
- **Why release = 0 (joint):**
  - The release fee is an exit tax.
  - B (sim `flat_fee.py` Parts C/G), 0.002-tier release is underwater below a graduation market cap of **~212 SOL at 10k** and **~42 SOL
    at 50k**.
  - A's PoC 04 (fee ÷ value, without tx) gives the same thresholds: 200 SOL at 10k and 40 SOL at 50k for 0.002 SOL. **Our numbers agree.**
  - 10k is now dropped (Barton 4:55 PM), but 50k keeps 0.002.
  - A zero release fee also removes the insider's exit-side arbitrage (FF-03) and keeps M-06 closed (capture = re-roll).
  - `sol_fee` returns early on 0 (`vault_token_ops.rs`), so 0 is safe.
- **Tests:**
  - `per_instruction_token_and_lamport_deltas_exact`.
  - `release_pays_exactly_n`.
  - `release_charges_zero_lamports` (if 0), otherwise `release_fee_le_capture_fee_every_tier`.
  - `fuzz_inv1_inv2_supply_constant`.

### M-15 — Medium — Fee recipient state must not DoS converts (was: fee ATA closed)
- **Status:** **closed in WT (v2.1, verified 16:58 MT)**: no fee token account exists; release fee → `FeeVault`; tiers ≥ 890,880. Regression test required. **Owner:** engineer + QA. **Source:** F-05. **Fee change:** the ATA part is superseded (no token fee;
  `ensure_fee_account` is gone).
- **Requirement:**
  - A missing or zero-balance recipient must not fail a conversion. Every tier is ≥ 890,880 lamports.
  - Release is isolated via `FeeVault` (M-36).
  - Capture and re-roll still credit the recipient directly. They'd fail only if the recipient were executable or reserved (M-05 launch check).
- **Test:** `capture_succeeds_to_zero_balance_recipient`.

### M-17 — Medium — Fee wallet as a hot wallet; changeability of the recipient
- **Status:** needs Barton. **Owner:** Barton + engineer. **Source:** F-09, B-07.
- **Now:**
  - The WT recipient is a **program constant** (`PLATFORM_FEE_RECIPIENT`) recorded in every `LaunchConfig`. It is **not changeable**
    without a program upgrade (3-of-5 + 7 days).
  - A mainnet build refuses to compile until it's set.
  - The devnet value is a throwaway key.
- **Risk:** a hot wallet that's compromised loses future fees until an upgrade re-points it. It can't steal backing.
- **Remediation:** the recipient is a Squads vault (≥ 2-of-3 hardware keys; name the signers). Barton confirms "never changeable except
  by upgrade" (current code) or asks for an in-program `FeeConfig` with a ≥ 7-day timelock.
- **Tests:** `mainnet_build_requires_fee_recipient_set`; `fee_recipient_is_squads_vault` (deploy script).
- **v2.1 (B FF-08, Medium custody):** a stolen key steals accumulated revenue only, never principal, so "never changeable" (current constant) is the better choice (B). Rotating it by upgrade must not break release (M-41). Assert off-curve/system-owned at launch.

### M-19 — Medium — Grinding floor and disclosure: mid tiers and the top ratios are market-priced lotteries (incl. B FF-04, FF-05)
- **Status:** needs Barton (disclosure, and whether to keep 5M). **Owner:** Barton + engineer (UI). **Sources:** F-02, B-04, B-05,
  **B FF-04 (Medium, disclosure)**, **B FF-05 (Low)**. **D6 resolved** (flat constant, no rarity scaling).
- **Numbers** (A PoC 04 Part F; m* = N × (fee + tx) / floor; FDVs are **examples**; 10k dropped by Barton 4:55 PM, collections ≤ 10,000):

  | R | fee | N (≤ 10k cap) | m* @ FDV 100 | @ 410 | @ 1,500 |
  |---|---|---|---|---|---|
  | 50k | 0.002 | 10,000 | 4,200× | 1,024× | 280× |
  | 100k | 0.005 | 10,000 | 5,100× | 1,244× | 340× |
  | 200k | 0.005 | 5,000 | 1,275× | 311× | 85× |
  | 500k | 0.01 | 2,000 | 404× | 98.5× | 26.9× |
  | 1M | 0.01 | 1,000 | 101× | 24.6× | 6.7× |
  | 2.5M | 0.01 | 400 | 16.2× | 3.9× | 1.1× |
  | 5M | 0.01 | 200 | 4.0× | 1.0× | 0.3× |

  - B's FF-04 (`flat_fee.py` Part G) agrees in shape.
  - At graduation, epics at 1M+ (m* ≤ 3.5×) and every tier at 2.5M/5M (1/1 ≈ 5× and 2×) are +EV to grind at plausible premiums.
  - Once the floor is past ~0.25 SOL, mid tiers are a lottery at every ratio ≥ 50k.
  - A higher cap doesn't fix it (B §1.5), because a flat fee can't track the floor. So this is **disclosure, not a fee fix**.
- **FF-05 (pool depletion, Low):**
  - Draws come from the current pool, not the full census.
  - At the pool floor `max(5, 2% × C)`, the 1/1 at C = 10k, F = 0.25 falls from 405× to **9.1×**; at C = 1k, F = 0.1, from 102× to **3.0×**.
  - Each draw is still fair and the odds are public. The protection figures assume a full pool.
- **Remediation:**
  - The re-roll button shows **live pool odds per tier** (pool count / pool size, not the launch census) and "expected cost to hit:
    fee / p SOL".
  - Optionally drop 5M, or keep it with a warning that its rares are lottery-priced.
- **Tests:**
  - `fee_charged_on_every_request`.
  - Frontend `expected_cost_equals_fee_over_live_pool_odds` (FF-04/05).
  - `ui_pool_census_matches_chain` (M-24).

### M-20 — Medium — Escrow cornering and head-of-line queue DoS
- **Status:** open (partly fixed in WT). **Owner:** engineer + QA. **Source:** B-06, B H2/H4.
- **Remediation:**
  - The non-refundable fee makes stalls cost f per attempt.
  - Deadline ≈ 9,000 slots (**v1's ≤ 150 is withdrawn**, M-04 H3).
  - A program-chosen oracle (M-04 H2).
  - Capped recommits, then principal-only expire clearing K heads per call.
  - `merge_incoming` stays permissionless (settle's `MergeBacklog` > 32 is outcome-independent).
  - Donations never affect accounting.
- **Tests:**
  - `queue_of_100_unrevealed_requests_clears_in_bounded_calls`.
  - `donation_to_vault_changes_no_outcome`.
  - `attacker_dead_oracle_head_cannot_stall_queue_beyond_bound`.

### M-21 — Medium — Metadata commitment excludes image bytes
- **Status:** open (partly fixed in WT). **Owner:** engineer + QA. **Source:** B-11.
- **WT:** URIs must start with `ipfs://` or `ar://` (`ALLOWED_URI_PREFIXES`). There is still no `image_sha256` in the committed leaf
  (committed leaf = `leaf_hash(index, name, uri)`, B).
- **Remediation:** the JSON includes `image_sha256`; the leaf commits to `sha256(metadata_json)`; a verify script hashes every image.
- **Tests:** `launch_rejects_mutable_http_image_uri`; `verify_script_detects_swapped_image`.

### M-22 — Medium — Graduation mint crank stall and funding
- **Status:** open. **Owner:** engineer + QA. **Sources:** B-13, B N-02.
- **WT:**
  - `mint_assets` is permissionless, idempotent and Merkle-proven.
  - `open_vault` requires `minted_count == collection_size`.
  - The caller pays rent. The graduation-proceeds carve-out isn't implemented.
  - `MAX_COLLECTION_SIZE = 10,000` (engineer measured 0.00509 SOL per NFT).
- **Remediation:** a program-held carve-out PDA (not an operator wallet, B N-02a) funds the crank. Fail closed if it's short. Publish the
  max N from a CU benchmark.
- **Tests:**
  - `converting_closed_until_fully_minted`.
  - `mint_crank_resumes_after_interruption_by_any_caller`.
  - `mint_crank_idempotent_no_double_mint`.
  - `carve_out_only_spendable_by_mint_crank`.
- **v2.1:** Barton caps collections at 10,000 (B: agree, not lower). The WT enforces it in `hybrid_launch` only; add the same assert in `hybrid_vault::init_vault` (B) — `MAX_COLLECTION_SIZE` is not referenced in `hybrid_vault/src` (checked 16:59 MT).

### M-23 — Medium — FIFO/seq merge and Feistel/Merkle binding correctness
- **Status:** open. **Owner:** engineer + QA. **Sources:** A-D03, A-D04.
- **Note (B):** the WT has no Feistel permutation. Leaves are bound by raw index with VRF selection from a request-time pool. That's
  acceptable, but `hybrid-rarity` §2 still promises a permutation (docs drift, M-33).
- **Tests:** `fuzz_fifo_candidate_set_matches_reference_model`; `leaf_binding_rejects_wrong_index_proof`.

### M-25 — Medium (High if we build our own curve) — Curve/AMM integration math and slippage
- **Status:** open. **Owner:** engineer + QA. **Sources:** T-CURVE-02/04/06.
- **Remediation:** composite flows carry on-chain `min_out`/`max_in`. `request_capture` takes `expected_ratio` and `expected_fee`
  (lamports) and fails on mismatch.
- **Tests:** `composite_buy_nft_respects_max_in`; `request_fails_on_expected_fee_mismatch`.

### M-26 — Medium — Pause scope
- **Status:** **resolved in WT, pending documentation.**
  - The guardian pause was **removed** (`lib.rs:22` "NO pause/guardian of any kind (ADR-015)"; `admin.rs:1-2` cites "Barton
    2026-09-25 4:57 PM MT"). Pause error variants are reserved.
  - `unwrap.rs:68` has no pause and no open gate.
  - A `paused_until` grep finds no uses (checked 16:59 MT).
  - The 4:57 PM decision and ADR-015 aren't in BRIEF.md or DECISIONS.md yet (checked 16:59 MT), so record them.
- **Owner:** Barton (confirm) + engineer. **Sources:** B-10(4), B N-01c.
- **Remaining:**
  - Add ADR-015.
  - A regression test that no instruction has a pause gate.
  - The release-path fee/config validation is a separate issue: M-41.
- **Tests:** `no_instruction_checks_a_pause_flag` (IDL/state: no pause field); `release_and_settle_always_available`.

### M-27 — Medium — Creator/team NFT allocations outside the VRF pool
- **Status:** needs Barton (Q-H6). **Owner:** Barton + engineer. **Source:** B-10(5).
- **Remediation:** forbid them. `mint_assets` mints only into the vault (WT: yes, "into the vault").
- **Test:** `mint_crank_destination_is_vault_only`.

### M-28 — Medium (B: Low, upstream moot) — Discarded CPI result; raw account init (upstream)
- **Status:** moot-if-hybrid_vault. **Owner:** engineer. **Sources:** A-09, A-10.
- **Requirement:** propagate every CPI result; Anchor `init` only.
- **Test:** `attack_reinitialize_vault_fails`.

### M-33 — Medium (was Low; reopened as B FF-11) — Doc and code drift: 2% token fee, burn, refunds; ADR-013/ADR-015 missing
- **Status:** open. **Owner:** engineer (docs, ADRs) + QA (TEST_PLAN) + Creative Director (copy). **Sources:** B-12, A-D08, cross-review §5,
  **B FF-11**.
- **Evidence (checked 16:59–17:03 MT):**
  - `docs/DECISIONS.md` (313 lines, mtime 16:31) ends at ADR-010 plus open questions. **No ADR-013 (flat SOL fee) or ADR-015 (no pause)**,
    though WT code cites both.
  - Several docs still describe burned or escrowed token fees, refunds on expire, ≤ 1,000 bps, or the 2% token fee. The refreshed list
    is in "Doc bugs" below.
- **Remediation:** write ADR-013, ADR-015, and the 4:55 PM graduation decisions. Mark every token-fee, burn and refund passage
  SUPERSEDED. Update user copy.
- **Test:** `rg -n -i "2%|bps|burn|refund|token fee|guardian|pause" docs design qa`. Every hit is historical (marked), an LP/DBC-buffer
  statement, or a correct "no pause" statement.

### M-37 — Medium — VRF cost funding; the 0.002 tier doesn't cover VRF plus crank (incl. B FF-13, Low)
- **Status:** open. **Owner:** engineer + Barton. **Sources:** A; B FF-13; BRIEF.md:89 ("Engineering to confirm 0.002 SOL covers
  per-request VRF cost").
- **Cost figure (checked):**
  - B's ~0.002 SOL per request is a **third-party figure, not an official price**. QuickNode's Switchboard Randomness On-Demand guide
    says "costs about 0.002 SOL per request"
    (quicknode.com/guides/solana-development/3rd-party-integrations/generate-onchain-random-numbers-with-switchboard-vrf).
  - Switchboard's own older post says "just under 0.002 SOL", but that's for its **legacy V2 VRF**, not On-Demand
    (switchboardxyz.medium.com/verifiable-randomness-on-solana-46f72a46d9cf).
  - Official Switchboard docs publish no per-request price.
  - ORAO: 0.001 SOL (co-founder via solana.stackexchange.com/questions/14889).
  - **Unverified on devnet.** Measure it.
- **Finding:**
  - If the platform crank pays the oracle reward and randomness rent, the **0.002 tier (now 50k only)** loses ≈ 0.0002 SOL per draw
    (B), so Barton subsidizes every 50k draw.
  - 0.005/0.01 tiers are covered (+0.0028 / +0.0078).
  - A weak grief (the spammer pays 10× what it costs Barton), not an exploit.
  - WT: `request_*` has the user pay `Request`/`RandLock` rent (`request.rs:55, 58, 122, 125`). `init_randomness`/
    `reveal_randomness` take a separate `payer` and the SB `sb_reward_escrow` (`randomness_ix.rs`), so who funds VRF depends on the crank.
- **Fix:** (a) the requester funds randomness rent and the oracle reward inside `request_*`; or (b) raise the lowest tier to
  **0.003 SOL**. B notes 50k stays fine at 0.003 (capture 15% of V at graduation).
- **Tests:** devnet `measure_switchboard_lamports_per_request` (record in DECISIONS); `requester_funds_randomness_rent_and_reward` or
  `min_tier_ge_measured_vrf_plus_crank`.

---

## Findings: Low

### M-18 — Low (was Medium) — Fee accrual → sell pressure
- **Status:** superseded by fee change: fees are SOL, not the coin, so there's no sell pressure on the coin from fees.
  **Kept:** disclosure of the fee wallet, cumulative fees and outflows (Stonk.fun pattern). **Owner:** Barton. **Source:** F-10.
- **Test:** `ledger_reconciles_fee_wallet` (Σ on-chain fee transfers + `FeeVault` sweeps == the site's figure).

### M-24 — Low (resolved, D3) — Wrap/unwrap arbitrage vs stale listings; pool adverse selection
- **Status:** resolved at Low. Goes back to Medium if the UI shows the launch census as the current odds. **Owner:** engineer (UI).
  **Source:** B-07.
- **Remediation:** show the live unwrap value (R × price, **minus the release fee in SOL**), live pool census and odds.
- **Test:** `ui_pool_census_matches_chain`.

### M-29 — Low — Fee arithmetic
- **Status:** closed/superseded by fee change (v2.1: verified `needs_barton.rs:71-73` compile-time asserts, mtime 16:56): no token fee, so no bps rounding. **Kept:** the fee is a stored lamport constant from
  `FEE_TIERS`, within [MIN, MAX], with checked math. The WT compile-time asserts cover it. **Owner:** engineer + QA. **Source:** F-11.
- **Test:** `fee_tiers_within_bounds_and_cover_all_ratios`.

### M-30 — Low — SOL fee below rent-exempt to an empty recipient
- **Status:** **closed in WT (v2.1)** ( `MIN_FEE_LAMPORTS = 2,000,000 ≥ 890,880`, compile-time assert). **Owner:** engineer. **Source:** F-12.
- **Test:** `fee_transfer_succeeds_to_empty_recipient`.

### M-31 — Low — MEV around converts
- **Status:** open. **Owner:** engineer. **Sources:** B-08, T-CURVE-06.
- **Remediation:** as M-25.

### M-32 — Low — Shipped TODOs; authority footgun (upstream)
- **Status:** moot-if-hybrid_vault. **Owner:** engineer. **Sources:** A-11, now **five** TODOs (`release_v2.rs:110`, `release.rs:99`
  [errata], `update_recipe.rs:58`, `update_escrow.rs:56`, `migrate_tokens_v1.rs:74`), A-12.
- **Requirement:** CI fails on "TODO"/"Need to add" in `programs/`.

### M-40 — Low (B) — The WT solvency invariant omitted pending re-rolls
- **Status:** **fixed in WT** (verified: `invariants.rs:14-20`, mtime 16:49:39: owed = R × (`assets_outside + pending_captures +
  pending_rerolls`)). Regression test required. **Owner:** engineer + QA. **Source:** B M-B3.
- **Test:** `invariant_counts_pending_reroll_handins`.

---

## Findings: Info

### M-34 — Info — Deployed MPL-Hybrid ≠ pinned commit; externally upgradeable
- **Status:** moot-if-hybrid_vault. **Owner:** engineer.
- **Requirement:** no Mintmark value is ever held by MPL-Hybrid.

### M-35 — Info — VRF integration hygiene
- **Status:** open. **Owner:** engineer.
- Pin program IDs per cluster (M-38).
- Parse with the provider SDK or an audited parser.
- **Corrected per B (H1):** reveal and settle do **not** need to be one instruction. Pin `seed_slot`, record the value at reveal, and
  read the stored value at settle (the WT does this).
- Benchmark settle CU at max N.

### M-39 — Info (new, B M-B5) — Audit-baseline drift
- **Status:** open. **Owner:** engineer.
- **Evidence:**
  - `wip/hybrid-vault` was re-pointed `f4761af` → `977f8f2` during review (B).
  - ~50 uncommitted paths changed while we were reading (16:31–16:55 MT), including the fee model, `expire.rs` deleted and re-added,
    and recommit caps.
  - WT findings here are a snapshot.
- **Remediation:** tag an audit commit per round. The third-party audit is pinned to a tag. CI checks that the deployed hash == the
  audited tag.
- **Test:** `deployed_program_hash_equals_audit_tag` (deploy script).

---

## Disagreements between A and B: final state (all resolved)

| # | Topic | Auditor A | Auditor B | Final |
|---|---|---|---|---|
| D1 | Severity of upstream cherry-pick / reroll (M-02) | Concedes Critical if MPL-Hybrid is in the swap path | Critical (conditional) | **Resolved: Critical, moot-if-hybrid_vault.** B13 full drain (1,000,000 → 0 plus fee reroute) added to M-01 evidence. PoC 13 now executes A-04 on-chain |
| D2 | Curve sniping (M-03) | Critical until the venue's activation point and anti-snipe rules are tested | High, as a blocking launch gate: no drain, converts closed on the curve, no curve code, audited venue by default | **Resolved: High, launch-blocking gate.** B's facts verified (`main@658fb95`: only a `launch` ix; 1B in `ATA(launch_vault)`; no curve code). A's Critical rationale recorded in M-03. No drain path found. New: neither DBC nor LaunchLab accepts a pre-existing mint |
| D3 | Stale-listing arbitrage (M-24) | Low | Concedes Low (Medium only if the UI shows the launch census as odds) | **Resolved: Low**, with that condition |
| D4 | VRF failure handling (M-04) | Fee never refunded; PDA authority; bounded recommit; principal-only expire | Accepts, with H1–H6 | **Resolved:** A's rule plus H1 (separate reveal records the value once; recommit only if never revealed; deterministic unskippable settle), H2 (program-chosen, rotating oracle), H3 (~9,000 slots), H4 (cap 3, then principal-only batch expire), H5/H6 noted |
| D5 | Code in scope | `hybrid_launch` exists | Concedes; adds that `hybrid_vault` exists too | **Resolved.** M-11 retitled. WIP/WT findings added (M-04, M-26, M-36–M-40) |
| D6 | SOL-minimum scaling (M-19) | Flat constant suffices | Concedes (sim) | **Resolved: flat constant.** Superseded in magnitude by the fee change: the flat SOL fee is now the whole fee. New PoC 04 numbers in M-19; thin at R ≥ 2.5M (needs Barton) |

## Fee-model re-check (4:49 PM flat SOL fee + 4:52 PM tiers; v2.1 rows = B flat-fee notes)

| Item | Before | Now |
|---|---|---|
| F-01 / M-07 | `hybrid_launch` 0–10% creator bps, BURN | WT: tiered flat SOL fee from a const table, cap 0.01, never creator-set. Fixed in WT; commit + tests |
| F-02 / M-19 | SOL minimum is the grinding floor | The flat fee is the whole floor. Prohibitive at R ≤ 1M; **thin at R = 2.5M/5M** (m* 1–4× at FDV 410). Needs Barton |
| F-03 / M-06 | Release+capture bypass | Closed by construction (one fee for all three; re-roll f < 2f). Test required |
| F-04 / M-16 | Insider discount on token fee | Now the whole fee: insider cost = tx + VRF only. **Tabled by Barton**; impact restated |
| F-05 / M-15 | Fee ATA closed → DoS | Token part superseded. Recipient state can't block release (`FeeVault`); captures only if the recipient is executable/reserved |
| F-06 / M-05 | Fee account substitution | Token-account parts superseded. Kept: recipient == LaunchConfig == platform constant, system-owned, never caller-supplied. Fixed in WT |
| F-07 / M-14 | Fee ≠ backing; no release fee | Release now **has** a SOL fee (Barton). Fee still never touches `vault_tokens`; release returns exactly N. Deltas restated in lamports |
| F-08 / M-08 | "Never raised" false while upgradeable | Copy: "fee set per collection at launch; no setting can raise it" + beta caveat. Cap 0.01 checked at use in WT. Needs Barton (with CD) |
| F-10 / M-18 | Token fee sell pressure | Superseded (SOL, not the coin). Disclosure kept; now Low |
| F-11 / M-29 | Token fee rounding | Superseded. Lamport bounds kept (compile-time asserts in WT) |
| F-12 / M-30 | SOL min below rent | Fixed in WT (floor 0.002 ≥ 890,880) |
| M-26 | Pause scope | Release must never be pausable (WT: it isn't). Guardian concerns (creator-set key) remain |
| New | — | M-36 release-fee trap (soft trap at low ratio × low FDV; consider no release fee), M-37 VRF cost funding |
| v2.1 B FF-14 → M-41 | — | **New High (A) / Medium (B):** `econ()` on the release path; an upgrade changing tier, wallet or version freezes release everywhere |
| v2.1 B FF-07 → M-08 | Cap in code, upgradeable | Charge `min(stored, tier, MAX)`; release never reverts on mismatch |
| v2.1 B FF-03 → M-16 | Medium | High (insider): 21% of public cost at 0.01; 1/1 102× → 22×. Tabled by Barton |
| v2.1 B FF-01 → M-14/M-36 | Release fee = tier | Joint recommendation: release fee 0 (needs Barton) |
| v2.1 B FF-13 → M-37 | VRF unverified | ~0.002 SOL/request (QuickNode 2026, third-party); 0.002 tier ≈ −0.0002/draw; raise to 0.003 or requester pays |
| v2.1 B FF-04/05 → M-19 | Thin at 2.5M/5M | Lottery at mid tiers (F ≳ 0.25) and all tiers at 2.5M/5M; pool depletion up to ~50× cheaper; disclose live odds |
| v2.1 B FF-11 → M-33 | Low | Medium: docs still describe token fee/burn/refund; ADR-013/015 missing |

## Needs Barton
1. **Release fee = 0 (M-14/M-36):** joint A+B recommendation.
   - It's an exit tax, underwater at 50k below a ~42 SOL market cap.
   - Removing it also removes the insider's exit-side arbitrage.
   - M-06 stays closed without it.
2. **Fee copy and freeze date (M-08, M-06), with the Creative Director.**
   - Copy: "fee set per collection at launch; no setting can raise it", plus the beta caveat.
   - Engineering adopts `min(stored, tier)` either way.
3. **Lowest tier vs VRF cost (M-37):** raise 50k from 0.002 to 0.003 SOL, or have the requester pay VRF. Needs a devnet measurement.
4. **Disclosure of lottery-priced tiers and whether to keep 5M (M-19, FF-04):**
   - Live pool odds and expected cost per hit on the re-roll button.
   - Optionally drop 5M, or keep it with a warning.
5. **Fee wallet custody (M-17):** Squads vault, name the signers. B and A now both prefer "never changeable except by upgrade"
   (current code), provided M-41 is fixed so a rotation can't freeze release.
6. **Confirm the pause removal (M-26, ADR-015 "Barton 4:57 PM" in code comments; not yet in BRIEF/DECISIONS).**
7. **Creator allocations outside the pool (M-27).** A recommends forbidding them.
8. **VRF provider (M-04):** Switchboard, pending the devnet real-reveal proof, or ORAO.
9. **Multisig signers, threshold and timelock for upgrades (M-09).**
10. **Tabled, for information: M-16 insider discount.**
    - Now High per B's numbers: 21% of public cost at 0.01; 1/1 break-even 102× → 22× floor.
    - Income-preserving options, none blocking:
      - disclose fee-wallet re-rolls publicly;
      - exclude the fee wallet from `request_*` on-chain;
      - route re-roll fees only (or a share) to a sink.
11. **Ratios and sizes: decided by Barton at 4:55 PM** (drop 10k, keep 50k, cap 10,000, minimum 100).
    - This matches the **joint auditor recommendation**: B would drop 10k (the fee is 52% of NFT value at graduation), keep 50k, and cap
      at 10,000 but no lower, because farming protection scales with collection size.
    - A's numbers agree: PoC 04 shows the release/capture fee exceeding the value of N tokens at 10k below FDV ~200 SOL, and 50k at
      0.002 is 10.5% of V at graduation (B).
    - Remaining: add the vault-side 10,000 assert (M-22).

## Doc bugs — file:line (refreshed v2.1, checked 16:59–17:03 MT; M-33 / B FF-11)
Every line below still describes a superseded design, contradicting BRIEF 4:49/4:52/4:55 PM, the WT code, or ADR-015 (no pause):
- a **burned** or **escrowed/refunded** token fee;
- **bps / ≤ 10% / 1,000 bps**;
- the **4:27 PM 2% token fee**;
- `fee_destination = BURN`;
- a guardian/pause power.

LP/DBC buffer statements, upstream `BurnOn*` paths and Core-burn rent notes are excluded.

**Missing records**
- `docs/DECISIONS.md` has no ADR-013 (flat SOL fee), no ADR-015 (no pause), and no record of the 4:55 PM graduation decisions. The WT
  code cites ADR-013/015.

**Code and tests**
- Committed `main@658fb95` `hybrid_launch`:
  - `constants.rs:44-50`;
  - `validation.rs:15-16, 31-37, 84`;
  - `state.rs:6, 41-42`;
  - `error.rs:17-18`;
  - `lib.rs:14`;
  - `launch.rs:4, 186-190`;
  - tests `launch.rs:16, 39, 161, 281-283` and `qa_launch.rs:25, 57, 186-187, 247, 373-377, 880-881` (BURN / bps).
  - Superseded by the WT. Commit it.
- Committed `977f8f2` `expire.rs:1-4, 62-63, 109` (fee refund). Superseded by the WT `expire.rs` (principal only).
- WT `graduation.rs:12-13` cites a non-existent `scripts/deploy-devnet.sh` (M-38).

**docs/**
- `docs/BRIEF.md:62` (token fees burned) and `:71-76` (4:27 PM 2% token fee): add "SUPERSEDED by 4:49/4:52 PM" markers.
- `docs/DECISIONS.md`:
  - `:117, 151, 192, 198, 206, 214, 239-242` (ADR-008/009/010 burn, `fee_destination = BURN`, ≤ 1,000 bps);
  - `:289` (Q-H4 token fees);
  - `:299-300` (N1);
  - `:306-307` (N4 pause: now decided);
  - `:311-313` (N7 burn at settle, refund on expire);
  - `:66` (withheld-fee refund).
- `docs/ARCHITECTURE.md`:
  - `:20, 124` (pause/guardian);
  - `:22, 38, 73, 77, 92, 96, 99` (burned bps token fee, `fee_destination = BURN`);
  - `:52` ("optional guardian").
- `docs/THREAT_MODEL.md:22, 24` (burn; pause as the one admin power), `:72, 83, 94, 96, 146` (T-TOK-04, T-HL-06, T-BURN-01/03,
  T-HV-12 token-fee defenses).
- `docs/hybrid-rarity-and-assignment.md`:
  - `:22, 184-187, 207, 253, 255, 257` (escrowed token fee, burn at settle, **refund on expire**);
  - `:308-315, 325, 331, 336` (token fee bps recommendation, 1,000-bps cap, Σburned);
  - `:351, 356-358, 372` (token-fee defenses, "refunded minus VRF", "≤ 10%", "no destination account").
- `docs/qa-answers.md:22-33, 45, 79` (pause), `:28-29, 72, 74-76, 81` (burn, refund, bps, "burned by re-roll fees").
- `docs/admin-multisig-timelock.md:20` ("fees are burned, no destination account"), `:23-47, 61, 65` (the pause power, now removed).
- `docs/marketplaces-and-ratios.md:7, 136` ("2% of N in tokens", "the 2% fee divides exactly"). Also uses N = 10k examples
  (`:89`); 10k is dropped.
- `docs/graduation-design.md:13, 429` ("2% of N in collection tokens", "+ the 2% token fee"), `:413` (10k ratio, now dropped: mark
  historical).

**qa/**
- `qa/TEST_PLAN.md` is partly updated (v0.5 §17 FEE-01..11; INV-10/RR-02 marked obsolete). Stale lines remain:
  - `:28, 48, 87` ("VRF re-roll with burned fee", "re-roll burns can only reduce it");
  - `:178` (CFG-08 burns);
  - `:233` (RR-08: engineering default refund);
  - `:236` (RR-11 `capture_fee ≥ reroll_fee` via `CaptureFeeBelowRerollFee`: now a single tier fee);
  - `:237` (RR-12 "burned total").
- `qa/FINDINGS_TRACKER.md`: re-key to v2.1 (M-41 new; M-16 High; M-33 Medium; M-06/07/15/26/29/30/40 closed in WT).

**design/ (user-facing copy)**
- `design/directions/README.md:35` ("Re-roll fees are paid in the collection's token and burned").
- The a-obsidian pages were updated at 16:58. `NOTE.md:84` records the removal, and no stale fee/burn copy was found in
  `token.html`/`launch.html`.
- Copy to use:
  - "Flat SOL fee per capture, release and re-roll, set per collection at launch; no setting can raise it", plus the beta caveat.
  - Supply: "Fixed at 1,000,000,000."

**security/** (errata only)
- `security/auditor-a/round1.md`: errata added.
- `security/auditor-b/round1.md:38, 177, 288, 309-323`: burn-based (B's to annotate).

## `hybrid_vault` requirements checklist (each with a passing test)

**Economics and config**
- [ ] Reads ratio, mint, N, fee and fee recipient from the immutable `LaunchConfig` v3. No local copies. (M-01, M-07) — *WT: yes*
- [ ] No instruction modifies ratio, mint, N, trait root, vault, fee or recipient. No config/vault close. (M-01)
- [ ] Fee = `FEE_TIERS[ratio]`, within [0.002, 0.01] SOL, re-checked at use (`FeeAboveHardCap`). (M-07, M-08, M-29) — *WT: yes*
- [ ] Recipient == `PLATFORM_FEE_RECIPIENT`, never a parameter or argument; launch/deploy check that it's system-owned and not
  executable. (M-05, M-17) — *WT: first part yes*
- [ ] Same fee on capture, release and re-roll, or at least `capture ≥ re-roll`. (M-06) — *WT: yes*
- [ ] Release: exactly N back; fee into `FeeVault` (recommended 0); no pause/open gate; succeeds regardless of recipient state. (M-36, M-26) — *WT: yes*
- [ ] Release uses a release-only reader (mint, ratio, size; fee = min(stored, MAX) or 0): **no version/tier/recipient equality**, survives any tier, recipient or version upgrade. (M-41) — *WT: no (`unwrap.rs:70` → `config.rs:23-27`)*
- [ ] Requests charge `min(stored fee, current tier, MAX)`; an upgrade can only lower an existing launch's fee. (M-08) — *WT: no (equality)*
- [ ] Fee never touches `vault_tokens`. INV-1 counts pending re-rolls. Supply constant. (M-14, M-40) — *WT: INV-1 fixed 16:49*
- [ ] One vault PDA per collection. Nothing burns from the vault. Fuzzed. (M-12)

**Selection and VRF**
- [ ] `request_*` takes no asset/index. Rejection sampling, without replacement, pool pinned at request. Hand-in excluded. (M-02) — *WT: yes*
- [ ] PDA-authority randomness; freshness and not-revealed checked; pubkey/slot pinned. (M-04) — *WT: yes*
- [ ] **Program-chosen** oracle, different on each recommit; skip stale oracles. (M-04 H2) — *WT: yes (`randomness.rs:108-109, 198-218`); no heartbeat filter*
- [ ] Fee at request, never refunded. (M-04) — *WT: yes*
- [ ] Separate permissionless reveal records the value once. Settle deterministic from it, FIFO, unskippable. Recommit only if never
  revealed. (M-04 H1) — *WT: yes*
- [ ] Deadline ≈ 9,000 slots; ≤ 3 recommits; then principal-only expire after ≥ 1 day, batch-capable. (M-04 H3/H4, M-20) — *WT: yes except batch*
- [ ] Requester funds randomness rent/reward, or the lowest tier ≥ measured VRF + crank (0.003 proposed). (M-37)
- [ ] `init_vault` asserts `collection_size ≤ 10,000` itself. (M-22)
- [ ] Optional (Barton): `request_*` rejects the fee wallet; re-roll fee share to a sink. (M-16)
- [ ] No SlotHashes/Clock/counter in selection. (M-02)

**Lifecycle, authority, shipping**
- [ ] Converting opens only after verified venue graduation **and** full mint. Crank permissionless, resumable, idempotent, funded by a
  program-held carve-out. (M-10, M-22) — *WT: gate yes (fail-closed); carve-out no*
- [ ] Mint crank mints only to the vault. (M-27) — *WT: yes*
- [ ] Content-addressed URIs **and** `image_sha256` committed. (M-21) — *WT: first part only*
- [ ] No pause. (M-26) — *WT: yes (guardian removed, ADR-015 to be written)*
- [ ] No vault outflow except unwrap, settle and principal-only expire. `FeeVault` → recipient only. (M-09)
- [ ] Upgrade authority = Squads ≥ 3-of-5 with a ≥ 7-day timelock, then `--final`. (M-08, M-09)
- [ ] No mock markers or test features in release builds; deploy guard exists; real-Switchboard devnet reveal proven. (M-38)
- [ ] Audit tag pinned; deployed hash == tag. (M-39)
- [ ] PoCs 10/11/12/13 and B13 ported as `regress_*` and green in CI. (M-01, M-02)

## Launch readiness
**Mintmark is NOT launch-ready.**
- `hybrid_vault` is real but unaudited, uncommitted, and was still changing during this review.
- The committed WIP refunds fees on expire and ships an unauthenticated mock reveal path in tests.
- The real Switchboard PDA-authority reveal is unproven on devnet.
- Release can be frozen for every collection by a routine fee/wallet/version upgrade (M-41).
- Meteora DBC is chosen (4:55 PM) but creates its own mint, so the `hybrid_launch`/graduation design must change.
- LP custody and the mint carve-out aren't implemented.
- Mid tiers and the 2.5M/5M ratios are lottery-priced (disclosure), and the 0.002 tier doesn't cover VRF.

Close every Critical/High item and pass the checklist on localnet, then on devnet with the real Switchboard, before any public devnet
launch.

**A professional third-party audit of `hybrid_vault`, `hybrid_launch`, the graduation/migration and mint-crank flow, the VRF integration,
the venue integration, and the authority/deployment configuration is REQUIRED before mainnet. This internal A+B review is not a
substitute.**
