#!/usr/bin/env bash
# Build the TEST-ONLY SBF binaries used by the LiteSVM tests into target/test-sbf/ (never deployed):
#  - hybrid_vault with `test-mock-graduation` (mock graduation verifier, TEST-ONLY),
#  - mock_switchboard (stand-in for Switchboard On-Demand at its program id).
# The production binaries in target/deploy/ (from `anchor build`) contain neither.
source "$(dirname "$0")/env.sh"
export PATH="$HOME/.cargo/bin:$PATH"
set +u
ROOT=$(pwd)
OUT="$ROOT/target/test-sbf"
mkdir -p "$OUT"
# --sbf-out-dir MUST be absolute (a relative one fails with "Failed create folder") and MUST be
# given: cargo-build-sbf's default out dir is target/deploy, which would put the TEST-ONLY
# binary where the production one lives.
(cd programs/hybrid_vault && cargo build-sbf --arch "$ANCHOR_BUILD_SBF_ARCH" --features test-mock-graduation --sbf-out-dir "$OUT" -- --locked)
(cd tests/track-a-hybrid/mock-switchboard && cargo build-sbf --arch "$ANCHOR_BUILD_SBF_ARCH" --sbf-out-dir "$OUT" -- --locked)
# Guard: the production vault binary must NOT contain the TEST-ONLY marker.
if [ -f target/deploy/hybrid_vault.so ] && grep -aq "TEST-ONLY mock graduation" target/deploy/hybrid_vault.so; then
  echo "build-test-sbf.sh: target/deploy/hybrid_vault.so contains the TEST-ONLY mock; refusing" >&2
  exit 1
fi
grep -aq "TEST-ONLY mock graduation" "$OUT/hybrid_vault.so" || { echo "test build lacks mock marker" >&2; exit 1; }
echo "build-test-sbf.sh: test binaries in $OUT"
