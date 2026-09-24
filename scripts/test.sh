#!/usr/bin/env bash
# Build (SBPF v2) and run all tests against a local validator. Localnet only.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh
exec anchor test --validator legacy "$@"
