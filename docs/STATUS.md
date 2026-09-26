# STATUS: Track A hybrid launch + vault (engineering handoff)

Updated 2026-09-25 (MT). Branch `wip/hybrid-vault`, mirrored to `onchain/hybrid-launch`. The baseline tag
`audit-baseline-r1` = `a1cfa8f`. Localnet/LiteSVM only; **not audited, not deployed anywhere**.

## Done (on top of the baseline)
- **Economics docs:** ADR-018 records QA-FEE-04 (first-mint cost comes from the escrow; M-06 holds on average). It
  also records the QA-FEE-05 disclosure (the average holds with ≥ ~613 NFTs left; exact frontend wording). CD Q1/Q2
  answers use measured lamport figures (`docs/DECISIONS.md`).
- **Batch expire:** `expire_requests(count ≤ 7)`, M-04 (`2be21da`).
- **Build and deploy hygiene:** production build and devnet deploy use an allowlist (no mock `.so` can ship); marker
  guards plus a self-test; platform-tools pinned to v1.54; scripts honour `CARGO_TARGET_DIR` (QA-HYG-01).
- **Real Switchboard On-Demand:** compiles with Anchor 1.2. The real devnet program runs our init/commit/recommit
  calls in LiteSVM and rejects a forged reveal (`vault/real_switchboard.rs`).
- **Meteora DBC (ADR-014):**
  - `hybrid_launch::register_dbc_launch` checks the config is on the platform allowlist `APPROVED_DBC_CONFIGS`,
    then verifies DBC's pool, config and mint.
  - The real graduation verifier in `hybrid_vault` requires the recorded pool to have migrated.
  - The unsold buffer goes to a locked `["dbc_buffer"]` PDA that can never be withdrawn.
  - Tests: `vault/dbc_graduation.rs`, 10 tests, run against the real DBC program.
- **Docs cleanup (QA-DOC-01):** stale parts of ADR-008 and ADR-010 marked superseded; first-mint cost table in
  `hybrid-rarity-and-assignment.md`.

## Open
- **Devnet run (M-37/M-38):**
  - The throwaway deployer `An3ZmiB4SaA7FCJ4bpad2qDRmqhu4F9d5UqUAKHiAB5Z` has 0 SOL. Every faucet or RPC refused.
    Barton could use the web faucet (https://faucet.solana.com) for 1–2 SOL.
  - Then: `./scripts/deploy-devnet.sh`, a third-party reveal, and measuring the oracle fee and reveal cost per draw.
    Record the results in DECISIONS CD Q2 and M-37.
  - A devnet DBC end-to-end run also needs a platform-created devnet DBC config whose `leftover_receiver` is the
    buffer PDA.
- **Graduation (ADR-014 Limits):** the DAMM v2 migration is simulated in tests. Fields were confirmed on a real
  migrated devnet pool.
- **Needs Barton:**
  - mainnet `APPROVED_DBC_CONFIGS` and `PLATFORM_FEE_RECIPIENT`; the mainnet build refuses to compile until both
    are set;
  - the multisig/timelock setup (M-09);
  - fee-wallet custody (M-17).
- **QA review triggers (QA-owned tests):** `register_dbc_launch` affects 3 tests in `qa_launch`, and
  `expire_requests` affects 1 in `qa_regression`. See `audit-fixes-round1.md`, last section.
- **Not yet shown:** the full suite passing from a separate `CARGO_TARGET_DIR`. The scripts support it, but that
  run was interrupted.
- The audit itself: round-1 status is in `docs/audit-fixes-round1.md`.

## Build and test
```bash
set +u; source scripts/env.sh          # pins platform-tools v1.54; TARGET=${CARGO_TARGET_DIR:-target}
./scripts/build.sh                     # production .so + IDL (allowlist: hybrid_launch, hybrid_vault)
./scripts/build-test-sbf.sh            # TEST-ONLY binaries (mock graduation, mock_switchboard) -> $TARGET/test-sbf
cargo test --workspace --locked --no-fail-fast
./scripts/test-deploy-guards.sh        # deploy guard self-test
```
- Don't run a bare `anchor build`: it defaults to platform-tools v1.57, which is QA's.
- Use your own `CARGO_TARGET_DIR` when QA is building at the same time.
- Expected at HEAD: all engineer suites green; the only failures are the 4 QA review triggers above.
- Fixtures: `tests/track-a-hybrid/fixtures/{switchboard-devnet,dbc}`, read-only public dumps; see their READMEs.
