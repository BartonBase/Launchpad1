#!/usr/bin/env python3
"""
Round 1 (Review B): re-roll / rare-farming economics for Mintmark SPL-404 hybrid launches.
Extends reroll_ev.py (reuses its MPL-Hybrid pick model, confirmed against mpl-hybrid @ aacf1a53
capture.rs L167-183, see research-sources.md section 5).

Design numbers taken FROM THE DOCS (docs/hybrid-rarity-and-assignment.md section 4, DECISIONS.md ADR-008):
  - reroll fee = token fee in bps of the ratio R. Default 200 bps (2%), hard cap 1,000 bps (10%).
  - plus a small SOL fee "~0.003 SOL" (VRF + worst-case asset rent), capped at 0.05 SOL.
  - capture_fee_bps >= reroll_fee_bps enforced, release has no token fee -> unwrap+rewrap costs >= a reroll.
  - VRF: ORAO ~0.001 SOL per request (doc); Switchboard = reclaimable rent + oracle fee (no number given).
  - Doc sizing rule: break-even when fee f >= p * m  (p = pool frequency, m = premium in floors).
  - Stock MPL-Hybrid: protocol fee 0.005 SOL on capture AND on release (research-sources.md section 5).
  - BRIEF SCOPE CHANGE: reroll fee is paid in the collection token and BURNED (the hybrid doc says
    "no burn"; this contradiction does not change the attacker's cost, only where the fee goes).
  - Because NFT floor F = R x token price, a bps-of-R fee costs (bps/1e4) x F in SOL for ANY ratio, so the
    results below are ratio-independent except where noted.

EXAMPLE parameters (NOT from the docs; the docs give no rarity table or premiums; edit to match a real launch):
  - rarity tiers and premiums, floors F, tx fee, curve parameters in part F/H.
"""
import math
import os
import random
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from reroll_ev import mpl_hybrid_pick, slot_hashes_bytes  # reuse the confirmed model

random.seed(20260924)

# ---------------------------------------------------------------- design parameters (docs)
FEE_BPS_GRID = [0, 50, 100, 200, 500, 1000]     # 200 = doc default, 1000 = doc hard cap
SOL_FEE_DEFAULT = 0.003                          # doc: "~0.003 SOL, capped"
SOL_FEE_CAP = 0.05                               # doc: "<= 0.05 SOL"
ORAO_VRF_SOL = 0.001                             # doc: "ORAO ~0.001 SOL per request"
MPLH_PROTOCOL_FEE = 0.005                        # per capture and per release (Metaplex, code-derived)

# ---------------------------------------------------------------- example parameters (labelled)
TX_FEE = 0.00005                # EXAMPLE: base 5,000 lamports + modest priority fee, per tx
TXS_PER_VRF_ATTEMPT = 2         # request tx + settle tx (settle may be cranked; counted anyway)
TIERS = [                       # EXAMPLE rarity tiers: (name, pool frequency p, example premium in floors m)
    ("1/1",       0.0001, 100.0),
    ("legendary", 0.001,   20.0),   # 0.1% at 20x is the doc's own worked example
    ("epic",      0.01,     3.0),
    ("rare",      0.05,     0.5),
]
FLOORS = [0.03, 0.3, 3.0]       # EXAMPLE NFT floor in SOL (= R x token price)
F_MAIN = 0.3


def vrf_attempt_cost(F, bps, sol_fee=SOL_FEE_DEFAULT):
    """SOL cost of one honest VRF reroll attempt (hybrid_vault as documented)."""
    return bps / 1e4 * F + sol_fee + TXS_PER_VRF_ATTEMPT * TX_FEE


def breakeven_m(F, bps, p, sol_fee=SOL_FEE_DEFAULT):
    """Premium (in floors) above which grinding for a tier with frequency p has positive EV."""
    return vrf_attempt_cost(F, bps, sol_fee) / p / F


def hr(t):
    print("\n" + "=" * 100 + f"\n{t}\n" + "=" * 100)


# ---------------------------------------------------------------- Part A
def part_a():
    hr("Part A: honest VRF (hybrid_vault as documented). Break-even premium m* in FLOORS (grind is +EV if premium > m*)")
    print("m* = (fee_bps/1e4 + (sol_fee + 2 tx)/F) / p.  The doc's rule f >= p*m ignores the SOL part; it is shown for comparison.")
    for F in FLOORS:
        print(f"\n-- floor F = {F} SOL (EXAMPLE), SOL fee {SOL_FEE_DEFAULT} (doc default)")
        print(f"{'tier':10s} {'p':>8s} | " + " ".join(f"{str(b)+'bps':>9s}" for b in FEE_BPS_GRID) + " | doc rule f/p @200bps")
        for name, p, _ in TIERS:
            row = " ".join(f"{breakeven_m(F, b, p):9.1f}" for b in FEE_BPS_GRID)
            print(f"{name:10s} {p:8.4%} | {row} | {0.02 / p:9.1f}")
    print("\nSame table in SOL of premium (m* x F) at F = 0.3 SOL:")
    print(f"{'tier':10s} | " + " ".join(f"{str(b)+'bps':>9s}" for b in FEE_BPS_GRID))
    for name, p, _ in TIERS:
        print(f"{name:10s} | " + " ".join(f"{breakeven_m(F_MAIN, b, p) * F_MAIN:9.2f}" for b in FEE_BPS_GRID))


# ---------------------------------------------------------------- Part B
def part_b():
    hr(f"Part B: EV per hit, honest VRF, F = {F_MAIN} SOL, EXAMPLE premiums (1/1 100x, legendary 20x, epic 3x, rare 0.5x)")
    print(f"{'tier':10s} {'premium SOL':>12s} | " + " ".join(f"{str(b)+'bps':>9s}" for b in FEE_BPS_GRID))
    for name, p, m in TIERS:
        prem = m * F_MAIN
        evs = [prem - vrf_attempt_cost(F_MAIN, b) / p for b in FEE_BPS_GRID]
        print(f"{name:10s} {prem:12.2f} | " + " ".join(f"{e:9.2f}" for e in evs))
    print("\nVariance (honest odds): attempts for 50% / 95% chance of >= 1 hit")
    for name, p, _ in TIERS:
        n50 = math.ceil(math.log(0.5) / math.log(1 - p)); n95 = math.ceil(math.log(0.05) / math.log(1 - p))
        c = vrf_attempt_cost(F_MAIN, 200)
        print(f"   {name:10s} p={p:.4%}: n50={n50:6d} ({n50*c:8.2f} SOL @200bps)  n95={n95:6d} ({n95*c:8.2f} SOL)")


# ---------------------------------------------------------------- Part C
def part_c():
    hr("Part C: cost to farm EVERY copy of a tier out of a full pool (coupon-collector, honest VRF, no refills)")
    N = 10_000
    print(f"EXAMPLE pool N = {N} NFTs (e.g. ratio 100k at max size), F = {F_MAIN} SOL. Expected attempts = N x H_L")
    for name, L in [("1/1", 1), ("legendary", 10), ("epic", 100)]:
        H = sum(1 / k for k in range(1, L + 1))
        att = N * H
        cells = []
        for b in FEE_BPS_GRID:
            c = vrf_attempt_cost(F_MAIN, b)
            cells.append(f"{b}bps: {att*c:9.1f} SOL ({att*c/L:7.2f}/each)")
        print(f"   {name:10s} L={L:3d}: attempts ~ {att:8.0f}\n      " + "\n      ".join(cells))


# ---------------------------------------------------------------- Part D
def part_d():
    hr("Part D: STOCK MPL-HYBRID (predictable pick / caller-chosen asset) vs proper VRF")
    rng_mean = mean_wait_slots(legendary_share=0.001, trials=1000)
    print(f"Reused model: mean slots a predict-and-abort bot waits for a 0.1% pick = {rng_mean:.0f} "
          f"(~{rng_mean*0.4/60:.1f} min at 400 ms/slot); original sim said 994.")
    for bps in [0, 200, 1000]:
        cap_tok = bps / 1e4 * F_MAIN
        cycle = 2 * MPLH_PROTOCOL_FEE + cap_tok + 2 * TX_FEE      # release + capture, project SOL fee 0 (EXAMPLE)
        print(f"\n-- project capture token fee {bps} bps (EXAMPLE), cycle = release+capture = {cycle:.4f} SOL")
        print(f"{'tier':10s} {'honest VRF E[cost]':>19s} {'MPLH choose-asset':>18s} {'MPLH predict+Jito':>18s} "
              f"{'MPLH predict no-Jito':>21s} {'reduction':>10s} {'EV/hit MPLH':>12s}")
        for name, p, m in TIERS:
            honest = vrf_attempt_cost(F_MAIN, bps) / p
            choose = MPLH_PROTOCOL_FEE + cap_tok + TX_FEE              # NoRerollMetadata: name the asset, 1 capture
            jito = cycle                                                # failed bundles never land (Jito docs)
            nojito = cycle + (1 / p) * 5000 / 1e9                       # each failed attempt lands and pays base fee
            print(f"{name:10s} {honest:19.2f} {choose:18.4f} {jito:18.4f} {nojito:21.4f} "
                  f"{honest/jito:9.0f}x {m*F_MAIN - jito:12.2f}")
    print("\nNote: with NoRerollMetadata the capturer NAMES the asset (capture_v2.rs:52-54), so every visible rare in escrow"
          "\ncosts one capture. With reroll on, the pick is a pure function of SlotHashes/unix_timestamp/count.")


def mean_wait_slots(legendary_share, trials):
    coll = 10_000
    target = int(coll * legendary_share)
    slot, ts, count, waits = 300_000_000, 1_790_000_000, 7, []
    for _ in range(trials):
        n = 0
        while True:
            n += 1
            h = random.getrandbits(256).to_bytes(32, "little")
            if mpl_hybrid_pick(slot_hashes_bytes(slot + n, h), ts + n * 4 // 10, count, 0, coll) < target:
                break
        waits.append(n)
    return sum(waits) / trials


# ---------------------------------------------------------------- Part E
def part_e():
    hr("Part E: proper VRF but with the documented `expire` refund used as a free abort (R1-B-02, Switchboard only)")
    print("If a requester learns the Switchboard value off-chain before reveal and nobody else reveals before deadline_slot,")
    print("`expire` refunds principal + fee minus VRF cost. A bad draw then costs only the VRF cost; only good draws settle.")
    print(f"{'tier':10s} {'per-abort cost':>15s} {'E[cost]/hit':>12s} {'vs honest @200bps':>18s} {'EV/hit':>9s}")
    for name, p, m in TIERS:
        for vrf in [ORAO_VRF_SOL, SOL_FEE_DEFAULT]:
            per = vrf + TX_FEE                     # request tx; the expire tx can be anyone's
            succ = vrf_attempt_cost(F_MAIN, 200)   # the one settled draw pays the full fee
            e = (1 / p - 1) * per + succ
            honest = vrf_attempt_cost(F_MAIN, 200) / p
            print(f"{name:10s} {per:15.4f} {e:12.2f} {honest/e:17.1f}x {m*F_MAIN - e:9.2f}   (VRF cost {vrf})")
    print("Selective expiry of an attacker's OWN earlier request also changes the pool the later request draws from")
    print("(k own pending requests ahead -> up to 2^k pool states to choose between). Same fix.")


# ---------------------------------------------------------------- Part F
V_SOL, V_TOK = 30.0, 1_073_000_000.0   # EXAMPLE constant-product virtual reserves (pump.fun-style shape; UNSOURCED)


def curve_price(tokens_sold):
    """Marginal price (SOL per token) on an x*y=k virtual curve after `tokens_sold` have been bought."""
    k = V_SOL * V_TOK
    tok = V_TOK - tokens_sold
    return k / tok / tok


def curve_cost(a, b):
    """SOL to buy tokens from sold=a to sold=b."""
    k = V_SOL * V_TOK
    return k / (V_TOK - b) - k / (V_TOK - a)


def part_f():
    hr("Part F: launch-window rare farming (reroll priced in tokens is cheapest when the token is cheapest)")
    print(f"EXAMPLE curve: constant product, virtual reserves {V_SOL} SOL / {V_TOK:,.0f} tokens (unsourced example).")
    R = 1_000_000
    for sold in [0, 50_000_000, 200_000_000, 600_000_000]:
        F0 = R * curve_price(sold)
        c = vrf_attempt_cost(F0, 200)
        print(f"   tokens sold {sold/1e6:5.0f}M: floor(R=1M) = {F0:.4f} SOL, attempt @200bps = {c:.5f} SOL, "
              f"E[cost] legendary = {c/0.001:7.3f} SOL, 1/1 = {c/0.0001:8.2f} SOL")
    F1 = 1.0
    print(f"\nIf the collection later trades at floor {F1} SOL (EXAMPLE) and a legendary at 20x floor:")
    F0 = R * curve_price(0)
    e = vrf_attempt_cost(F0, 200) / 0.001
    print(f"   legendary farmed at launch costs ~{e:.2f} SOL expected vs later premium {20*F1:.0f} SOL -> EV ~{20*F1 - e:.2f} SOL")
    print(f"   same farm after F reaches {F1}: E[cost] {vrf_attempt_cost(F1,200)/0.001:.2f} SOL -> EV {20*F1 - vrf_attempt_cost(F1,200)/0.001:.2f} SOL")
    print("   => the doc's scale-invariant fee is invariant to the CURRENT floor, not to the FUTURE premium. Early grinding")
    print("      is dominated by the flat SOL fee, and the snipers who already hold cheap tokens can do it first.")


# ---------------------------------------------------------------- Part G
def part_g():
    hr("Part G: SOL-only fee (the design mock's 0.01 / 0.02 SOL) -- break-even legendary premium in floors")
    for fee in [0.01, 0.02]:
        print("   fee {:.2f} SOL: ".format(fee) + "  ".join(
            f"F={F}: m*={(fee + 2*TX_FEE)/0.001/F:8.1f}" for F in [0.03, 0.3, 3.0, 30.0]))
    print("   => a fixed SOL fee gets cheaper relative to the floor exactly when rares are worth most (doc's own argument).")


# ---------------------------------------------------------------- Part H
def part_h():
    hr("Part H: cost to corner a whole (small) collection from the first slot on the EXAMPLE curve (R1-B-06)")
    for N, R in [(10, 1_000_000), (100, 1_000_000), (100, 200_000), (1_000, 100_000), (365, 1_000_000)]:
        need = N * R
        c = curve_cost(0, need)
        cap = vrf_attempt_cost(R * curve_price(need), 200) * N
        print(f"   N={N:5d} x R={R:>9,} = {need/1e6:6.0f}M tokens ({need/1e9:6.1%} of supply): "
              f"curve cost ~{c:8.2f} SOL + capture fees ~{cap:6.3f} SOL")
    print("   (365M = the 36.5% of supply MELT found in bundled accounts at migration, research-sources.md section 3)")


if __name__ == "__main__":
    part_a(); part_b(); part_c(); part_d(); part_e(); part_f(); part_g(); part_h()
