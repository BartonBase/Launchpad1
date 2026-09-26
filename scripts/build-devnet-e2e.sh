#!/usr/bin/env bash
# DEVNET E2E ONLY. Builds hybrid_launch with `--features devnet-e2e` (0.1 SOL graduation floor, see
# needs_barton.rs) into $E2E_TARGET (default target/devnet-e2e), NEVER into target/deploy, and without
# an IDL (the committed IDL and QA's IDL constant checks stay on the default build). hybrid_vault is
# unchanged by the feature and is not rebuilt. The mainnet feature can't be combined (compile_error).
source "$(dirname "$0")/env.sh"
source scripts/lib/deploy-guards.sh
set +u
E2E_TARGET=${E2E_TARGET:-$PWD/target/devnet-e2e}
mkdir -p "$E2E_TARGET/deploy"
cp target/deploy/hybrid_launch-keypair.json "$E2E_TARGET/deploy/"   # same program id (declare_id!)
CARGO_TARGET_DIR="$E2E_TARGET" anchor build -p hybrid_launch --no-idl --tools-version "$SBF_TOOLS_VERSION" \
  --arch "$ANCHOR_BUILD_SBF_ARCH" -- --features devnet-e2e
so="$E2E_TARGET/deploy/hybrid_launch.so"
[ -f "$so" ] || { echo "build-devnet-e2e.sh: $so missing" >&2; exit 1; }
r=$(guard_no_markers "$so") || { echo "build-devnet-e2e.sh: $r" >&2; exit 1; }
if cmp -s "$so" target/deploy/hybrid_launch.so; then echo "build-devnet-e2e.sh: identical to the default build (feature not applied?)" >&2; exit 1; fi
echo "build-devnet-e2e.sh: $so ($(stat -c %s "$so") bytes)"
