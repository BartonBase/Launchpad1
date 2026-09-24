#!/usr/bin/env bash
# Shared environment for launchpad program scripts. LOCALNET/DEVNET ONLY.
set -euo pipefail
[ -f "$HOME/.solana-dev-env.sh" ] && . "$HOME/.solana-dev-env.sh"
# Anchor 1.2.0 defaults to SBPF v3, but the LiteSVM version in Anchor's own
# litesvm test template (litesvm 0.10.0) cannot load v3 ELFs
# (add_program -> InvalidAccountData). Build SBPF v2 until LiteSVM is bumped.
export ANCHOR_BUILD_SBF_ARCH="${ANCHOR_BUILD_SBF_ARCH:-v2}"
cd "$(dirname "${BASH_SOURCE[0]}")/.."
