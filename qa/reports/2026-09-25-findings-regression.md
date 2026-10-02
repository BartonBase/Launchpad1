# QA report: audit findings as regression tests, fees, graduation, ratios (2026-09-25)

Written 2026-09-25 ~4:58 PM MT. Localnet / LiteSVM only; no mainnet transactions and no real keys (one read-only
`solana program dump` of Metaplex Core from 9/24). Nothing under `programs/` or the engineer's own test files was edited.

## Code state
- **main = 658fb95** (committed 16:30 MT): the QA-HL-01 and QA-HL-02 fixes, the new ratio set and min 100. The fee model on main is still the legacy one (token bps + BURN).
- **hybrid_vault is not on main.**
  - `wip/hybrid-vault` **f4761af** builds, and its tests pass (4 + 7 unit, 19 + 1 LiteSVM).
  - The branch was rebased to **977f8f2** on top of 658fb95. Its vault harness doesn't compile against main (it still uses `launch_destination_owner`), which is logged as QA-HV-02.
- **The engineer's working tree is uncommitted and changed several times during this run** (16:31–16:49). In order:
  - It first added a 2% token fee with `fee_bps` as a launch param.
  - It then switched to a fixed `FEE_BPS = 200` constant, `MAX_COLLECTION_SIZE = 10_000` and a capture SOL fee in [0, 0.05].
  - On the vault side it added `mint_assets` / `open_vault` / a fail-closed `graduation::verify`, its own randomness instructions, and removed `expire`.
  - The whole token-fee part is superseded by BRIEF 4:49 PM (SOL-only fee).

## Findings inventory
| Source | Critical | High | Medium | Low | Info | Total |
|---|---|---|---|---|---|---|
| Auditor A round 1 (13 code + 8 design) | 3 | 7 | 8 | 2 | 1 | 21 |
| Auditor B round 1 | 1 | 5 | 5 | 2 | 1 | 14 |
| Merged (M-01..M-35, primary IDs) | 3 | 10 | 15 | 5 | 2 | 35 |
| QA (new or kept) | 0 | 2 (QA-HL-02, QA-HV-01) | 4 | 1 | 1 | 8 |

Tracker (`qa/FINDINGS_TRACKER.md`, 43 rows): **open 31, fixed 3, verified 9**.
- **Verified:** M-01, M-02, M-13, M-14 (token side), M-28, M-32, M-34 (N/A), QA-HL-01, QA-HL-02.
- **Fixed (not yet test-verified):**
  - M-12: fixed by construction; the two-vault isolation test is still to be written.
  - M-18 and M-29: fixed by the "no token fee" decision.

## Ported PoCs (tests/regression, run against f4761af with `run-against-wip.sh`)
Result: **23 passed, 0 failed, 9 ignored.**

| PoC | Merged ID | Tests | Result |
|---|---|---|---|
| 10_escrow_drain (A-01) | M-01 | `qa_M01_A01_*` (5) | **Blocked.** The IDL has no instruction that edits economics. 128 random discriminators leave the vault bytes unchanged. Re-running init_vault fails. A full capture/release cycle by an authority stand-in nets ≤ 0, and each release pays exactly the ratio. A double release fails. |
| 12_token_swap (A-02) | M-01 | `qa_M01_A02_*` (6) | **Blocked.** A junk user_token, a junk mint, spoofed vault_tokens / fee escrow (on capture and release), a Token-2022 program swap, an init_vault rebind with a junk mint, and init_vault by a non-creator are all rejected. |
| 11_cherrypick (A-03) | M-02 | `qa_M02_A03_*` (6) | **Blocked.** request_capture takes no asset. Settling with any non-selected asset fails with `WrongAsset`. A crank can't redirect the NFT to another recipient. Foreign or vault-held release fails. A re-roll can't return the handed-in or a non-selected asset. There's no abort after reveal. |

No exploit works against f4761af.

Limits of the port:
- The randomness is a TEST-ONLY mock Switchboard, so M-04 (selective reveal) and M-35 remain open.
- A front-run of the *first* init_vault is not covered: the harness always runs init in setup.

## Fees: SOL-only model (BRIEF 4:49 PM)
- All 2% token-fee tests written earlier today were deleted, and the changelog lists them.
- `qa_launch.rs` no longer asserts any token-fee values.

Active and passing on wip:
- `qa_FEE02_M14_release_returns_exactly_ratio_no_token_deduction`
- `qa_FEE07_M06_reroll_cost_le_release_plus_capture`. On wip, a re-roll costs 5,000 lamports and release + capture costs 10,000 lamports; tokens are equal. Wip has no SOL fee yet.
- `qa_M26_release_still_works_while_paused`
- 3 × `qa_M14_INVESC_escrow_exact_random_sequence` (200 random ops)

Ignored until the SOL-fee model is committed:
- Launch side: FEE01 (no token fee), FEE03 (cap), FEE04 (fee = 0), FEE05 (fee wallet fixed).
- Vault side: FEE03, FEE05 (substitution), FEE06 (totals = ops × fee), FEE08 (release fee never blocks release).

## Graduation
- Invariants GRAD-01..16 are in TEST_PLAN §18.
- 5 compiled tests are ignored: `qa_M11_GRAD01`, `qa_M10_GRAD02`, `qa_M22_GRAD03`, `qa_M22_GRAD04`, `qa_M10_GRAD05`.
- **New finding QA-HV-01 (High):** the committed hybrid_vault opens captures as soon as the creator has deposited every asset, with no graduation gate. A fix is in progress in the uncommitted tree.

## Ratios and HL-02
All of these are un-ignored and pass on main 658fb95:
- the HL-01 test
- the three HL-02 tests (launch-vault PDA destination, caller-supplied owner rejected, no withdraw path)
- the new ratio set, min 100 and max 1B/R

The pending 10k cap is read from the IDL automatically.

## Core mint cost
See `qa/reports/2026-09-25-core-mint-cost.md`.
- Per asset, the cost is 0.0035–0.0067 SOL. Of that, 0.0015 SOL is the Metaplex Core create fee; the rest is rent.
- Each create uses 12k–29k CU. 1–4 assets fit in a tx, limited by the 1,232-byte packet size.
- This agrees with graduation-design §1 within about 4%. The older 0.0016 SOL estimate in hybrid-rarity §Q-H5 is stale (QA-GRAD-01).

## Suite results
- **Clean `git archive main` (658fb95) + QA files, `anchor build` + `cargo test --workspace --locked`: 54 passed, 0 failed, 5 ignored.**
  - hybrid_launch unit: 5 passed
  - engineer `launch.rs`: 25 passed
  - QA `qa_launch.rs`: 24 passed, 5 ignored
- `./scripts/test.sh` was not used, because it tests the engineer's live, uncommitted tree.
- **Regression suite on wip f4761af:** 23 passed, 0 failed, 9 ignored.
- The engineer's wip suite at f4761af passed earlier (31 tests).
- The intermediate working tree's hybrid_launch was built separately (`cargo build-sbf`). `anchor build` failed in that copy with a missing platform-tools v1.57 toolchain, which was not investigated. All `qa_launch` tests passed against it except one CU-headroom assert (109k CU); that threshold has since been relaxed.

## Doc conflicts and blockers
1. **The fee design changed twice today.** Several places still describe burned or escrowed token fees, a 2% token fee, or a refund on expire (M-33):
   - BRIEF lines 39–57 and 62: burn, and "re-roll burns can only reduce" supply.
   - BRIEF lines 71–76: the 4:27 PM 2% token fee.
   - DECISIONS N1 and N7: burn at settle, refund on expire.
   - qa-answers Q3.
   - hybrid-rarity lines 22, 186–187, 207–208, 253, 257 and 356.
   - The working-tree code: `FEE_BPS`, the fee ATA, and the vault's `user_pays` token fee.
2. **SOL-fee hard cap value is undefined** in BRIEF and DECISIONS (QA-FEE-03). The code's 0.05 SOL cap comes from the superseded design. The working tree also has a separate re-roll minimum of 0.001 SOL and a capture fee range of [0, 0.05], which doesn't match "flat 0.01 on each".
3. **"Charge a SOL fee on every release" vs "the release fee must never block release" (M-26)** (QA-FEE-02). A holder with less than 0.01 SOL can't release. Barton needs to decide whether to waive the fee, defer it, or accept this.
4. **Fee = 0 semantics are undefined.** QA assumes a re-roll fee of 0 is rejected (anti-grind floor, M-19) and that any non-zero fee is ≥ 890,880 lamports (M-30).
5. **Expiry refunds:** merged M-04 says fees are paid at request and never refunded, and expire returns principal only after capped recommits. DECISIONS N7 says refund on expire. The committed wip refunds on expire; the working tree removed expire and has uncapped recommit.
6. **The escrow rule has no conflict** with the SOL-only fee: fees never touch token escrow, and supply is fixed at 1B with no burns.
7. **M-16:** with SOL fees, the fee wallet still re-rolls for only the tx fee. This relies on the published policy.
8. **Error-code slots** 6005 and 6006 were re-purposed in the working tree, and 6005 is now "reserved" (QA-FEE-01, Info).
9. **Graduation decisions still open:** curve (DBC recommended), proceeds-slice size, residue destination, 10k cap and ratio drop, art-key custody. GRAD-05..10 and 12–14 wait on these.

## Addendum (~5:15 PM MT): merged v2 IDs, tiered fee, 10k drop, pause removal

- **IDs:** merged v2 (final) = M-01..M-40. M-01..M-35 keep their numbers and subjects, and M-36..M-40 are new. "Release fee never blocks release" moved M-26 → M-36. The tracker has a v1-ID column; the old tracker is at `qa/archive/FINDINGS_TRACKER.v1-ids.md`.
- **Tiers:** implemented only in the engineer's uncommitted WT (`needs_barton.rs` FEE_TIERS; fee derived from the ratio; `MAX_FEE_LAMPORTS` = 0.01 SOL; FeeVault + `sweep_fees`; pause removed; principal-only expire). Main 658fb95 = bps + BURN; f4761af = no SOL fee, has pause.
- **Runs:** main export 55 passed / 0 failed / 10 ignored. Regression on f4761af: 24 / 0 / 14.
- **WT qa_launch run:** not done. The engineer's WT `tests/track-a-hybrid/Cargo.toml` sets `autotests = false`, which drops QA's shim ("targets the pre-M-07 LaunchParams; excluded until QA rewrites it"). QA's rewrite now handles the WT schema. A run in an isolated copy with the target re-added was aborted.
- **Tracker:** 48 rows. Open 33, fixed 3, verified 9, accepted-risk 1 (M-16), resolved-by-design (pending test) 2 (M-36, QA-FEE-02).
