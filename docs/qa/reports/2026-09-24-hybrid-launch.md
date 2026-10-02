# QA report: `hybrid_launch::launch` (Track A), 2026-09-24

- **Commit under test:** local `main` @ `c02be68` ("Scope change (ADR-009): shelve Track B, add hybrid_launch (ADR-010) with LiteSVM tests").
- **Program:** `programs/hybrid_launch`, ID `9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8` (localnet throwaway key). Built SBPF v2 with Anchor 1.2.0 / Agave 4.1.2 / Rust 1.89.0.
- **Environment:** LiteSVM 0.10.0 only. No validator deploys, no devnet, no mainnet.
- **QA files (new; nothing in `programs/` or the engineer's test files was edited):**
  - `tests/track-a-hybrid/launch/qa_launch.rs`: the QA tests.
  - `tests/track-a-hybrid/tests/qa_launch.rs`: a 3-line shim (`#[path = "../launch/qa_launch.rs"] mod qa_launch;`). Cargo's test auto-discovery (`tests/*.rs`) picks it up, so `cargo test --workspace` / `./scripts/test.sh` run the QA tests without touching the engineer's `Cargo.toml`.
- **Author:** QA. Written 2026-09-24, around 3:05–3:30 PM MT.

## 1. Summary

| Suite | Tests | Result |
|---|---|---|
| Engineer: unit (`validation::tests`) | 4 | ✅ 4 pass |
| Engineer: LiteSVM (`tests/track-a-hybrid/launch/launch.rs`) | 19 | ✅ 19 pass |
| QA: LiteSVM (`qa_launch.rs`), default run | 24 | ✅ 24 pass |
| QA: known-finding tests (`#[ignore]`, run with `--ignored`) | 1 | ❌ 1 fails, as expected (QA-HL-01) |

- **No real bug was found in the validation logic, the supply math or the authority revocation.** Every parameter edge and the 400-case property test (plus 3×600 extra seeds) match QA's independent u128 oracle exactly.
- **Findings:**
  - one Low robustness bug (QA-HL-01, pre-funded mint address blocks launch)
  - one design gap already open as DECISIONS N2 (QA-HL-02, caller-chosen destination owner)
  - two documentation inconsistencies (§5)
- **Test issues seen during the run (not program bugs):**
  1. `./scripts/test.sh` fails with `no such command: +1.95.0-sbpf-solana-v1.57` if `~/.cargo/bin` isn't first in `PATH`. The fix is to put rustup's cargo first.
  2. The shared box currently has another `solana-test-validator` bound to UDP 8000, so `anchor test --validator legacy` times out ("gossip_addr bind_to port 8000: Address already in use"). The tests themselves don't use the validator.
  3. Since about 3:10 PM MT the working tree has an untracked, half-written `programs/hybrid_vault/` (no `src/lib.rs` yet), which makes `cargo metadata` fail for the whole workspace. It's the engineer's work in progress, so I left it alone.

  Because of (2) and (3), the authoritative full run was done on a clean `git archive c02be68` export in `/tmp/qa-c02be68`, plus the two QA files, using the Anchor.toml test script directly.

## 2. How to reproduce

```bash
export PATH="$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$HOME/.avm/bin:/usr/local/bin:/usr/bin:/bin:$HOME/.local/bin"
cd /workspace/launchpad
./scripts/test.sh              # builds + runs everything; needs port 8000 free and a parseable workspace
# equivalent without the validator (what was actually run, after `anchor build`):
cargo test --workspace --locked -- --show-output
# only QA:
cargo test -p track-a-hybrid-tests --locked --test qa_launch -- --show-output
# known findings (expected to FAIL until fixed):
cargo test -p track-a-hybrid-tests --locked --test qa_launch -- --ignored
# property test with another seed / more iterations:
QA_SEED=42 QA_ITERS=600 cargo test -p track-a-hybrid-tests --locked --test qa_launch -- --show-output qa_property
```

Clean-tree variant: `git archive c02be68 | tar -x -C /tmp/qa-c02be68`, copy the two QA files in, copy `target/deploy/hybrid_launch-keypair.json` in, then `anchor build` and the `cargo test` line above.

## 3. What passes (QA tests, all against the real SBF binary)

Every successful launch in these tests goes through `assert_post_launch`, which checks:
- **INV-01:** the mint is owned by classic Token, is 82 bytes, has `supply == 1e9 × 10^d` exactly, the right decimals, and mint and freeze authority `None`. The destination holds the whole supply, with the right owner and mint, and no delegate or close authority.
- **INV-07:** every `LaunchConfig` field equals QA's oracle, including `ratio_base`, `max_tokens_in_nft_form ≤ total_supply_base`, exact fee amounts (`amount × 10000 == ratio_base × bps`), `version == 1`, and `launched_at == Clock.unix_timestamp`.
- **INV-13:** the PDA addresses match, and the stored `bump` and `mint_authority_bump` are canonical.
- The config size is `8 + INIT_SPACE`.

| Test | What it proves |
|---|---|
| `qa_ratio_neighbours_extremes_and_non_allowed_divisors_rejected` | 22 ratios are rejected with `RatioNotAllowed` (6001) and leave no mint or config behind: 0, 1, 2, 5k, 20k, 25k, 500k, 2M, 1B, 1B+1, u64::MAX−1, u64::MAX, and ±1 around each allowed value. |
| `qa_every_ratio_exactly_at_1b_passes_and_one_nft_over_fails_for_decimals_0_6_9` | For all 5 ratios × decimals {0, 6, 9}: N = 1B/R succeeds with `max_tokens_in_nft_form == total_supply_base` (exactly 100%), and N+1 fails with `CollectionTooLargeForSupply` (6003). That's 30 launches. |
| `qa_collection_size_u64_overflow_boundaries_rejected` | Decimals {0, 9} × 5 ratios × N ∈ {u64::MAX/ratio_base (fits u64, far above 1B), that value +1 (first overflow), u64::MAX/2, u64::MAX}: all fail with 6003 and never wrap. N=0 at decimals 0 fails with `ZeroCollectionSize` (6002). |
| `qa_every_decimals_0_to_9_mints_exactly_1b_times_10_pow_d` | Decimals 0 through 9 all pass INV-01 and INV-07. The literal edges are asserted: supply is 1,000,000,000 at d=0 and 1,000,000,000,000,000,000 (1e18, which fits in u64) at d=9. |
| `qa_decimals_10_to_255_rejected` | Decimals 10, 11, 18, 19, 20, 38, 128, 254 and 255 fail with `InvalidDecimals` (6000). The large values would overflow `10^d` or the supply without the early check. |
| `qa_fee_edges_at_cap_equal_zero_and_over` | Accepted at d=0 and d=9 with exact amounts: (1000,1000) at the cap, (1000,0), (0,0), (500,500), (1,1). Rejected with `FeeAboveCap` (6004): (1000,1001), (1001,1000), (65535,0), (65535,65535). Rejected with `CaptureFeeBelowRerollFee` (6005): (499,500), (0,1), (999,1000). |
| `qa_every_non_burn_fee_destination_value_rejected` | All 255 values 1..=255 fail with `FeeDestinationNotBurn` (6006). |
| `qa_real_classic_mint_with_supply_and_foreign_authority_cannot_be_launched` | A real classic mint (made with InitializeMint2, attacker as mint and freeze authority, supply 5 in the attacker's ATA) can't be launched. The mint account is byte-identical afterwards and no LaunchConfig exists. The engineer's test used a `set_account` fake instead. |
| `qa_real_token_2022_mints_with_and_without_extensions_cannot_be_launched` | Real Token-2022 mints, one plain (82 bytes) and one with PermanentDelegate (202 bytes), are rejected with either `token_program` (classic or Token-2022). They stay unchanged and get no config. |
| `qa_mint_equal_to_creator_is_rejected` | Passing the creator's own wallet as the mint fails, and the wallet stays system-owned. |
| `qa_wrong_program_ids_rejected` | Eight substitutions are rejected with `InvalidProgramId` (3008) or a runtime program error, and nothing is created: `token_program` ∈ {System, ATA, hybrid_launch, random}, `associated_token_program` ∈ {Token, random}, `system_program` ∈ {Token, random}. |
| `qa_destination_ata_of_other_owner_or_token_2022_derivation_rejected` | Four bad destinations fail atomically. A different owner's ATA and the Token-2022-derived ATA of the right owner fail with `InvalidLaunchDestination` (6007). The config PDA and the mint itself also fail. |
| `qa_prefunded_destination_ata_config_and_mint_authority_do_not_block_launch` | An attacker who sends lamports to the destination ATA, the `launch_config` PDA or the `mint_authority` PDA before launch can't block it. Launch still succeeds and passes INV-01/07/13. The attacker also can't initialize the ATA early, because the mint doesn't exist yet. |
| `qa_off_curve_pda_destination_owner_is_supported` | An off-curve PDA works as `launch_destination_owner`. N2's recommended curve/sale vault PDA depends on this. |
| `qa_observation_creator_can_direct_entire_supply_to_own_wallet` | Documents current behaviour: see QA-HL-02. |
| `qa_two_launches_by_same_creator_are_fully_isolated` | Two launches by one creator get distinct mint, config, mint_authority and destination accounts. Launch 1's accounts are byte-identical after launch 2, and both pass INV-01/07. |
| `qa_same_mint_twice_in_one_transaction_fails_atomically` | Two `launch` instructions with the same mint in one tx fail, and nothing persists. |
| `qa_relaunch_of_existing_mint_with_different_destination_owner_fails_and_changes_nothing` | An attacker re-running `launch` on a launched mint, with different params and their own destination owner, fails. The mint, config and destination are unchanged, and no second destination is created. |
| `qa_launch_config_cannot_be_modified_or_closed_by_any_instruction` | Covers INV-07. About 80 payloads are sent to hybrid_launch with the config writable in two account positions: empty, 1 byte, 0xff×8, the `launch` discriminator truncated and with garbage, the Anchor IDL tag `0x0a69e9a778bcf440` with variants 0–7, and 64 seeded random discriminators. System `transfer`/`assign` and SPL `close_account` are also tried on the config. The config account (data, lamports, owner) stays byte-identical. `SetAuthority(MintTokens)` on the mint fails. |
| `qa_launch_config_and_mint_are_exactly_rent_exempt` | The config and mint lamports equal `minimum_balance(len)` exactly, and the destination is rent-exempt. |
| `qa_non_canonical_bump_pdas_rejected` | Valid off-curve PDAs built with a non-canonical bump, for both the config and the mint_authority, fail with `ConstraintSeeds` (2006). At least 2 cases are found and tested per run. |
| `qa_payer_too_poor_fails_atomically_and_exact_budget_succeeds` | A payer with `rent(config) + rent(82) + rent(165) + 2 × 5000 − 1` lamports fails, and no mint, config or ATA is left behind. With exactly that amount, launch succeeds and the payer ends at 0. |
| `qa_launch_fits_default_compute_budget_in_worst_case_params` | Launch at d=9, R=10k, N=100k, fees 1000/1000 uses about 62k–68k CU (varies with bump search), under the default 200k. |
| `qa_property_launch_succeeds_iff_all_constraints_hold` | Seeded xorshift loop (proptest isn't in the lockfile). 400 cases per run, default seed `0xC0FFEE0020260924`, biased toward edges. Launch succeeds iff QA's independent u128 oracle says every constraint holds. On success, INV-01/07/13 hold. On failure, the error code is one of the violated constraints and nothing persists. Default seed: 38 valid / 362 rejected. Seeds 1, 42 and 987654321 at 600 iterations each also pass. |

## 4. What fails

### QA-HL-01: a pre-funded mint address blocks `launch` (real bug, severity **Low**, griefing/DoS)

- **Test:** `qa_finding_prefunded_mint_address_should_not_block_launch` (marked `#[ignore]` so the shared suite stays green).
- **Assertion:** `send(...).expect("pre-funded (1 lamport) mint address should not block launch")`, at `launch/qa_launch.rs` in that test.
- **Actual:**
  ```
  InstructionError(0, Custom(0)) … "Create Account: account Address { address: 7ycB…3ySF, base: None } already in use"
  "Program 11111111111111111111111111111111 failed: custom program error: 0x0"
  ```
- **Expected vs actual:** Expected: `launch` succeeds even if someone sent lamports to the fresh mint address first, as Anchor's own `init` does. Actual: the mint is created with a raw `system_program::create_account` (ADR-010 step 1), which fails if the address holds any lamports, so 1 lamport from anyone blocks the launch for that mint keypair.
- **Impact:** Only exploitable if an attacker learns the mint pubkey before the launch lands. Examples: a dropped or expired first attempt that the wizard retries with the same keypair, a vanity or pre-announced mint address, or a front-runner watching the leader's forwarded txs. The creator has to start over with a new keypair; no funds are at risk.
- **Fix options for the engineer:**
  - Use Anchor `init` / `create_account` with the "transfer + allocate + assign if lamports > 0" pattern.
  - Or make the wizard always generate a new mint keypair on retry and document that.
- **Repro:** `cargo test -p track-a-hybrid-tests --locked --test qa_launch -- --ignored qa_finding_prefunded_mint_address_should_not_block_launch`

### QA-HL-02: `launch_destination_owner` is caller-chosen (design gap, already open as DECISIONS N2; severity **High if shipped as-is**, informational today)

- **Test:** `qa_observation_creator_can_direct_entire_supply_to_own_wallet`. It **passes**, because it documents current behaviour.
- **What happens:** The creator can send 100% of the 1B supply to their own wallet in the launch tx. Nothing on-chain ties the destination to a curve or sale vault.
- **Why it matters:** This contradicts ARCHITECTURE principle 4 ("No outflow to a caller-chosen address. Every destination is derived (PDA/ATA) or stored at init") and the Stonk.fun lessons (single-wallet launches). The program behaves as ADR-010 says (N2 is marked open), so this isn't a program bug against the current spec.
- **Recommendation:** Before any public launch, `launch` should derive the destination owner (e.g. the curve/sale vault PDA, or a CPI from the curve program), or record and cap an explicit creator allocation. When that lands, QA will flip this test to assert rejection. `qa_off_curve_pda_destination_owner_is_supported` already confirms a PDA owner works.

## 5. Documentation inconsistencies found (not code bugs; for the engineer)

1. **Engine status.** The engineer's 3:04 PM MT update says the custom `hybrid_vault` + Metaplex Core engine is decided. But `docs/hybrid-rarity-and-assignment.md` (header), `docs/DECISIONS.md` Q-H1 / ADR-008 ("Proposed") and `docs/qa-answers.md` ("Q1 engine: [needs Barton]") still say it's open.
2. **When the token fee is burned.** `docs/hybrid-rarity-and-assignment.md` §3.3 (iii) says `request_capture` burns the token fee at request time. `docs/qa-answers.md` Q3 and DECISIONS N7 say the engine should burn at `settle`, so `expire` can refund it. QA's RR-08 depends on which one is true.

## 6. Not covered yet

- **Real localnet validator run** of `launch`: the validator port is busy on the shared box, and all coverage is LiteSVM. Repeat SUP-01 on `anchor test --validator legacy` once port 8000 is free.
- **CPI into `launch` from another program**, e.g. a future curve program calling it. The mint must sign, so the risk is low, but it's needed if N2 is solved via CPI.
- **Trident fuzzing** (`fuzz_config`): not set up; the seeded property loop stands in for now.
- **Upgrade-authority checks (SUP-06, GOV-07):** no deploy/launch-readiness script exists yet.
- **`launched_at` under clock warps:** only checked against the default LiteSVM clock.
- **Everything in the engine** (`hybrid_vault`: capture/settle/release/re-roll/expire, VRF, pause), the curve and the UI. The code doesn't exist at c02be68 (`programs/hybrid_vault/` is untracked work in progress).
- **Mutation testing** of the QA suite (e.g. flip `<=` to `<` in `validate()` and confirm a QA test fails). By inspection, the boundary tests (max vs max+1, cap vs cap+1, capture == reroll) would catch those mutants, but this hasn't been run.

## 7. Engineer's coverage vs QA additions (why each QA test isn't a duplicate)

The engineer's 19 cover:
- the happy path; max size at 10k; the IDL has only `launch`
- Token-2022 substituted as `token_program`; `set_account`-faked pre-existing mints; relaunch of the same mint
- ratios {0, 1, 30k, 250k, 1B}; N 1001@1M and 100001@10k; N u64::MAX; N 0; decimals 10
- fee 1001 capture and 1001/1001; capture 100 < reroll 200; destination {1, 2, 255}
- random-pubkey destination; spoofed mint_authority; spoofed config; mint not signing

QA adds:
- all other ratio neighbours and extremes; max/max+1 for every ratio at 3 decimals
- exact u64 overflow boundaries; decimals 0–9 exact supply; decimals up to 255
- fee boundaries at the cap, equality, and all 255 destinations
- real (not faked) classic and Token-2022 mints, including extensions
- wrong ATA and System program IDs; Token-2022-derived and other-owner ATAs; pre-funded PDA/ATA griefing
- two-launch isolation; same-mint-twice atomicity; an immutability sweep over random and IDL instructions
- exact rent; non-canonical bumps; payer boundary; CU; the property test
