# Launchpad QA Test Plan: SPL-404 hybrid launches

Status: **DRAFT v0.8** (2026-09-25, ~6:05 PM MT). Owner: QA. Previous: `qa/archive/TEST_PLAN.v0.7.1.md`.
Networks: **localnet and devnet only.** Nothing in this plan deploys to mainnet or uses real funds or keys.

**Changelog**
- **v0.8 (2026-09-25 ~6:05 PM MT): baseline `audit-baseline-r1` = a1cfa8f** (local wip/hybrid-vault, not pushed). See §22.
  - **QA-FEE-04 ACCEPTED (Barton):** M-06 (re-roll cost <= release + capture) is checked ON AVERAGE; the per-draw check
    EXCLUDES the first-mint delta. FEE-07 / LAZY-08 rewritten accordingly (exact per-draw: re-roll f + tx, release +
    capture f + 2tx, all 7 tiers, on-chain). New QA-FEE-05 (Info): the average holds only for P >= m/tx drawable indices.
  - Mint escrow 6,338,100 lamports in its own PDA ["mint_escrow", vault, seq] (all four instructions); refunded in full
    when the pick is already minted or on expire; the fee is never refunded on expire. Measured first-mint cost at r1:
    3,066,000 lamports (asset rent + 0.0015 SOL Core fee).
  - tests/regression ported to the r1 harness (53 tests, 0 ignored; old files in `qa/archive/regression-f4761af/`);
    LAZY-01..14 implemented; qa_launch fee / lazy / 10k tests un-ignored (FEE-04 stays ignored: 0-fee semantics).
  - Build hygiene: QA builds only in `qa/.target` from a detached worktree (`qa/scripts/qa-build.sh`, `qa-run.sh`).
- **v0.7.1 (2026-09-25 ~5:25 PM MT):** §21 aligned to `docs/lazy-mint-interface.md` ("frozen for QA", 17:15 MT): settle_capture /
  settle_reroll(mint: Option<MintArgs>), request `mint_escrow_lamports` = MINT_ESCROW_LAMPORTS 6,338,100, pool v2, errors
  MintArgsMissing / MintEscrowShort / AlreadyMinted / MintCostConstantStale.
  - LAZY-04: exact expire refund (principal + request and rand_lock rent + the full unspent escrow; tier fee kept).
  - New LAZY-14: a pick on an already-minted index gets the unused escrow back at settle (provisional, per the interface).
  - New LAZY-13: Merkle leaf + proof checks and trait_root immutability.
  - LAZY-10 split into 10a (model chi-square over all held indices, active) and 10b (on-chain sampling, ignored).
  - LAZY-05: attributes are not on-chain (no plugins); they're bound through the leaf.
- **v0.7 (2026-09-25 ~5:30 PM MT; main = 658fb95, vault wip f4761af, engineer WT uncommitted):**
  - **Release is FREE (Barton 5:04 PM MT).** The tier fee applies to capture and re-roll only; re-roll fee == capture
    fee. Release takes no fee account; there's no holding account (FeeVault) and no sweep. **M-36 and QA-FEE-02:
    resolved by removing the fee.**
    - **OBSOLETE, deleted:** FEE-15 (`qa_FEE15_M36_sweep_only_to_fixed_fee_wallet_and_permissionless`), FEE-16
      (`qa_FEE16_M36_fee_vault_lamports_exit_only_via_sweep`), and the FeeVault cases of FEE-08.
    - FEE-08 is now "release takes no fee account" (ignored: f4761af still has the token `fee_escrow`) plus
      "payer with only the tx fee can release" (**active, passes on f4761af**).
    - New FEE-18: re-roll fee == capture fee per tier, and no release fee. FEE-07 (M-06) now uses release = tx fee only.
    - FEE-14 is renamed to M-15.
  - **Lazy minting (Barton 5:13 PM MT).** Nothing is pre-minted at graduation. Asset i is minted in settle the first time
    VRF picks it, and the capturer pays rent + the Core fee (graduation-design §2.5/§4.4, T-HV-16). §18 is reworked.
    - **OBSOLETE:** batch pre-mint, crank_mint, "fully minted before opening", the proceeds-funded mint, the mint budget
      and the unfundable-launch refusal (old GRAD-02/03/04/08/09/14/17; tests `qa_M22_GRAD03_*`, `qa_M22_GRAD04_*`,
      `qa_M10_GRAD05_*` and `qa_GRAD_M22_unfundable_launch_rejected` are deleted).
    - New GRAD-17 "10,000 at 85 SOL ACCEPTED" and SIZE-01 "N ≤ 10,000 enforced" (both ignored until the code lands).
    - New §21 LAZY-00..LAZY-12.
  - **Flagged QA-FEE-04 (design question):** under lazy minting, a re-roll that lands on an unminted index costs more
    than a release + capture that lands on a minted one, by m − tx (0.00346–0.00667 SOL). M-06 holds in expectation and
    when both land the same way, but not for every outcome.
  - v0.6 archived at `qa/archive/TEST_PLAN.v0.6.md`.
- **v0.6 (2026-09-25 ~5:15 PM MT; main = 658fb95, vault wip f4761af, engineer WT uncommitted):**
  - **Finding IDs remapped to merged v2 (final), M-01..M-40.** v2 keeps M-01..M-35 with the same numbers and subjects
    (titles/severities updated) and adds M-36..M-40, so v1→v2 is the identity for 01..35. Effective moves: "release fee
    never blocks release" M-26 → **M-36**; M-26 = pause scope only; F-06 = M-05. The tracker has a v1-ID column.
  - **Tiered flat SOL fee (BRIEF 4:52 PM + 10k drop 4:55 PM):** 50k = 0.002 SOL; 100k/200k = 0.005; 500k/1M/2.5M/5M = 0.01.
    Fixed per collection in LaunchConfig, immutable, hard cap **0.01 SOL** (resolves the cap half of QA-FEE-03; fee = 0 still
    undefined). Code (WT only) DERIVES the fee from the ratio, so "mismatched fee rejected" is tested as "no fee param +
    recorded fee == tier". New FEE-12..FEE-17; FEE-07/09 parameterized over all 7 tiers.
  - **FEE ID reconciliation:** FEE-06 = fee-destination substitution (vault side), FEE-09 = SOL totals, FEE-10 = escrow
    property; the old FEE-10 "immutability" is now FEE-13 (per tier). Test names follow.
  - **10k ratio dropped:** ratios {50k, 100k, 200k, 500k, 1M, 2.5M, 5M}; N ∈ [100, 10,000]; `qa_RATIO_10k_dropped_launch_rejected`
    (ignored until committed; main still accepts 10k). **Unfundable launches refused** (ADR-014): GRAD-17
    `qa_GRAD_M22_unfundable_launch_rejected`.
  - **Pause removed (ADR-015):** RR-14, GOV-04, the §9 pause row and `qa_M26_release_still_works_while_paused` are
    **OBSOLETE** (test deleted). Added `qa_M26_no_pause_instruction_or_field_in_launch` (active, passes on main) and
    `qa_M26_no_pause_instruction_or_field_in_vault` (ignored: f4761af still has pause/unpause).
  - **QA-FEE-02 resolved by design (pending test):** release fee → per-vault program-owned FeeVault PDA, permissionless
    `sweep_fees` ONLY to the fixed fee wallet; launch checks the wallet is system-owned / not executable. Tests FEE-08, 14, 15, 16.
  - **M-16 accepted risk:** its test is documentation only (prints insider cost per tier, never fails).
  - **Expire (M-04 wins over N7):** principal refunded, SOL fee NOT refunded. FEE-17 `qa_M04_expire_refunds_principal_not_fee`.
  - v0.5 archived at `qa/archive/TEST_PLAN.v0.5.md`.
- **v0.5 (2026-09-25; main = 658fb95, hybrid_vault on `wip/hybrid-vault` f4761af / 977f8f2, engineer working tree uncommitted and changing):**
  - **Primary finding IDs are now the merged list** (`security/merged/round1-merged.md`, M-01..M-35). Original auditor IDs are a cross-reference column. New §16 "Regression tests for audit findings" maps every M-ID (criticals first) to setup / attack / expected result and a concrete test. `qa/FINDINGS_TRACKER.md` restructured to match.
  - **Auditor A PoCs ported** to Rust LiteSVM against hybrid_vault's real instructions (`tests/regression/`, M-IDs in names): A-01 → M-01, A-02 → M-01, A-03 → M-02. All exploits are blocked at f4761af.
  - **Fees: SOL-only (BRIEF 2026-09-25 4:49 PM).** New §17. No token fee of any kind; a flat SOL fee (default 0.01 SOL, hard cap) on every capture, release and re-roll to a fee wallet fixed in LaunchConfig.
    - **OBSOLETE and removed from the suite:** every 2% token-fee test and invariant written earlier today for the 4:27 PM decision (fee = ceil(2% × ratio) exactness over ratio × decimals × bps, bps hard cap 200, fee-ATA spoofing, capture-SOL ≥ re-roll-SOL ordering). The tests `qa_fee_amount_is_exact_for_every_ratio_decimals_and_bps`, `qa_fee_v2_bps_hard_capped_at_2_percent`, `qa_fee_v2_fee_account_fixed_to_platform_owner_ata_spoof_rejected`, `qa_fee_v2_reroll_sol_minimum_and_capture_ge_reroll`, `qa_fee_config_edges_for_the_built_fee_model` were deleted.
    - **OBSOLETE (burn design):** v0.4 INV-10 "token fee burned at settle", SUP/CFG checks `fee_destination == BURN`, `capture_fee_bps ≥ reroll_fee_bps`, "supply only decreases by burns", DECISIONS N7 "escrow fee in request PDA, burn at settle, refund on expire". Supply is now exactly 1B forever (no burns). `qa_launch.rs` no longer asserts anything about token-fee values; its oracle still predicts main's (legacy) acceptance rules so the suite runs against 658fb95.
  - **Graduation** section redone (§18, GRAD-01..16), including the Core mint-cost benchmark (`qa/reports/2026-09-25-core-mint-cost.md`).
  - **Ratio/supply (§19):** ratio set {10k, 50k, 100k, 200k, 500k, 1M, 2.5M, 5M}, min 100, max min(1B/R, MAX_COLLECTION_SIZE if built). HL-01 / HL-02 / new-ratio tests **un-ignored and verified on main 658fb95**. The pending 10,000 cap (working tree) is followed automatically when built.
  - `qa_launch.rs` rewritten schema-driven (IDL at build time) so it compiles against main and the engineer's changing tree.
  - v0.4 archived at `qa/archive/TEST_PLAN.v0.4.md`.
- **v0.4 (2026-09-24, after the engineer's 3:04 PM MT update; code at local `main` c02be68):**
  - **Q1 resolved:** the swap engine is a custom **`hybrid_vault`** with **Metaplex Core** NFTs. MPL-Hybrid is reference only. Capture and re-roll are two-step request → VRF → settle; unwrap (`release`) is instant with no token fee; token fees are burned.
  - Folded in `docs/qa-answers.md` (cited as **[QA-ANS Qn]**) and `docs/admin-multisig-timelock.md` (cited as **[AMT §n]**). Q2, Q6, Q7 and Q9 are resolved or narrowed, and Q11 (mints) is answered. See §14.
  - Launch is real code now: `hybrid_launch::launch`. §3 uses the real instruction, account, seed and error names, and SUP-02/03/04/06 are reworded for the real design (there is no separate `assert_launch_ready`/`open_sale` instruction).
  - **Track A rejects Token-2022.** This replaces all separation content; there's no second track.
  - Engine instruction names now come from the design doc (`request_capture`, `request_reroll`, `settle`, `release`, `expire`, optional `pause_new_requests`), still _(prov.)_ until `hybrid_vault` code lands. `propose_fees`/`execute_fees` are removed because fees are fixed at init (ADR-009).
  - §13 layout matches the real `tests/track-a-hybrid/` crate. §15 coverage is updated, and the first QA report is `qa/reports/2026-09-24-hybrid-launch.md`.
  - v0.3 is archived at `qa/archive/TEST_PLAN.v0.3.md`.
- **v0.3 (2026-09-24):** Barton's scope change (docs/BRIEF.md "SCOPE CHANGE", 2:51 PM MT).
  - **Token-2022 is dropped for now** (deferred, not dropped for good). That removes Track B entirely: transfer tax, tax treasury and NFT buys, holder raffle, the Token-2022 fee math and extension matrix, and the track-separation section (old INV-13…16 and its E2E cases). All of it is now the single rule **Track A rejects Token-2022**.
  - The product is **SPL-404 hybrid launches only**.
  - Added: fixed-supply launch checks, blind NFT assignment, the VRF re-roll with burned fee, multisig + timelock governance, and bonding-curve anti-sniping.
  - Invariants renumbered INV-01…16 (old IDs don't carry over).
  - v0.2 is archived at `qa/archive/TEST_PLAN.v0.2.md`, and shelved Track B notes are in `tests/_shelved/track-b/README.md`.
- v0.3 (2026-09-24): SPL-404 only (scope change). Archived.
- v0.2 (2026-09-24): two tracks (A: SPL-404, B: Token-2022 tax). Archived.
- v0.1 (2026-09-24): initial draft. Archived.

---

## 1. Scope and assumptions

### 1.1 Product (only this)

An **SPL-404 hybrid launch** works like this:
- A **classic SPL Token** mint with a fixed supply of **exactly 1,000,000,000** tokens. **Track A rejects Token-2022**: `token_program: Program<Token>`, and the mint is created by the program as a fresh keypair.
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
| Launch: `hybrid_launch::launch` creates the mint, mints 1B × 10^d to the launch ATA, revokes the mint authority, writes an immutable `LaunchConfig` | `programs/hybrid_launch` | **code @ c02be68** (4 unit + 19 LiteSVM engineer tests; 24 + 1 QA tests) |
| Swap engine `hybrid_vault` (Metaplex Core): `request_capture`, `request_reroll`, `settle`, `release`, `expire` _(prov.)_ | `hybrid_vault` | **engine decided (Q1)**; code not in c02be68 (untracked WIP in the tree) |
| Governance: Squads v4 multisig + timelock for program upgrades, and the optional `pause_new_requests` (no economic parameter is mutable) | `gov` | no code (Squads, not ours); needs Barton (N4/N5) |
| Bonding curve / launch window (our program or a third party) | `curve` | no code yet, venue is Q4 |
| Next.js site (launch wizard, token page, convert, re-roll) | `app` | `app/` exists; E2E not started |

Out of scope: Token-2022 anything (deferred), lottery/raffle (shelved), mainnet deployment, running the audits themselves.

### 1.2 Assumptions and sources (read first)

- **Sources:**
  - `docs/BRIEF.md`, including the SCOPE CHANGE section.
  - `docs/stonkfun-lessons.md`.
  - `docs/hybrid-rarity-and-assignment.md` and `docs/DECISIONS.md` ADR-008, ADR-009 and ADR-010.
  - `docs/admin-multisig-timelock.md` [AMT].
  - `docs/THREAT_MODEL.md` T-HY/T-HV/T-MKT rows.
  - `security/auditor-b/design-requirements.md` R-01, R-03, R-10, R-11 and R-12.
  - `docs/qa-answers.md` [QA-ANS], the engineer's answers to QA's questions.
- **Swap engine: DECIDED (Q1 resolved, engineer update 2026-09-24 3:04 PM MT).**
  - It's a custom **`hybrid_vault`** with **Metaplex Core** assets. MPL-Hybrid is **reference only**: it's not in the swap path and not deployed by us.
  - Capture and re-roll are two-step: request (payment locked) → VRF → permissionless `settle`, in FIFO `seq` order. `release` is instant and pays exactly R_base with no token fee. Token fees are burned.
  - Why not MPL-Hybrid (from its source @ `aacf1a53`): the capturer names the asset (`capture_v2.rs:52-54`), the reroll is predictable (`:187-203`), `update_recipe` is mutable with no timelock (`update_recipe.rs:91-138`), and it can't burn fees.
  - _Doc lag:_ `docs/hybrid-rarity-and-assignment.md`, DECISIONS Q-H1/ADR-008 and QA-ANS "Q1 engine" still say "needs Barton". This is flagged to the engineer in the QA report §5.
  - Engine instruction names below follow hybrid-rarity §3.3 (iii) and are _(prov.)_ until `hybrid_vault` code is committed.
- **Code status (c02be68):**
  - `programs/hybrid_launch` exists (ADR-010) and is tested. See §3 for its real names.
  - `hybrid_vault` isn't committed yet. Track B programs are shelved (`shelved/`).
- **Track A rejects Token-2022.**
  - On-chain: `launch` requires `token_program == spl_token::ID` (`Program<Token>`, Anchor error 3008 otherwise) and creates the mint itself, so an existing Token-2022 mint can't be passed in (QA-ANS Q11).
  - QA covers plain and extension-bearing Token-2022 mints (`qa_real_token_2022_mints_with_and_without_extensions_cannot_be_launched`).
- **Mints are launchpad-created only** [QA-ANS Q11]. Existing classic or Token-2022 mints can't be onboarded.
- **No registry program** [QA-ANS Q9]. `hybrid_launch` owns the `LaunchConfig` PDAs, and the engine reads `LaunchConfig` by pinned program ID + seeds instead of keeping its own copy. So the engine tests must include "fake LaunchConfig from another program / other seeds is rejected".
- **Supply after burns:**
  - The token re-roll fee is burned (BRIEF scope change: "default is BURN"; "re-roll burns can only reduce it").
  - So the rule is **exactly 1B at launch, and afterwards `supply = 1B − Σ recorded burns`**, never more.
  - Burns must never touch the vault or exact unwrap.
- **Freeze authority revoked:**
  - BRIEF hard requirement #1 only says *mint* authority is revoked.
  - Freeze authority `None` comes from ADR-004/ADR-010, [QA-ANS Q6] and Auditor B R-03.1. `hybrid_launch` never sets it, and both are tested.
- **No lottery on hybrid launches, no transfer tax, no discretionary platform wallet** (stonkfun lessons 1–2).

### 1.3 Test levels and tooling

| Level | Tool | Where it runs |
|-------|------|---------------|
| Unit (supply/ratio math, fee bps, Feistel permutation, rejection sampling, timelock arithmetic) | `cargo test` | local |
| Program integration | Rust + LiteSVM 0.10 (same as the engineer's harness: Anchor 1.2.0, Rust 1.89, SBPF v2) and the localnet validator via `anchor test --validator legacy` | local |
| VRF | Mock VRF accounts in LiteSVM (owner and layout of the chosen provider, Q5); devnet queue for smoke tests only | localnet / devnet |
| Upstream programs | Metaplex Core, dumped read-only from devnet (MPL-Hybrid only as reference, not loaded) | localnet |
| Property tests | Seeded xorshift loops (`proptest` isn't in the lockfile; add it only if the engineer agrees) | local |
| Fuzzing | Trident 0.12 | local |
| Security regression | one test per finding ID (§11) | CI + local |
| E2E / UI | Playwright + mocked wallet adapter (§12) | localnet, then devnet |

---

## 2. Invariants

Every integration test ends with `assert_invariants()`, and every Trident flow asserts them after each instruction. Notation: `D = 10^decimals`, `R_base = R × D`, `S0 = 1_000_000_000 × D`.

| ID | Invariant |
|----|-----------|
| **INV-01** Fixed launch supply | After `hybrid_launch::launch`: `mint.supply == S0 == LaunchConfig.total_supply_base`, `mint_authority == None`, `freeze_authority == None`, the mint is owned by the classic Token program (82 bytes, no Token-2022), and the whole S0 sits in `LaunchConfig.launch_destination`. All of this happens in one tx, or nothing persists. |
| **INV-02** Supply fixed forever _(v0.5)_ | After launch, `mint.supply == S0` forever. No instruction mints or burns (BRIEF 4:49 PM: no token fee, so no burns). _(v0.4 "only shrinks by burns" is obsolete.)_ |
| **INV-03** Exact per-instruction token deltas _(v0.5)_ | capture: user −R_base, vault +R_base. release: vault −R_base, user +R_base. re-roll: no token moves. No token fee anywhere; no other token balance changes. SOL: each of capture / release / re-roll moves exactly `sol_fee` from the user to `LaunchConfig.fee_owner` (FEE-06). |
| **INV-04** Escrow backing (INV-ESC) | `vault_token_balance == R_base × (nfts_outside + pending_captures + pending_rerolls)` after every instruction (a pending re-roll's handed-in NFT is back in the vault while its R_base stays escrowed for the replacement). With no pending requests: `== R_base × nfts_outside`. Donations: `≥` with surplus tracked (Q11). |
| **INV-05** NFT conservation | `pool + incoming + nfts_outside_vault + pending_reroll_handins == collection_size`, counting unminted indices as pool. No NFT is created beyond N, and none is lost. |
| **INV-06** Exact unwrap | `release` always pays exactly `R_base` tokens: no token deduction ever and no SOL fee (release is free, BRIEF 5:04 PM; M-36 resolved, FEE-08). |
| **INV-07** Supply cap at config _(v0.6)_ | `ratio_whole_tokens ∈ {50k, 100k, 200k, 500k, 1M, 2.5M, 5M}` (10k dropped 4:55 PM), `100 ≤ collection_size ≤ min(1B/R, 10,000)`, graduation fundable (GRAD-17), `max_tokens_in_nft_form ≤ total_supply_base` (checked), `decimals ≤ 9`, SOL fee ≤ hard cap (FEE-03), fee wallet = program-fixed owner (FEE-05). No token-fee fields. `LaunchConfig` can't be updated or closed. |
| **INV-08** Blind assignment | The recipient never chooses the NFT. The pick depends only on a VRF value produced **after** payment is locked, and on a candidate pool fixed at request time (deposits with `seq < s`). |
| **INV-09** Single-use randomness | Each request consumes exactly one fulfilled VRF result from the account pinned at request time, exactly once. There's no cancel or refund after fulfilment, and `expire` works only if the request is unfulfilled past `deadline_slot`. |
| **INV-10** ~~Re-roll fee burned~~ **OBSOLETE (v0.5)** | Replaced by FEE-01..FEE-11 (§17): no token fee, flat SOL fee. |
| **INV-11** Fee destination public and fixed _(v0.7)_ | All fees are SOL (capture and re-roll only; release free), paid directly by the user to `LaunchConfig.fee_recipient` (program-fixed, recorded at launch, never changeable; M-05/M-17). No instruction takes a destination argument; no operator wallet custodies user tokens. Supply goes to the launch-vault PDA (QA-HL-02 verified at 658fb95). |
| **INV-12** Governance: multisig + timelock | No economic parameter is mutable [AMT §2]. The only powers are the program upgrade (Squads v4 vault PDA, `time_lock` proposed at 7 days, then `--final`) and optionally `pause_new_requests` (guardian or governance multisig, auto-expiring after `MAX_PAUSE_SLOTS`) [AMT §3–5, QA-ANS Q3/Q6]. Pause never blocks `release`, `settle` or `expire`. There's no immediate path to change anything else. |
| **INV-13** Access control and account validation | Privileged instructions need the documented signer. Every PDA is checked (seeds, **canonical** bump, owner, discriminator), and every external program is pinned. Launch seeds: `["launch_config", mint]` and `["mint_authority", launch_config]`, with the bumps stored in `LaunchConfig.bump` / `mint_authority_bump`. |
| **INV-14** No stuck value / close safety | No account holding user tokens, NFTs or escrowed fees can be closed. `expire` returns the principal. Closing returns rent only to the recorded payer. |
| **INV-15** Launch fairness window | No trade or capture happens before `open_slot` (stored at init). During the anti-snipe window, per-wallet, per-tx and per-slot caps hold (Q4). |
| **INV-16** UI truthfulness | Ratio, collection size, "% convertible", fees, fee destinations, supply, authorities and pending timelocked changes on the site equal the on-chain values. |

---

## 3. Launch and supply (INV-01, 02, 07): `hybrid_launch::launch`

**Real interface (c02be68):**
- `launch(params: LaunchParams { decimals, ratio_whole_tokens, collection_size, capture_fee_bps, reroll_fee_bps, fee_destination })`.
- Accounts `hybrid_launch::accounts::Launch`:
  - `creator` (mut, signer)
  - `mint` (mut, **signer**, fresh, created by the program)
  - `launch_config` (init PDA `["launch_config", mint]`)
  - `mint_authority` (PDA `["mint_authority", launch_config]`)
  - `launch_destination_owner` (unchecked, caller-chosen, see QA-HL-02)
  - `launch_destination` (= classic ATA(owner, mint), created non-idempotently)
  - `token_program: Program<Token>`, `associated_token_program`, `system_program`
- Errors `LaunchError` 6000–6009: `InvalidDecimals`, `RatioNotAllowed`, `ZeroCollectionSize`, `CollectionTooLargeForSupply`, `FeeAboveCap`, `CaptureFeeBelowRerollFee`, `FeeDestinationNotBurn`, `InvalidLaunchDestination`, `MathOverflow`, `PostLaunchCheckFailed`.
- There's no separate `assert_launch_ready`/`open_sale`. The post-launch check is inside `launch` (step 5, ADR-010). A sale-open check belongs to the curve (N6).
- Tests:
  - Engineer: `tests/track-a-hybrid/launch/launch.rs`.
  - QA: `tests/track-a-hybrid/launch/qa_launch.rs`, report `qa/reports/2026-09-24-hybrid-launch.md`.

| ID | Case | Expected | Covered by |
|----|------|----------|------------|
| SUP-01 | Launch happy path, decimals 0…9 | supply == 1e9 × 10^d exactly (1e18 at d=9), mint & freeze authority None, classic Token owner, the whole supply in `launch_destination` (INV-01) | eng `happy_path…`; QA `qa_every_decimals_0_to_9…` ✅ |
| SUP-02 | Mint authority left set | impossible: `SetAuthority(MintTokens → None)` in the same tx, and the post-check fails with `PostLaunchCheckFailed` otherwise | eng happy path; QA INV-07 sweep (SetAuthority attempt fails) ✅ |
| SUP-03 | Freeze authority set | impossible: `InitializeMint2(freeze = None)`, post-checked | ✅ (every QA success asserts it) |
| SUP-04 | Supply ≠ S0 | impossible: exact `mint_to` + post-check; property test over random params ✅ | QA `qa_property…` ✅ |
| SUP-05 | **Track A rejects Token-2022:** Token-2022 mint (with or without extensions) or the Token-2022 program passed in | rejected (3008 / address in use); mint untouched, no `LaunchConfig` | eng `attack_existing_token_2022…`, T22 program test; QA `qa_real_token_2022_mints…` ✅ |
| SUP-06 | Upgrade authority is a single EOA instead of the Squads vault PDA | deploy-readiness script fails [AMT §5.2] | not started (no script yet) |
| SUP-07 | Attempt to mint after launch (any path) | impossible; INV-02 | QA INV-07 sweep ✅ (engine paths when they exist) |
| SUP-08 | Many re-rolls | ~~supply decreases by Σ fees~~ OBSOLETE v0.5: supply stays exactly S0 (INV-02) | engine |
| SUP-09 | Existing mint (classic with supply / foreign authority, Token-2022), mint == creator, mint not signing | rejected, nothing changes [QA-ANS Q11] | eng (set_account fakes, non-signer); QA real mints ✅ |
| SUP-10 | Wrong `token_program` / `associated_token_program` / `system_program` | 3008, nothing created | eng (T22); QA `qa_wrong_program_ids_rejected` ✅ |
| SUP-11 | `launch_destination` ≠ classic ATA(owner, mint) (other owner, Token-2022 derivation, PDA, mint) | `InvalidLaunchDestination` / fail atomically | eng (random pubkey); QA ✅ |
| SUP-12 | Pre-funded addresses (lamports sent to the mint / ATA / config / mint_authority before launch) | launch still succeeds | QA ✅ for ATA/config/authority; **❌ mint: QA-HL-01 (Low)** |
| SUP-13 | Relaunch of the same mint (any caller, any params); two launches in one tx; two launches by one creator | relaunch fails with no state change; two launches by one creator are fully isolated | eng relaunch; QA ✅ |
| SUP-14 | Payer too poor (needed − 1), exact budget | atomic failure / success; config and mint exactly rent-exempt | QA ✅ |
| SUP-15 | Compute | worst case (d=9, R=10k, N=100k) < 200k CU (measured ~62–68k) | QA ✅ |
| SUP-16 | Launch destination owner | **must be the curve/sale vault PDA or a capped, disclosed creator allocation (N2)**. Today it's caller-chosen; PDA owners work | QA `qa_observation…` documents it (QA-HL-02); flip to an assert when N2 lands |
| CFG-01 | Every allowed R at max N (`1B / R`) at decimals 0/6/9 | accepted, `max_tokens_in_nft_form == total_supply_base` | eng (10k only); QA all ✅ |
| CFG-02 | Every allowed R at max N + 1 | `CollectionTooLargeForSupply` | eng (1M, 10k); QA all ✅ |
| CFG-03 | R not in the allowed set: 0, 1, ±1 around each allowed value, 20k, 500k, 2M, 1B, 1B+1, u64::MAX−1, u64::MAX | `RatioNotAllowed` | eng (5 values); QA 22 values ✅ |
| CFG-04 | N = 0 | `ZeroCollectionSize` | eng; QA ✅ |
| CFG-05 | Overflow edges: `N × R_base` just below and just above the u64 overflow point at d=0 and d=9, N = u64::MAX; decimals 10…255 | clean 6003 / 6000, never wraps | eng (N=u64::MAX, d=10); QA ✅ |
| CFG-06 | Undersized collection (`N × R < 1B`) | accepted; UI copy "Up to {N×R} … at launch" (INV-16, [QA-ANS Q9]) | ✅ on-chain; UI not started |
| CFG-07 | Change or close `LaunchConfig` after launch | no instruction exists; random discriminators, IDL-namespace ixs, garbage `launch` data, and System/Token close/transfer/assign all leave it byte-identical (INV-07) | eng IDL test; QA sweep ✅ |
| CFG-08 | Burns push supply below `N × R_base` | releases stay exact; captures limited only by users' tokens; no insolvency (INV-04/06); copy shows live supply + burned [QA-ANS Q9] | engine, not started |
| CFG-09 | Fees: at cap (1000), cap+1, capture == reroll, capture < reroll, u16::MAX, `fee_destination` 1…255 | accept / `FeeAboveCap` / accept / `CaptureFeeBelowRerollFee` / `FeeAboveCap` / `FeeDestinationNotBurn`; fee amounts exact (`ratio_base × bps / 10000`) | eng (partial); QA ✅ |
| CFG-10 | PDAs: canonical bumps stored; non-canonical-bump PDAs for `launch_config` / `mint_authority` | stored bumps == `find_program_address`; non-canonical → `ConstraintSeeds` (INV-13) | eng (spoofed accounts); QA ✅ |
| CFG-11 | Property: random (ratio, N, decimals, fees, destination) | success iff every constraint holds; on success INV-01/07/13; on failure the error is one of the violated constraints and nothing persists | QA `qa_property…` (seeded, 400/run) ✅ |

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
| BA-01 | IDL review: `request_capture` has no asset/index argument and no caller-selectable asset account | pass (T-HV-01). This is the property MPL-Hybrid lacks (`capture_v2` takes the asset), which is why it's reference only |
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
| RR-02 | ~~Fee burned~~ OBSOLETE v0.5 → FEE-06/07 | re-roll moves no tokens; user pays exactly `sol_fee` to the fee wallet |
| RR-03 | Stale fulfilment: VRF account from an earlier or other request, wrong seed, foreign owner, unfulfilled | rejected (INV-09) |
| RR-04 | Duplicate fulfilment / double settle / reuse of a consumed VRF result | rejected; request closed after settle |
| RR-05 | Cancel or `expire` after fulfilment (the "saw the result, want out" path) | rejected (T-HV-02/04) |
| RR-06 | Abandon: requester never settles | anyone (crank) can settle; outcome identical; requester can't block it |
| RR-07 | Recipient griefing: requester makes delivery fail (closes accounts, changes wallet state) hoping to reach `expire` | settle still succeeds (Core asset owner set directly) or can't be turned into a refund; INV-09 |
| RR-08 | VRF never fulfils | `expire` after `deadline_slot` returns the NFT/principal; engineering default [QA-ANS cross-ref Q3, N7]: SOL fee refunded minus VRF cost, and the token fee (burned only at `settle`) refunded in full. Not before the deadline. _Doc conflict: hybrid-rarity §3.3 burns at request (report §5)._ |
| RR-09 | Pool has only the caller's own NFT / reservations exhausted | request rejected (T-HV-14) |
| RR-10 | ~~Fee change while pending~~ | **obsolete:** fees are fixed at init and read from `LaunchConfig` (ADR-009, [QA-ANS Q6]). Replaced by: the engine uses `LaunchConfig.capture_fee_amount` / `reroll_fee_amount` exactly, and a fake `LaunchConfig` (other program ID or seeds) is rejected [QA-ANS Q9] |
| RR-11 | `capture_fee ≥ reroll_fee` | enforced at launch (`CaptureFeeBelowRerollFee`, QA CFG-09 ✅); engine test: unwrap→rewrap is never cheaper than a re-roll; the capture fee is burned (N1) |
| RR-12 | Re-roll loop × 1k | INV-02/04/05/10 after each; burned total matches the counter |
| RR-13 | SOL part of the fee (VRF + rent) | covers VRF and lazy-mint rent; a zero-balance cranker can settle (T-HV-16); paid only to the VRF/rent payer, never an operator wallet (N7 open) |
| RR-14 | ~~Pause~~ **OBSOLETE v0.6 (pause removed, ADR-015)** | replaced by `qa_M26_no_pause_instruction_or_field_in_{launch,vault}`: no pause/guardian instruction or field exists |

---

## 7. Governance: multisig + timelock (INV-12, 13)

Per [AMT §2–5] and [QA-ANS Q6], there is **no in-program propose/execute** (fees are fixed). Governance is:
- Squads v4 (`SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf`), autonomous (`configAuthority = null`), with its vault PDA as the upgrade authority and `time_lock` proposed at 604,800 s (7 days).
- The optional `pause_new_requests`: guardian 2-of-3 with no timelock (option a), or governance (option b).
- Signers, threshold and timelock length need Barton (N5/Q8).

| ID | Case | Expected |
|----|------|----------|
| GOV-01 | `pause_new_requests` (if kept) called by an EOA, the creator key or a random key | rejected; only the configured multisig PDA |
| GOV-02 | Squads upgrade proposal executed at `time_lock − 1s`, exactly at `time_lock`, and after (localnet with Squads dumped from devnet) | fails / succeeds / succeeds |
| GOV-03 | Pause edges: `MAX_PAUSE_SLOTS − 1`, `MAX_PAUSE_SLOTS`; back-to-back renewal | paused / lapsed / renewal rejected [AMT §3] |
| GOV-04 | ~~Pause never blocks exits~~ **OBSOLETE v0.6** | no pause exists (M-26 no-pause tests) |
| GOV-05 | **No immediate path:** enumerate every instruction in each IDL and try to change each immutable field (R, N, mint, fees, fee destination, trait root, vault, VRF provider, open slot, curve params) | every attempt fails. `hybrid_launch`: ✅ (eng IDL test + QA sweep) |
| GOV-06 | ~~Pending requests during a fee change~~ | obsolete (fees immutable) |
| GOV-07 | Program upgrade authority | Squads vault PDA on mainnet (SUP-06); `hybrid_launch` is the first candidate for `--final` [AMT §5.4] |
| GOV-08 | Multisig config: members/threshold/`time_lock` match the documented set | launch-script assertion (after N5) |
| GOV-09 | Visibility: pending Squads proposals targeting our programs and an active pause are shown with their times | E2E-G01 |
| GOV-10 | Settings declared immutable in the docs | IDL has no setter: `hybrid_launch` ✅ (only `launch`) |

---

## 8. Bonding-curve launch window and anti-sniping (INV-15) _(prov.; venue and mechanism are Q4 / N6)_

Options and the engineering leaning are in [AMT §6]: Meteora DBC (`activation_point` + fee scheduler) or our own curve (a commit window with uniform clearing, then per-tx/per-slot caps). Any anti-snipe fee must burn or go to LP, never to an operator (ADR-009 C2). SN-xx stays provisional until Barton picks.

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

## 9. Generic program matrix (real names for `hybrid_launch`; engine names provisional)

Columns: **HP** happy path · **AC** access control/signer · **PDA** spoofing/wrong accounts · **PROG** wrong token program / wrong mint / fake Core or VRF program · **MATH** overflow/rounding · **CPI** ordering/reentrancy · **PAUSE** paused/closed ([QA-ANS Q3]) · **EDGE** zero/max. ✅ = covered at c02be68.

| Instruction | HP | AC | PDA | PROG | MATH | CPI | PAUSE | EDGE |
|-------------|----|----|-----|------|------|-----|-------|------|
| `hybrid_launch::launch` | SUP-01 ✅ | creator + fresh mint signer ✅ | config/mint_authority seeds + canonical bump ✅; destination ATA ✅ | classic Token only ✅; wrong ATA/System program ✅ | S0 / N×R checked math ✅ | init mint → mint_to → revoke → post-check ✅ | n/a (no admin) | CFG-01…11 ✅ |
| `hybrid_vault` init _(prov.)_ | reads `LaunchConfig` | creator/launch only; re-init fails | vault/config seeds; **fake `LaunchConfig` (other program/seeds) rejected** | Metaplex Core pinned; collection created by the program (T-HV-07) | n/a (values from `LaunchConfig`) | Core collection CPI with no Permanent plugins | n/a | — |
| `request_permutation_seed` / `store_seed` _(prov.)_ | seed stored once | anyone / pinned VRF | VRF account pinned | foreign VRF owner rejected | n/a | n/a | capture closed until the seed is stored | second store fails |
| `request_capture` _(prov.)_ | WR-01 | user signs | vault ATA / request PDA spoofing | classic Token only; wrong mint | R_base + fee overflow | tokens locked before anything else | **pausable** [QA-ANS Q3] | exact balance; empty pool (RR-09) |
| `settle` _(prov.)_ | WR-01, RR-01 | permissionless | request/VRF/asset PDA checks | pinned VRF + Core | index bounds | mint-on-exit CPI order; Merkle check before mint | **never pausable** | last NFT in pool |
| `release` _(prov.)_ | WR-02 | NFT owner only | fake asset / collection | fake Core program | exact R_base, no fee | NFT in before tokens out, atomic | **never pausable** | last NFT; after burns |
| `request_reroll` _(prov.)_ | RR-01 | NFT owner | asset from another collection | pinned programs | fee = `LaunchConfig.reroll_fee_amount` | hand-in before fee, atomic | **pausable** | only NFT in pool |
| `expire` _(prov.)_ | RR-08 | permissionless | request PDA | n/a | refund exact | n/a | **never pausable** | deadline ± 1 slot |
| ~~`pause_new_requests`~~ OBSOLETE v0.6 (pause removed) | — | — | — | — | — | — | — | — |
| curve `buy` / `sell` (if ours) | SN-xx | user | curve PDAs | classic Token | curve math overflow/rounding favours the pool | n/a | window/pause | SN-02…08 |

Close safety (INV-14): try to close every program-owned account while it holds tokens, NFTs, escrowed lamports or a pending request. All attempts must fail, and rent may go only to the recorded payer. `LaunchConfig` is never closable: ✅ QA sweep.

---

## 10. Fuzzing plan (Trident 0.12)

- `fuzz_config` can start now against `hybrid_launch` (until then, the seeded property test `qa_property_launch_succeeds_iff_all_constraints_hold` stands in). `fuzz_hybrid` starts once `hybrid_vault` is committed. Set up with `trident init`. Run `trident fuzz run <target> [seed] --with-exit-code` in CI; reproduce with `trident fuzz debug <target> <seed>`.
- The VRF is a **mock account owned by the pinned VRF program id**. The fuzzer controls fulfilment timing and value, including never fulfilling, fulfilling late, and fulfilling twice.

| Target | Instructions | Assertions after every instruction |
|--------|--------------|------------------------------------|
| `fuzz_hybrid` | request_capture, settle (random order attempts), release, request_reroll, expire, random VRF fulfilment, random donations to the vault | INV-02…06, 08, 09, 10, 14; out-of-order settle always fails; settle never finds an empty pool |
| `fuzz_config` | `hybrid_launch::launch` with random `LaunchParams` and account substitutions | INV-01, 07, 13; no panic, only clean errors; atomic failure |
| `fuzz_governance` | `pause_new_requests` (if kept) with random signers and slot warps, interleaved with user requests | INV-12, 13; exits never blocked; pause lapses |
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
7. MPL-Hybrid is reference only (Q1), so no MPL-Hybrid regression tests. QA-found issues use `QA-HL-xx` (launch) / `QA-HV-xx` (vault) IDs until merged into `security/merged/`. Known-failing tests are `#[ignore = "QA-…"]` so the shared suite stays green, and they run with `-- --ignored`.

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
| E2E-L02 | Post-launch token page, "What nobody can change" panel | values equal on-chain `LaunchConfig` (INV-16); copy "Up to {N×R} $TICKER could be held as NFTs at launch" plus a live line "Current supply: {supply} ({burned} burned by re-roll fees)" [QA-ANS Q9]; mint/freeze authority "revoked"; pause state if any |
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

## 13. `tests/` layout (actual, c02be68)

QA files are prefixed `qa_` and never edit the engineer's files. QA test binaries are registered through Cargo auto-discovery (`tests/track-a-hybrid/tests/*.rs` shims that `#[path]`-include the real file), so `./scripts/test.sh` (`anchor test --validator legacy` → `cargo test --workspace --locked`) runs them without changing the engineer's `Cargo.toml`.

```
tests/
├── .keys/                         # LOCALNET/DEVNET-ONLY throwaway keypairs (gitignored)
├── track-a-hybrid/                # crate `track-a-hybrid-tests` (engineer)
│   ├── Cargo.toml                 # engineer's; [[test]] launch = launch/launch.rs
│   ├── lib.rs
│   ├── launch/
│   │   ├── launch.rs              # engineer: 19 LiteSVM tests
│   │   └── qa_launch.rs           # QA: SUP-xx / CFG-xx (24 + 1 ignored finding)
│   ├── tests/
│   │   └── qa_launch.rs           # QA shim: #[path = "../launch/qa_launch.rs"] mod qa_launch;
│   └── vault/ (planned)           # engine tests; QA adds qa_wrap.rs, qa_blind.rs, qa_reroll.rs, qa_pause.rs
├── regression/ (planned)          # qa-sec_xxx (one file per merged finding ID)
├── fuzz/ (planned)                # Trident targets + regressions/
├── e2e/ (planned)                 # Playwright specs, wallet mock
└── _shelved/track-b/              # shelved Token-2022 QA notes
qa/reports/                        # dated QA run reports (first: 2026-09-24-hybrid-launch.md)
```

---

## 14. Open questions (status after `docs/qa-answers.md`)

| Q | Status | Answer / what's still open |
|---|---|---|
| **Q1 Swap engine** | ✅ **Resolved** (engineer, 2026-09-24 3:04 PM MT) | Custom `hybrid_vault` + Metaplex Core; MPL-Hybrid reference only; two-step request/settle via VRF; instant exact unwrap; token fees burned. _Docs still say "needs Barton" (report §5)._ |
| **Q2 Capture fee** | ✅ Resolved on-chain, pending Barton's confirmation | Token bps of R, ≤ 1000 bps, `capture ≥ reroll`, burned (N1) [QA-ANS cross-ref Q2]. Enforced and tested in `hybrid_launch` (CFG-09). |
| **Q3 SOL fee / expire** | ⏳ Narrowed, **needs Barton (N7)** | Default: SOL fee pays VRF + rent only, refunded minus VRF cost on `expire`; token fee burned at `settle`, so an expired request refunds it [QA-ANS cross-ref Q3]. Open conflict with hybrid-rarity §3.3 (burn at request). |
| **Q4 Bonding curve** | ⏳ **Needs Barton (N6)** | Options in [AMT §6] (Meteora DBC vs own curve with a commit window + caps). Blocks §8. Related: **N2** launch destination (QA-HL-02). |
| **Q5 VRF provider** | ⏳ Open (DECISIONS Q1) | Switchboard On-Demand vs ORAO; deadline and expire policy follow the provider. |
| **Q6 Multisig and timelock** | ⏳ Narrowed, **needs Barton (Q8/N5)** for signers/threshold/length | Mint + freeze authority None; `LaunchConfig` and engine config immutable (nobody); upgrade = Squads vault PDA, 7-day `time_lock` proposed, then `--final`; vault/collection authorities are program PDAs [QA-ANS Q6, AMT §4–5]. |
| **Q7 Pause** | ✅ Scope resolved; whether to keep it **needs Barton (N4)** | Only `request_capture` / `request_reroll` are pausable; `release` / `settle` / `expire` never; auto-expiring after `MAX_PAUSE_SLOTS`; `hybrid_launch` has no admin [QA-ANS Q3, AMT §3]. Tests: RR-14, GOV-01/03/04. |
| **Q8 Hybrid design details** | ⏳ Open | Lazy vs pre-mint (Q-H5), creator allocations (Q-H6, now also N2), public pool (Q-H7), two-tx UX (Q-H8). Max collection size is settled by `1B / R` (CFG-01). |
| **Q9 Supply copy after burns** | ✅ Resolved (engineering proposal) | "Up to {N×R} $TICKER could be held as NFTs at launch" plus a live "Current supply: {supply} ({burned} burned by re-roll fees)" [QA-ANS cross-ref Q9]. Test: E2E-L02. |
| **Q10 Decimals** | ⏳ Default **needs Barton (N3)** | The program accepts 0–9. QA covers all ten values plus 10…255 (CFG-05, SUP-01), so any choice is already tested. |
| **Q11 Vault donations** | ⏳ Open (engine) | Surplus (`≥` backing) vs reject; decide with `hybrid_vault` code. |
| **Mints: launchpad-created only?** | ✅ Resolved | Yes; enforced by a fresh signer mint created in `launch` [QA-ANS Q11]. **Track A rejects Token-2022.** |
| **Registry?** | ✅ Resolved | No registry program. The engine reads `LaunchConfig` by pinned program ID + seeds [QA-ANS Q9]. |

---

## 15. Coverage status (2026-09-24, c02be68)

Engineer: 4 unit + 19 LiteSVM tests, all green. QA: 24 LiteSVM tests green, plus 1 known-finding test (`--ignored`, fails: QA-HL-01). Details are in `qa/reports/2026-09-24-hybrid-launch.md`.

| Area | Section | Status |
|------|---------|--------|
| Launch & supply (SUP-01…16) | §3 | **mostly covered** (eng + QA); SUP-06 not started; SUP-08 needs the engine; SUP-12 ❌ QA-HL-01 (Low); SUP-16 open (QA-HL-02 / N2) |
| Ratio × size × fees config (CFG-01…11) | §3 | **covered** (eng + QA, incl. seeded property test); CFG-06 UI part and CFG-08 need the engine/UI |
| QA helpers (supply/authority, backing, NFT conservation, balance diff, mock VRF, IDL scanner) | §13 | launch helpers exist inside `qa_launch.rs` (`assert_post_launch`, oracle); move to shared helpers when the engine lands |
| Wrap/unwrap exactness (WR-01…10) | §4 | not started (engine not committed) |
| Blind assignment (BA-01…13) | §5 | not started (engine not committed; no longer blocked on Q1) |
| VRF re-roll (RR-01…14) | §6 | not started (engine; VRF provider Q5) |
| Governance (GOV-01…10) | §7 | GOV-05/10 ✅ for `hybrid_launch`; rest blocked on N4/N5 |
| Anti-sniping launch window (SN-01…14) | §8 | not started (blocked on Q4/N6) |
| Generic matrix + close safety | §9 | `hybrid_launch` row ✅; engine rows not started |
| Fuzz | §10 | not started (`fuzz_config` can start now) |
| Security regression tests | §11 | not started (security/merged is empty); QA-HL-01 test exists (ignored) |
| E2E desktop + mobile, usability / a11y | §12 | not started |

---

## 16. Regression tests for audit findings (v0.6: merged v2 IDs M-01..M-40)

Primary IDs = `security/merged/round1-merged.md` (M-xx); auditor IDs in brackets. Status lives in `qa/FINDINGS_TRACKER.md`.
Vault tests live in `tests/regression/` and run with `tests/regression/run-against-wip.sh [ref]` (hybrid_vault isn't on main;
default ref f4761af, the last wip commit whose harness builds). `L` = `tests/track-a-hybrid/launch/qa_launch.rs`,
`R` = `tests/regression/`. "Non-repro" = the finding exists only in stock MPL-Hybrid; the test asserts hybrid_vault doesn't
reproduce it.

### 16.1 Critical

| ID | Test (file::name) | Setup | Attack | Expected |
|---|---|---|---|---|
| **M-01** [A-01] non-repro | `R/qa_M01_A01_escrow_drain.rs::qa_M01_A01_no_instruction_can_change_ratio_fees_or_mint` | built vault IDL | look for any update/set/withdraw/close/migrate ix or a ratio/amount/fee/mint/destination arg | none exists (new ix → test fails until reviewed) |
| M-01 [A-01] | `…::qa_M01_A01_random_discriminators_cannot_mutate_vault` | sealed vault, 4 assets | 128 random discriminators + "amount"-like payloads with vault/pool/escrow accounts | every tx fails; vault account bytes identical |
| M-01 [A-01] | `…::qa_M01_A01_reinit_vault_rejected_and_state_unchanged` | 1 NFT captured | creator re-sends `init_vault` | fails; ratio/economics unchanged |
| M-01 [A-01] PoC 10 port | `…::qa_M01_A01_authority_capture_release_cycle_cannot_drain_backing` | alice holds 1 NFT; attacker (authority stand-in) holds supply | capture n−1 NFTs, release them all (PoC: amount 0 → capture → amount 3R → release) | each release = +ratio exactly; attacker net ≤ 0; alice's backing intact; escrow exact after each step |
| M-01 [A-01] | `…::qa_M01_A01_double_unwrap_rejected` | NFT captured + released | release it again | fails; balances unchanged |
| **M-01** [A-02] PoC 12 port | `R/qa_M01_A02_token_swap.rs::qa_M01_A02_capture_paid_in_junk_mint_rejected` | user holds a junk mint | request_capture with junk `user_token`, then junk `mint` + junk account | both fail; no request; junk untouched |
| M-01 [A-02] | `…::qa_M01_A02_spoofed_vault_tokens_or_fee_escrow_rejected` / `…_unwrap_from_spoofed_vault_tokens_rejected` | attacker-owned token account of the real mint | substitute vault_tokens / fee escrow in capture; vault_tokens in release | rejected; NFT and escrow unchanged |
| M-01 [A-02] | `…::qa_M01_A02_token2022_program_substitution_rejected` | — | Token-2022 in `token_program` | rejected |
| M-01 [A-02] | `…::qa_M01_A02_init_vault_cannot_rebind_mint_or_be_run_by_non_creator`, `…_rebind_on_unsealed_vault_by_non_creator_rejected` | vault exists | init_vault with junk mint; non-creator signs as creator | rejected; vault.mint unchanged. _Gap VAULT-INIT-01: front-run of the FIRST init_vault needs a launch-only harness helper._ |
| **M-02** [A-03, B-01] PoC 11 port | `R/qa_M02_A03_cherrypick.rs::qa_M02_A03_request_capture_takes_no_asset_or_index` | IDL | — | request_capture has no args and no asset/index account |
| M-02 [A-03] | `…::qa_M02_A03_settle_with_non_selected_asset_rejected` | pending capture, revealed | settle with every non-selected (rare) asset | each → `WrongAsset`; rares stay in escrow; correct settle succeeds |
| M-02 [A-03] | `…::qa_M02_A03_settle_to_different_recipient_rejected` | alice's request revealed | crank settles to bot | rejected; alice gets it |
| M-02 [A-03, A-12] | `…::qa_M02_A03_unwrap_of_vault_held_or_foreign_asset_rejected` | alice owns one NFT | bot releases each index | all fail; bot balance unchanged |
| M-02 [A-03] | `…::qa_M02_A03_reroll_settle_with_handed_in_or_non_selected_asset_rejected` | re-roll pending | settle with handed-in / other assets | rejected; pick ≠ handed-in |
| M-02 [B-01 predict-abort] | `…::qa_M02_A03_no_abort_after_reveal` | capture revealed | cancel (no ix), expire before and after deadline | all fail; settle succeeds |
| M-02 [A-04, A-05] | engineer `selection.rs` unit tests; REG-M02-5 (to add): 10k draws χ² over pool, never-selected index = none | — | — | uniform, full range (A-05 `max never selected` not reproduced) |
| **M-03** [A/B-05, B-03] | CURVE-01..08 (§8), planned on a Meteora DBC LiteSVM fixture | pool created at launch | same-slot bundle buys, pre-open buys, creator dev-buy bundle | fee-scheduler cliff fee applied; no buy before open; creator allocation per spec. **Depends on curve choice (DECISIONS, pending Barton).** |

### 16.2 High

| ID | Test | Setup → attack → expected |
|---|---|---|
| M-04 (expire: principal only, fee kept; FEE-17 `R/…::qa_M04_expire_refunds_principal_not_fee`, ign.) [B-02, A/B-02] | REG-M04-1 `attack_requester_withholds_reveal_gets_no_refund`, -2 `attack_foreign_randomness_account_rejected`, -3 `third_party_can_reveal_and_settle`, -4 `selective_expiry_cannot_shift_later_draw`, + B's `attack_settle_failure_cannot_trigger_recommit_after_reveal`, `recommit_capped_then_principal_only_expire` (merged-doc specs). Partial now: `qa_M02_A03_no_abort_after_reveal` | Requester never reveals → no fee refund, principal only after capped recommits; foreign/revealed randomness rejected; anyone can reveal. **Depends on VRF choice (Switchboard vs ORAO) and DECISIONS N7 (superseded).** |
| M-05 [F-06] | `L::qa_FEE05_M05_fee_wallet_fixed_at_launch_and_immutable` (ign.), `R/qa_M05_M06_M36_sol_fee.rs::qa_FEE06_M05_fee_destination_substitution_rejected_on_every_op` (ign.) | Replace fee wallet with user's wallet / random account / PDA / vault → rejected on capture, release, re-roll; real wallet +fee on honest op. |
| M-06 [F-03] | `R/…::qa_FEE07_M06_reroll_cost_le_release_plus_capture` (active) | Same user, same vault: lamports (incl. tx fees, rent deposits and refunds) and tokens for re-roll vs release+capture → re-roll ≤ both. |
| M-07 [F-01, B-05] | `L::qa_FEE01_M07_no_token_fee_and_no_fee_params` (ign.) | IDL: no bps / fee_destination / fee_amount; no fee ATA at launch; a SOL fee for capture, release, re-roll. |
| M-08 [F-08, B-10] | `L::qa_FEE03_M08_fee_hard_cap_is_exactly_0_01_sol` (ign.), `R/…::qa_FEE03_M08_vault_rejects_forged_config_off_tier_or_above_cap` (ign.); GOV-06 deploy check | cap+1 rejected; forged LaunchConfig above cap refused by vault; upgrade authority = Squads 3-of-5 + 7-day. **Cap value undefined (QA-FEE-03).** |
| M-09 [B-10, A-13] | GOV-06 (deploy-script, manual) | program data upgrade authority == Squads vault PDA with time_lock ≥ 7 days; fails otherwise. Needs Barton. |
| M-10 [A/B-06, B-09] | §18 GRAD-01..16; `R/qa_M10_M22_graduation.rs::qa_M10_GRAD02_*`, `qa_M10_GRAD05_*` (ign.) | see §18 |
| M-11 [A/B-01] | whole `tests/regression` suite + `qa_M11_GRAD01_*` (ign.) | vault exists on main with open_vault/mint_assets and passes every M test |
| M-12 [A-06, A-07] non-repro | `R/qa_M14_escrow_sequence.rs::qa_M14_INVESC_*`; REG-M12-B (to add): two launches, two vaults, release in vault B can't touch vault A's escrow | escrow exact; per-vault PDAs; no burn from escrow |
| M-13 [A/B-07, T-CURVE-01/09] | `L::qa_post_launch_immutability_sweep`, `qa_hl02_*`, decimals/supply tests | mint/freeze None; supply S0 in launch-vault PDA ATA; no withdraw path (verified 658fb95) |

### 16.3 Medium / Low / Info

| ID | Test | Expected |
|---|---|---|
| M-14 [F-07] | `R/qa_M14_escrow_sequence.rs::qa_M14_INVESC_escrow_exact_random_sequence_seed_{1,2,3}`, `R/…::qa_FEE02_M14_release_returns_exactly_ratio_no_token_deduction` | escrow == ratio×(outside+pending) after every random op; release = +ratio exactly |
| M-15 [F-05] | `qa_FEE08_M36_release_fee_never_blocks_release` case 1 (ign.) | closed / 0-lamport fee wallet never blocks capture/release/re-roll |
| M-16 [F-04] **accepted-risk** | `R/…::qa_M16_ACCEPTED_RISK_fee_wallet_rerolls_for_tx_cost_only` (documentation, always passes); FEE-11 policy | fee wallet never converts / re-rolls (published policy); accepted trust assumption |
| M-17 [F-09] | `L::qa_FEE05_M05_*` (ign.) | fee wallet fixed; no change path |
| M-18 [F-10] | `L::qa_FEE01_M07_*` (ign.) | no token fee → nothing accrues (fixed by decision) |
| M-19 [F-02, B-04, B-05] | `L::qa_FEE04_M19_zero_fee_behaviour` (ign.) | re-roll fee 0 rejected; **fee = 0 semantics undefined, needs Barton** |
| M-20 [B-06] | REG-M20-1 queue flood of unrevealed requests then recommit/expire clears head; REG-M20-2 cornering cost report per ratio | head-of-line never stuck > deadline; cost ≥ N×R |
| M-21 [B-11] | GRAD-12/13 | art bytes bound to commitment |
| M-22 [B-13] | v0.7 lazy: `R/qa_M22_lazy_mint.rs::qa_LAZY04_*` (ign.), GRAD-17 / SIZE-01 (ign.); crank tests obsolete | no crank; liveness moves to every settle (manifest availability + principal-refund expire) |
| M-23 [A/B-03, A/B-04] | engineer `pool.rs`/`merkle.rs` unit tests + INV-ESC sequences; Trident `fuzz_vault` (§10) | merge/reservation invariants hold under fuzz |
| M-24 [B-07] | E2E-ARB-01 | UI shows live floor vs ratio value; stale-listing warning |
| M-25 [T-CURVE-02/04/06] | CURVE-05..08 | curve math boundaries, slippage args honoured |
| M-26 [B-10(4)] | `L::qa_M26_no_pause_instruction_or_field_in_launch` (**active, passes main**), `R/…::qa_M26_no_pause_instruction_or_field_in_vault` (ign.; f4761af has pause) | no pause/guardian instruction or field anywhere (pause removed, ADR-015). `qa_M26_release_still_works_while_paused` OBSOLETE/deleted |
| M-36 [new v2] **resolved (fee removed, 5:04 PM)** | `R/…::qa_FEE08_M36_release_takes_no_fee_account` (ign.), `R/…::qa_FEE08_M36_release_with_payer_holding_only_tx_fee_succeeds` (**active, passes**) | release is free; nothing about any fee can block it |
| M-37 [new v2] | REG-M37 (planned) | VRF randomness rent / oracle reward paid by the requester; crank budget not drainable |
| M-38 [new v2] | REG-M38 (planned CI check) | release build has no mock-switchboard / test features; mainnet build refuses without fee recipient |
| M-39 [new v2] | process | audit a fixed commit |
| M-40 [new v2] | `R/qa_M14_escrow_sequence.rs::qa_M14_INVESC_*` (QA invariant includes pending re-rolls; passes f4761af) | engineer's invariant must count pending re-rolls |
| M-27 [B-10(5)] | REG-M27 | no allocation/mint-to-creator path in vault IDL; all NFTs enter the VRF pool |
| M-28 [A-09, A-10] non-repro | `qa_M01_A01_reinit_vault_*`, `qa_M01_A02_init_vault_*` | re-init rejected; CPI errors propagate |
| M-29 [F-11] | none (token fee removed; flat lamports) | fixed by decision |
| M-30 [F-12] | `L::qa_FEE04_M19_*` (ign.) | any non-zero SOL fee ≥ 890,880 lamports |
| M-31 [B-08] | E2E-MEV-01 | composite swap+convert has slippage bounds |
| M-32 [A-11, A-12] non-repro | `qa_M02_A03_settle_to_different_recipient_rejected`, `…unwrap_of_vault_held_or_foreign_asset_rejected`; `rg TODO programs/hybrid_vault` empty | typed signers; no TODOs |
| M-33 [A/B-08, B-12] | doc review each release (report "Doc conflicts") | no burn / 2% / refund language remains |
| M-34 [A-13] | none — truly N/A: product never invokes MPL-Hybrid (ADR-008) | — |
| M-35 [B-14] | REG-M35 devnet E2E with real Switchboard (or ORAO) | reveal by third party works; CU within budget |

QA-found: QA-HL-01/02 (verified 658fb95), QA-HV-01 (`qa_M10_GRAD02_*`), QA-HV-02, QA-FEE-01..03, QA-GRAD-01: see tracker.

---

## 17. Fees: tiered flat SOL fee on capture and re-roll; release free _(v0.7)_

Rule: **no token fee of any kind.** Capture and re-roll each charge ONE flat SOL fee, **tiered by ratio**:
50k = 0.002 SOL; 100k / 200k = 0.005 SOL; 500k / 1M / 2.5M / 5M = 0.01 SOL (10k dropped). **Re-roll fee == capture fee.**
**Release is free** (BRIEF 5:04 PM): no fee account, no holding account, no sweep; it returns exactly N tokens. The fee is fixed
per collection in `LaunchConfig` (derived from the ratio, never caller-supplied), immutable, **hard cap 0.01 SOL**, paid to
`LaunchConfig.fee_recipient` (program constant). VRF cost is paid by the requester at cost, separately from the tier fee.
Lazy minting adds a **first-mint cost** (rent + Core fee, ~0.0035–0.0067 SOL) to a capture/re-roll whose draw lands on an
unminted index (§21). **Fee = 0: still undefined in BRIEF.** Code: tiers and free release exist only in the uncommitted WT;
main 658fb95 is legacy bps + BURN; f4761af has no SOL fee.
L = `tests/track-a-hybrid/launch/qa_launch.rs`, R = `tests/regression/qa_M05_M06_M36_sol_fee.rs`.

| ID | Invariant / test | Status |
|---|---|---|
| **FEE-01** No token fee, no fee param (M-07, M-18, M-29) | LaunchParams has no fee field; LaunchConfig has `fee_lamports`, no bps / fee_destination / fee ATA. `L::qa_FEE01_M07_no_token_fee_and_no_fee_params` | ignored |
| **FEE-02** Release exact (INV-06, M-14) | release pays exactly R_base tokens, repeatedly. `R::qa_FEE02_M14_release_returns_exactly_ratio_no_token_deduction` | **active, passes (f4761af)** |
| **FEE-03** Hard cap 0.01 SOL (M-08, QA-FEE-03) | cap constant == 10,000,000 exactly (0.05 SOL = finding); every tier ≤ cap; no param can attempt cap+1; vault rejects forged LaunchConfig with 10,000,001, off-tier fee, or 0. `L::qa_FEE03_M08_fee_hard_cap_is_exactly_0_01_sol`, `R::qa_FEE03_M08_vault_rejects_forged_config_off_tier_or_above_cap` | ignored |
| **FEE-04** Fee = 0 (M-19, M-30) | spec silent (OPEN). Floor ≥ 890,880 lamports; no param. `L::qa_FEE04_M19_zero_fee_behaviour` | ignored; **needs Barton** |
| **FEE-05** Fee wallet fixed at launch (M-05, M-17) | program constant, recorded in LaunchConfig, spoof → ConstraintAddress / FeeRecipientInvalid; no change path. `L::qa_FEE05_M05_fee_wallet_fixed_at_launch_and_immutable` | ignored |
| **FEE-06** Fee destination substitution, vault side (M-05 / F-06) | capture and re-roll with a substituted destination → rejected, nothing moves. `R::qa_FEE06_M05_fee_destination_substitution_rejected_on_every_op` | ignored |
| **FEE-07** Re-roll ≤ release + capture, **per tier** (M-06) | release = tx fee only; re-roll f + tx ≤ release + capture 2tx + f, for all 7 ratios; lazy: both-minted and both-unminted cases hold, mixed case flagged QA-FEE-04. `R::qa_FEE07_M06_reroll_cost_le_release_plus_capture` (measured, f4761af, no fee: 5,000 vs 10,000), `R::qa_FEE07_M06_tier_model_reroll_le_release_plus_capture` (spec model, prints lazy worst case), `R::qa_FEE07_M06_per_tier_reroll_le_release_plus_capture_all_ratios` | 2 **active, pass**; per-tier on-chain ignored |
| **FEE-08** Release is free (M-36 resolved by removing the fee) | release takes no fee account and no sweep exists: `R::qa_FEE08_M36_release_takes_no_fee_account`; a payer holding only the 5,000-lamport tx fee releases and gets exactly N: `R::qa_FEE08_M36_release_with_payer_holding_only_tx_fee_succeeds` | 1 ignored (f4761af has token `fee_escrow`); 1 **active, passes** |
| **FEE-09** SOL totals, **per tier** | fee_recipient Δ == (captures + re-rolls) × tier fee exactly; releases 0; expired: fee kept. `R::qa_FEE09_per_tier_sol_fee_totals_equal_ops_times_tier_fee` | ignored |
| **FEE-10** Escrow property (INV-ESC, M-14, M-40) | escrow == ratio × (outside + pending captures + pending re-rolls) after random sequences. `R/qa_M14_escrow_sequence.rs::qa_M14_INVESC_*` | **active, passes** |
| **FEE-11** Insider policy (M-16, accepted-risk) | fee wallet never captures/re-rolls: policy + monitor; `R::qa_M16_ACCEPTED_RISK_fee_wallet_rerolls_for_tx_cost_only` (doc only) | manual / doc |
| **FEE-12** Tier set exactly per ratio (M-07) | every ratio × decimals 0/6/9 × N min/max → `fee_lamports == tier`; fee not caller-supplied (mismatch unrepresentable). `L::qa_FEE12_M07_tier_fee_set_exactly_for_every_ratio`; also asserted in every successful launch (`assert_post_launch`) | ignored |
| **FEE-13** Fee immutable after launch, **each tier** (M-08, M-01) | only `launch` in IDL; relaunch as every other tier fails; 64 random discriminators fail; config bytes unchanged. `L::qa_FEE13_M08_fee_immutable_after_launch_each_tier` | ignored |
| **FEE-14** Fee wallet type checked at launch (M-15) | PDA-like / program-owned / executable / system-with-data / token account → FeeRecipientInvalid; missing or funded system account OK. `L::qa_FEE14_M15_launch_rejects_pda_program_owned_or_executable_fee_wallet` | ignored |
| ~~**FEE-15** Sweep only to the fixed wallet~~ | **OBSOLETE v0.7** (release free; no sweep). Test deleted. | — |
| ~~**FEE-16** FeeVault exit only via sweep~~ | **OBSOLETE v0.7** (no holding account). Test deleted. | — |
| **FEE-17** Expire refunds principal, not fee (M-04 over N7) | N tokens / handed-in NFT + rent back; fee kept. `R::qa_M04_expire_refunds_principal_not_fee` | ignored (f4761af refunds the fee) |
| **FEE-18** Re-roll fee == capture fee; no release fee (M-06) | LaunchConfig has exactly one lamport fee field and no release fee: `L::qa_FEE18_single_fee_for_capture_and_reroll_none_for_release`; per tier on-chain capture Δ == re-roll Δ == tier, release Δ == 0: `R::qa_FEE18_reroll_fee_equals_capture_fee_per_tier` | ignored |

Obsolete: the v0.5 FEE-08 "paused vault" case and `qa_M26_release_still_works_while_paused` (pause removed); v0.6
FEE-15/16 and the FeeVault design (release is free, v0.7). QA-FEE-02 / M-36: resolved by removing the release fee.

---

## 18. Graduation (v0.7: LAZY MINTING)

Model: Barton 5:13 PM MT: **nothing is pre-minted at graduation.** Converting opens on graduation. Asset i is minted inside
settle the first time VRF picks it (§21). Curve = Meteora DBC (4:55 PM). _(D)_ = depends on an open Barton question.
Graduation-design §2.6 ("batch pre-mint") is superseded.

| ID | Invariant / test (setup → attack → expected) | Where |
|---|---|---|
| GRAD-01 | Converting closed before graduation via **every** entry point: request_capture, request_reroll, settle_*, merge → all rejected (`VaultNotOpen`). | `R/qa_M10_M22_graduation.rs::qa_M10_GRAD02_capture_rejected_before_graduation` (ign., QA-HV-01) |
| GRAD-02 | ~~`open_vault` requires minted_count == N~~ **OBSOLETE v0.7.** Now: the gate exists and takes a graduation proof (pinned owner, mint, graduated flag), does NOT require any minting, and the deposit path is gone; default build fails closed. | `R/…::qa_M11_GRAD01_graduation_gate_exists_and_deposit_path_removed` (ign.); LAZY-11 |
| GRAD-03 | ~~Batch mints resumable / idempotent~~ **OBSOLETE v0.7** (no batch mint). Idempotence moves to LAZY-01. | test deleted |
| GRAD-04 | ~~Crank griefing (asset-PDA pre-fund)~~ **OBSOLETE as a crank case**; the same attack at settle time is LAZY-06. | test deleted |
| GRAD-05 | Graduation can't be front-run or sandwiched (DAMM v2 pool creator = DBC pool authority; pinned owner / PDA / mint). | planned (D) |
| GRAD-06 | Pinned program IDs (DBC, DAMM v2, Core); permissionless migration, no privileged migrate key. | planned (D) |
| GRAD-07 | LP permanently locked, owned by our PDA; no LP withdraw; fee claims to the fixed destination. | planned (D) |
| GRAD-08 | ~~Mint cost only from the proceeds slice~~ **OBSOLETE v0.7**: the protocol pays 0 for minting; capturers pay (LAZY-03). | test deleted |
| GRAD-09 | ~~Mint budget covers N~~ **OBSOLETE v0.7**. | `R/qa_M22_lazy_mint.rs::qa_LAZY00_model_10k_collection_costs_protocol_nothing` (model, passes) |
| GRAD-10 | Supply accounting at graduation: curve reserves + locked 25% buffer + circulating == S0 (INV-02). | planned |
| GRAD-11 | One graduation per launch; converting opens once; no reverse path. | planned |
| GRAD-12 | Revealed art matches the pre-graduation commitment (image-bytes hash in the leaf, M-21), now checked in settle at first mint. | LAZY-05 |
| GRAD-13 | Art can't be swapped after trading starts: URIs immutable; off-chain verifier. | planned + verifier (D) |
| GRAD-14 | ~~Throughput of a 10k batch mint~~ **OBSOLETE v0.7**. Replaced by settle-with-mint CU / tx-size budget (fits one v0 tx with ALT): LAZY-08. | — |
| GRAD-15 | Tensor / Magic Eden: **only minted assets appear; the collection page is PARTIAL at graduation** and grows with first captures; rarity ranks move. Listing, buy, trait filters on minted assets work. | **E2E / manual** (LAZY-12) |
| GRAD-16 | Converting-open UI state equals on-chain `opened` flag; the UI explains the partial collection. | E2E |
| GRAD-17 | ~~Unfundable launch refused~~ **OBSOLETE v0.7** (affordability rule likely removed). Now: **a 10,000-NFT launch at the default 85 SOL is ACCEPTED** (50k and 100k ratios). | `L::qa_GRAD17_LAZY_10k_collection_at_default_85_sol_accepted` (ign.; WT still refuses) |
| SIZE-01 | N ≤ 10,000 still enforced for every ratio (10,001 → CollectionAboveCap / TooLargeForSupply). | `L::qa_SIZE01_collection_cap_10000_enforced_every_ratio` (ign.; main has no cap) |

---

## 19. Ratio and supply updates (v0.6)

- **v0.6:** 10k DROPPED (Barton 4:55 PM). Ratio set {50k, 100k, 200k, 500k, 1M, 2.5M, 5M}; N ∈ [100, min(1B/R, 10,000)].
  Ratio 10k must be rejected: `L::qa_RATIO_10k_dropped_launch_rejected` (ignored; main 658fb95 still accepts 10k). The
  generic ratio tests follow whichever set is built.


- Ratio set {10k, 50k, 100k, 200k, 500k, 1M, 2.5M, 5M}; `collection_size ≥ 100`; max = 1B/R (and ≤ `MAX_COLLECTION_SIZE` = 10,000 if the pending cap is committed — the tests read the constant from the IDL). Verified on main 658fb95: `L::qa_new_ratio_set_min_100_max_1b_over_r`, `qa_every_ratio_at_max_and_min_passes_and_max_plus_one_fails`, `qa_collection_size_zero_and_below_minimum_rejected`, `qa_ratio_outside_allowed_set_rejected`, property test.
- HL-02: destination = ATA of PDA `[launch_vault, mint, launch_config]`, no caller-chosen owner, no withdraw instruction: `qa_hl02_*` (3) verified on 658fb95.
- HL-01: pre-funded mint address tolerated; data/program-owned → `MintAccountInUse`: `qa_hl01_*` verified on 658fb95.
- (D) Pending Barton: drop 10k/50k ratios; 10,000 cap.

## 20. Coverage status (2026-09-25)

- main 658fb95: engineer 5 unit + 25 launch; QA `qa_launch` 24 pass, 5 ignored (SOL-fee model not committed).
- wip f4761af: QA regression 23 pass, 9 ignored (graduation gate, SOL-fee vault tests).
- Report: `qa/reports/2026-09-25-findings-regression.md`; mint cost: `qa/reports/2026-09-25-core-mint-cost.md`.

---

## 20a. Coverage status (audit-baseline-r1, 2026-09-25 ~6:05 PM MT)

- a1cfa8f: unit 14 + 13; engineer launch 27/27, vault 70/70; QA `qa_launch` 36 pass / 1 ignored (FEE-04; passes with
  `--include-ignored`); QA `qa_regression` 53 pass / 0 ignored (needs `--features qa-regression` until the engineer drops
  `required-features`). Report: `qa/reports/2026-09-25-audit-baseline-r1.md`.

---

## 21. Lazy minting invariants (v0.7, Barton 5:13 PM MT)

Source: `docs/lazy-mint-interface.md` (frozen for QA, 17:15 MT; ADR-016), graduation-design §2.5 / §4.4, THREAT_MODEL T-HV-16.
**Lazy code is in progress in the uncommitted WT only** (not committed; f4761af = creator deposit), so the on-chain tests are ignored. File: `R/qa_M22_lazy_mint.rs` (except where noted).

| ID | Invariant / test | Status |
|---|---|---|
| **LAZY-00** Protocol pays 0 | 10,000 NFTs: protocol 0 SOL; capturers pay 34.6–66.7 SOL total over first captures. `qa_LAZY00_model_10k_collection_costs_protocol_nothing` | **active (model), passes** |
| **LAZY-01** Mint exactly the picked index, once (M-02) | settle creates only asset i for the drawn i; two same-slot settles resolving to the same index → one mint, no duplicate; re-settle fails. `qa_LAZY01_M02_settle_mints_picked_index_once_even_same_slot` | ignored |
| **LAZY-02** minted ≤ N; minted index transferred, not re-minted (M-14) | `qa_LAZY02_M14_minted_count_le_n_and_minted_index_transferred_not_reminted` | ignored |
| **LAZY-03** Mint cost from request escrow, never the settler (T-HV-16, M-37) | settler holding only the tx fee settles a first-mint request and ends at 0 (tip 0); escrow → vault_authority → Core rent + 1,500,000; the rest goes to the user. `qa_LAZY03_M37_mint_cost_paid_from_request_escrow_not_settler` | ignored |
| **LAZY-04** Underfunded request; exact expire refund (M-04, M-22) | Rent raised → request_* fails `MintCostConstantStale`. Forged short request → settle fails atomically (`MintEscrowShort` / INV-4). **Expire refunds exactly:** ratio_base tokens (or the handed-in NFT) + request rent + rand_lock rent + mint_escrow_lamports (6,338,100, unspent). fee_recipient unchanged (tier fee kept). `qa_LAZY04_M22_underfunded_request_cannot_settle_or_strand_principal_exact_refund` | ignored |
| **LAZY-14** Minted pick: unused escrow back at settle (M-37) | pick already minted → transfer, no Core create; user gets request rent + rand_lock rent + full 6,338,100; settler 0. First-mint pick → user gets escrow − actual rent − Core fee (+ drained pre-fund). **Provisional** (interface §3; re-check when the code lands). `qa_LAZY14_M37_minted_pick_refunds_unused_mint_escrow_at_settle` | ignored |
| **LAZY-05** Asset is correct (M-21) | in vault.collection (update authority = vault_authority); owner = user (vault_authority after release / hand-in); name `#<i>`; uri == leaf.uri (content-addressed, ≤ 200); no plugins. Attributes are bound through the leaf, not on-chain. `qa_LAZY05_M21_minted_asset_in_collection_owned_named_as_committed` | ignored |
| **LAZY-13** Merkle leaf + proof verified; root immutable (M-21) | rejected atomically: wrong / short / long proof; valid leaf for another index; leaf from another launch; tampered uri / trait_values / salt / image_sha256 / json_sha256; proof against a different root; bad uri; missing MintArgs → `MintArgsMissing`. trait_root never changes after init (IDL enumeration, random discriminators, byte check across sequences; no re-init). `qa_LAZY13_M21_merkle_leaf_proof_verified_and_root_immutable` | ignored |
| **LAZY-06** No pre-create / griefing of asset address (T-GRAD-01) | pre-funded PDA still mints, attacker gains nothing; nobody else can allocate the PDA; a foreign asset can't stand in for i. `qa_LAZY06_asset_address_cannot_be_precreated_or_griefed` | ignored |
| **LAZY-07** Release of a lazy asset exact; recapture not re-minted (M-14) | `qa_LAZY07_M14_release_of_lazy_asset_exact_and_recapture_not_reminted` | ignored |
| **LAZY-08** Per-tier capture cost = tier fee + first-mint cost (or tier fee only); M-06 incl. first mint (M-06, QA-FEE-04) | `qa_LAZY08_M06_capture_cost_tier_fee_plus_first_mint_per_tier`; model in `qa_FEE07_M06_tier_model_*` prints the mixed case | ignored (model active) |
| **LAZY-09** Escrow backing with a virtual pool (M-14, M-40) | escrow == ratio × (outside + pending); minted + unminted == N; no double ownership. `qa_LAZY09_M14_escrow_backing_holds_with_virtual_pool` | ignored |
| **LAZY-10a** Pick uniform over ALL held indices, model (M-02) | QA replica of the v2 pool + the real `hybrid_vault::selection` functions. N = 24, 18 held (14 never-minted + 4 minted-and-returned), 60,000 seeded draws: chi-square 12.66 vs p=0.001 critical 40.93; returned group gets its share ±2%; non-held never drawn; a re-roll hand-in isn't drawable for its own request. `qa_LAZY10a_M02_model_pick_uniform_over_all_held_indices` | **active, passes** |
| **LAZY-10b** Same, on-chain sampling (M-02) | N = 16, 13 held; ≥ 3,200 settles from snapshots of the same state; chi-square p=0.001; the program's pick == QA replay for every sample. `qa_LAZY10b_M02_onchain_pick_uniform_over_all_held_indices` | ignored |
| **LAZY-11** No batch pre-mint path; open not gated on full mint (M-10) | IDL has no `mint_assets` / `crank_mint` / `deposit_asset`. `qa_LAZY11_M10_no_batch_premint_path_and_open_not_gated_on_full_mint`  `CollectionNotFullyMinted` must be Reserved*. | ignored |
| **LAZY-12** Marketplaces show only minted assets | Tensor / ME collection page is partial at graduation and grows; the site copy says so (GRAD-15/16). | **E2E / manual** |
| **GRAD-17 / SIZE-01** | 10,000 at 85 SOL accepted; N ≤ 10,000 enforced (L, §18) | ignored |

**QA-FEE-04: ACCEPTED (v0.8).** M-06 on average; per draw excluding the first-mint delta. Original flag: per-outcome M-06. Re-roll landing on an unminted index = f + tx + m. Release + capture
landing on a minted index = f + 2·tx. So the re-roll costs more by m − tx: 3,458,000–6,666,000 lamports in every tier. It holds in
expectation and whenever both draws land the same way. Options: accept (disclose), or make the first-mint cost a pooled
per-request surcharge charged equally on every capture and re-roll.


---

## 22. audit-baseline-r1 (v0.8)

Baseline: tag `audit-baseline-r1` = a1cfa8f; interface `docs/lazy-mint-interface.md` (frozen; mint escrow is its own PDA
["mint_escrow", vault, seq] in request_capture / request_reroll / settle_* / expire_request; vault_authority read-only in
settle); fix list `docs/audit-fixes-round1.md` (M-01..M-41).

**M-06 rule (QA-FEE-04 accepted).** Per draw, excluding the first-mint delta m: re-roll = f + tx <= release + capture =
f + 2tx (exact on-chain, all 7 tiers, all four minted/unminted combinations). On average: E[re-roll] - E[release +
capture] = m·U/(P(P+1)) - tx for P drawable indices, U never minted (the hand-in is not drawable for its own request; a
released NFT is). <= 0 iff P·tx >= m·U/(P+1); always for P >= m/tx (~613 at the measured m). Smaller pools: excess
printed, recorded as QA-FEE-05 (Info, disclosure).

**Mint escrow rules tested.** Request: user pays fee + tx + rents + 6,338,100 escrow. Settle, pick never minted: escrow
pays asset rent + Core fee (asset account lamports), remainder + rents to the user, settler tx only, vault_authority
unchanged, escrow PDA ends at 0. Settle, pick minted: full escrow + rents back, no Core create. Expire: principal + full
escrow + rents back; fee kept; caller tx only. Underfunded: Rent raised -> MintCostConstantStale at request; escrow PDA
forged short -> MintEscrowShort at settle, atomic.

**Not covered yet (QA):** M-41 exit-path upgrade safety (engineer tests only); pool-floor rejection (M-19; engineer tests
only); FIFO out-of-order settle and oracle heartbeat/selection (engineer tests only); K-per-call expire (not built);
real Switchboard reveal and VRF cost (M-37/M-38, devnet); DBC graduation verifier and 25% buffer (M-10, not built); M-03
(DBC mint); curve/MEV (M-25, M-31); v0 tx + ALT settle at N = 10,000 (depth-14 proof, ~1.2 KB legacy tx); fuzzing.
