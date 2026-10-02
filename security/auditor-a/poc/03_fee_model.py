#!/usr/bin/env python3
"""Auditor A — PoC 03: model of the 2026-09-25 fee design (2% of ratio on capture and each re-roll,
paid in collection tokens to ONE fixed fee address, plus a fixed SOL minimum on re-rolls).
Off-chain model only. All market numbers (graduation FDV, premiums, LP depth) are EXAMPLE parameters,
not sourced. Integer fee math mirrors what hybrid_vault must do (u128, ceil, nonzero floor).
Run: python3 03_fee_model.py > 03_fee_model.out
"""
from fractions import Fraction
import math

SUPPLY = 1_000_000_000
FEE_BPS = 200
RATIOS = [10_000, 50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000]
MAXN = {r: SUPPLY // r for r in RATIOS}
VRF_COST = 0.002          # SOL, example Switchboard/ORAO cost per request (unsourced estimate)
checks = []
def check(name, ok):
    checks.append((name, ok)); print(f"  [{'PASS' if ok else 'FAIL'}] {name}")

print("=" * 100)
print("PART 1 — per-attempt cost and break-even rarity premium after graduation")
print("  floor F = R * FDV_grad / 1B (SOL). Attempt cost = 0.02*F + s (s = SOL minimum).")
print("  Expected attempts to hit a tier with pool share p ~ 1/p (ignores pool drift).")
print("  Break-even premium m* (in floors) = (0.02 + s/F) / p. Grinding is +EV iff market premium > m*.")
for fdv in [100, 400, 1500]:
    print(f"\n  Graduation FDV = {fdv} SOL (example)")
    print(f"  {'ratio':>9} {'floor SOL':>10} | {'s':>6} {'cost/att':>9} {'SOL share':>9} | "
          f"{'m* p=1%':>8} {'m* p=0.1%':>9} {'m* 1-of-maxN':>12} {'cost 1/1 (SOL)':>14}")
    for R in [10_000, 100_000, 1_000_000, 5_000_000]:
        F = R * fdv / SUPPLY
        for s in [0.0, 0.003, 0.01]:
            att = 0.02 * F + s
            share = s / att if att else 0
            p11 = 1 / MAXN[R]
            m = lambda p: (0.02 + s / F) / p
            print(f"  {R:>9,} {F:>10.4f} | {s:>6.3f} {att:>9.5f} {share:>8.0%} | "
                  f"{m(0.01):>8.1f} {m(0.001):>9.1f} {m(p11):>12,.0f} {att/p11:>14,.1f}")
F = 10_000 * 100 / SUPPLY
check("at R=10k, FDV=100 SOL the 2% token part is 0.00002 SOL/attempt (<1% of a 0.003 SOL min): SOL min dominates",
      0.02 * F < 0.01 * 0.003 * 100)

print("\n" + "=" * 100)
print("PART 2 — farming a whole tier: cost rises as the tier depletes (coupon-collector, pool = N)")
print("  expected attempts to take all c items of a tier from pool N ~ N * H_c; last item alone ~ N attempts")
for R, fdv, s in [(10_000, 400, 0.003), (1_000_000, 400, 0.003), (100_000, 400, 0.01)]:
    N = MAXN[R]; F = R * fdv / SUPPLY; att = 0.02 * F + s
    for c in sorted({1, 10, max(1, N // 1000)}):
        H = sum(1 / i for i in range(1, c + 1))
        tot = N * H * att
        print(f"  R={R:>9,} N={N:>7,} floor={F:.4f} s={s}: tier c={c:>4}: total {tot:>10,.1f} SOL "
              f"({tot/c:,.1f}/item avg, first {N/c*att:,.2f}, last {N*att:,.2f})")

print("\n" + "=" * 100)
print("PART 3 — INSIDER: the fee owner pays the 2% token fee to itself")
print("  If the fee address (or anyone it funds) re-rolls, the token fee round-trips to the same owner.")
print("  Its marginal cost per attempt = s_to_third_party + VRF cost only. If the SOL min ALSO goes to the fee")
print("  owner, the insider's marginal cost is VRF cost only.")
print("  Assumes the VRF cost (VRF_COST) is paid out of the SOL minimum s.")
for R, fdv in [(10_000, 400), (1_000_000, 400)]:
    N = MAXN[R]; F = R * fdv / SUPPLY
    for s in [0.003, 0.01]:
        pub = (0.02 * F + s) * N
        ins_owner = VRF_COST * N            # token fee + (s - VRF) both return to the owner
        ins_sink = s * N                    # token fee returns; s goes to a sink the owner doesn't control
        print(f"  R={R:>9,} s={s}: public cost per 1-of-{N:,} = {pub:>9,.1f} SOL | insider if SOL min -> owner: "
              f"{ins_owner:>7,.1f} SOL | insider if SOL min -> non-owner sink: {ins_sink:>7,.1f} SOL")
R = 1_000_000; N = MAXN[R]; F = R * 400 / SUPPLY
check("insider grind cost < 25% of public cost when SOL min goes to the fee owner (R=1M, s=0.003)",
      (VRF_COST * N) < 0.25 * ((0.02 * F + 0.003) * N))

print("\n" + "=" * 100)
print("PART 4 — release + capture as a substitute re-roll (bypass of the re-roll SOL minimum)")
for R, fdv in [(10_000, 400), (100_000, 400), (1_000_000, 400)]:
    F = R * fdv / SUPPLY
    for s in [0.003, 0.01]:
        rr = 0.02 * F + s
        rc = 0.02 * F + 0.0      # release free; capture = 2% only if capture has no SOL minimum
        print(f"  R={R:>9,} s={s}: re-roll {rr:.5f} SOL vs release+capture {rc:.5f} SOL "
              f"-> bypass saves {1 - rc/rr:.0%}")
check("without a capture SOL minimum >= re-roll SOL minimum, release+capture is cheaper than re-roll",
      0.02 * (10_000 * 400 / SUPPLY) < 0.02 * (10_000 * 400 / SUPPLY) + 0.003)

print("\n" + "=" * 100)
print("PART 5 — rounding: fee = 2% of ratio in base units")
def fee_floor(rb, bps=FEE_BPS): return (rb * bps) // 10_000
def fee_ceil(rb, bps=FEE_BPS): return -(-(rb * bps) // 10_000)
allexact = True
for R in RATIOS:
    for d in range(0, 10):
        rb = R * 10**d
        assert rb * FEE_BPS < 2**128
        if (rb * FEE_BPS) % 10_000: allexact = False
        assert fee_floor(rb) > 0
check("every allowed ratio x decimals 0..9: 2% is exact and > 0 (min = 200 tokens at R=10k, d=0)", allexact)
print("  Hypothetical ratios outside the allowed set (why ceil + nonzero floor must still be coded):")
for rb in [1, 25, 49, 50, 99, 12_345]:
    print(f"    ratio_base={rb:>6}: floor={fee_floor(rb):>4} ceil={fee_ceil(rb):>4}")
check("floor rounding gives a ZERO fee for ratio_base < 50 (free re-rolls) — ceil gives >= 1", fee_floor(49) == 0 and fee_ceil(49) == 1)
u64max = 2**64 - 1
check("u64 intermediate R_base*bps overflows for R=5M, d=9 only if bps > 3689 (safe at 200, but compute in u128)",
      5_000_000 * 10**9 * 200 < u64max and 5_000_000 * 10**9 * 3690 > u64max)

print("\n" + "=" * 100)
print("PART 6 — tokens accrued at the fee address (sell-pressure disclosure)")
print("  fee tokens = 2% * R * (captures + re-rolls). As % of 1B supply for a max-size collection (N*R = 1B):")
for k in [0, 1, 3, 10]:
    captures = 1.0   # each NFT captured once (full collection)
    share = 0.02 * (captures + k)
    print(f"    full capture + {k:>2} re-rolls per NFT on average -> {share:.0%} of total supply sent to the fee address")
print("  Price impact of the fee address selling X tokens into a constant-product pool holding T_lp tokens:")
for T in [100_000_000, 200_000_000]:
    for X in [10_000_000, 20_000_000, 60_000_000]:
        pi = 1 - (T / (T + X)) ** 2
        print(f"    LP tokens {T/1e6:>4.0f}M, sell {X/1e6:>3.0f}M -> spot price -{pi:.1%}")
check("a full capture of a max-size collection alone routes 20,000,000 tokens (2% of supply) to one address",
      abs(0.02 * SUPPLY - 20_000_000) < 1)

print("\n" + "=" * 100)
print("PART 7 — SOL minimum vs rent-exempt minimum for the SOL leg")
RENT_MIN_SYSTEM = 890_880   # lamports, 0-data system account (current Rent defaults)
for s_l in [500_000, 890_880, 3_000_000]:
    ok = s_l >= RENT_MIN_SYSTEM
    print(f"  SOL min {s_l/1e9:.6f} SOL: a lamport transfer to an EMPTY system-account fee wallet "
          f"{'succeeds' if ok else 'FAILS (InsufficientFundsForRent) -> re-roll DoS until someone funds it'}")
check("a SOL minimum below 890,880 lamports can DoS re-rolls if the SOL receiver is emptied", 500_000 < RENT_MIN_SYSTEM)

print("\n" + "=" * 100)
print(f"SUMMARY: {sum(ok for _, ok in checks)}/{len(checks)} checks PASS")
