#!/usr/bin/env python3
"""PoC 04 (model only, no chain): Barton's 4:49 PM MT 2026-09-25 flat SOL fee model.
No token fee. Each capture / release / re-roll pays a flat s SOL (default 0.01) to the fee wallet.
Same framework as Auditor B's sim/fee_stress.py (Part E) so the numbers are comparable:
  R = tokens per NFT (the "ratio"), N = 1B / R = max collection size, F = floor = R x FDV / 1B (SOL),
  expected draws to land a specific 1-of-N ~ N, break-even premium m* = N x (s + TX) / F  (in floors).
FDV values are EXAMPLES, not forecasts. TX = 2 x 0.00005 SOL (priority fee incl.) as in B's sim.
VRF cost per draw is an UNVERIFIED example parameter (Switchboard/ORAO pricing not confirmed here)."""
TX = 0.0001
FDVS = [100.0, 410.0, 1500.0]
RATIOS = [10_000, 50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000]
def F(R, fdv): return R * fdv / 1e9
results = {}
def check(n, c, d=""): results[n] = bool(c); print(("PASS " if c else "FAIL ") + n + (" :: " + d if d else ""))

print("A. 1/1 break-even premium m* (floors) for an outsider, flat fee s per re-roll, no token fee")
for s in (0.005, 0.01):
    print(f"\n  s = {s} SOL")
    print(f"  {'R':>10} {'N':>7} | " + " ".join(f"{'F@'+str(int(f)):>9} {'m*@'+str(int(f)):>10}" for f in FDVS))
    for R in RATIOS:
        N = 1e9 / R
        cells = []
        for f in FDVS:
            fl = F(R, f); cells.append(f"{fl:9.4f} {N*(s+TX)/fl:10.1f}")
        print(f"  {R:>10,} {int(N):>7,} | " + " ".join(cells))
# at FDV 410 compare with B's 2%+0.005 numbers
b = {10_000:126390.2, 100_000:1443.9, 500_000:89.8, 1_000_000:32.4, 5_000_000:4.5}
print("\n  vs B's D6 table (2% token fee + s=0.005, FDV 410):")
for R, bm in b.items():
    N = 1e9 / R; fl = F(R, 410); m = N * (0.01 + TX) / fl
    print(f"   R={R:>9,}: B {bm:>10.1f}x  ->  flat 0.01 {m:>10.1f}x  (change {m/bm:5.2f}x)")
m5 = 200 * (0.01 + TX) / F(5_000_000, 410)
check("A1_flat_fee_R5M_breakeven_thin", m5 < 5, f"R=5M FDV410 m*={m5:.2f}x: a 1/1 worth >~{m5:.1f} floors is +EV to grind")
m5_1500 = 200 * (0.01 + TX) / F(5_000_000, 1500)
check("A2_high_fdv_R5M_breakeven_near_1x", m5_1500 < 1.5, f"R=5M FDV1500 m*={m5_1500:.2f}x floor")
m10k = 100_000 * (0.01 + TX) / F(10_000, 100)
check("A3_R10k_grind_prohibitive", m10k > 1e5, f"R=10k FDV100 m*={m10k:,.0f}x")

print("\nB. Release economics: fee as a share of the value returned (F), per release and per round trip")
print(f"  {'R':>10} | " + " ".join(f"{'fee/F@'+str(int(f)):>11}" for f in FDVS) + " | round-trip capture+release (0.02) / F @100")
trap = []
for R in RATIOS:
    row = [0.01 / F(R, f) for f in FDVS]
    rt = 0.02 / F(R, 100)
    print(f"  {R:>10,} | " + " ".join(f"{x:10.1%} " for x in row) + f" | {rt:8.1%}")
    if row[0] >= 1: trap.append(R)
check("B1_release_uneconomic_at_low_ratio_low_fdv", len(trap) > 0,
      f"at FDV 100 the 0.01 SOL release fee >= value of N tokens for R in {trap} (release is a soft trap)")
# fee exceeds value -> holder can still sell the NFT on Tensor/ME instead; disclose.

print("\nC. Insider (fee wallet owner) re-roll cost: fee returns to self, pays only TX + VRF")
for vrf in (0.0, 0.001, 0.002):
    m = 1000 * (TX + vrf) / F(1_000_000, 410)
    print(f"  VRF cost {vrf} SOL (example): insider 1/1 m* at R=1M FDV410 = {m:.2f}x (outsider {1000*(0.01+TX)/F(1_000_000,410):.1f}x)")
check("C1_insider_discount_large", (1000*(0.01+TX)) / (1000*(TX+0.001)) > 9, "insider pays ~1/9 of outsider per draw at VRF 0.001")

print("\nD. Program-funded VRF budget drain (only if the program pays VRF from a budget)")
for vrf in (0.001, 0.002, 0.005, 0.02):
    net = 0.01 - vrf
    print(f"  VRF {vrf} SOL/draw vs fee 0.01 -> attacker cost {0.01+TX:.4f}/draw, budget drain {vrf:.4f}/draw; fee covers VRF: {net>=0}")
check("D1_fee_must_cover_vrf_if_budget_funded", all(0.01 - v >= 0 for v in (0.001, 0.002, 0.005)) and (0.01 - 0.02) < 0,
      "If the fee goes to Barton and VRF is paid from a program budget, every draw drains the budget by the VRF cost; "
      "requester must pay VRF directly (Switchboard reward escrow funded by the requester) or the fee split must cover it")

print("\nE. Rent-exempt edge for a fresh fee wallet (system transfer to a 0-lamport account)")
RENT0 = 890_880
for s in (0.001, 0.005, 0.01):
    lam = int(s * 1e9)
    print(f"  s={s} SOL = {lam:,} lamports: first transfer to an empty wallet {'OK' if lam >= RENT0 else 'FAILS (InsufficientFundsForRent)'}")
check("E1_0.01_clears_rent_floor", 10_000_000 >= RENT0, "0.01 SOL = 10,000,000 >= 890,880; floor must stay >= 890,880 (M-30)")

print("\nF. Barton's 4:52 PM MT refinement: fee TIERED BY RATIO (same fee on capture, release, re-roll; cap 0.01 SOL)")
TIERS = {10_000:0.002, 50_000:0.002, 100_000:0.005, 200_000:0.005, 500_000:0.01, 1_000_000:0.01, 2_500_000:0.01, 5_000_000:0.01}
print(f"  {'R':>10} {'fee':>6} {'N':>7} | " + " ".join(f"{'m*@'+str(int(f)):>10}" for f in FDVS) + " | " + " ".join(f"{'rel/F@'+str(int(f)):>10}" for f in FDVS))
worst_trap = {}
for R, s_ in TIERS.items():
    N = 1e9 / R
    ms = [N * (s_ + TX) / F(R, f) for f in FDVS]
    rel = [s_ / F(R, f) for f in FDVS]
    worst_trap[R] = rel[0]
    print(f"  {R:>10,} {s_:6.3f} {int(N):>7,} | " + " ".join(f"{m:10.1f}" for m in ms) + " | " + " ".join(f"{r:9.1%} " for r in rel))
print("  (N = 1B/R; with MAX_COLLECTION_SIZE = 10,000 in the WT, N = min(1B/R, 10,000): m* scales with N, so at R=10k/50k")
print("   with N capped at 10,000 the m* column is 10x / 2x lower than shown.)")
for R in (10_000, 50_000):
    s_ = TIERS[R]; Ncap = min(1e9 / R, 10_000)
    print(f"   R={R:,} N capped {int(Ncap):,}: m*@100={Ncap*(s_+TX)/F(R,100):.1f}x, @410={Ncap*(s_+TX)/F(R,410):.1f}x, @1500={Ncap*(s_+TX)/F(R,1500):.1f}x")
check("F1_tiers_keep_R5M_thin", 200 * (0.01 + TX) / F(5_000_000, 410) < 2, "top ratios unchanged at 0.01: R=5M m*~1x floor at FDV 410")
check("F2_tiers_cut_release_trap_at_R10k", worst_trap[10_000] < 0.01 / F(10_000, 100), f"R=10k FDV100 release fee/F: {worst_trap[10_000]:.0%} (was 1000% at 0.01)")
check("F3_release_still_exceeds_value_R10k_FDV100", worst_trap[10_000] > 1, "0.002 SOL fee > 0.001 SOL value of N tokens at FDV 100: soft trap persists at the lowest ratio")

print("\nSUMMARY:", sum(results.values()), "/", len(results), "checks passed")
