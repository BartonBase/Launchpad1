#!/usr/bin/env bash
# Self-test of the production-artifact guards (M-38). No network, no deploy.
#  - a stray target/deploy/mock_switchboard.so is removed by build.sh and never staged;
#  - the TEST build of hybrid_vault (mock graduation) and the mock Switchboard are REJECTED;
#  - the production build passes; non-allowlisted names and Switchboard ids are rejected.
source "$(dirname "$0")/env.sh"
source scripts/lib/deploy-guards.sh
set +u
ok=0; bad() { echo "FAIL: $*"; ok=1; }
echo junk > $TARGET/deploy/mock_switchboard.so
DRY_RUN=1 ./scripts/deploy-devnet.sh > /tmp/deploy-guards-dry.log 2>&1 || bad "dry run failed: $(tail -1 /tmp/deploy-guards-dry.log)"
[ ! -e $TARGET/deploy/mock_switchboard.so ] || bad "stray mock .so survived build.sh"
[ ! -e $TARGET/deploy-devnet/mock_switchboard.so ] || bad "mock staged for deploy"
[ "$(ls $TARGET/deploy-devnet/*.so | wc -l)" -eq 2 ] || bad "staging not exactly the allowlist"
[ -f $TARGET/test-sbf/hybrid_vault.so ] || ./scripts/build-test-sbf.sh >/dev/null
guard_no_markers $TARGET/test-sbf/hybrid_vault.so >/dev/null && bad "test hybrid_vault.so passed the marker guard"
guard_no_markers $TARGET/test-sbf/mock_switchboard.so >/dev/null && bad "mock_switchboard.so passed the marker guard"
guard_allowlisted mock_switchboard >/dev/null && bad "mock_switchboard allowlisted"
guard_not_switchboard_id Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2 >/dev/null && bad "Switchboard id accepted"
guard_no_markers $TARGET/deploy-devnet/hybrid_vault.so || bad "prod vault rejected"
guard_devnet_switchboard $TARGET/deploy-devnet/hybrid_vault.so || bad "prod vault not devnet-pinned"
[ $ok = 0 ] && echo "test-deploy-guards.sh: all guard checks passed"
exit $ok
