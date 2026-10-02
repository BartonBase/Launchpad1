#!/usr/bin/env python3
"""
MPL-Hybrid reroll economics + predictability demo (design-level, no chain access).

Part A: EV of "reroll until rare" when the metadata pick is *honestly random*.
Part B: Re-implementation of the MPL-Hybrid capture-time metadata pick, transcribed from
        metaplex-foundation/mpl-hybrid @ aacf1a53 (programs/mpl-hybrid/src/instructions/capture.rs
        lines 169-183, identical logic in capture_v2.rs):

            most_recent = data[12..20] of the SlotHashes sysvar       (array_ref![data, 12, 8])
            seed  = u64_le(most_recent).saturating_sub(unix_timestamp).wrapping_mul(escrow.count)
            index = seed % (max - min) + min

        Every input is public before the capture transaction executes (SlotHashes' newest entry
        is the *parent* slot, unix_timestamp is the bank clock, escrow.count is account state), so
        a bot can compute the index off-chain, or on-chain in a wrapper instruction that aborts
        unless the pick is rare. Inside a Jito bundle a failed attempt never lands, so it costs nothing.

        SlotHashes layout (bincode Vec<(u64 slot, [u8;32] hash)>): bytes 0..8 = vec length,
        8..16 = newest slot number, 16..48 = newest hash. So data[12..20] = the HIGH 4 bytes of the
        slot number (zero while slot < 2^32) followed by the first 4 bytes of the hash.
        As a little-endian u64 that is  hash[0..4] << 32  -- only 32 bits of entropy, and when
        (max - min) divides 2^32 the hash term vanishes entirely (index depends only on
        timestamp and count). This is OUR READING of the source; a PoC test must confirm it.

Fees at that commit (constants.rs / capture.rs / release.rs):
  capture: escrow.amount tokens -> escrow, escrow.fee_amount tokens -> project,
           protocol fee (rent(590 bytes)+2720 lamports = 0.005 SOL), escrow.sol_fee_amount SOL
  release: protocol fee 0.005 SOL + escrow.sol_fee_amount SOL; user receives escrow.amount tokens
All other numbers below are illustrative assumptions; edit them to match the real design.
"""
import random
import struct

random.seed(4242)
LAMPORTS = 1_000_000_000

# ------------------------------------------------------------------ Part A parameters
PROTOCOL_FEE_SOL = ((128 + 590) * 3480 * 2 + 2720) / LAMPORTS   # = 0.005 SOL (computed)
PROJECT_SOL_FEE = 0.01          # escrow.sol_fee_amount per swap leg (assumption)
SWAP_AMOUNT_TOKENS = 1_000_000  # escrow.amount (assumption)
CAPTURE_FEE_TOKENS = 20_000     # escrow.fee_amount, 2% of swap amount (assumption)
TOKEN_PRICE_SOL = 0.000_001     # 1 token = 1e-6 SOL => swap amount worth 1 SOL (assumption)
T22_FEE_BPS = 100               # IF the design forks MPL-Hybrid to use the Token-2022 fee mint
TX_FEES_SOL = 2 * 0.000_05      # base + modest priority fee, two txs (assumption)

# rarity table: (name, probability, market value in SOL). Common value ~= swap value (floor is
# pinned to the swap rate by wrap/unwrap arbitrage).
RARITY = [
    ("legendary", 0.001, 60.0),
    ("epic",      0.010, 6.0),
    ("rare",      0.050, 2.0),
    ("common",    0.939, 1.0),
]


def cycle_cost_sol(t22_fee_bps=0):
    """One reroll = release (NFT -> tokens) + capture (tokens -> NFT with new random metadata)."""
    sol = 2 * PROTOCOL_FEE_SOL + 2 * PROJECT_SOL_FEE + TX_FEES_SOL
    tokens = CAPTURE_FEE_TOKENS
    # Token-2022 transfer fee applies to each token leg: escrow->user on release, user->escrow on capture
    tokens += 2 * SWAP_AMOUNT_TOKENS * t22_fee_bps / 10_000
    return sol + tokens * TOKEN_PRICE_SOL


def part_a():
    print("=== Part A: reroll-until-rare EV with HONEST randomness ===")
    print(f"protocol fee per leg = {PROTOCOL_FEE_SOL:.6f} SOL (computed from mpl-hybrid constants.rs)")
    for label, bps in [("classic SPL token (MPL-Hybrid as shipped)", 0),
                       (f"hypothetical Token-2022 fork, {T22_FEE_BPS} bps fee", T22_FEE_BPS)]:
        c = cycle_cost_sol(bps)
        print(f"\n-- {label}: cost per reroll cycle = {c:.4f} SOL")
        print(f"{'target':10s} {'p':>7s} {'E[attempts]':>12s} {'E[cost] SOL':>12s} "
              f"{'value SOL':>10s} {'EV SOL':>9s}  verdict")
        common_value = RARITY[-1][2]
        for name, p, v in RARITY[:-1]:
            e_att = 1 / p
            e_cost = c * e_att
            ev = (v - common_value) - e_cost
            print(f"{name:10s} {p:7.3%} {e_att:12.0f} {e_cost:12.2f} {v:10.1f} {ev:9.2f}  "
                  f"{'PROFITABLE to grind' if ev > 0 else 'not profitable'}")
        # breakeven rarity premium for legendary
        p_leg = RARITY[0][1]
        print(f"   breakeven: grinding legendary is profitable iff premium over floor > {c / p_leg:.2f} SOL")
    # probability of success within n attempts
    p = RARITY[0][1]
    print("\nP(at least one legendary within n honest rerolls), p=0.1%:")
    for n in [100, 693, 1000, 3000]:
        print(f"   n={n:5d}: {1 - (1 - p) ** n:.2%}")


# ------------------------------------------------------------------ Part B: predictability
def slot_hashes_bytes(slot, h):
    """Minimal SlotHashes prefix: len=1, then (slot, hash)."""
    return struct.pack("<Q", 1) + struct.pack("<Q", slot) + h


def mpl_hybrid_pick(sysvar_data, unix_ts, count, mn, mx):
    most_recent = struct.unpack("<Q", sysvar_data[12:20])[0]
    seed = max(0, most_recent - unix_ts)                  # saturating_sub
    seed = (seed * count) & 0xFFFF_FFFF_FFFF_FFFF         # wrapping_mul
    return seed % (mx - mn) + mn


# ---- [auditor-b fix 2026-09-24] The file was truncated here (no return, no part_b, no main), so it
# ---- printed nothing. Completed minimally below, following the docstring. Part B is a model of OUR
# ---- READING of MPL-Hybrid's pick, not a verified PoC.
def part_b():
    print("\n=== Part B: predictability of the MPL-Hybrid capture-time pick (model of our reading) ===")
    slot = 300_000_000          # < 2^32, so high 4 bytes of the slot number are zero
    ts = 1_790_000_000          # example unix timestamp
    count = 7                   # example escrow.count

    # B1: entropy check -- vary only the slot hash, keep ts and count fixed
    for coll in (10_000, 1_024, 4_096):
        picks = set()
        for _ in range(5_000):
            h = random.getrandbits(256).to_bytes(32, "little")
            picks.add(mpl_hybrid_pick(slot_hashes_bytes(slot, h), ts, count, 0, coll))
        note = "(max-min) divides 2^32 -> hash term vanishes" if (1 << 32) % coll == 0 else ""
        print(f"collection range {coll:6d}: distinct picks over 5,000 random slot hashes = "
              f"{len(picks):5d}  {note}")

    # B2: the pick is a pure function of public inputs, so a bot can decide before the tx lands
    h = random.getrandbits(256).to_bytes(32, "little")
    data = slot_hashes_bytes(slot, h)
    a = mpl_hybrid_pick(data, ts, count, 0, 10_000)
    b = mpl_hybrid_pick(data, ts, count, 0, 10_000)
    print(f"same public inputs -> same pick: {a} == {b}: {a == b}")

    # B3: conditional-capture bot. Legendary = indices 0..9 of 10,000 (0.1%). Each new slot gives a
    # new parent hash; the bot only lands a capture when the predicted index is legendary.
    coll, legendary = 10_000, 10
    trials, waits = 2_000, []
    for _ in range(trials):
        n = 0
        while True:
            n += 1
            hh = random.getrandbits(256).to_bytes(32, "little")
            t = ts + n * 4 // 10             # ~400 ms slots
            if mpl_hybrid_pick(slot_hashes_bytes(slot + n, hh), t, count, 0, coll) < legendary:
                break
        waits.append(n)
    mean_wait = sum(waits) / trials
    c = cycle_cost_sol(0)
    print(f"conditional bot: mean slots waited for a legendary pick = {mean_wait:.0f} "
          f"(~{mean_wait * 0.4 / 60:.1f} min at 400 ms/slot)")
    print(f"   landed swap cycles needed = 1 -> cost ~= {c:.4f} SOL (+ wrapper CU / Jito tip), "
          f"vs honest grinding E[cost] = {c / 0.001:.2f} SOL")
    print(f"   cost reduction factor ~= {1 / 0.001:.0f}x; legendary EV at value 60 SOL = "
          f"{60.0 - 1.0 - c:.2f} SOL per grab")


if __name__ == "__main__":
    part_a()
    part_b()
