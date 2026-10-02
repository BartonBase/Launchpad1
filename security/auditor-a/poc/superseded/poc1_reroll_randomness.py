#!/usr/bin/env python3
"""
PoC-1 (off-chain, executed): MPL-Hybrid capture_v2 metadata "reroll" randomness.
Faithful re-implementation of
  programs/mpl-hybrid/src/instructions/capture_v2.rs:189-203  (and capture.rs:170-183, same math)
at pinned commit aacf1a53c8a43bf395db7b562c02cf016ce604c5:

    let most_recent = array_ref![data, 12, 8];           // SlotHashes sysvar bytes 12..20
    let seed = u64::from_le_bytes(*most_recent)
        .saturating_sub(clock.unix_timestamp as u64)
        .wrapping_mul(recipe.count);
    let remainder = seed.checked_rem(recipe.max - recipe.min)? + recipe.min;
    uri = recipe.uri + remainder + ".json"

SlotHashes sysvar layout (bincode Vec<(Slot, Hash)>): [u64 len][u64 slot][32-byte hash]...
=> bytes 12..20 = slot.to_le_bytes()[4..8] ++ hash[0..4]
=> u64 value = (u32_le(hash[0..4]) << 32) | (slot >> 32)       (slot >> 32 == 0 today)
Every input (SlotHashes, Clock, recipe.count) is readable by any program in the same
transaction, so the outcome is known before the capture CPI executes.
No network access; pure arithmetic.
"""
import os, random, struct, collections

U64 = (1 << 64) - 1

def slot_hashes_bytes(slot: int, h: bytes, n_entries: int = 512) -> bytes:
    # most recent entry first; only the first entry matters for bytes 12..20
    out = struct.pack("<Q", n_entries) + struct.pack("<Q", slot) + h
    return out

def mplh_reroll(sysvar: bytes, unix_ts: int, count: int, mn: int, mx: int) -> int:
    most_recent = struct.unpack("<Q", sysvar[12:20])[0]
    seed = max(most_recent - unix_ts, 0)            # saturating_sub
    seed = (seed * count) & U64                     # wrapping_mul
    rng = mx - mn
    if rng == 0:
        raise ZeroDivisionError("RandomnessError")  # checked_rem(0) -> None
    return seed % rng + mn

def check(name, cond):
    print(("PASS " if cond else "FAIL ") + name)
    assert cond

def main():
    rnd = random.Random(1337)
    slot = 330_000_000            # realistic 2026 slot, < 2^32
    ts = 1_790_000_000            # 2026-09 unix time

    # --- 1. Layout: bytes 12..20 = high half of slot (0) ++ first 4 byt
