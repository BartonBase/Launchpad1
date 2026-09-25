#!/usr/bin/env bash
# Devnet deploy with hard guards (audit M-38). NEVER deploys test builds or mocks.
#  1. Refuses unless the cluster is devnet (mainnet deploys are a separate, multisig-only process).
#  2. Rebuilds production binaries with `anchor build` (no test features) and refuses any .so that
#     contains a TEST-ONLY marker or mock symbol, or that came from target/test-sbf.
#  3. Refuses to deploy the mock Switchboard program at all.
#  4. Checks the Switchboard program id compiled in is the DEVNET one (pinned per cluster).
#  5. Sets the upgrade authority to UPGRADE_AUTHORITY (a Squads multisig vault; placeholder on devnet)
#     and prints the checklist for Squads 3-of-5 + 7-day timelock (mainnet requirement).
# Usage: UPGRADE_AUTHORITY=<pubkey> DEPLOYER_KEYPAIR=.keys/devnet-deployer.json scripts/deploy-devnet.sh
set -euo pipefail
source "$(dirname "$0")/env.sh"
set +u
ROOT=$(pwd)
: "${UPGRADE_AUTHORITY:?set UPGRADE_AUTHORITY (Squads vault pubkey; throwaway placeholder on devnet)}"
: "${DEPLOYER_KEYPAIR:?set DEPLOYER_KEYPAIR (throwaway devnet key under .keys/)}"
URL="https://api.devnet.solana.com"

fail() { echo "deploy-devnet.sh: REFUSING: $*" >&2; exit 1; }

genesis=$(solana genesis-hash --url "$URL")
[ "$genesis" = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG" ] || fail "cluster at $URL is not devnet (genesis $genesis)"

anchor build
MARKERS=("TEST-ONLY" "mock graduation" "MOCKGRAD" "mock_switchboard" "test-mock-graduation")
for so in target/deploy/hybrid_launch.so target/deploy/hybrid_vault.so; do
  [ -f "$so" ] || fail "$so missing"
  for m in "${MARKERS[@]}"; do
    if grep -aq "$m" "$so"; then fail "$so contains test/mock marker '$m'"; fi
  done
done
[ ! -f target/deploy/mock_switchboard.so ] || fail "target/deploy/mock_switchboard.so exists; mocks are built only into target/test-sbf"

SB_DEVNET="Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2"
SB_MAINNET="SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv"
python3 - "$SB_DEVNET" "$SB_MAINNET" <<'PY' || fail "hybrid_vault.so does not pin the devnet Switchboard id"
import sys
def b58d(s):
    a='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'; n=0
    for c in s: n=n*58+a.index(c)
    return n.to_bytes(32,'big')
so=open('target/deploy/hybrid_vault.so','rb').read()
dev,main=b58d(sys.argv[1]),b58d(sys.argv[2])
sys.exit(0 if (dev in so and main not in so) else 1)
PY

for p in hybrid_launch hybrid_vault; do
  solana program deploy --url "$URL" --keypair "$DEPLOYER_KEYPAIR" \
    --program-id "target/deploy/$p-keypair.json" "target/deploy/$p.so"
  solana program set-upgrade-authority --url "$URL" --keypair "$DEPLOYER_KEYPAIR" \
    "$(solana address -k target/deploy/$p-keypair.json)" --new-upgrade-authority "$UPGRADE_AUTHORITY" --skip-new-upgrade-authority-signer-check
done
cat <<TXT
Deployed. Upgrade authority -> $UPGRADE_AUTHORITY.
Mainnet checklist (docs/admin-multisig-timelock.md): upgrade authority is a Squads v4 vault with
threshold 3-of-5 and a 7-day time lock; verify with 'solana program show <id>' and the Squads UI.
TXT
