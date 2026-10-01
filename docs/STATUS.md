# STATUS: Track A hybrid launch + vault (engineering handoff)

Updated 2026-10-01. Branch `onchain/hybrid-launch`. **Not audited.** Devnet only. Not mainnet.

## Mode 1 (plain SPL) — 2026-10-01

`hybrid_launch::launch_plain` and `hybrid_launch::register_plain_dbc` write `PlainLaunchConfig`
(not `LaunchConfig`). Classic SPL, fixed 1B, mint and freeze revoked, no update. The curve path
accepts only `APPROVED_DBC_CONFIGS`. No ratio, collection, fee transfer, wrap, unwrap, or NFT.
`hybrid_vault::init_vault` still loads `LaunchConfig` only, so a plain mint cannot open a vault.
Mode 2 instruction arguments and `LaunchConfig` layout are unchanged. Mode 3 is local-only in this tree
(`launch_burn`, `register_burn_dbc`, `init_permanent_vault`, `open_permanent_vault`, `wrap_permanent`).
Wrap burns exactly the ratio and mints the next committed NFT in the same instruction. No release.
The index is sequential, not a Switchboard draw. Mode 2 unwrap still returns the NFT to inventory.
Mode 4 is local-only and does not burn. `launch_token22` creates a Token-2022 mint whose only
extension is a transfer tax of 1..=1000 bps, chosen at launch, with the fee-config authority left
unset so the rate cannot change. Withheld tax is withdrawn by the vault tax PDA (`harvest_tax`)
and the program then pays that frozen share itself, NFT 0 then 1 then 2, only to the current
owner of each. A round does not open until the pot, valued at the locked buyback price, reaches
the Stonk-style market-cap tier ($50, $200, $250, then 0.1% of market cap, capped at $50,000,
at 1 SOL = $100). There is no $20 holder minimum and no live price the caller can set. A wallet
with 10 NFTs that were minted before the freeze receives 10 times one share. An NFT minted after
the freeze gets none of that round. The crank cannot pick the NFT,
the amount, the order, or the destination, and there is no admin who can. `buyback` spends
SOL already on that same PDA to buy tokens from the launch inventory at `buyback_lamports_per_whole`
(1 lamport to 1 SOL per whole token, also chosen at launch and not updatable). Bought tokens accrue
to those NFT holders only. `wrap_token22` locks the ratio and mints the NFT; supply does not
decrease. There is no external DEX swap: the buy is from this launch's own inventory. No Token-2022
curve register. Mode 5 is a separate launch (`launch_raffle`, `init_raffle_vault`). Same locked
tax and locked buyback price. When the pot clears the same market-cap tier, the program freezes
the NFTs that already exist and records each owner before Switchboard is asked for a number.
One of those NFTs wins the whole pot. Ten NFTs is ten chances, not ten payouts. Buying the
winning NFT after the number is public does not move the prize. The program picks the oracle,
accepts only that randomness account, and will not draw again because someone dislikes the
result. Mode 4 still splits the pot across every minted NFT. Upgrade authority is still the throwaway deployer,
not the Squads timelock. Not deployed. Local LiteSVM on this tree: launch 30, vault 83, dbc
graduation 16, real Switchboard 4. Host units: hybrid_launch 19, hybrid_vault 15.
`qa_launch` and `qa_regression` stay commented out; those files are not on this branch.

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
  - **Full vault e2e on devnet: done (2026-09-26, ~7:30 AM MT).**
    - The run used `scripts/devnet-e2e.sh`, with the re-roll and release steps under `EXTRAS=1`: DBC pool →
      register → vault → buy to 0.1 SOL → DAMM v2 migration → leftover to the buffer → open_vault → randomness
      → capture → third-party reveal → settle-with-mint (#11) → re-roll (#25 minted) → release (exactly 1M tokens
      back). Figures and signatures are in DECISIONS CD Q1.
    - **Fixes, all script-only, no program bugs:**
      - ExtendProgram needs at least 10,240 bytes, so the script extends by that;
      - resuming now skips steps that are already settled.
    - **`hybrid_launch` on devnet now runs the `devnet-e2e` build** (0.1 SOL graduation floor; program data
      231,216 B; no leftover buffers). **Keep it for devnet:** it's the only build the platform devnet config
      registers with. Before any audit/mainnet comparison, redeploy the default build or verify against
      `target/devnet-e2e`. The mainnet build can't include the feature.
    - Deployer after the run: 4.9347 SOL (net ≈ 0.24 SOL, including 0.052 SOL of extend rent).
      The throwaway keys under `.keys/devnet-e2e` are swept.
- **QA-FEE-03 (ADR-019, 2026-09-26):** exact-tier fee check on capture and re-roll (vault 6061 `FeeNotTier`), and
  at config creation (launch 6024 `FeeNotTier`), through one shared `tier_fee_lamports`.
  - Suites are green on the default build and on the devnet-e2e build
    (`cargo test -p track-a-hybrid-tests --features devnet-e2e` with `CARGO_TARGET_DIR=target/devnet-e2e`).
  - **Devnet is NOT upgraded with this yet.** `hybrid_vault` grew to 692,152 B, above its 691,744 B program data,
    so it needs a 10,240 B extend (≈ 0.052 SOL, not refunded) plus a ~3.5 SOL temporary buffer. The devnet
    programs still run the pre-QA-FEE-03 code.
- **Graduation (ADR-014 Limits):** the DAMM v2 migration is simulated in tests. Fields were confirmed on a real
  migrated devnet pool.
- **Needs Barton:**
  - mainnet `APPROVED_DBC_CONFIGS` and `PLATFORM_FEE_RECIPIENT`; the mainnet build refuses to compile until both
    are set;
  - the multisig/timelock setup (M-09);
  - fee-wallet custody (M-17).
- **QA suites (QA-owned tests):** fully green at HEAD after QA updated their files (2026-09-26): `qa_launch` 37/37,
  `qa_regression` 56/56, including `qa_FEE04_M19_zero_fee_behaviour` and
  `qa_sol_fee::qa_FEE03_M08_vault_rejects_forged_config_off_tier_or_above_cap`. The earlier review triggers are gone.
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
- Expected at HEAD: all engineer suites and QA's `qa_launch` / `qa_regression` green.
- Fixtures: `tests/track-a-hybrid/fixtures/{switchboard-devnet,dbc}`, read-only public dumps; see their READMEs.
