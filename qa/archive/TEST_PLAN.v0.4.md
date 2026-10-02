# Launchpad QA Test Plan: SPL-404 hybrid launches

Status: **DRAFT v0.4** (2026-09-24). Owner: QA.
Networks: **localnet and devnet only.** Nothing in this plan deploys to mainnet or uses real funds or keys.

**Changelog**
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
| **INV-02** Supply only shrinks, only by burns | After launch, `mint.supply == S0 − Σ recorded re-roll burns` (and any other documented burn). No instruction ever mints. |
| **INV-03** Exact per-instruction token deltas | capture: user −(R_base + token fee), vault +R_base, fee handled per its policy (Q2). release: vault −R_base, user +R_base. reroll: user −fee (burned), vault ±0. No other token balance changes. |
| **INV-04** Escrow backing | `vault_token_balance == R_base × nfts_outside_vault`. If unsolicited donations are possible, it's `≥`, with the donation surplus tracked separately (Q11). |
| **INV-05** NFT conservation | `pool + incoming + nfts_outside_vault + pending_reroll_handins == collection_size`, counting unminted indices as pool. No NFT is created beyond N, and none is lost. |
| **INV-06** Exact unwrap | `release` always pays exactly `R_base`: no fee, no rounding, independent of fees, burns, pool state or pending requests. |
| **INV-07** Supply cap at config | `LaunchConfig`: `ratio_whole_tokens ∈ {10k, 50k, 100k, 200k, 1M}`, `collection_size ≥ 1`, `max_tokens_in_nft_form = collection_size × ratio_base ≤ total_supply_base` (checked_mul, overflow fails cleanly), `decimals ≤ 9`, fees ≤ 1000 bps with `capture ≥ reroll`, `fee_destination == BURN`. **`LaunchConfig` can't be updated or closed by anyone** (no instruction exists) [QA-ANS Q6]. The engine's config is also fixed at init (ADR-009 C3). |
| **INV-08** Blind assignment | The recipient never chooses the NFT. The pick depends only on a VRF value produced **after** payment is locked, and on a candidate pool fixed at request time (deposits with `seq < s`). |
| **INV-09** Single-use randomness | Each request consumes exactly one fulfilled VRF result from the account pinned at request time, exactly once. There's no cancel or refund after fulfilment, and `expire` works only if the request is unfulfilled past `deadline_slot`. |
| **INV-10** Re-roll fee burned | A re-roll lowers `mint.supply` by exactly the token fee. No token account's balance goes up because of it, and the vault is untouched. |
| **INV-11** Fee destinations public and fixed | Token fees (capture and re-roll) are **burned** (`fee_destination = BURN`, N1). The SOL cost fee pays only VRF + rent (N7, open). No instruction takes a free destination argument, and no operator wallet ever custodies user value. _Known gap: `launch_destination_owner` is caller-chosen today (QA-HL-02, N2)._ |
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
| SUP-08 | Many re-rolls | supply decreases by exactly Σ fees; recorded burn counter matches (INV-02, INV-10) | engine, not started |
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
| RR-02 | Fee burned | `mint.supply` −fee exactly; no account's balance goes up; vault unchanged (INV-10) |
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
| RR-14 | Pause (if kept, N4) | while paused, `request_capture` / `request_reroll` fail and `release` / `settle` / `expire` succeed; the pause lapses at `MAX_PAUSE_SLOTS`; can't be renewed back-to-back without a timelocked proposal [QA-ANS Q3, AMT §3] |

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
| GOV-04 | Pause never blocks exits | `release`, `settle`, `expire` succeed while paused (RR-14) |
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
| `pause_new_requests` _(prov., only if N4 keeps it)_ | RR-14 | multisig PDA only | config PDA | n/a | auto-expiry math | n/a | n/a | GOV-03 |
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
