#!/usr/bin/env bash
# ONE devnet airdrop attempt for the throwaway devnet-only deployer key.
# Never loop this; devnet faucets rate-limit and that's fine.
source "$(dirname "$0")/env.sh"
KEY=.keys/devnet-only-deployer.json
solana airdrop 2 "$(solana-keygen pubkey "$KEY")" --url devnet || echo "airdrop failed/rate-limited (ok)"
