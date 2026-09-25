#!/usr/bin/env bash
# Build (SBPF v2), start an ISOLATED local validator, deploy, and run all tests. Localnet only.
#
# Several agents share this machine, so we never use the default validator ports (8899/8900/9900/
# 8000-8020) or a shared ledger: a free block of ports is picked at random each run and the ledger
# lives in a unique temp dir that is deleted on exit. The Rust tests themselves run in LiteSVM.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/env.sh
# Rust toolchain from rustup (~/.cargo/bin) must win over any other cargo on PATH.
export PATH="$HOME/.cargo/bin:$PATH"

PORTS=$(python3 - <<'PY'
import random, socket
def free(p):
    for kind in (socket.SOCK_STREAM, socket.SOCK_DGRAM):
        s = socket.socket(socket.AF_INET, kind)
        try:
            s.bind(("127.0.0.1", p))
        except OSError:
            return False
        finally:
            s.close()
    return True
for _ in range(500):
    base = random.randrange(20000, 59000)
    if all(free(p) for p in range(base, base + 64)):
        print(base)
        break
else:
    raise SystemExit("no free 64-port block found")
PY
)
RPC_PORT=$PORTS                 # websocket = RPC_PORT + 1
FAUCET_PORT=$((PORTS + 2))
GOSSIP_PORT=$((PORTS + 3))
DYN_RANGE="$((PORTS + 4))-$((PORTS + 63))"
LEDGER=$(mktemp -d "${TMPDIR:-/tmp}/launchpad-test-ledger.XXXXXX")
WALLET=".keys/devnet-only-deployer.json"
# Build first, then load every [programs.localnet] program at genesis (like Anchor's own
# `upgradeable = false` validator; the 4.x loader refuses to *deploy* SBPF v2 ELFs).
if [[ " $* " != *" --skip-build "* ]]; then ./scripts/build.sh && ./scripts/build-test-sbf.sh; fi
mapfile -t BPF_ARGS < <(python3 - <<'PY'
import tomllib
progs = tomllib.load(open("Anchor.toml", "rb"))["programs"]["localnet"]
for name, pid in progs.items():
    print("--bpf-program"); print(pid); print(f"target/deploy/{name}.so")
PY
)
[ -f "$WALLET" ] || solana-keygen new --no-bip39-passphrase --silent -o "$WALLET"

solana-test-validator --reset --quiet \
  --ledger "$LEDGER" --bind-address 127.0.0.1 \
  --rpc-port "$RPC_PORT" --faucet-port "$FAUCET_PORT" --gossip-port "$GOSSIP_PORT" \
  --dynamic-port-range "$DYN_RANGE" "${BPF_ARGS[@]}" \
  --mint "$(solana-keygen pubkey "$WALLET")" >"$LEDGER/validator.out" 2>&1 &
VALIDATOR_PID=$!
cleanup() { kill "$VALIDATOR_PID" 2>/dev/null || true; wait "$VALIDATOR_PID" 2>/dev/null || true; rm -rf "$LEDGER"; }
trap cleanup EXIT

URL="http://127.0.0.1:$RPC_PORT"
for _ in $(seq 1 120); do
  solana cluster-version -u "$URL" >/dev/null 2>&1 && break
  kill -0 "$VALIDATOR_PID" 2>/dev/null || { cat "$LEDGER/validator.out"; exit 1; }
  sleep 0.5
done
echo "test.sh: isolated validator on $URL (gossip $GOSSIP_PORT, faucet $FAUCET_PORT, dynamic $DYN_RANGE, ledger $LEDGER)"

anchor test --skip-local-validator --skip-deploy --skip-build --provider.cluster "$URL" --provider.wallet "$WALLET" "$@"
