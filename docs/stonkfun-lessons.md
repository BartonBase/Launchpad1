# Stonk.fun: what actually happened (source: Bitquery investigation, measured 2026-09-22)
https://bitquery.io/investigations/is-stonkfun-dumping-on-holders

- Not a hack or exploit bot. The viral "STONK FEE DRAIN" wallet is Stonk.fun's own reward wallet. Reward coins carry a 1% or 3%
  transfer tax (Token-2022 style, withheld in the recipient account). One wallet sweeps it, sells it into each coin's own pool,
  and pays holders in the pair asset. $56.3M sold vs $56.2M paid out over 30 days.
- The harm is the design: the tax sells into every chart. 160 coins lost >50% of supply to tax (BUDDY 69%, ZCAT 61%).
  Snipers churning in the first minutes generate the heaviest tax (ZCAT lost 20% of supply in hour one, sold for ~$18K).
- Trust failures: >=$1.41M of reward money went in single transfers to Stonk.fun-linked wallets (creator, treasury, a 1,000 SOL transfer), omitted from its public ledger.
  On 2,178 LaunchLab coins the reward wallet can still raise the tax to 100% after a delay of a few days (never used).
  Claimed buybacks ($2.09M) were overstated (chain shows $1.23M).

## Lessons for Mintmark (SPL-404 track)
1. No transfer tax (already dropped with Token-2022). Confirms the scope change.
2. No discretionary platform wallet that custodies or routes user value. Every flow is program-enforced; nothing is swept by an off-chain operator.
3. No retained power to change economics after launch. If anything must be adjustable, a multisig plus a long, visible timelock, with hard caps set in code. Prefer immutable.
4. Every fee destination is on-chain, public, and matches what the site says. No side payments outside the documented flows.
5. Launch-window sniping is a real, measurable threat on bonding-curve launches. Design anti-sniping for the curve's first minutes.
6. Publish a verifiable ledger and let anyone reconcile it against the chain.
