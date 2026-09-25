#!/usr/bin/env bash
# Shared environment for launchpad program scripts. LOCALNET/DEVNET ONLY.
set -euo pipefail
[ -f "$HOME/.solana-dev-env.sh" ] && . "$HOME/.solana-dev-env.sh"
# Anchor 1.2.0 defaults to SBPF v3, but the LiteSVM version in Anchor's own
# litesvm test template (litesvm 0.10.0) cannot load v3 ELFs
# (add_program -> InvalidAccountData). Build SBPF v2 until LiteSVM is bumped.
export ANCHOR_BUILD_SBF_ARCH="${ANCHOR_BUILD_SBF_ARCH:-v2}"
cd "$(dirname "${BASH_SOURCE[0]}")/.."
# Pin the SBF platform-tools version for EVERY build (anchor defaults to v1.57, cargo-build-sbf to
# v1.54) and never (re)install tools, so our builds never re-link another team's rustup toolchain
# (QA uses v1.57 on the same box). Install once with: cargo build-sbf --install-only --tools-version v1.54
export SBF_TOOLS_VERSION="${SBF_TOOLS_VERSION:-v1.54}"
SBF_TOOLS_ARGS=(--tools-version "$SBF_TOOLS_VERSION")
if [ -d "$HOME/.cache/solana/$SBF_TOOLS_VERSION/platform-tools" ]; then SBF_TOOLS_ARGS+=(--skip-tools-install); fi
# Production program allowlist: the ONLY programs built into target/deploy and deployable.
PROD_PROGRAMS=(hybrid_launch hybrid_vault)
