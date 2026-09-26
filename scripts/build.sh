#!/usr/bin/env bash
# Build the PRODUCTION programs (allowlist PROD_PROGRAMS: .so + IDL) into $TARGET/deploy and $TARGET/idl ($TARGET = ${CARGO_TARGET_DIR:-target}).
# Test-only programs (mock_switchboard) and test features are NEVER built here; they go to
# target/test-sbf via scripts/build-test-sbf.sh. A stale target/deploy/mock_switchboard.so left by an
# older full-workspace `anchor build` is removed so target/deploy only ever holds production artifacts.
source "$(dirname "$0")/env.sh"
source scripts/lib/deploy-guards.sh
set +u
rm -f "$TARGET/deploy/mock_switchboard.so" "$TARGET/deploy/mock_switchboard-keypair.json"
# A separate target dir must reuse the committed program ids: copy the program keypairs from the default
# target dir (anchor would otherwise mint new ones, and the .so would not match declare_id!).
if [ "$TARGET" != "$PWD/target" ]; then
  mkdir -p "$TARGET/deploy"
  for p in "${PROD_PROGRAMS[@]}"; do
    [ -f "$TARGET/deploy/$p-keypair.json" ] || [ ! -f "target/deploy/$p-keypair.json" ] || cp "target/deploy/$p-keypair.json" "$TARGET/deploy/"
  done
fi
for p in "${PROD_PROGRAMS[@]}"; do
  # --tools-version pins OUR platform-tools link (anchor's default is v1.57 = QA's; using it is what
  # re-linked QA's toolchain mid-build before). Cargo args after `--` would also reach the IDL build's
  # `cargo test`, so none are passed.
  anchor build -p "$p" --tools-version "$SBF_TOOLS_VERSION" --arch "$ANCHOR_BUILD_SBF_ARCH" "$@"
done
# anchor writes the IDL under <workspace>/target/idl regardless; mirror it into $TARGET/idl.
if [ "$TARGET" != "$PWD/target" ]; then
  mkdir -p "$TARGET/idl"
  for p in "${PROD_PROGRAMS[@]}"; do
    [ -f "$TARGET/idl/$p.json" ] && [ "$TARGET/idl/$p.json" -nt "target/idl/$p.json" ] || cp "target/idl/$p.json" "$TARGET/idl/" 2>/dev/null || true
  done
fi
for f in "$TARGET"/deploy/*.so; do
  n=$(basename "$f" .so)
  r=$(guard_allowlisted "$n") || { echo "build.sh: unexpected artifact $f: $r" >&2; exit 1; }
  r=$(guard_no_markers "$f") || { echo "build.sh: $r" >&2; exit 1; }
done
echo "build.sh: production artifacts: ${PROD_PROGRAMS[*]}"
