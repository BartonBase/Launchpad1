# Tests (on-chain, Track A)

- **Unit tests** (supply/ratio/fee math): `programs/hybrid_launch/src/validation.rs`.
- **LiteSVM integration tests**: `tests/track-a-hybrid/` (crate `track-a-hybrid-tests`). They load the real SBF build
  from `target/deploy/hybrid_launch.so` and the IDL from `target/idl/`, so build first (`./scripts/test.sh` does).
  - `launch/launch.rs`: `hybrid_launch::launch`. One happy path, one max-size path, one IDL immutability check, and one
    test per rejection, each named for the attack it proves (`attack_*`).
  - `vault/vault.rs` + `vault/harness.rs`: `hybrid_vault` (capture/re-roll/settle/expire/release, lazy mint, M-xx
    regressions). Loads the TEST build from `target/test-sbf` (`./scripts/build-test-sbf.sh`: `test-mock-graduation`
    + `mock_switchboard`), never the production `.so`.
  - The crate uses `autotests = false`: every test binary is an explicit `[[test]]` in `track-a-hybrid/Cargo.toml`
    (`launch`, `vault`, QA's `qa_launch` → `launch/qa_launch.rs`, and `qa_regression` → shim
    `qa-roots/qa_regression.rs`). `qa_regression` has `required-features = ["qa-regression"]` because QA's
    `tests/regression` targets an older harness API; build it with `--features qa-regression` once QA updates it.
- **Shelved:** the Token-2022 localnet smoke test and the `fee_treasury`/`holder_lottery` tests are in
  `../shelved/` and on local branch `shelved/track-b-t22` (ADR-009). `tests/_shelved/` and `tests/.keys/` are QA's.

Run everything with `./scripts/test.sh` (see ../docs/DEV_SETUP.md). QA's plan is in `../qa/TEST_PLAN.md`.
