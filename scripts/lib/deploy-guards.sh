#!/usr/bin/env bash
# Shared production-artifact guards (audit M-38). Sourced by build.sh, deploy-devnet.sh and
# test-deploy-guards.sh. Each function prints a reason and returns non-zero on failure.
GUARD_MARKERS=("TEST-ONLY" "mock graduation" "MOCKGRAD" "mock_switchboard" "test-mock-graduation")
SB_IDS=("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2" "SBondMDrcV3K4kxZR1HNVT7osZxAHVHgYXL5Ze1oMUv")

# guard_no_markers <so>: no test/mock marker anywhere in the binary.
guard_no_markers() {
  local m
  for m in "${GUARD_MARKERS[@]}"; do
    if grep -aq "$m" "$1"; then echo "$1 contains test/mock marker '$m'"; return 1; fi
  done
}
# guard_allowlisted <name>: the program is in PROD_PROGRAMS.
guard_allowlisted() {
  case " ${PROD_PROGRAMS[*]} " in *" $1 "*) return 0;; esac
  echo "$1 is not in the production allowlist (${PROD_PROGRAMS[*]})"; return 1
}
# guard_not_switchboard_id <pubkey>
guard_not_switchboard_id() {
  local id
  for id in "${SB_IDS[@]}"; do [ "$1" != "$id" ] || { echo "program id $1 is a Switchboard id"; return 1; }; done
}
# guard_devnet_switchboard <hybrid_vault.so>: compiled for the devnet Switchboard id, never mainnet.
guard_devnet_switchboard() {
  grep -aq "hybrid_vault:switchboard=devnet:Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhy" "$1" || { echo "$1 does not pin the devnet Switchboard id"; return 1; }
  if grep -aq "hybrid_vault:switchboard=mainnet" "$1"; then echo "$1 pins the MAINNET Switchboard id"; return 1; fi
}
