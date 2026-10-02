#!/usr/bin/env bash
# QA-only build of the LiteSVM test artifacts for a checkout (default: the QA worktree at audit-baseline-r1).
# Build hygiene (engineer request 2026-09-25): never touches the global rustup default, never installs or switches
# platform-tools, never edits ~/.solana-toolchain-env.sh or shared cargo/solana config. Everything goes to a
# QA-only CARGO_TARGET_DIR; toolchain choices are per-command. Localnet only.
set -euo pipefail
SRC="${1:-/workspace/launchpad-qa}"
export CARGO_TARGET_DIR="${QA_TARGET:-/workspace/launchpad/qa/.target}"
export PATH="$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$HOME/.avm/bin:$PATH"
SBF=(--tools-version v1.54 --skip-tools-install --arch v2)   # repo-pinned platform-tools, already installed
mkdir -p "$CARGO_TARGET_DIR"/{deploy,idl,test-sbf}
cd "$SRC"
# production launch binary + both IDLs (tests read CARGO_TARGET_TMPDIR/../{deploy,idl,test-sbf})
(cd programs/hybrid_launch && cargo build-sbf "${SBF[@]}" --sbf-out-dir "$CARGO_TARGET_DIR/deploy" -- --locked)
(cd programs/hybrid_vault && cargo build-sbf "${SBF[@]}" --sbf-out-dir "$CARGO_TARGET_DIR/deploy" -- --locked)
# TEST-ONLY binaries (mock graduation vault, mock switchboard), never in deploy/
(cd programs/hybrid_vault && cargo build-sbf "${SBF[@]}" --features test-mock-graduation --sbf-out-dir "$CARGO_TARGET_DIR/test-sbf" -- --locked)
(cd tests/track-a-hybrid/mock-switchboard && cargo build-sbf "${SBF[@]}" --sbf-out-dir "$CARGO_TARGET_DIR/test-sbf" -- --locked)
grep -aq "TEST-ONLY mock graduation" "$CARGO_TARGET_DIR/test-sbf/hybrid_vault.so"
! grep -aq "TEST-ONLY mock graduation" "$CARGO_TARGET_DIR/deploy/hybrid_vault.so"
for p in hybrid_launch hybrid_vault; do
  anchor idl build -p "$p" -o "$CARGO_TARGET_DIR/idl/$p.json"
done
# QA-HYG-01 fixed at 5cd7a7e (harness target_artifact() follows CARGO_TARGET_DIR): no worktree IDL copy any more.
echo "qa-build: artifacts in $CARGO_TARGET_DIR"
