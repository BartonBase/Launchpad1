#!/usr/bin/env bash
# Build, start solana-test-validator (NOT surfpool), deploy both programs to
# localnet, then run `cargo test --workspace --locked`:
#   * LiteSVM tests in programs/*/tests (in-process)
#   * tests/localnet-smoke (real RPC against the local validator)
source "$(dirname "$0")/env.sh"
anchor test --validator legacy "$@"
