# Tests (on-chain)

- **LiteSVM integration tests** live next to each program: `programs/fee_treasury/tests/`, `programs/holder_lottery/tests/`.
- **Unit tests** (ticket math, including the sybil-split property) are in `programs/holder_lottery/src/math.rs`.
- **Localnet smoke test**: `tests/localnet-smoke/` runs against a real `solana-test-validator` started by
  `anchor test --validator legacy`. It creates a real Token-2022 TransferFee mint and calls both `initialize`s. It
  skips unless `ANCHOR_PROVIDER_URL` is set, and it refuses non-localhost URLs.

Run everything with `./scripts/test.sh` (see ../docs/DEV_SETUP.md). QA's plan is in `../qa/TEST_PLAN.md`.
