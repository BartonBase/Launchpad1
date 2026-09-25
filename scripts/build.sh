#!/usr/bin/env bash
# Build the PRODUCTION programs (allowlist PROD_PROGRAMS: .so + IDL) into target/deploy and target/idl.
# Test-only programs (mock_switchboard) and test features are NEVER built here; they go to
# target/test-sbf via scripts/build-test-sbf.sh. A stale target/deploy/mock_switchboard.so left by an
# older full-workspace `anchor build` is removed so target/deploy only ever holds production artifacts.
source "$(dirname "$0")/env.sh"
source scripts/lib/deploy-guards.sh
set +u
rm -f target/deploy/mock_switchboard.so target/deploy/mock_switchboard-keypair.json
for p in "${PROD_PROGRAMS[@]}"; do
  # --tools-version pins OUR platform-tools link (anchor's default is v1.57 = QA's; using it is what
  # re-linked QA's toolchain mid-build before). Cargo args after `--` would also reach the IDL build's
  # `cargo test`, so none are passed.
  anchor build -p "$p" --tools-version "$SBF_TOOLS_VERSION" --arch "$ANCHOR_BUILD_SBF_ARCH" "$@"
done
for f in target/deploy/*.so; do
  n=$(basename "$f" .so)
  r=$(guard_allowlisted "$n") || { echo "build.sh: unexpected artifact $f: $r" >&2; exit 1; }
  r=$(guard_no_markers "$f") || { echo "build.sh: $r" >&2; exit 1; }
done
echo "build.sh: production artifacts: ${PROD_PROGRAMS[*]}"
