#!/usr/bin/env bash
# Devnet deploy with hard guards (audit M-38). NEVER deploys test builds or mocks.
#  1. Refuses unless the cluster is devnet (mainnet deploys are a separate, multisig-only process).
#  2. Rebuilds production binaries with `anchor build` (no test features) and refuses any .so that
#     contains a TEST-ONLY marker or mock symbol, or that came from target/test-sbf.
#  3. Deploys ONLY the allowlist PROD_PROGRAMS (env.sh), staged from a fresh production build into
#     target/deploy-devnet/. Nothing else in target/deploy is ever deployed, so a stray
#     mock_switchboard.so can't ship; a non-allowlisted program, or one whose id is a Switchboard id, is refused.
#  4. Checks the Switchboard program id compiled in is the DEVNET one (pinned per cluster).
#  5. Sets the upgrade authority to UPGRADE_AUTHORITY (a Squads multisig vault; placeholder on devnet)
#     and prints the checklist for Squads 3-of-5 + 7-day timelock (mainnet requirement).
# Usage: UPGRADE_AUTHORITY=<pubkey> DEPLOYER_KEYPAIR=.keys/devnet-deployer.json scripts/deploy-devnet.sh
set -euo pipefail
source "$(dirname "$0")/env.sh"
source scripts/lib/deploy-guards.sh
set +u
ROOT=$(pwd)
: "${UPGRADE_AUTHORITY:=${DRY_RUN:+DRYRUN}}"; : "${UPGRADE_AUTHORITY:?set UPGRADE_AUTHORITY (Squads vault pubkey; throwaway placeholder on devnet)}"
: "${DEPLOYER_KEYPAIR:=${DRY_RUN:+DRYRUN}}"; : "${DEPLOYER_KEYPAIR:?set DEPLOYER_KEYPAIR (throwaway devnet key under .keys/)}"
URL="https://api.devnet.solana.com"

fail() { echo "deploy-devnet.sh: REFUSING: $*" >&2; exit 1; }

# DRY_RUN=1: run every build/guard step, skip the cluster check and the deploy (guard self-test).
if [ "${DRY_RUN:-0}" != 1 ]; then
genesis=$(solana genesis-hash --url "$URL")
[ "$genesis" = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG" ] || fail "cluster at $URL is not devnet (genesis $genesis)"
fi

./scripts/build.sh   # production allowlist only; refuses non-allowlisted artifacts and markers
STAGE=target/deploy-devnet
rm -rf "$STAGE"; mkdir -p "$STAGE"
for p in "${PROD_PROGRAMS[@]}"; do
  so="target/deploy/$p.so"; kp="target/deploy/$p-keypair.json"
  [ -f "$so" ] && [ -f "$kp" ] || fail "$so or its keypair missing"
  r=$(guard_allowlisted "$p") || fail "$r"
  r=$(guard_no_markers "$so") || fail "$r"
  if [ -f "target/test-sbf/$p.so" ] && cmp -s "$so" "target/test-sbf/$p.so"; then fail "$so is identical to the TEST build"; fi
  r=$(guard_not_switchboard_id "$(solana address -k "$kp")") || fail "$p: $r"
  cp "$so" "$kp" "$STAGE/"
done
[ "$(ls "$STAGE"/*.so | wc -l)" -eq "${#PROD_PROGRAMS[@]}" ] || fail "staging holds unexpected artifacts"

# Pubkey constants are inlined as immediates, so the cluster is verified through the marker static
# HYBRID_VAULT_SWITCHBOARD_CLUSTER (constants.rs): devnet, never mainnet.
r=$(guard_devnet_switchboard "$STAGE/hybrid_vault.so") || fail "$r"

if [ "${DRY_RUN:-0}" = 1 ]; then echo "DRY_RUN: guards passed; would deploy: $(ls $STAGE/*.so | xargs -n1 basename | tr '\n' ' ')"; exit 0; fi
for p in "${PROD_PROGRAMS[@]}"; do
  solana program deploy --url "$URL" --keypair "$DEPLOYER_KEYPAIR" \
    --program-id "$STAGE/$p-keypair.json" "$STAGE/$p.so"
  solana program set-upgrade-authority --url "$URL" --keypair "$DEPLOYER_KEYPAIR" \
    "$(solana address -k $STAGE/$p-keypair.json)" --new-upgrade-authority "$UPGRADE_AUTHORITY" --skip-new-upgrade-authority-signer-check
done
cat <<TXT
Deployed. Upgrade authority -> $UPGRADE_AUTHORITY.
Mainnet checklist (docs/admin-multisig-timelock.md): upgrade authority is a Squads v4 vault with
threshold 3-of-5 and a 7-day time lock; verify with 'solana program show <id>' and the Squads UI.
TXT
