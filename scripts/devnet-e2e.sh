#!/usr/bin/env bash
# DEVNET ONLY: upgrade hybrid_launch to the devnet-e2e build, then run the full end-to-end
# (DBC pool on the platform config -> register -> init_vault -> buys to 0.1 SOL -> DAMM v2 migration
# -> leftover to the buffer -> open_vault -> init_randomness -> request_capture -> third-party
# gateway reveal -> settle_capture with mint). hybrid_vault is NOT touched (its code is unchanged).
#
# Usage: DEPLOYER_KEYPAIR=.keys/devnet-only-deployer.json scripts/devnet-e2e.sh
#   SKIP_UPGRADE=1   run only the e2e (program already upgraded)
#   DRY_RUN=1        build + guards + budget check only; no transactions
# Budget (devnet rent, measured 2026-09-25 unless marked est.): peak ~1.35 SOL, of which ~1.124 SOL is
# the upgrade buffer and comes back when the upgrade closes it; net spend ~0.2 SOL.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
URL="https://api.devnet.solana.com"
PROGRAM_ID=9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8
: "${DEPLOYER_KEYPAIR:?set DEPLOYER_KEYPAIR (throwaway devnet key under .keys/)}"
fail() { echo "devnet-e2e.sh: REFUSING: $*" >&2; exit 1; }
[ "$(solana genesis-hash --url "$URL")" = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG" ] || fail "not devnet"
DEPLOYER=$(solana address -k "$DEPLOYER_KEYPAIR")
bal() { solana balance --url "$URL" --lamports "$1" | awk '{print $1}'; }

if [ "${SKIP_UPGRADE:-0}" != 1 ]; then
  ./scripts/build-devnet-e2e.sh
  SO=target/devnet-e2e/deploy/hybrid_launch.so
  source scripts/lib/deploy-guards.sh
  r=$(guard_no_markers "$SO") || fail "$r"
  [ "$(solana address -k target/devnet-e2e/deploy/hybrid_launch-keypair.json)" = "$PROGRAM_ID" ] || fail "program id mismatch"
  AUTH=$(solana program show --url "$URL" "$PROGRAM_ID" | awk '/^Authority:/{print $2}')
  [ "$AUTH" = "$DEPLOYER" ] || fail "upgrade authority is $AUTH, not the deployer $DEPLOYER"
  CUR=$(solana program show --url "$URL" "$PROGRAM_ID" | awk '/^Data Length:/{print $3}')
  NEW=$(stat -c %s "$SO"); EXTRA=$(( NEW > CUR ? NEW - CUR : 0 ))
  BUF=$(solana rent --url "$URL" --lamports $((NEW + 45)) | awk '/Rent-exempt minimum/{print $3}')
  NEED=$(( BUF + 250000000 ))   # buffer (refunded) + e2e run (~0.2 SOL) + margin
  HAVE=$(bal "$DEPLOYER")
  echo "upgrade: program $CUR B -> $NEW B (extend $EXTRA B); buffer rent $BUF lamports (refunded); need ~$NEED, have $HAVE"
  [ "$HAVE" -ge "$NEED" ] || fail "short by $((NEED - HAVE)) lamports ($(awk "BEGIN{print ($NEED-$HAVE)/1e9}") SOL)"
  if [ "${DRY_RUN:-0}" = 1 ]; then echo "DRY_RUN: guards and budget OK; no transactions sent"; exit 0; fi
  # ExtendProgram needs >= 10,240 extra bytes (loader rule); extend by that when any growth is needed.
  [ "$EXTRA" -eq 0 ] || solana program extend --url "$URL" --keypair "$DEPLOYER_KEYPAIR" "$PROGRAM_ID" $(( EXTRA > 10240 ? EXTRA : 10240 ))
  # Upgrade in place (write buffer + upgrade; the CLI closes the buffer and refunds its rent).
  solana program deploy --url "$URL" --keypair "$DEPLOYER_KEYPAIR" --upgrade-authority "$DEPLOYER_KEYPAIR" \
    --program-id "$PROGRAM_ID" "$SO"
  # Reclaim any buffer left by an interrupted write.
  solana program close --buffers --url "$URL" --keypair "$DEPLOYER_KEYPAIR" --bypass-warning || true
fi
[ "${DRY_RUN:-0}" = 1 ] && { echo "DRY_RUN: skipping the e2e run"; exit 0; }
( cd scripts/devnet-e2e && [ -d node_modules ] || npm ci --no-audit --no-fund )
DEPLOYER_KEYPAIR="$DEPLOYER_KEYPAIR" node scripts/devnet-e2e/e2e.cjs
echo "deployer balance after: $(bal "$DEPLOYER") lamports; results: .keys/devnet-e2e/run.json"
