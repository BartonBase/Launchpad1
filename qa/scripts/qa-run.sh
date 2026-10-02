#!/usr/bin/env bash
# Run every suite (engineer + QA) against a git ref in a detached QA worktree, with QA-only build artifacts.
# Usage: qa/scripts/qa-run.sh [ref=audit-baseline-r1]. Never touches the engineer's working tree, the rustup default,
# platform-tools or shared cargo/solana config. Localnet/LiteSVM only.
set -euo pipefail
REF="${1:-audit-baseline-r1}"
REPO=/workspace/launchpad
WT="${QA_WT:-/workspace/launchpad-qa}"
export CARGO_TARGET_DIR="${QA_TARGET:-$REPO/qa/.target}"
export PATH="$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$HOME/.avm/bin:$PATH"
if [ ! -d "$WT" ]; then git -C "$REPO" worktree add --detach "$WT" "$REF"; else git -C "$WT" checkout --detach "$REF"; fi
mkdir -p "$WT/tests/regression"
cp "$REPO"/tests/regression/*.rs "$WT/tests/regression/"
cp "$REPO/tests/track-a-hybrid/launch/qa_launch.rs" "$WT/tests/track-a-hybrid/launch/"
"$REPO/qa/scripts/qa-build.sh" "$WT"
cd "$WT"
cargo test --workspace --locked --no-fail-fast
cargo test --locked -p track-a-hybrid-tests --features qa-regression --test qa_regression -- --show-output
