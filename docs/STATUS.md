# STATUS: Track A hybrid launch + vault (engineering handoff)

Updated 2026-09-25 (MT). Branch `wip/hybrid-vault`, mirrored to `onchain/hybrid-launch`. The baseline tag
`audit-baseline-r1` = `a1cfa8f`. **Not audited.** Deployed to **devnet only** (2026-09-25); not on mainnet.

## Done (on top of the baseline)
- **Economics docs:** ADR-018 records QA-FEE-04 (first-mint cost comes from the escrow; M-06 holds on average). It
  also records the QA-FEE-05 disclosure (the average holds with ≥ ~613 NFTs left; exact frontend wording). CD Q1/Q2
  answers use measured lamport figures (`docs/DECISIONS.md`).
- **Batch expire:** `expire_requests(count ≤ 7)`, M-04 (`2be21da`).
- **Build and deploy hygiene:** production build and devnet deploy use an allowlist (no mock `.so` can ship); marker
  guards plus a self-test; platform-tools pinned to v1.54; scripts honour `CARGO_TARGET_DIR` (QA-HYG-01). Shown with `CARGO_TARGET_DIR=/workspace/scratch/tgt-hyg01`: a cold
  build, then launch 27, vault 76, dbc 10 and real_switchboard 4, all green.
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
- **Devnet deployment (2026-09-25, ~6:45 PM MT; M-37/M-38):**
  - Deployed with `scripts/deploy-devnet.sh` (production allowlist, no mock `.so`). Devnet accepted SBPF v2, so no v1
    rebuild was needed. Upgrade authority = the throwaway deployer `An3ZmiB4…` (placeholder for the Squads vault).
    - `hybrid_launch` `9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8`, deploy tx `4KzJVCFF…`
    - `hybrid_vault` `BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG`, deploy tx `5KEs59s5…`
  - Real Switchboard e2e: a gateway-signed reveal was settled by a third party. Measured per draw: commit 5,000, reveal
    10,000, oracle fee 0 (≈ 20,000 lamports with settle). A forged reveal is rejected (6033) and a replay is rejected
    (2001). Our vault's `init_randomness` CPI into live Switchboard works, with the authority set to our PDA. `open_vault`
    on a native launch fails closed (6037). Figures and tx sigs: DECISIONS CD Q2.
  - Deployer balance after the run: ≈ 0.296 SOL. Program rent (~4.64 SOL) can be reclaimed with `solana program close`
    if the deployment isn't needed. The e2e scripts and throwaway keys are in `/workspace/scratch/sbe2e` (not in git).
  - **Open:** the vault's own request → reveal → settle-with-mint has not run on devnet. `request_capture` needs an
    open vault, and `open_vault` needs a migrated DBC pool from a platform-created devnet DBC config whose
    `leftover_receiver` is the `["dbc_buffer"]` PDA (and ≥ 10 SOL of real buys to graduate). The settle-with-mint
    cost is the LiteSVM figure: 3,066,000 lamports spent from the 6,338,100 escrow.
- **Graduation (ADR-014 Limits):** the DAMM v2 migration is simulated in tests. Fields were confirmed on a real
  migrated devnet pool.
- **Needs Barton:**
  - mainnet `APPROVED_DBC_CONFIGS` and `PLATFORM_FEE_RECIPIENT`; the mainnet build refuses to compile until both
    are set;
  - the multisig/timelock setup (M-09);
  - fee-wallet custody (M-17).
- **QA review triggers (QA-owned tests):** `register_dbc_launch` affects 3 tests in `qa_launch`, and
  `expire_requests` affects 1 in `qa_regression`. See `audit-fixes-round1.md`, last section.
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
