#!/usr/bin/env python3
"""
Lottery ticket-formula sybil / flash-snapshot simulation (design-level, no chain access).

Questions answered:
  1. For a fixed attacker capital C, how does splitting it across k wallets change the
     attacker's expected share of lottery tickets under different ticket formulas?
  2. What does splitting cost (Token-2022 transfer fee on each split transfer + rent for each
     new Token-2022 token account)?
  3. How much share does a "flash" holder (buys right before a point-in-time snapshot and sells
     right after) get under a point snapshot vs a time-weighted average balance (TWAB)?

All numbers are synthetic. Holder balances are drawn from a lognormal distribution (a common
rough shape for token holder distributions); change the parameters to match real data later.
Deterministic seed so output is reproducible.
"""
import math
import random

random.seed(1337)

# ---------------------------------------------------------------- parameters
N_HOLDERS = 5_000                  # honest holders
TOTAL_SUPPLY = 1_000_000_000       # tokens
HONEST_FRACTION = 0.60             # fraction of supply held by honest, lottery-eligible wallets
ATTACKER_CAPITAL = 5_000_000       # tokens (0.5% of supply)
MIN_BALANCE = 1_000                # eligibility threshold (tokens)
CAP = 1_000_000                    # per-wallet cap for "capped" formulas (tokens)
TRANSFER_FEE_BPS = 100             # 1% Token-2022 transfer fee (example tier)
# Token-2022 ATA with TransferFeeAmount + ImmutableOwner extensions:
#   165 (base) + 1 (account type) + 12 (TransferFeeAmount TLV: 2+2+8) + 4 (ImmutableOwner TLV: 2+2)
#   = 182 bytes. Rent-exempt minimum = (128 + size) * 3480 lamports/byte-year * 2 years.
ATA_SIZE = 182
RENT_LAMPORTS = (128 + ATA_SIZE) * 3480 * 2
BASE_FEE_LAMPORTS = 5_000          # per signature
LAMPORTS_PER_SOL = 1_000_000_000
TOKEN_PRICE_SOL = 0.000_001        # example: 1 token = 1e-6 SOL (1B supply => 1,000 SOL FDV)

FORMULAS = {
    "flat_per_wallet (1 ticket if >= min)": lambda b: 1.0 if b >= MIN_BALANCE else 0.0,
    "linear (tickets = balance)":           lambda b: float(b) if b >= MIN_BALANCE else 0.0,
    "sqrt(balance)":                        lambda b: math.sqrt(b) if b >= MIN_BALANCE else 0.0,
    "log10(balance)":                       lambda b: math.log10(b) if b >= MIN_BALANCE else 0.0,
    "capped linear min(balance, CAP)":      lambda b: float(min(b, CAP)) if b >= MIN_BALANCE else 0.0,
    "tiered floor(balance/100k)":           lambda b: float(b // 100_000),
}


def honest_balances():
    raw = [random.lognormvariate(0, 2.0) for _ in range(N_HOLDERS)]
    s = sum(raw)
    target = TOTAL_SUPPLY * HONEST_FRACTION
    return [max(1, int(x / s * target)) for x in raw]


def attacker_share(formula, honest, capital, k):
    """Attacker splits `capital` across k wallets. Each split transfer pays the transfer fee
    (fee is withheld from the amount received), except the first wallet keeps its funds."""
    per_wallet_sent = capital / k
    received = per_wallet_sent * (1 - TRANSFER_FEE_BPS / 10_000)
    wallets = [per_wallet_sent] + [received] * (k - 1)
    t_att = sum(formula(int(w)) for w in wallets)
    t_hon = sum(formula(b) for b in honest)
    return t_att / (t_att + t_hon) if (t_att + t_hon) > 0 else 0.0


def split_cost_sol(capital, k):
    tokens_lost = (capital / k) * (k - 1) * TRANSFER_FEE_BPS / 10_000
    rent = (k - 1) * RENT_LAMPORTS / LAMPORTS_PER_SOL          # recoverable only if account closed
    tx_fees = (k - 1) * BASE_FEE_LAMPORTS / LAMPORTS_PER_SOL    # ~1 sig per transfer (batchable)
    return tokens_lost * TOKEN_PRICE_SOL, rent, tx_fees


def monte_carlo_check(formula, honest, capital, k, draws=20_000):
    """Empirically draw winners proportional to tickets and count attacker wins."""
    per = capital / k
    rec = per * (1 - TRANSFER_FEE_BPS / 10_000)
    att = [formula(int(per))] + [formula(int(rec))] * (k - 1)
    hon = [formula(b) for b in honest]
    t_att, t_all = sum(att), sum(att) + sum(hon)
    wins = sum(1 for _ in range(draws) if random.random() * t_all < t_att)
    return wins / draws


def main():
    honest = honest_balances()
    eligible = sum(1 for b in honest if b >= MIN_BALANCE)
    print("=== Lottery sybil-splitting simulation ===")
    print(f"honest holders={N_HOLDERS} (eligible>=min: {eligible}), honest supply={int(sum(honest)):,}")
    print(f"attacker capital={ATTACKER_CAPITAL:,} tokens "
          f"({ATTACKER_CAPITAL / TOTAL_SUPPLY:.2%} of supply), transfer fee={TRANSFER_FEE_BPS} bps")
    print(f"Token-2022 ATA rent (182 bytes) = {RENT_LAMPORTS} lamports = {RENT_LAMPORTS / 1e9:.7f} SOL\n")

    # k is bounded by capital*(1-fee)/MIN_BALANCE (= 4,950 here): below MIN a wallet earns nothing.
    ks = [1, 10, 100, 1_000, 4_000]
    header = f"{'formula':40s}" + "".join(f"{'k=' + str(k):>11s}" for k in ks) + f"{'x(k=1000)':>11s}"
    print("Attacker expected share of tickets (= P(win) for a single-winner draw):")
    print(header)
    print("(x(k=1000) = share multiplier vs a single wallet; >1x means the formula rewards sybil splitting)")
    for name, f in FORMULAS.items():
        shares = [attacker_share(f, honest, ATTACKER_CAPITAL, k) for k in ks]
        mult = shares[3] / shares[0] if shares[0] > 0 else float("inf")
        print(f"{name:40s}" + "".join(f"{s:11.4%}" for s in shares) + f"{mult:10.1f}x")

    print("\nMonte Carlo sanity check (20k draws), k=100:")
    for name in ["flat_per_wallet (1 ticket if >= min)", "linear (tickets = balance)", "sqrt(balance)"]:
        f = FORMULAS[name]
        print(f"  {name:40s} analytic={attacker_share(f, honest, ATTACKER_CAPITAL, 100):.4%}"
              f"  empirical={monte_carlo_check(f, honest, ATTACKER_CAPITAL, 100):.4%}")

    print("\nCost of splitting (example token price = "
          f"{TOKEN_PRICE_SOL} SOL/token):")
    print(f"{'k':>6s} {'fee loss SOL':>14s} {'rent SOL (locked)':>18s} {'tx fees SOL':>12s}")
    for k in ks:
        fee, rent, tx = split_cost_sol(ATTACKER_CAPITAL, k)
        print(f"{k:6d} {fee:14.4f} {rent:18.4f} {tx:12.5f}")

    # ------------------------------------------------ flash snapshot vs TWAB
    print("\n=== Flash-buy before snapshot: point snapshot vs time-weighted balance ===")
    SLOTS_PER_EPOCH = 216_000   # ~1 day at ~400ms slots (window for TWAB)
    flash = 50_000_000          # tokens bought 1 slot before snapshot, sold right after (5% supply)
    hold_slots = 2
    t_hon = sum(honest)
    point = flash / (flash + t_hon)
    twab_att = flash * hold_slots / SLOTS_PER_EPOCH
    twab = twab_att / (twab_att + t_hon)
    rt_fee = 2 * TRANSFER_FEE_BPS / 10_000  # buy leg + sell leg transfer fees (approx.)
    print(f"flash position={flash:,} tokens held {hold_slots} slots; honest supply={t_hon:,}")
    print(f"  point-in-time snapshot share : {point:.4%}")
    print(f"  TWAB over {SLOTS_PER_EPOCH} slots share  : {twab:.6%}")
    print(f"  round-trip transfer fee cost : {rt_fee:.2%} of position "
          f"= {flash * rt_fee * TOKEN_PRICE_SOL:.2f} SOL at example price (+ AMM spread/price impact)")
    print("  => breakeven prize for point snapshot attack: prize * share > round-trip cost, i.e. prize >"
          f" {flash * rt_fee * TOKEN_PRICE_SOL / point:.1f} SOL (ignoring price impact)")


if __name__ == "__main__":
    main()
