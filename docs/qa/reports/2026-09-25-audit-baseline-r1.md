# QA report: audit-baseline-r1 (a1cfa8f), 2026-09-25 ~6:05 PM MT

Baseline: tag `audit-baseline-r1` = a1cfa8f on local branch `wip/hybrid-vault` (not pushed; lightweight tag). Localnet /
LiteSVM only; no mainnet, no real keys. Test plan v0.8 (§22), tracker v0.8.

## How it was run (build hygiene)

- Detached QA worktree `/workspace/launchpad-qa` at the tag. The engineer's tree (`/workspace/launchpad`) was not touched
  except for QA-owned files synced back (tests/regression/*, launch/qa_launch.rs, qa/*). Their uncommitted harness.rs /
  Cargo.toml work is additive and compatible with the ported QA files.
- `CARGO_TARGET_DIR=/workspace/launchpad/qa/.target` for every build and test. SBF via `cargo build-sbf --tools-version v1.54
  --skip-tools-install --arch v2` (repo-pinned platform-tools, already installed; nothing installed or switched). Rust
  1.89.0 is picked per directory by the repo's rust-toolchain.toml. `rustup default` is still
  `stable-x86_64-unknown-linux-gnu`; `~/.solana-toolchain-env.sh` and shared cargo/solana config weren't touched.
- Scripts: `qa/scripts/qa-build.sh` (artifacts + IDLs; production .so checked for the mock marker), `qa/scripts/qa-run.sh [ref]`.

## Results at a1cfa8f

| Suite | Pass | Fail | Ignored |
|---|---|---|---|
| hybrid_launch unit | 14 | 0 | 0 |
| hybrid_vault unit | 13 | 0 | 0 |
| engineer `launch` | 27 | 0 | 0 |
| engineer `vault` | 70 | 0 | 0 (*) |
| QA `qa_launch` | 36 | 0 | 1 (FEE-04; passes with `--include-ignored`: 37/0/0) |
| QA `qa_regression` (`--features qa-regression`) | 53 | 0 | 0 |

(*) The first run gave 66/4: four engineer tests read `CARGO_MANIFEST_DIR/../../target/idl/hybrid_vault.json` and ignore
`CARGO_TARGET_DIR` (QA-HYG-01). With a worktree-local copy of the QA-built IDL: 70/70, matching the engineer.

**Real program bugs found: none.** Every failure was a stale QA test or a harness issue, and all were fixed:

- `qa_launch::qa_two_launches_are_isolated_and_same_mint_twice_fails`: used the dropped 10k ratio (RatioNotAllowed). Now
  ratio 50,000, N = 10,000.
- `qa_launch::qa_property_random_params_match_oracle_for_built_version`: the oracle modelled the dropped affordability
  rule. Now ratio in set, 100 <= N <= 10,000, ratio x N <= 1B (the affordability check applies only while the IDL has an
  active `GraduationUnfundable`; at r1 it is `ReservedGraduationUnfundable`).
- `qa_launch::qa_FEE14_*` (newly un-ignored): LiteSVM refuses to store a non-ELF executable account
  (InvalidAccountData). The executable case now clones a real program account. The program check itself passes.
- Engineer `vault` 4 tests: QA-HYG-01 above (no program issue).

Un-ignored, now passing: qa_launch FEE01, FEE03, FEE05, FEE12, FEE13, FEE14, FEE18, RATIO_10k, GRAD17 (10k at 85 SOL),
SIZE01, planned property; regression FEE06, FEE07 per tier, FEE08 (no fee account), FEE18, FEE09, FEE03 (forged
config), M04 expire, M26 no pause, GRAD01, GRAD02, LAZY-01..14.

## Regression port (tests/regression)

Ported from the f4761af harness to the r1 harness (old files in `qa/archive/regression-f4761af/`), with the frozen
interface, including the mint-escrow PDA. 53 tests: A01 5, A02 6, A03 6, M14 sequence 3, fee 14, graduation 3, lazy 16.
`qa_M22_lazy_mint.rs` is included from `qa_M10_M22_graduation.rs`, so the engineer's root runs it without an edit.

**Needs an engineer edit (QA can't change their files):** `tests/track-a-hybrid/Cargo.toml` still has
`required-features = ["qa-regression"]` on `qa_regression`. Remove it and the suite runs by default. Their comment says
it's opt-in "until QA ports it", and it's ported now.

Checks implemented against the interface:

- Request: the user pays fee + tx + request/rand_lock rent + 6,338,100 into the mint-escrow PDA. INV-4 is checked while
  pending.
- Settle, first mint:
  - the escrow pays the asset account (rent + Core fee = 3,066,000 measured);
  - the rest plus rents goes to the user;
  - the settler pays only the tx fee (a settler funded with exactly 5,000 ends at 0);
  - vault_authority is unchanged and the escrow ends at 0.
- Settle, already-minted pick: the full escrow and rents are refunded and no Core create appears in the logs.
- Expire: principal (tokens or the handed-in NFT) + the full escrow + rents come back; the fee is kept; the caller pays
  only the tx fee.
- Underfunded:
  - with Rent raised, the request fails with MintCostConstantStale;
  - with the escrow PDA forged short, settle fails with MintEscrowShort and changes nothing.
- Leaf/proof: 12 tamper cases are rejected atomically, MintArgsMissing is enforced and trait_root is immutable.
- Asset: in the vault collection, UA = Collection, name `#i`, uri from the leaf, no plugins. Collection counters equal
  minted_count.
- Forced re-mint (bitmap bit cleared) is rejected with AssetStateMismatch (6023).
- Pre-funded asset PDA: the lamports go to the user. Lookalike Core accounts are rejected.
- Two settles in one tx work, one mint per index.
- On-chain pick == replay (300 samples); chi-square over the 97 held indices passes (94.4 < 144.6).
- Exact per-op lamport model in random sequences (captures, re-rolls, releases, expires) at every tier; fee_recipient
  delta == fee ops x tier fee.

## M-06 / QA-FEE-04 (accepted)

Per draw, excluding the first-mint delta, this holds exactly on-chain in all 7 tiers and all four minted/unminted
combinations: re-roll = f + tx and release + capture = f + 2tx. Example at the 1M tier: 10,005,000 vs 10,010,000; with
first mints, 13,071,000 vs 13,076,000.

The average is modelled exactly: E[A] - E[B] = m·U/(P(P+1)) - tx. This is <= 0 for pools of P >= m/tx ≈ 613 drawable
indices. For smaller pools a re-roll costs on average up to ~25,000 lamports more at P = 100 (m = 3,066,000). The cause:
the hand-in isn't drawable for its own request, while a released NFT is. This is new finding **QA-FEE-05 (Info)**, a
disclosure-wording matter only, not asserted as a failure.

## Open QA items re-checked

| ID | Result |
|---|---|
| QA-HV-01 (no graduation gate) | **Verified fixed.** Capture before open -> VaultNotOpen. 5 bad proofs + a malformed one -> GraduationNotVerified. Second open -> VaultAlreadyOpen. The production .so (no mock marker) fails closed with GraduationCheckUnavailable. |
| QA-HV-02 (vault tests don't compile) | **Moot / verified.** Vault suite compiles, 70/70. |
| QA-FEE-01 (6005/6006 reused) | **Verified.** hybrid_launch codes are append-only. Retired slots are `Reserved*` (6004 ReservedFeeAboveCap, 6005 ReservedCaptureFeeBelowRerollFee, 6006 ReservedFeeDestinationNotBurn, 6015 ReservedGraduationUnfundable), never reused. Same pattern in hybrid_vault (6006-6009, 6022, 6024-6026, 6028, 6030, 6035, 6049, 6052). |
| QA-FEE-03 (0 fee undefined) | **Still open (needs Barton).** Unreachable through launch (floor 0.002 SOL). The vault charges min(stored, tier, cap), so a forged 0 config would make requests free (only possible by injecting account bytes). |
| QA-GRAD-01 (mint estimate) | **Still open.** hybrid-rarity §cost says ~0.0016 SOL; the banner says "actual ≈ 0.00507". Measured at r1: 0.003066 SOL per lazily minted asset. |
| Doc drift (burn / 2%) | **Partly fixed; new QA-DOC-01 (Low).** Fixed: hybrid-rarity has a "current model" banner; DECISIONS ADR-009 D, ADR-011, N1, N7 are marked superseded. Still unmarked: DECISIONS ADR-008 §4 (token bps fee "burned"; sizes up to 100,000) and ADR-010 §6 / validation (fee bps, `fee_destination = BURN`, 10k ratio). Also: hybrid-rarity cost bullets (0.0016 SOL, 100,000-NFT pool); launch test name `fee_tier_is_derived_from_ratio_for_all_8_ratios_*` checks 7. `docs/BRIEF.md` is untracked (not in the baseline); it is a chronological log whose burn/2% lines (50-62, 71-74) are superseded only by later entries, which is acceptable for a log. audit-fixes-round1 marks M-33 closed; QA keeps it open until the DECISIONS markers are added. |

## M-xx status (QA view at r1)

- **Verified (QA test passes):** M-01, M-02, M-05, M-06, M-07, M-12, M-13, M-14, M-15, M-22, M-26, M-28, M-29, M-30,
  M-32, M-34, M-36, M-39, M-40; M-08 in code (disclosure: Barton).
- **Partially verified:**
  - M-04: expire rule verified; oracle heartbeat/selection only in engineer tests; K-per-call expire and devnet reveal
    open.
  - M-21: on-chain verified; off-chain image verifier not built.
  - M-23: uniform pick and hand-in exclusion verified; FIFO order only in engineer tests.
  - M-37: settler pays nothing verified; VRF cost blocked.
  - M-38: production fail-closed and no mock marker verified; real Switchboard reveal unproven.
- **Blocked:** M-10. The gate is verified with the mock; the DBC verifier and 25% buffer aren't built.
- **Fixed, no QA test:** M-41 (engineer m41_* tests pass). M-18 is superseded.
- **Open:**
  - M-03 (launch blocker);
  - M-09, M-17, M-27 (Barton);
  - M-11 (unaudited);
  - M-19 (pool floor: no QA rejection test);
  - M-20, M-24, M-25, M-31, M-35;
  - M-33 (see QA-DOC-01).
- **Accepted:** M-16 (accepted-risk), QA-FEE-04.

## Not covered yet

- M-41 upgrade safety: forged LaunchConfig with appended fields or a version bump on release / settle / expire.
- Pool-floor rejection (M-19).
- FIFO out-of-order settle.
- Oracle heartbeat / selection attacks. Engineer tests cover these three; QA has no independent tests yet.
- K-per-call expire (not built).
- Real Switchboard reveal and VRF cost on devnet. The engineer is adding `vault/real_switchboard.rs` (uncommitted, not
  run by QA).
- DBC graduation verifier and buffer (M-10); M-03; curve / MEV.
- Settle-with-mint at N = 10,000 (depth-14 proof) as a v0 tx with an ALT. QA tests use N = 100.
- Fuzzing.

## Files

- `tests/regression/*` (ported), `tests/track-a-hybrid/launch/qa_launch.rs` (stale tests fixed, 11 un-ignored).
- `docs/qa/FINDINGS_TRACKER.md` v0.8, `docs/qa/TEST_PLAN.md` v0.8 (archives: `qa/archive/FINDINGS_TRACKER.v0.7.md`,
  `TEST_PLAN v0.7.1 (git history)`, `qa/archive/regression-f4761af/`).
- `qa/scripts/qa-build.sh`, `qa/scripts/qa-run.sh`; `tests/regression/run-against-wip.sh` now forwards to qa-run.sh.
- Logs: `/tmp/qa-r1-final-ws.log`, `/tmp/qa-r1-final-reg.log`, `/tmp/qa-r1-final-ql-all.log`.
