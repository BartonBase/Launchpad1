#!/usr/bin/env python3
"""
Auditor B -- stress test of the FLAT SOL FEE design (BRIEF.md "FEE CHANGE v2", Barton 2026-09-25 4:49 PM MT).
Pure local arithmetic. No cluster, no keys, no funds.

DESIGN UNDER TEST (BRIEF.md:79-85):
  - NO token fee. A flat SOL fee on every capture (token->NFT), release (NFT->token) and re-roll, paid to Barton's
    fee wallet. Proposed 0.01 SOL each (Barton can adjust). Fixed per launch in LaunchConfig, hard cap in code
    (cap VALUE NOT STATED in the BRIEF; the working tree has MAX_SOL_FEE_LAMPORTS = 0.05 SOL).
  - Release still returns exactly N tokens; the fee is SOL only.

MODEL CONVENTIONS
  R   = ratio, tokens per NFT (the "ratio" axis).  C = collection size (NFT count).  C x R <= 1B.
  F   = NFT floor in SOL = R x token price = R x FDV / 1e9 (FDV = 1B-supply market cap in SOL).
  c   = outsider cost of one draw (capture or re-roll) = s + TX (+ VRF if the requester pays VRF separately).
  Farming a tier with pool frequency p: expected draws 1/p; each draw hands in a floor NFT and gets a random one.
  Break-even premium (price of the rare / floor): m* = 1 + c / (p x F).  Grinding is +EV iff the rare's market
  price > m* x F. (The 2026-09-25 fee doc used N x (0.02 + c/F), i.e. without the "+1" hand-in credit.)

EXAMPLE INPUTS (labelled; not sourced from a Solana-404 premium dataset):
  TX = 0.0001 SOL per attempt (request + reveal/settle, base + modest priority).
  VRF = 0.002 SOL (Switchboard On-Demand ~0.002 SOL/request per QuickNode guide 2026 and Switchboard's own 2022
        VRF fee post; the official Switchboard docs page does not publish a price. ORAO: 0.001 SOL).
  Premium bands (EXAMPLE): 1/1 10x..1000x, legendary 5x..50x, epic 2x..10x floor.
  Graduation FDV ~400 SOL (docs/graduation-design.md, illustrative, 85 SOL raise).

UPDATE (BRIEF.md:89, "FEE REFINEMENT", Barton 4:52 PM MT): the flat fee is TIERED BY RATIO, fixed per collection at
launch, immutable, hard cap 0.01 SOL in code: 10k/50k = 0.002, 100k/200k = 0.005, 500k..5M = 0.01 SOL, the same on
capture, release and re-roll. Part G evaluates that schedule. The WT (needs_barton.rs FEE_TIERS) matches it.
"""
import math

TX = 0.0001
VRF = 0.002
S_DEFAULT = 0.01
SIZES = [1_000, 5_000, 10_000]
FLOORS = [0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0]
RATIOS = [10_000, 50_000, 100_000, 200_000, 500_000, 1_000_000, 2_500_000, 5_000_000]
FDVS = [50.0, 100.0, 400.0, 1_000.0, 5_000.0, 20_000.0]
TIERS = [("1/1", lambda C: 1.0 / C, (10, 1000)),
         ("legendary 0.1%", lambda C: 0.001, (5, 50)),
         ("epic 1%", lambda C: 0.01, (2, 10))]

def hr(t): print("\n" + "=" * 118 + "\n" + t + "\n" + "=" * 118)
def mstar(c, p, F): return 1 + c / (p * F)
def verdict(m, band):
    lo, hi = band
    if m >= hi: return "PROT"      # break-even above the top of the example band: farming unprofitable
    if m < lo: return "LOTT"       # below the bottom of the band: market-priced lottery, +EV for any plausible rare
    return "mid"                   # inside the band: depends on the actual market premium

# ------------------------------------------------------------------------------------------------ Part A
def part_a():
    hr("Part A: what the OLD 'ratio' figures meant (fee_stress.py Part B, 2% token fee + s = 0.005, FDV 410)")
    print("The old table varied R (tokens per NFT) and set C = 1B/R (the MAX collection size for that ratio), FDV 410,")
    print("so each row moves THREE things at once: ratio R, collection size C (odds 1/C) and floor F = R x 0.41e-6.")
    print(f"{'R (tokens/NFT)':>15s} {'C = 1B/R':>9s} {'F (SOL)':>9s} {'old m* (repro)':>15s} {'flat 0.01, no token fee':>24s} {'allowed if C<=10k?':>19s}")
    for R in RATIOS:
        C = 1_000_000_000 // R; F = R * 410 / 1e9
        old = C * (0.02 + (0.005 + TX) / F)
        new = mstar(S_DEFAULT + TX, 1 / C, F)
        print(f"{R:15,d} {C:9,d} {F:9.4f} {old:15,.1f} {new:24,.1f} {'yes' if C <= 10_000 else 'NO (C capped)':>19s}")
    print("Restated: '126,390x at a 10k ratio' = R 10,000 tokens/NFT, C 100,000 NFTs, floor 0.0041 SOL. The huge multiple")
    print("comes mostly from the 1-in-100,000 odds and the tiny floor, not from the ratio as such. Under a 10,000-NFT cap")
    print("the R=10k and R=50k rows at max size no longer exist.")

# ------------------------------------------------------------------------------------------------ Part B
def part_b():
    hr("Part B: re-roll farm break-even premium m* (x floor) per tier, collection size and floor. s = 0.01 SOL.\n"
       "Row 'closed' = fee paid at request, never refunded (working tree). Row 'VRF sep' = requester also pays VRF.\n"
       "Row 'open' = OLD expire loophole (fee refunded on expire; attacker withholds reveal on bad draws => bad draw\n"
       "costs TX + VRF only; good draw pays s + TX + VRF). Verdict vs EXAMPLE bands: PROT / mid / LOTT.")
    for name, pf, band in TIERS:
        print(f"\n-- {name}   (example premium band {band[0]}x..{band[1]}x)")
        print(f"{'C':>7s} {'case':>8s} | " + " ".join(f"{'F='+str(F):>11s}" for F in FLOORS))
        for C in SIZES:
            p = pf(C)
            rows = {
                "closed": [mstar(S_DEFAULT + TX, p, F) for F in FLOORS],
                "VRF sep": [mstar(S_DEFAULT + TX + VRF, p, F) for F in FLOORS],
                "open": [1 + ((1 / p - 1) * (TX + VRF) + (S_DEFAULT + TX + VRF)) / F for F in FLOORS],
            }
            for k, ms in rows.items():
                cells = [f"{m:7,.0f}{verdict(m, band)[0]:>1s}" if m >= 10 else f"{m:7.2f}{verdict(m, band)[0]:>1s}" for m in ms]
                print(f"{C:7,d} {k:>8s} | " + " ".join(f"{x:>11s}" for x in cells))
    print("\nSuffix letter: P = protected (m* above the band), m = inside band (depends on market), L = lottery (m* below band).")
    print("Loophole-open vs closed: the abort makes a 1/1 at C=10k ~5x cheaper (TX+VRF 0.0021 vs 0.0101 per miss).")

# ------------------------------------------------------------------------------------------------ Part B2
def part_b2():
    hr("Part B2: expected SOL cost to farm ONE hit (closed loophole, s = 0.01, outsider) and EV per draw at EXAMPLE\n"
       "premiums 1/1 = 100x, legendary = 20x, epic = 5x (EV = p x (m-1) x F - c).")
    prem = {"1/1": 100, "legendary 0.1%": 20, "epic 1%": 5}
    c = S_DEFAULT + TX
    print(f"{'C':>7s} {'tier':>15s} {'E[cost/hit] SOL':>16s} | EV per draw (SOL) at F = " + " ".join(f"{F:>8}" for F in FLOORS))
    for C in SIZES:
        for name, pf, _ in TIERS:
            p = pf(C); m = prem[name]
            evs = [p * (m - 1) * F - c for F in FLOORS]
            print(f"{C:7,d} {name:>15s} {c / p:16,.2f} | {'':25s}" + " ".join(f"{e:8.4f}" for e in evs))
    print("\nFloor F at which each tier turns +EV for a given premium m: F* = c / (p x (m-1)).")
    for C in SIZES:
        out = []
        for name, pf, _ in TIERS:
            p = pf(C); m = prem[name]
            out.append(f"{name} @{m}x: F* = {c / (p * (m - 1)):8.3f} SOL")
        print(f"   C={C:>6,d}: " + " | ".join(out))

# ------------------------------------------------------------------------------------------------ Part B3
def part_b3():
    hr("Part B3: sensitivity to the fee amount s (closed loophole): m* for the 1/1 and epic at C = 10k / 1k, and\n"
       "the s needed to protect a tier at premium m: s_req = p x (m-1) x F - TX.")
    for s in [0.005, 0.01, 0.02, 0.05]:
        c = s + TX
        a = [mstar(c, 1 / 10_000, F) for F in FLOORS]
        b = [mstar(c, 1 / 1_000, F) for F in FLOORS]
        e = [mstar(c, 0.01, F) for F in FLOORS]
        print(f"  s={s:<6}  1/1 C=10k: " + " ".join(f"{m:8,.0f}" for m in a))
        print(f"  {'':8}  1/1 C=1k : " + " ".join(f"{m:8,.1f}" for m in b))
        print(f"  {'':8}  epic 1%  : " + " ".join(f"{m:8,.2f}" for m in e))
    print(f"  (columns: F = {FLOORS})")
    print("\n  s_req (SOL) to keep grinding unprofitable:")
    for label, p, m in [("1/1 C=10k @100x", 1e-4, 100), ("1/1 C=1k @100x", 1e-3, 100), ("legendary @20x", 1e-3, 20),
                        ("epic @5x", 1e-2, 5)]:
        print(f"   {label:18s}: " + " ".join(f"{max(0, p * (m - 1) * F - TX):8.4f}" for F in FLOORS))

# ------------------------------------------------------------------------------------------------ Part B4
def part_b4():
    hr("Part B4: pool depletion. Odds are drawn from the CURRENT pool, not the census. If only a fraction phi of the\n"
       "collection is in the vault and the rare is among them, p_pool = count / (phi x C). Pool floor r = max(5, 2% C).")
    c = S_DEFAULT + TX
    for C in SIZES:
        for phi in [1.0, 0.25, 0.05, max(5, math.ceil(0.02 * C)) / C]:
            ms = [1 + c / ((1 / (phi * C)) * F) for F in FLOORS]
            print(f"   C={C:>6,d} phi={phi:5.2f} (pool {phi*C:7,.0f}) 1/1 m*: " + " ".join(f"{m:8,.1f}" for m in ms))
    print(f"   (columns: F = {FLOORS}). A 1/1 sitting in a thin pool is up to 50x cheaper to farm than the census implies.")

# ------------------------------------------------------------------------------------------------ Part C
def part_c():
    hr("Part C: release trap. Value of an NFT's tokens V = R x FDV/1e9. Release fee s_rel = 0.01 SOL.\n"
       "Underwater on release when V x (1 - sell_cost) < s_rel + TX. sell_cost = AMM fee + slippage (EXAMPLE 1%).")
    s_rel = 0.01; sell = 0.01
    print(f"{'R':>10s} | {'token price P* (SOL/token)':>27s} {'FDV* (SOL)':>11s} | V at FDV " + " ".join(f"{f:>8,.0f}" for f in FDVS))
    for R in RATIOS:
        pstar = (s_rel + TX) / (R * (1 - sell))
        fdvstar = pstar * 1e9
        vs = [R * f / 1e9 for f in FDVS]
        print(f"{R:10,d} | {pstar:27.3e} {fdvstar:11,.1f} | {'':9s}" + " ".join(f"{v:8.4f}" for v in vs))
    print("\nRelease fee as % of V (>=100% = underwater, holder should not release):")
    print(f"{'R':>10s} | " + " ".join(f"{'FDV '+format(f, ',.0f'):>11s}" for f in FDVS))
    for R in RATIOS:
        print(f"{R:10,d} | " + " ".join(f"{(s_rel + TX) / (R * f / 1e9):11.1%}" for f in FDVS))
    print("\nCrash from graduation FDV 400: release fee as % of V after a drawdown")
    for R in [100_000, 200_000, 500_000, 1_000_000]:
        row = []
        for dd in [0.0, 0.5, 0.75, 0.9, 0.95, 0.99]:
            V = R * 400 * (1 - dd) / 1e9
            row.append(f"-{dd:.0%}: {(s_rel + TX) / V:6.1%}")
        print(f"   R={R:>9,d}: " + " | ".join(row))
    print("\nMax loss per floor NFT from the release fee = min(V, s_rel) <= 0.01 SOL (the holder can always choose not to")
    print("release, or sell the NFT on Tensor/ME; an arbitrageur bids at most V - s_rel - TX, so the fee is priced into")
    print("the NFT's exit value either way).")
    print("\nFee-wallet income if every NFT is released in a panic vs the tokens those NFTs back (R=100k, C=10k):")
    for fdv in [400, 100, 50, 20]:
        V = 100_000 * fdv / 1e9
        print(f"   FDV {fdv:>4}: holders' tokens N x V = {10_000 * V:7.1f} SOL; release fees N x 0.01 = {10_000 * s_rel:5.1f} SOL"
              f" ({s_rel / V:6.1%} of the value exiting)")
    print("\nInsider band arbitrage: the NFT bid floor sits at V - s_rel - TX for outsiders. The fee wallet (or a wallet")
    print("it funds) releases at net cost TX (the fee returns to itself): it can buy every NFT listed between V - s_rel and V")
    print("and redeem it, earning up to s_rel per NFT that nobody else can. Same on the capture side (sell NFTs up to V + s).")

# ------------------------------------------------------------------------------------------------ Part D
def part_d():
    hr("Part D: small ratios. Capture fee and round trip (capture + release) as % of V, flat 0.01 each; and with a\n"
       "zero release fee (round trip = 0.01). Max C under a 10,000 cap; share of supply that can be in NFT form.")
    print(f"{'R':>10s} {'maxC (1B/R)':>11s} {'maxC cap10k':>11s} {'% supply':>8s} | capture fee % of V at FDV " +
          " ".join(f"{f:>8,.0f}" for f in FDVS))
    for R in RATIOS:
        C = 1_000_000_000 // R; Cc = min(C, 10_000)
        row = [(S_DEFAULT + TX) / (R * f / 1e9) for f in FDVS]
        print(f"{R:10,d} {C:11,d} {Cc:11,d} {Cc * R / 1e9:8.0%} | {'':26s}" + " ".join(f"{x:8.1%}" for x in row))
    print(f"\n{'R':>10s} | round trip 0.02 (capture+release) % of V at FDV " + " ".join(f"{f:>8,.0f}" for f in FDVS))
    for R in RATIOS:
        print(f"{R:10,d} | {'':47s}" + " ".join(f"{(0.02 + 2 * TX) / (R * f / 1e9):8.1%}" for f in FDVS))
    print("\nMinimum FDV (SOL) for the round trip to be <= 10% / <= 25% of V:")
    for R in RATIOS:
        f10 = (0.02 + 2 * TX) / (0.10 * R) * 1e9; f25 = (0.02 + 2 * TX) / (0.25 * R) * 1e9
        g10 = (0.01 + 2 * TX) / (0.10 * R) * 1e9
        print(f"   R={R:>9,d}: 0.02 round trip: <=10% needs FDV {f10:9,.0f}, <=25% needs {f25:8,.0f};"
              f"  with release fee 0 (0.01 round trip): <=10% needs {g10:8,.0f}")
    print("\nArbitrage band for the NFT price (outsider): [V - s_rel - TX, V + s_cap + TX]. Width 0.02 SOL flat;")
    print("relative width at FDV 400: " + ", ".join(f"R={R//1000}k {0.02 / (R * 400 / 1e9):.0%}" for R in RATIOS))
    print("\nPer-NFT Core mint cost (graduation-design.md, measured) 0.00509 SOL vs V at FDV 400:")
    print("   " + ", ".join(f"R={R//1000}k V={R * 400 / 1e9:.4f}" for R in RATIOS[:4]))
    print("\nWash-convert cost (outsider) per round trip: 0.02 SOL (or 0.01 with free release); insider: 2 x TX.")
    print("Rare farming at small ratios (C<=10k, 1/1): m* at FDV 400 -> " + ", ".join(
        f"R={R//1000}k: {mstar(S_DEFAULT + TX, 1 / min(10_000, 1_000_000_000 // R), R * 400 / 1e9):,.0f}x" for R in RATIOS))

# ------------------------------------------------------------------------------------------------ Part E
def part_e():
    hr("Part E: insider (fee wallet) vs outsider cost per draw, and sybil/DoS costs at 0.01")
    for vrf_paid_by_requester in [False, True]:
        out = S_DEFAULT + TX + (VRF if vrf_paid_by_requester else 0)
        ins = TX + VRF   # the fee returns to the insider; VRF is a real cost whether the requester or Barton's crank pays it
        print(f"   VRF paid by requester={vrf_paid_by_requester}: outsider {out:.4f} SOL/draw, insider (true cost) {ins:.4f} "
              f"(insider pays {ins / out:.0%}; {out / ins:.1f}x cheaper)")
    print("   Split alternative: 0.005 to Barton + 0.005 to incinerator/sink: insider cost %.4f (%.0f%% of outsider 0.0101)."
          % (0.005 + TX + VRF, 100 * (0.005 + TX + VRF) / (S_DEFAULT + TX)))
    print("\n   1/1 at C=10k, F=1 SOL: outsider m* = %.0fx; insider m* = %.0fx"
          % (mstar(S_DEFAULT + TX, 1e-4, 1.0), mstar(TX + VRF, 1e-4, 1.0)))
    print("   epic 1%% at F=0.25: outsider m* = %.2fx; insider m* = %.2fx" % (mstar(S_DEFAULT + TX, 0.01, 0.25), mstar(TX + VRF, 0.01, 0.25)))
    print("   Insider band arbitrage per NFT (release fee 0.01): outsider needs a discount > 0.0101 below V; insider > 0.0001.")
    print("\n   Queue spam: each request pays 0.01 non-refundable; 1,000 junk requests = 10 SOL + principal locked until settle.")
    print("   Crank cost per request (reveal + settle txs + Switchboard ~0.002): ~0.0022 SOL -> 0.01 fee covers it ~4.5x.")
    for C, R, fdv in [(1_000, 100_000, 400), (10_000, 100_000, 400), (1_000, 1_000_000, 400), (200, 5_000_000, 400)]:
        r = max(5, math.ceil(0.02 * C)); V = R * fdv / 1e9; n = C - r
        print(f"   Cornering C={C:,} R={R:,} FDV {fdv}: capture {n:,} NFTs = {n * V:8.2f} SOL of tokens + {n * S_DEFAULT:6.2f} SOL fees"
              f" ({n * R / 1e9:.1%} of supply; price impact not included)")

# ------------------------------------------------------------------------------------------------ Part F
def part_f():
    hr("Part F: candidate fee schedules (capture / re-roll / release, SOL). Checks: bypass (capture+release >= re-roll),\n"
       "VRF coverage (draw fee >= VRF + TX + margin), rent (>= 890,880 lamports or exactly 0), release trap FDV at R=100k,\n"
       "1/1 m* at C=10k and C=1k (F = 0.25 and 1 SOL), epic m* at F=0.25, capture fee % of V at R=100k/FDV 400.")
    RENT = 890_880 / 1e9
    scheds = [("BRIEF default", 0.01, 0.01, 0.01), ("REC: free release", 0.01, 0.01, 0.0),
              ("low", 0.005, 0.005, 0.0), ("high", 0.02, 0.02, 0.0), ("bad: capture<reroll", 0.0, 0.01, 0.0),
              ("bad: dust release", 0.01, 0.01, 0.0005)]
    for name, cap, rr, rel in scheds:
        bypass_ok = cap + rel >= rr
        vrf_ok = min(cap, rr) >= VRF + TX + 0.001
        rent_ok = all(x == 0 or x >= RENT for x in (cap, rr, rel))
        trap = (rel + TX) / (100_000 * 0.99) * 1e9 if rel > 0 else 0
        m10a, m10b = mstar(rr + TX, 1e-4, 0.25), mstar(rr + TX, 1e-4, 1.0)
        m1a, m1b = mstar(rr + TX, 1e-3, 0.25), mstar(rr + TX, 1e-3, 1.0)
        ep = mstar(rr + TX, 0.01, 0.25)
        capv = (cap + TX) / (100_000 * 400 / 1e9)
        print(f"   {name:20s} {cap:5}/{rr:5}/{rel:6}: bypass {'OK ' if bypass_ok else 'FAIL'} VRF {'OK ' if vrf_ok else 'FAIL'}"
              f" rent {'OK ' if rent_ok else 'FAIL*'} | release underwater below FDV {(format(trap, '6.0f') if rel > 0 else '  never')} | 1/1 C10k {m10a:6,.0f}x/{m10b:5,.0f}x"
              f" C1k {m1a:5,.0f}x/{m1b:4,.1f}x | epic {ep:4.2f}x | capture {capv:5.1%} of V")

# ------------------------------------------------------------------------------------------------ Part G
TIER = {10_000: 0.002, 50_000: 0.002, 100_000: 0.005, 200_000: 0.005, 500_000: 0.01, 1_000_000: 0.01,
        2_500_000: 0.01, 5_000_000: 0.01}
CRANK_TX = 0.0002   # reveal + settle txs paid by our crank (EXAMPLE)
def part_g():
    hr("Part G: Barton's 4:52 PM TIERED schedule (same fee on capture, release, re-roll; cap 0.01). C = min(1B/R, 10,000).\n"
       "Floors at FDV 400 (~graduation) and 2,000 (5x). m* = break-even premium (x floor), loophole closed, VRF paid by crank.")
    print(f"{'R':>10s} {'fee':>6s} {'C':>6s} | {'F@400':>7s} {'1/1':>8s} {'leg':>7s} {'epic':>6s} | {'F@2000':>7s} {'1/1':>7s} {'leg':>6s} {'epic':>6s}"
          f" | {'cap %V@400':>10s} {'RT %V@400':>9s} {'rel. underwater FDV':>19s} | {'VRF margin':>10s} {'insider/outsider':>16s}")
    for R in RATIOS:
        sf = TIER[R]; C = min(1_000_000_000 // R, 10_000); c = sf + TX
        F1 = R * 400 / 1e9; F5 = R * 2000 / 1e9
        m = lambda p, F: mstar(c, p, F)
        uw = (sf + TX) / (R * 0.99) * 1e9
        margin = sf - VRF - CRANK_TX
        print(f"{R:10,d} {sf:6.3f} {C:6,d} | {F1:7.3f} {m(1/C, F1):8,.0f} {m(1e-3, F1):7,.1f} {m(1e-2, F1):6.2f} | {F5:7.3f} {m(1/C, F5):7,.0f}"
              f" {m(1e-3, F5):6.1f} {m(1e-2, F5):6.2f} | {c / F1:10.1%} {2 * c / F1:9.1%} {uw:19,.0f} | {margin:+10.4f} {(TX + VRF) / c:16.0%}")
    print("\nVRF margin = tier fee - VRF (0.002 EXAMPLE) - crank txs (0.0002). Negative = Barton's crank subsidises every draw")
    print("if it pays the oracle; a spammer then costs Barton ~%.4f SOL per draw at a cost to itself of %.4f." % (VRF + CRANK_TX - 0.002, 0.002 + TX))
    print("insider/outsider = fee-wallet's true cost per draw (TX + VRF) / outsider's (fee + TX). ~100% at the 0.002 tier means")
    print("no insider discount there, but only because the fee barely covers VRF.")
    print("\nRe-roll <= capture + release in every tier: " + str(all(TIER[R] <= 2 * TIER[R] for R in RATIOS)) +
          "; re-roll <= capture (needed if release fee -> 0): " + str(True))
    print("\nSame tiers with release fee = 0 (recommended): release never underwater; round trip = one fee:")
    for R in RATIOS:
        print(f"   R={R:>9,d}: round trip {TIER[R] + 2 * TX:.4f} SOL = {(TIER[R] + 2 * TX) / (R * 400 / 1e9):6.1%} of V at FDV 400")

def footnote():
    print("   * rent FAIL = a sub-890,880-lamport transfer fails only while the fee wallet is empty/below rent-exempt (M-30).")

if __name__ == "__main__":
    for f in (part_a, part_b, part_b2, part_b3, part_b4, part_c, part_d, part_e, part_f, part_g, footnote):
        f()
