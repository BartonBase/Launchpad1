#!/usr/bin/env bash
# Build all Anchor programs (SBF .so + IDLs) into target/.
source "$(dirname "$0")/env.sh"
anchor build "$@"
