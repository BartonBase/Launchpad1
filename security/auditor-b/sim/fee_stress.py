#!/usr/bin/env python3
"""
Auditor B -- economic stress test of the 2026-09-25 fee design (BRIEF.md "FEE CHANGE", 4:27 PM MT).
Pure local arithmetic + a Monte-Carlo invariant model. No cluster, no keys, no funds.

DESIGN INPUTS (docs/BRIEF.md 2026-09-25):
  - token fee f = 2% of the ratio R on capture and on EVERY re-roll, paid on top of R, to ONE fixed fee address
  - release (NFT -> tokens) returns exactly R, no fee
  - re-roll also pays a small fixed SOL minimum s (value TBD)
  Working tree hybrid_launch @ 16:39 MT (uncommitted): fee_bps in [100, 200], reroll SOL fee >= 0.001 SOL,
  capture SOL fee >= reroll SOL fee, both SOL fees paid to the fee owner.

EXAMPLE PARAMETERS (not from docs; labelled; edit for a real launch):
  - graduation FDV (1B tokens) 100 / 410 / 1,500 SOL. 410 SOL ~ pump.fun-style curve (30 SOL / 1.073B virtual
    reserves, graduation after ~793M sold) used in reroll_ev_r1.py Part F.
  - post-graduation CP AMM reserves 206.9M tokens / 85 SOL (pump.fun-like migration; unsourced example).
  - rarity tiers / premiums; tx fee 0.00005 SOL per tx, 2 tx per attempt (request + reveal/settle).
"""
import math, random

random.seed(20260925)
FEE = 0.02
TX = 0.00005 * 2
RENT_MIN_SOL = 890_880 / 1e9
RATIOS = [10_000, 50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000]
FDVS = [100.0, 410.0, 1500.0]
S_GRID = [0.001, 0.003, 0.005, 0.01, 0.02, 0.05]

def hr(t): print("\n" + "=" * 110 + "\n" + t + "\n" + "=" * 110)
def floor_sol(R, fdv): return R * fdv / 1e9
def attempt_cost(F, s, bps=200): return bps / 1e4 * F + s + TX

# ------------------------------------------------------------------------------------------ Part A
def part_a():
    hr("Part A: cost of one draw (capture or re-roll) = 2% x F + s + tx, and the share paid by the SOL minimum")
    print("F = R x FDV/1B.  Columns: s = SOL minimum per draw.")
    for fdv in FDVS:
        print(f"\n-- graduation FDV {fdv:.0f} SOL (EXAMPLE)")
        print(f"{'R':>10s} {'maxN':>7s} {'F (SOL)':>9s} {'2% fee':>9s} | " + " ".join(f"{'s='+str(s):>16s}" for s in S_GRID))
        for R in RATIOS:
            F = floor_sol(R, fdv); N = 1_000_000_000 // R
            cells = []
            for s in S_GRID:
                c = attempt_cost(F, s); cells.append(f"{c:8.5f} ({s/c:4.0%})")
            print(f"{R:10,d} {N:7,d} {F:9.4f} {FEE*F:9.6f} | " + " ".join(f"{x:>16s}" for x in cells))

# ------------------------------------------------------------------------------------------ Part B (D6)
def part_b():
    hr("Part B (settles D6): TOP tier = a 1-of-N. Break-even premium m* (in floors) = N x (0.02 + (s+tx)/F).\n"
       "Farming the top tier is unprofitable while its premium < m*. 'g' = floor growth after graduation (farm at\n"
       "the graduation floor, sell at g x floor): unprofitable iff m x g < m*.")
    PREM_BAND = (10, 1000)   # EXAMPLE 'realistic' 1/1 premium band in floors (no Solana-404 dataset read)
    print(f"EXAMPLE realistic top-tier premium band: {PREM_BAND[0]}x .. {PREM_BAND[1]}x floor.")
    for s in [0.001, 0.005, 0.02]:
        print(f"\n-- flat s = {s} SOL")
        print(f"{'R':>10s} {'N':>7s} | " + " ".join(f"{'FDV '+str(int(f)):>12s}" for f in FDVS) + " | verdict @FDV410 (g=1 / g=10)")
        for R in RATIOS:
            N = 1_000_000_000 // R
            ms = [N * (FEE + (s + TX) / floor_sol(R, f)) for f in FDVS]
            m410 = ms[1]
            v1 = "safe" if m410 >= PREM_BAND[1] else ("safe<%dx" % m410)
            v10 = "safe" if m410 / 10 >= PREM_BAND[1] else ("+EV if m>%.0fx" % (m410 / 10))
            print(f"{R:10,d} {N:7,d} | " + " ".join(f"{m:12,.0f}" for m in ms) + f" | {v1} / {v10}")
    print("\nB's round-1 fix (c) 's >= k / p_rarest' (p_rarest = 1/N => s = k x N) for comparison, k = 1e-6 SOL:")
    for R in RATIOS:
        N = 1_000_000_000 // R
        print(f"   R={R:>9,d} N={N:>7,d}: s = {1e-6*N:.4f} SOL  -> largest where m* is ALREADY largest (big N); smallest where"
              f" m* is smallest (small N). Wrong direction." if R in (10_000, 5_000_000) else f"   R={R:>9,d} N={N:>7,d}: s = {1e-6*N:.4f} SOL")
    print("\nConclusion printed by the model:")
    print("  * m* for the top tier grows linearly with N, so a flat s already scales cost-per-hit with rarity (A is right).")
    print("  * Where m* is small (N = 200..2,000, i.e. R >= 500k), F is large and s is a tiny share of the cost: NO flat s")
    print("    and NO rarest-tier scaling changes m*; only the token fee does (m* -> 0.02 x N = 4x..40x floor).")

# ------------------------------------------------------------------------------------------ Part C
TIERS = [("1/1(1/N)", None, 100.0), ("legendary", 0.001, 20.0), ("epic", 0.01, 5.0), ("rare", 0.05, 1.0)]
def part_c():
    hr("Part C: all tiers. Minimum FIXED s that makes grinding a tier unprofitable: s_req = F x (p x m x g - 0.02) - tx.\n"
       "EXAMPLE tiers: 1/1 = 1/N at 100x, legendary 0.1% at 20x, epic 1% at 5x, rare 5% at 1x. g = post-graduation growth.")
    for g in [1, 3, 10]:
        print(f"\n-- g = {g}")
        print(f"{'R':>10s} {'F@410':>8s} | " + " ".join(f"{n:>12s}" for n, _, _ in TIERS) + f" | {'ALL tiers':>10s} {'s/F burden':>10s}")
        for R in RATIOS:
            N = 1_000_000_000 // R; F = floor_sol(R, 410)
            reqs, epd = [], 0.0
            for n, p, m in TIERS:
                p = p if p is not None else 1 / N
                epd += p * m
                reqs.append(max(0.0, F * (p * m * g - FEE) - TX))
            allr = max(0.0, F * (epd * g - FEE) - TX)
            print(f"{R:10,d} {F:8.4f} | " + " ".join(f"{r:12.5f}" for r in reqs) + f" | {allr:10.5f} {allr/F:10.1%}")
    print("\nReading: at g=1 only tiers with p x m > 2% need s > 0 (epic 5%, rare 5%); the required s is proportional to F,")
    print("so a FIXED s that covers R=5M (F~2 SOL) is 500x the floor at R=10k. The token fee already covers p x m <= 2%.")

# ------------------------------------------------------------------------------------------ Part D
def part_d():
    hr("Part D: fixed s vs floor-tracking s across the price range (epic 1% at 5x, the worst EXAMPLE tier at g=1)")
    p, m = 0.01, 5.0
    print(f"{'F (SOL)':>9s} {'s_req':>9s} | fixed s=0.005: {'EV/draw':>9s} {'s/F':>7s} | tracking s=k F (k=0.03): {'EV/draw':>9s}")
    for F in [0.001, 0.004, 0.02, 0.04, 0.2, 0.41, 1.0, 2.05, 5.0, 20.0]:
        sreq = max(0, F * (p * m - FEE) - TX)
        ev_fixed = p * m * F - attempt_cost(F, 0.005)
        ev_track = p * m * F - attempt_cost(F, max(0.001, 0.03 * F))
        print(f"{F:9.3f} {sreq:9.5f} | {'':15s}{ev_fixed:9.5f} {0.005/F:7.1%} | {'':26s}{ev_track:9.5f}")
    print("A fixed s cannot bound mid-tier grinding at high floors without being prohibitive at low floors. A k x F term")
    print("does, but it is economically a higher percentage fee (2% + k): it conflicts with the '2% hard cap' copy.")

# ------------------------------------------------------------------------------------------ Part E
def part_e():
    hr("Part E: graduation window. Draws exist only after graduation (converting closed on the curve), so the lowest-price\n"
       "draws happen at the graduation floor F_g. Farming a 1-of-N at F_g and selling after growth g.")
    s = 0.005
    print(f"{'R':>10s} {'N':>7s} {'F_g':>8s} {'E[cost] 1/1':>12s} | premium needed for +EV at g=1 / 3 / 10 (floors of F_g)")
    for R in RATIOS:
        N = 1_000_000_000 // R; F = floor_sol(R, 410)
        cost = N * attempt_cost(F, s)
        print(f"{R:10,d} {N:7,d} {F:8.4f} {cost:12.2f} | " + " / ".join(f"{cost/F/g:8.1f}" for g in (1, 3, 10)))
    print("\nOptional decaying graduation surcharge on ALL draws: s(t) = s + k_g x F_g x max(0, 1 - t/72h), k_g = 0.10, F_g =")
    print("final curve price x R written on-chain at graduation (no oracle). Effect at t=0 on the 1/1 break-even (FDV 410):")
    for R in [100_000, 1_000_000, 5_000_000]:
        N = 1_000_000_000 // R; F = floor_sol(R, 410)
        base = N * (FEE + (s + TX) / F); sur = N * (FEE + (s + TX + 0.10 * F) / F)
        print(f"   R={R:>9,d}: m* {base:8.1f}x -> {sur:8.1f}x floor during the first hours")

# ------------------------------------------------------------------------------------------ Part F
def part_f():
    hr("Part F: bypass and insider paths (cost of one draw, FDV 410)")
    s_r, s_c_bad = 0.005, 0.0
    print(f"{'R':>10s} {'reroll':>9s} {'release+capture (capture s=0)':>30s} {'discount':>9s} {'insider (fee+s to self)':>24s} {'insider, s to sink':>19s}")
    for R in RATIOS:
        F = floor_sol(R, 410)
        rr = attempt_cost(F, s_r); byp = attempt_cost(F, s_c_bad); ins = TX + 0.001; ins_sink = TX + s_r
        print(f"{R:10,d} {rr:9.5f} {byp:30.5f} {1-byp/rr:9.0%} {ins:24.5f} {ins_sink:19.5f}")
    print("insider = fee owner (or a wallet it funds): the 2% returns to itself; if s also goes to the fee owner its marginal")
    print("cost is only tx + the VRF/oracle cost (EXAMPLE 0.001 SOL). Working tree enforces capture_sol >= reroll_sol (bypass")
    print("closed) but pays the SOL fee to the fee owner (insider path open).")

# ------------------------------------------------------------------------------------------ Part G
def part_g():
    hr("Part G: abort/refund economics. Draws obtained per fee paid, legendary (0.1%), R=1M, FDV 410, s=0.005")
    F = floor_sol(1_000_000, 410); c = attempt_cost(F, 0.005); p = 0.001
    rows = [("design docs / committed WIP 977f8f2: fee refunded on expire", 0.001 + TX),
            ("fee at request, never refunded (A M-04 / working tree)", c)]
    for name, per_bad in rows:
        print(f"   {name:62s}: E[cost]/legendary = {per_bad/p:8.2f} SOL (honest {c/p:.2f})")
    for k in [1, 3]:
        print(f"   A's ATOMIC reveal+settle, IF the requester can make settle fail at will, recommit cap {k}: up to {k+1} draws/fee"
              f" -> E[cost]/legendary ~ {c/p/(k+1):.2f} SOL")
    print("   Separate reveal (value recorded on-chain, then settle) + recommit only while unrevealed: no extra draws.")

# ------------------------------------------------------------------------------------------ Part H
def part_h(ops=200_000):
    hr("Part H: escrow invariant under the fee design (Monte-Carlo, integer base units, decimals 6)")
    for R_whole in RATIOS:
        d = 6; R = R_whole * 10**d; fee = -(-R * 200 // 10_000); N = min(1_000, 1_000_000_000 // R_whole)
        supply = 1_000_000_000 * 10**d
        vault = 0; fee_ata = 0; outside = 0; pool = N; pending = []  # pending: ('cap'|'rr', refundable principal)
        users = supply; ok = True; seq_leak = 0
        for _ in range(ops if R_whole == 1_000_000 else ops // 20):
            op = random.random()
            if op < 0.30 and pool - len(pending) > 0 and users >= R + fee:          # request_capture
                users -= R + fee; vault += R; fee_ata += fee; pending.append(("cap", R))
            elif op < 0.55 and outside > 0 and pool - len(pending) > 0 and users >= fee:  # request_reroll
                users -= fee; fee_ata += fee; outside -= 1; pending.append(("rr", 0))
            elif op < 0.85 and pending:                                             # settle (FIFO head)
                k, _ = pending.pop(0); outside += 1
                if k == "cap": pool -= 1
            elif outside > 0:                                                        # release
                outside -= 1; pool += 1; vault -= R; users += R
            # principal-only expire is modelled as: capture -> vault pays R back; reroll -> NFT back; fee kept
            if pending and random.random() < 0.01:
                k, pr = pending.pop(0)
                if k == "cap": vault -= pr; users += pr
                else: outside += 1
            owed = R * (outside + len(pending))   # pending re-roll hand-ins are still owed backing (see note)
            if vault != owed or users + vault + fee_ata != supply or vault < 0: ok = False; break
        rt = R + fee - R
        print(f"   R={R_whole:>9,d}: fee={fee//10**d:>7,d} tokens (exact={R*200 % 10_000 == 0})  vault==R x (outside+pending): {ok}"
              f"  supply conserved: {ok}  capture->release round trip loses {rt//10**d:,} tokens (2%) + SOL")
    print("   NOTE: a first version counted only pending CAPTURES as owed (as the WIP invariant does: required = ratio x")
    print("   (assets_outside + pending_captures), with assets_outside decremented at request_reroll). That check FAILS as an")
    print("   equality: each pending re-roll hand-in leaves R of unaccounted backing. '>=' still passes, so it is not exploitable,")
    print("   but the on-chain invariant is looser than it should be by R x pending_rerolls (see fee doc F-13).")
    print("   Release pays no fee; re-roll moves no backing; fee never enters the vault. Round trip cannot be arbitraged against")
    print("   the protocol (it always loses 2% + SOL); it only bounds the NFT floor band [R x P, 1.02 x R x P + s].")

# ------------------------------------------------------------------------------------------ Part I
AMM_TOK, AMM_SOL = 206_900_000.0, 85.0
def sell_impact(X, y=AMM_TOK): return 1 - (y / (y + X)) ** 2
def part_i():
    hr("Part I: fee-token accumulation at the fee address and market impact if sold (EXAMPLE volumes)")
    print(f"EXAMPLE post-graduation CP AMM: {AMM_TOK/1e6:.1f}M tokens / {AMM_SOL} SOL. Fee tokens/draw = 0.02 x R."
          " % of supply per draw = 0.02 x R / 1B.")
    scen = [("quiet", 0.02, 0.01), ("normal", 0.10, 0.10), ("hot", 0.30, 0.50), ("farm frenzy", 0.50, 3.00)]
    for R in [10_000, 100_000, 1_000_000]:
        N = 1_000_000_000 // R
        print(f"\n-- R={R:,} (max N={N:,}); scenario = captures/day and re-rolls/day as fractions of N")
        for name, cf, rf in scen:
            draws = (cf + rf) * N; toks = FEE * R * draws
            print(f"   {name:12s}: {draws:9,.0f} draws/day -> {toks/1e6:7.2f}M tokens/day ({toks/1e9:6.2%} supply); 30d {30*toks/1e9:6.1%};"
                  f" dump 1 day at once: price {-sell_impact(toks):6.1%}; 30-day pile dumped: {-sell_impact(30*toks):6.1%}")
    print("\nFarming cost -> fee tokens: farming all 10 legendaries in a 10,000 pool ~29,290 draws -> 0.02 x 29,290 x R")
    for R, N in [(100_000, 10_000)]:
        toks = 0.02 * 29_290 * R
        print(f"   R={R:,}: {toks/1e6:.1f}M tokens = {toks/1e9:.2%} of supply to the fee address; dump impact {-sell_impact(toks):.1%}")
    print("\nStreaming vs dumping 20M tokens (2% of supply) into the EXAMPLE pool (arbitrage refills between tranches ignored:"
          " conservative):")
    for tranches in [1, 10, 30, 100]:
        X = 20e6 / tranches
        print(f"   {tranches:3d} tranches of {X/1e6:6.2f}M: per-tranche impact {-sell_impact(X):6.2%}")
    print("Stonk.fun reference (Bitquery via stonkfun-lessons.md): 160 coins lost >50% of supply to tax; ZCAT 20% in hour one.")

# ------------------------------------------------------------------------------------------ Part J
def part_j():
    hr("Part J: rounding / small amounts")
    for d in [0, 6, 9]:
        for R in RATIOS:
            Rb = R * 10**d
            for bps in (100, 200):
                f_floor = Rb * bps // 10_000; f_ceil = -(-Rb * bps // 10_000)
                assert f_floor == f_ceil and f_floor > 0
    print("   All allowed ratios x decimals {0,6,9} x bps {100,200}: fee exact and > 0 (floor == ceil). Min fee = 100 tokens")
    print("   (R=10k at 1%). u64 headroom: R=5M, d=9 -> R_base x 200 = 1e18 < 1.8e19.")
    print(f"   SOL leg: reroll SOL min must be >= rent-exempt minimum {RENT_MIN_SOL:.6f} SOL if paid to a possibly-empty system")
    print("   account (working tree MIN_REROLL_SOL_FEE_LAMPORTS = 1,000,000 satisfies this).")
    print("   'Near zero' in SOL terms: 2%% of F at R=10k, FDV 100 = %.7f SOL per draw." % (FEE * floor_sol(10_000, 100)))

if __name__ == "__main__":
    for f in (part_a, part_b, part_c, part_d, part_e, part_f, part_g, part_h, part_i, part_j): f()
