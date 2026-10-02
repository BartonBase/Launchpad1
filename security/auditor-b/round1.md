# Mintmark: Security review round 1, Auditor B (economic and bot exploits against the SPL-404 hybrid model)

**Prepared for:** Barton
**Date:** 2026-09-24 (written 14:52 to 14:58 MT)
**Reviewer:** Auditor B worker (independent; no Auditor A material was opened, listed, or searched)
**Status:** Design review. No program code exists. Nothing deployed, no mainnet access, no real keys or funds used.

## Scope

- **In scope (round 1):** SPL-404 hybrid launches only. A classic SPL token (1B fixed supply, mint authority revoked) that converts to and from Metaplex Core NFTs at a creator-chosen ratio (10k / 50k / 100k / 200k / 1M tokens per NFT), with cosmetic rarity, blind assignment, a VRF re-roll, and multisig + timelock on anything changeable. The bonding-curve launch, graduation, and liquidity migration that sit in front of it are also in scope.
- **Lens:** economic and bot exploits: sniping and bundling, wrap/unwrap arbitrage, rare farming via re-rolls, escrow peeking and cherry-picking, MEV on converts, escrow griefing, graduation and migration, and operator powers that would recreate Stonk.fun's trust problems.
- **Out of scope / deferred (BRIEF SCOPE CHANGE, 2026-09-24 2:51 PM MT):** Token-2022, transfer tax, Rewards launch type, `fee_treasury`, `holder_lottery`, and the ticket lottery. My earlier findings that only apply there (B-02, B-04, B-06, B-07, B-08, B-09, B-13, B-14, and the Token-2022 parts of B-03/B-05) are **deferred, not deleted**. See the "Deferred" table at the end.

## Documents read (mtime, box-local MT, 2026-09-24)

| Doc | Last modified | Notes |
|---|---|---|
| docs/BRIEF.md | 14:52 | Includes the SCOPE CHANGE section (2:51 PM MT) |
| docs/DECISIONS.md | 14:45 | ADR-001..008. ADR-008 (custom `hybrid_vault`) is Proposed |
| docs/hybrid-rarity-and-assignment.md | 14:46 | The main hybrid design. Status "Proposed design, no program code yet" |
| docs/transfer-tax-vs-wrap.md | 14:46 | ADR-004 analysis. Rewards half now shelved |
| docs/stonkfun-lessons.md | 14:51 | Summary of Barton's Bitquery source |
| docs/ARCHITECTURE.md | 14:46 | Bonding curve / AMM still "third party (TBD)" |
| docs/THREAT_MODEL.md | 14:46 | Engineering threat model. T-HV-01..17 cover `hybrid_vault` |
| Bitquery, "Is StonkFun dumping on holders?" (https://bitquery.io/investigations/is-stonkfun-dumping-on-holders) | read 14:53 | Primary source behind stonkfun-lessons.md. Figures "verified September 22, 2026" |
| Switchboard randomness docs + tutorial (https://docs.switchboard.xyz/docs-by-chain/solana-svm/randomness, https://docs.switchboard.xyz/docs-by-chain/solana-svm/randomness/randomness-tutorial.md) | read 14:54 | New source for R1-B-02 |
| My prior work: threat-model.md (14:30), research-sources.md (14:28), design-requirements.md (14:16), code-review-no-commits.md (14:31), sim/reroll_ev.py + output (14:14), sim/lottery_sybil.py + output (14:13) | | Reused. Nothing in them was edited |

## Repo status

`BartonBase/Launchpad1` is **still empty**. At 14:54 MT the public page https://github.com/BartonBase/Launchpad1 returned HTTP 200 with "This repository is empty". The unauthenticated GitHub REST API returned 403 (rate limit exceeded for the box's IP), so I couldn't re-read the commits endpoint. The HTML check is the evidence. The repo wasn't cloned. Every finding below is therefore **design-level**, and each one is marked:
- **[DOCUMENTED]**: applies to the design as written today.
- **[CONDITIONAL]**: applies only "if the design does X", usually because the relevant part (bonding curve, migration, VRF provider, fee destination) isn't decided yet.

## Doc contradictions and changed assumptions (read this first)

1. **Swap engine.** The BRIEF SCOPE CHANGE says conversion is "MPL-Hybrid-based". ADR-008, hybrid-rarity-and-assignment.md §3, and ARCHITECTURE.md all say MPL-Hybrid is **not in the swap path** and propose a custom `hybrid_vault`. That's still **Proposed**, and Q-H1 is unanswered. If stock MPL-Hybrid is really used, R1-B-01 is Critical and applies as documented.
2. **Re-roll fee destination.** The BRIEF SCOPE CHANGE says the fee is paid in the collection token and **burned**. hybrid-rarity §4 and DECISIONS Q-H3 still say "**Burning isn't recommended**… Confirm no burn". The BRIEF supersedes, but the design doc hasn't been updated (see R1-B-12).
3. **Stonk.fun.** BRIEF hard requirement #5 says "a bot drained tokens and liquidity via Stonk.fun's distribution layer". docs/THREAT_MODEL.md and my research-sources.md §1 say no credible report was found. Barton's Bitquery source settles it: this was **not a hack or a bot**. The "STONK FEE DRAIN" wallet is Stonk.fun's own reward wallet. The documented harms are the tax design, off-ledger payments to insiders, and a retained power to raise the tax. My earlier "no exploit found" was correct, but I didn't have this investigation. The trust failures are now sourced (R1-B-10). Also note that the "single-wallet launches" point comes from The Coinomist's second-hand report of a StonkFun post (research-sources.md §1), not from Bitquery. Bitquery documents launch-minute sniper churn instead.
4. **Fee model.** My earlier sim (`reroll_ev.py`) assumed a 0.01 SOL project fee per leg, Metaplex fees, and Token-2022. The documented design is 200 bps of R in tokens (cap 1,000 bps) plus about 0.003 SOL (cap 0.05). There are no Metaplex fees if `hybrid_vault` is used. hybrid-rarity §4 still quotes my old "~50 SOL at a 0.05 SOL cycle" number, which those superseded parameters produced. The new numbers are in `sim/reroll_ev_r1_output.txt`.
5. **Bonding curve and migration are undecided.** ARCHITECTURE.md lists "Bonding curve / AMM: third party (TBD: Raydium CPMM / Meteora DAMM v2)". DECISIONS Q2 ("anti-sniping") is about the shelved lottery, not the curve. So stonkfun-lessons #5 (anti-sniping on the curve) has no design yet. R1-B-03 and R1-B-09 are therefore conditional.
6. **Pause.** ARCHITECTURE.md Principles say "Admin powers are limited to pausing and bounded parameter changes", but the `hybrid_vault` instruction table has no pause. Whether release can be paused is undefined (R1-B-10).
7. **Stale rows.** docs/THREAT_MODEL.md T-MKT-03 ("only venues that support TransferFee mints") and ADR-003/005/006 are Token-2022-era and should be marked deferred.

## Findings summary

| ID | Severity | Applies | Title |
|---|---|---|---|
| R1-B-01 | **Critical** | CONDITIONAL (stock MPL-Hybrid in the swap path) | Cherry-picking and predict-and-abort re-rolls on stock MPL-Hybrid: rares cost 0.005 to 0.04 SOL instead of 3 to 330 SOL |
| R1-B-02 | **High** | DOCUMENTED (if Switchboard, the preferred VRF) | `expire` refund plus off-chain early view of Switchboard randomness = free abort of bad draws and pool-shifting by selective expiry |
| R1-B-03 | **High** | CONDITIONAL (curve not designed) | Launch sniping, Jito and dev bundles on the bonding curve, amplified by a fixed NFT supply |
| R1-B-04 | **High** | DOCUMENTED | Launch-window rare farming: a bps-of-R fee is cheapest when the token is cheapest, so bots farm rares for about 3.7 SOL per legendary that later carry about 20 SOL premiums (example) |
| R1-B-05 | Medium | DOCUMENTED | Re-roll fee sizing: 2% default breaks even only at ~20 to 30x floor for a 0.1% tier. No minimum fee, and fee cuts via timelock favour insiders |
| R1-B-06 | Medium | DOCUMENTED | Escrow cornering and queue griefing: small collections can be fully captured for a few SOL; head-of-line DoS via unrevealed requests |
| R1-B-07 | Medium | DOCUMENTED | Wrap/unwrap arbitrage against stale SOL-priced listings and collection offers; rare-depleted pool vs the published census |
| R1-B-08 | Low (Medium if CONDITIONAL) | DOCUMENTED / CONDITIONAL (swap+convert bundled in one tx) | MEV and sandwiching around converts |
| R1-B-09 | **High** | CONDITIONAL (migration not designed) | Graduation and migration: privileged migrate key (pump.fun precedent), pool-creation front-running, price gap at graduation, LP ownership |
| R1-B-10 | **High** (Critical if any single key) | DOCUMENTED + CONDITIONAL | Operator and authority powers that recreate Stonk.fun's trust problems (shared upgrade key over every vault, fee and destination changes on a 72h timelock, pause, creator allocations, undocumented platform fees) |
| R1-B-11 | Medium | DOCUMENTED | Metadata commitment covers the JSON but not the image bytes, so a mutable image host lets the creator swap art after reveal |
| R1-B-12 | Low | DOCUMENTED | Burn vs no-burn contradiction; burns falsify the "up to N×R as NFTs" copy and strand the last NFTs at max size |
| R1-B-13 | Medium | DOCUMENTED | Creator can freeze all captures and re-rolls by withholding the trait manifest (a fulfilled head can't settle without a Merkle proof, and can't expire) |
| R1-B-14 | Info | DOCUMENTED | Pool census honesty, the `[min, max)` off-by-one, and other carry-overs (B-12/B-15/B-16 re-scoped) |

---

## Findings

Sim references: `sim/reroll_ev_r1.py` → `sim/reroll_ev_r1_output.txt` (Parts A to H). Unless a number is attributed to a source, all attacker costs are **estimates** from that sim. Its rarity tiers, premiums, floors, and curve reserves are **example parameters** (the docs don't give a rarity table), and the file labels them that way.

### R1-B-01: Cherry-picking and predict-and-abort re-rolls on stock MPL-Hybrid. **Critical**. [CONDITIONAL: MPL-Hybrid in the swap path]

**Condition.** Conversion uses stock MPL-Hybrid, which the BRIEF SCOPE CHANGE wording ("MPL-Hybrid-based") implies, instead of ADR-008's `hybrid_vault`. It doesn't matter whether reroll is on or off.

**Why.** Source-confirmed at `aacf1a53` (research-sources.md §5; hybrid-rarity §3.1):
- With `NoRerollMetadata` set, the capturer **names the asset** (`capture_v2.rs:52-54`). All escrow contents and metadata are public, so every visible rare costs one capture.
- With reroll on, the index is `(u64(SlotHashes[12..20]) − unix_timestamp) × count mod (max − min) + min` ([capture.rs L167-183](https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture.rs#L167-L183)). That's a pure function of values readable before execution, and capture is CPI-callable. Metadata URIs must be sequential `<base><index>.json`, which is the "Predictable URI" anti-pattern Metaplex warns about (research-sources.md §4). The draw is also with replacement, so the census isn't preserved, and index `max` is unreachable (B-16).

**Scenario (reroll on).**
1. The bot scrapes `<base>0.json … <base>N.json` and ranks indices by rarity.
2. It deploys a wrapper that recomputes the index from SlotHashes/Clock/count and aborts unless the index is in the target set. Otherwise it CPIs release+capture.
3. It sends the wrapper in a Jito bundle every slot. Jito: "If any transaction in a bundle fails, none of the transactions in the bundle will be committed" (https://docs.jito.wtf/lowlatencytxnsend/), so misses cost nothing on-chain.
4. The sim's mean wait for a 0.1% pick is about 948 slots (~6.3 min). It then lands one cycle. The bot repeats until the tier is empty.

**Cost to attacker (sim Part D, F = 0.3 SOL example floor).**
- One cycle is 2 × 0.005 SOL Metaplex protocol fee (sourced, research-sources.md §5) plus the project token fee.
- With a 2% project fee: **0.016 SOL per legendary or 1/1**, vs 9.10 / 91.00 SOL expected under honest VRF at the same fee. That's **565x / 5,652x cheaper**. EV per legendary is +5.98 SOL at a 20x premium.
- Without Jito, each miss lands and pays a 5,000-lamport base fee: +0.005 SOL per 1,000 misses.
- Choosing mode: **0.011 SOL per rare**.
- One-off wrapper build: a few engineer-days (estimate).

**Fix.** Keep MPL-Hybrid out of the swap path. Adopt ADR-008's `hybrid_vault`, with two-step VRF, FIFO, and the pool fixed at request time. Correct the BRIEF wording so engineering doesn't build on MPL-Hybrid by mistake. If Barton ever wants a "pick your NFT" mode, it must be a separate, disclosed launch type with value-uniform traits. Add the R-01 PoC test (predict-and-abort success rate = base rate) to CI.

### R1-B-02: Switchboard early view + `expire` refund = free abort of bad draws. **High**. [DOCUMENTED if Switchboard is chosen; DECISIONS Q1 leans Switchboard]

**Design as written (hybrid-rarity §3.3).** "If its VRF account is still *unfulfilled* after the deadline, anyone can `expire` it: principal is refunded, the fee is refunded minus VRF cost". "A requester who withholds a Switchboard reveal gains nothing (anyone can reveal)". "Our crank runs it".

**Why it breaks.** Switchboard's own flow says: "Alice sends the oracle assignment, randomness ID, and some other data to Crossbar to get the randomness … The Oracle creates a randomness object and sends it to Crossbar which passes it back to Alice. Alice sends the randomness object to the App" (https://docs.switchboard.xyz/docs-by-chain/solana-svm/randomness). So the requester can see the value **before** it's on-chain. Switchboard's tutorial names the resulting attack: if payment isn't locked irrevocably at commit, "a malicious user could: 1. Commit to randomness 2. Wait for reveal 3. Only reveal if they won (selective revelation attack)" (randomness-tutorial.md). `expire` gives the payment back, which recreates exactly that attack whenever nobody else reveals before `deadline_slot`. The docs I read don't say whether a third party can fetch the reveal from Crossbar for someone else's randomness account. "Anyone can reveal" is **unverified**, and the defence depends on our crank's liveness.

**Scenario A (abort).**
1. The attacker calls `request_reroll` (or `request_capture`) with a fresh randomness account.
2. They fetch the value from Crossbar off-chain and compute `uniform_below(r, pool_len)` against the pool, which is fully determined at request time by the FIFO/seq rule.
3. If the draw is rare, they reveal and settle.
4. If not, they don't reveal. They win if the crank is down, lagging, rate-limited, or can't obtain the reveal before `deadline_slot`. Then they call `expire` and get principal plus fee back, minus VRF cost.

**Scenario B (pool shifting by selective expiry).** The attacker holds requests s−1 and s. Knowing both values early, they choose whether s−1 settles (swap-removing an index) or expires. That changes the pool s draws from. With k own requests queued ahead, they choose among up to 2^k pool states. This defeats the FIFO defence that T-HV-03 relies on.

**Scenario C (stale randomness).** If `request_*` accepts a caller-created randomness account without Switchboard's documented checks (`seed_slot == clock.slot − 1` and "ensure randomness isn't already revealed", randomness-tutorial.md §Security Best Practices), the attacker submits an account whose value they already know.

**Cost to attacker (sim Part E, estimate).** Each aborted draw costs only the VRF cost plus one tx (0.001 to 0.003 SOL). The expected cost per legendary drops from 9.10 SOL (honest, 200 bps, F = 0.3) to **1.06 to 3.06 SOL (3 to 9x cheaper)**, EV +2.9 to +4.9 SOL per hit. For a 1/1 it drops from 91 to **10.5 to 30.5 SOL**. There are no capital costs, because principal is refunded.

**Fix.**
- **No refund of the token fee on `expire`, ever.** Also refund principal only after a long deadline (e.g. ≥ 1 day). Or prefer ORAO, where "nodes fulfil, so there's no withheld reveal" (DECISIONS Q1).
- `request_*` must enforce Switchboard's freshness and not-yet-revealed checks and pin the account (the THREAT_MODEL has this for the lottery, T-LOT-05, but not for T-HV).
- Verify in a devnet test that a third party can obtain and submit the reveal. If it can't, Switchboard is unfit for this flow.
- Run a redundant crank with alerts. Test: an attacker who withholds a reveal and calls `expire` gets back ≤ principal, and a later settle order is unchanged by the expiry. Treat expiry as a monitored incident.

### R1-B-03: Launch sniping and bundling on the bonding curve, amplified by a fixed NFT supply. **High**. [CONDITIONAL: the curve isn't designed; stonkfun-lessons #5 requires it]

**Evidence.**
- Stonk.fun (Bitquery): "Within a minute of one launch on 20 September, 522 wallets had traded 3.4 times the coin's entire supply between them"; "Sniper bots buy in the first seconds, flip, and buy again"; ZCAT "lost 20% of its supply to tax in its first hour". stonkfun-lessons.md: "Launch-window sniping is a real, measurable threat on bonding-curve launches. Design anti-sniping for the curve's first minutes."
- Single-wallet launches: The Coinomist, second-hand, research-sources.md §1.
- pump.fun (MELT, arXiv 2602.13480): "36.5% of the total token supply is held by bundled accounts at the point of migration" and "98.7% of 'create' events co-occur with developer buys in the same transaction".

**Hybrid-specific amplification.**
1. The NFT supply is a hard cap (`collection_size`), so early token buyers can pre-empt the NFTs themselves (R1-B-06).
2. Early token holders can also grind rares at launch prices (R1-B-04).
3. A creator's same-transaction dev buy is effectively a pre-mint allocation of NFTs, which is what Q-H6 wants to forbid.

**Scenario.**
1. The creator (or a bot watching the create instruction) bundles `[create + seed curve, dev buy, buy_1…buy_4]` in one Jito bundle.
2. The bundle takes 10 to 36% of supply at the lowest curve prices.
3. After the permutation seed is stored, the bundle wallets `request_capture` en masse and take most of a small collection.
4. They farm rares while fees are cheapest (R1-B-04), then dump tokens into retail buys.

**Cost to attacker (sim Part H, example curve 30 SOL / 1.073B virtual reserves, unsourced).** About **3.1 SOL for 10% of supply** and about **15.5 SOL for 36.5%**, plus Jito tips. Sandwich bundles in the IMC 2025 study tipped a median of more than 2,000,000 lamports (research-sources.md §3).

**Fix.**
- **Use a program-enforced launch window.** Trading opens at a stored `open_slot` (R-10.2). For the first N slots, enforce a per-wallet max and a per-slot max, **or** run a short commit window with uniform-price batch clearing (R-10.1).
- The creator's buy is disclosed on the page, capped (e.g. ≤ 2% of supply), and routed through the same caps. Better, lock it in a vesting PDA.
- Keep captures closed until the launch window has ended (see R1-B-04), and display top-holder concentration.
- Caps are sybil-able. Treat them as speed bumps and pair them with batch clearing.

### R1-B-04: Launch-window rare farming. **High**. [DOCUMENTED]

**Design as written.** The fee is "a token fee as a % of the ratio (scale-invariant) plus a small SOL fee" (~0.003 SOL). "A fee of f × R tokens stays the same fraction of floor at any price" (hybrid-rarity §4). Capture opens as soon as the permutation seed is stored at lock, which can be before or during the curve.

**Why it's exploitable.** The fee is invariant to the **current** floor. A rare's premium is priced on the **future** floor. At launch the floor is tiny, so the bps part is negligible and the flat SOL fee dominates. Sim Part F, R = 1M on the example curve: the floor at launch is **0.028 SOL**, and one re-roll at 200 bps costs **0.00366 SOL** (of which 0.003 is the SOL fee). The expected cost per 0.1% legendary is **3.66 SOL**, and per 0.01% 1/1 it's **36.6 SOL**. If the collection later trades at a 1 SOL floor with legendaries at 20x (example), the launch-window farm has **EV ≈ +16.3 SOL per legendary**, while the same farm at the 1 SOL floor has EV −3.1 SOL. Raising the bps cap to 1,000 still costs only 0.0059 SOL per attempt at launch. The fee cap can't close this.

**Scenario.**
1. Snipers (R1-B-03) hold cheap tokens in the first minutes.
2. They capture one NFT each and loop `request_reroll` about 1,000 times per legendary (2,995 for 95% confidence).
3. They sweep a large share of the legendaries and 1/1s before retail arrives.
4. They list rares after price discovery.

**Cost to attacker (estimate, sim Part F).** About 3.7 SOL per legendary and 37 SOL per 1/1 at launch, versus 23 and 230 SOL at a 1 SOL floor. Throughput: two txs per attempt with a few seconds of VRF latency. A fleet of wallets parallelises this (the FIFO serialises settles, not requests).

**Fix (pick one or combine).**
- **(a) No re-rolls until graduation**, or until a fixed time after `open_slot` (e.g. 72h), enforced on-chain.
- **(b) Delayed reveal.** Request the permutation seed only after the launch window, so early captures get indices whose traits are unknown and re-rolling has no information value. This preserves the "no peeking" property, and the trait root is still committed before launch.
- **(c) A minimum re-roll cost in SOL** that scales with collection size (e.g. ≥ `k / p_rarest` lamports). It's cruder.
- Also publish that rares minted during the window are subject to (a) or (b).

### R1-B-05: Re-roll fee sizing. **Medium**. [DOCUMENTED]

**Design as written.** Default `reroll_fee_bps` = 200 (2%), hard cap 1,000, SOL fee ~0.003 (cap 0.05). "Honest grinding … has break-even `f ≥ p × m`. For example, a legendary at 0.1% … at 20× floor is break-even at f = 2%" (hybrid-rarity §4). Fees change via `propose_fees` → ≥ 72h → `execute_fees`. No minimum fee is stated.

**Numbers (sim Parts A to C, honest VRF, example tiers).**
- **Break-even premium in floors at 200 bps:** legendary (0.1%) **30.3x at F = 0.3 SOL**, 21.0x at F = 3, 123x at F = 0.03. For a 1/1 (0.01%) it's 303x / 210x / 1,233x. The doc's rule gives 20x / 200x because it drops the SOL part, which dominates at small floors.
- **At 0 bps (if allowed):** break-even for a legendary falls to **10.3x** (F = 0.3) and **1.0x** (F = 3). Grinding epics (1%) is +EV at any premium above 0.1x at F = 3.
- **At the 1,000 bps cap:** legendary 110x, 1/1 1,103x (F = 0.3).
- **EV per hit at F = 0.3, 200 bps:** legendary at 20x −3.10 SOL; epic at 3x −0.01 (break-even); rare at 0.5x −0.03. So the 2% default sits **right at break-even for mid tiers**, and small premium moves flip it.
- **Farming all 10 legendaries from a 10,000-NFT pool:** about 29,290 attempts, **266.5 SOL at 200 bps** (26.65 SOL each), 90.8 SOL at 0 bps.

**Issues.**
1. No hard **minimum**. A creator (or the multisig after 72h) can set 0 bps, and insiders who know the change is queued can position to grind the moment it executes.
2. Odds are computed from the census, but the live pool drifts (R1-B-07).
3. The destination changed to burn (R1-B-12). That doesn't change attacker cost.

**Cost to attacker.** As above (estimate). No special capability is needed.

**Fix.**
- A hard-coded **floor** on `reroll_fee_bps` (e.g. ≥ 100) that can only be raised after launch, never lowered, plus `capture_fee_bps ≥ reroll_fee_bps` (already planned).
- Show live odds per tier from the **current pool census** on the re-roll button, with the expected cost to hit each tier.
- Re-run `reroll_ev_r1.py` with each launch's real census and floor as part of the launch wizard. Refuse launch if `(fee + sol/F)/p_rarest < 2 × creator's stated rarity multiple` (R-12.1).

### R1-B-06: Escrow cornering and queue griefing. **Medium**. [DOCUMENTED]

**Design as written.** `collection_size ≥ 1` is the only lower bound. Undersized collections are allowed ("Up to {N×R} … can be held as NFTs at any one time"). Reservations make `request_*` fail when nothing is unreserved. Each request locks R tokens or an NFT. `expire` refunds unfulfilled heads after `deadline_slot`.

**Scenarios.**
1. **Cornering.** For a small collection (e.g. N = 100 at R = 1M = 10% of supply), a first-slot buyer takes N×R tokens and captures every NFT. Honest users then see "No NFTs available to draw" until someone releases, and the cornerer sets the NFT floor. Sim Part H (example curve): **~0.28 SOL for N = 10, ~3.1 SOL for N = 100 at R = 1M, ~0.6 SOL for N = 100 at R = 200k**, plus capture fees.
2. **Holding rares out.** A holder can't target rares (VRF), but rational holders keep rares and release commons. This is market behaviour, not an exploit, and it feeds R1-B-07.
3. **Head-of-line DoS.** Under Switchboard, a requester who never reveals blocks the FIFO until `deadline_slot`. With k such requests queued and `expire` handling one head at a time, captures and re-rolls stall for about k × deadline. If `expire` refunds fees (R1-B-02), each stall costs only the VRF fee.
4. **Dust.** Anyone can send tokens to the vault ATA, or foreign Core assets to the vault PDA. That's harmless **only if** no instruction reads balances or owned-asset lists for accounting (the doc uses `≥` in the invariant, which is correct).

**Cost to attacker.** Cornering costs a few SOL for small collections (estimate, Part H). The DoS costs ~0.001 to 0.003 SOL per blocked deadline (estimate).

**Fix.**
- A minimum collection size and a minimum convertible share (e.g. N ≥ 1,000, or N×R ≥ 10% of supply).
- During the launch window, a per-wallet capture limit (sybil-able speed bump) and no captures before the window ends (R1-B-03).
- `expire` forfeits fees and can expire any number of consecutive stale heads in one call (bounded batch).
- A short `deadline_slot`, with our crank revealing within a few slots.
- Tests: donation of tokens or foreign assets doesn't change any outcome, and a queue of 100 unrevealed requests clears within one batch after the deadline.

### R1-B-07: Wrap/unwrap arbitrage against stale prices and a rare-depleted pool. **Medium**. [DOCUMENTED]

**Mechanism.** Conversion is exact both ways. Release is instant and one transaction; capture takes two transactions and seconds of VRF latency. Floor ≈ R × token price, within a band of [release cost, capture fee + SOL fee].

**Scenarios.**
1. **Stale listings.** NFT listings on marketplaces are priced in SOL. When the token pumps on the curve or AMM, a listing below R × price is free money. Bots sweep it, `release`, and sell R tokens in the same bundle (release is instant). Victims are NFT listers.
2. **Stale collection offers.** When the token dumps, standing SOL collection bids exceed R × price. Bots buy R tokens, capture, and hit the bid. Capture latency adds a few seconds of price risk. Victims are bidders.
3. **Pool adverse selection.** Holders release commons and keep rares, and re-rollers hand in commons. So the pool's rare share falls below the published census, and a capturer's real odds are lower than the census implies. That's a disclosure problem, not theft.
4. **Asymmetric exits.** In a crash, releases are instant and fully backed (no bank run, invariant holds). Captures during a pump lag. That's fine, but it should be disclosed.

**Cost to attacker.** Cost is the curve/AMM fee plus the network fee. There's no protocol fee on release. Profit is the listing's staleness (estimate; no Solana-404 dataset was found or read in this pass).

**Fix.**
- The UI and our marketplace metadata show a live "**unwrap value: R × price**" next to every listing and offer, and warn when a listing is below it.
- Show the **live pool census** and live odds, not just the launch census (the doc already suggests publishing it; make it mandatory and on-chain-derivable).
- Document release/capture asymmetry in convert copy.
- Optional: a small release token fee (bps) creates a band against micro-arbitrage. That trades against "exact unwrap" copy, so it's Barton's call.

### R1-B-08: MEV and sandwiching around converts. **Low** (documented) / **Medium** (conditional)

**As documented.** The convert leg has no price: exactly R tokens ⇄ one NFT, with fees locked per request (T-HV-11). A searcher can't sandwich a capture or release by itself. Settle is permissionless, but its outcome is fixed by VRF + FIFO, so a settler has nothing to extract. ORAO fulfilment makes `r` public before settle, but the FIFO pool is fixed at request time (T-HV-03). The residual risk is the Switchboard case in R1-B-02.

**Conditional (Medium).** If the app offers one-click "Buy NFT with SOL" (swap SOL→token on the curve/AMM + `request_capture`) or "Sell NFT for SOL" (`release` + swap), the swap leg is sandwichable. Sandwiching continued after the Jito mempool shutdown: 521,903 sandwiches and ≥ $7.7M of victim losses in Feb to Jun 2025 (IMC 2025, research-sources.md §3). A queued fee change could also race a user if requests didn't lock fees (they do).

**Cost to attacker.** Jito tip (median > 2,000,000 lamports for sandwich bundles, same study) plus pool fees. Profit scales with victim size × slippage tolerance (estimate).

**Fix.**
- Every composite flow carries `min_out` / `max_in` (R-11.1), and `request_capture` takes `expected_ratio`, `expected_fee_bps`, and `expected_sol_fee` (R-11.2).
- The frontend defaults to tight slippage and adds a `jitodontfront` account to user swaps ("rejected by the block engine unless that transaction appears first", Jito docs).

### R1-B-09: Graduation and liquidity migration. **High**. [CONDITIONAL: not designed; ARCHITECTURE says curve/AMM "TBD"]

**Precedent.** pump.fun, 2024-05-16: "a former employee, having illegitimately taken access of the withdraw authority … used flash loans" to push curves to 100% and trigger the privileged migrate path. About 12,300 SOL was taken, and the service account "acted as a cosigner for all the attacker's transactions" (The Block / Quadriga, research-sources.md §2).

**Attack scenarios (each applies if the design does X).**
1. **Privileged migrate key.** If migration requires an operator signer or withdraws curve reserves to an operator wallet before seeding the AMM, a compromised or insider key takes the reserves. The pump.fun pattern costs the insider about 0 (estimate).
2. **Pool-creation front-running.** If migration targets a permissionless AMM and the program creates the pool in a later, separate transaction, a squatter can pre-create the pool for (mint, WSOL) at a skewed price. Migration then either fails (liquidity stuck) or deposits at the squatter's price, and the squatter arbitrages. This depends on the AMM's pool-address derivation and hasn't been verified for Raydium CPMM or Meteora DAMM v2 in this pass.
3. **Threshold manipulation / price gap.** If the AMM's opening price ≠ the curve's final price, or graduation is triggered by a buy that the same bundle can follow, a bot bundles [buy to threshold → migrate → trade against the gap]. If the curve can be pushed to 100% with flash liquidity and migration is permissionless *and correct*, this is harmless. It's only dangerous combined with (1) or a price gap.
4. **LP ownership.** If LP tokens go to the platform or creator (not burned or locked in a PDA), post-graduation liquidity can be pulled. That's a classic rug.
5. **Migration fee skims.** An operator-set "migration fee" paid to a wallet is a Stonk.fun-style side payment (lesson 4).
6. **Hybrid interplay.** The vault's backing tokens must never be counted as curve reserves, or be withdrawable at migration. `hybrid_vault` holds them. Keep it that way and test it.

**Cost to attacker.** (1) ≈ 0 for an insider (pump.fun precedent, sourced). (2)/(3) cost pool-creation rent + tips + capital (estimate).

**Fix.**
- Graduation and migration happen **in the same program instruction** that completes the curve. It's permissionless, with no operator signer, and the destination pool is derived.
- The AMM opening price is set exactly to the curve's final price from the program's reserves. Fail closed if a pool already exists with another price, and have a documented recovery that seeds or corrects it.
- LP tokens are burned or locked in a PDA with no withdraw instruction.
- Any migration fee is a fixed on-chain constant paid to a disclosed PDA.
- The program has no instruction that moves curve reserves anywhere but the AMM.
- Fuzz: flash-buy to 100% and migrate in one bundle yields an AMM price equal to the final curve price.

### R1-B-10: Operator and authority powers that recreate Stonk.fun's trust problems. **High** (documented) / **Critical** (if any single key)

**What Stonk.fun actually did wrong (stonkfun-lessons.md, quoting Bitquery):**
- "Not a hack or exploit bot. The viral 'STONK FEE DRAIN' wallet is Stonk.fun's own reward wallet … One wallet sweeps it, sells it into each coin's own pool, and pays holders in the pair asset."
- "Trust failures: >=$1.41M of reward money went in single transfers to Stonk.fun-linked wallets (creator, treasury, a 1,000 SOL transfer), omitted from its public ledger." Bitquery: "StonkFun's reward ledger leaves them out, and we found no stated reason for them."
- "On 2,178 LaunchLab coins the reward wallet can still raise the tax to 100% after a delay of a few days (never used)."
- "Claimed buybacks ($2.09M) were overstated (chain shows $1.23M)."

The lessons Mintmark adopted: "No discretionary platform wallet that custodies or routes user value" (#2). "No retained power to change economics after launch. If anything must be adjustable, a multisig plus a long, visible timelock, with hard caps set in code. Prefer immutable." (#3). "Every fee destination is on-chain, public, and matches what the site says. No side payments outside the documented flows." (#4). "Publish a verifiable ledger" (#6).

**Where the documented design still falls short.**
1. **One upgrade key over every vault** [DOCUMENTED]. There's a single `hybrid_vault` program for all collections, with upgrade authority "Squads multisig, then `--final` after audit + stabilization" (ARCHITECTURE authority map). Until it's final, whoever controls that multisig can ship an upgrade that makes the vault PDA sign transfers of **every collection's backing tokens and pooled NFTs**. PDAs sign for whatever code is deployed (B-03). That's the pump.fun single-key pattern at platform scale, and it's a retained power stronger than Stonk.fun's tax switch. Signers and threshold are still open (DECISIONS Q8). No upgrade timelock is specified.
2. **Fee changes on a 72h timelock** [DOCUMENTED]. Bounded (≤ 10% of R, ≤ 0.05 SOL) and timelocked "≥ 72h". That's about as long as Stonk.fun's "few days" tax-raise delay, which Barton's lessons treat as the problem. Lesson #3 says "long, visible timelock … Prefer immutable".
3. **Destination changes** [DOCUMENTED, internally inconsistent]. §4 says destinations are "fixed at init, never an instruction argument", but the abuse table says "Changing one goes through the same timelock with a multisig". Under burn (BRIEF), there should be **no** destination at all for re-roll fees.
4. **Pause** [CONDITIONAL]. ARCHITECTURE says admin powers include "pausing". If `hybrid_vault` gets a pause that covers `release`, an operator can freeze every holder's backing. That's the freeze-authority rug by another name.
5. **Creator allocations** [CONDITIONAL, Q-H6 open]. If creators may take NFTs outside the VRF pool, or pre-mint via `reveal_batch` to themselves, they get rares with certainty.
6. **Undocumented platform flows** [CONDITIONAL]. Curve trading fees, graduation/migration fees, and "platform" shares of capture fees (Q-H3 "creator/platform split") aren't specified. Any of them paid to an operator wallet, or routed off-chain, is exactly lesson #4's failure. A "buyback" funded by platform revenue at operator discretion is Stonk.fun's overstated-buyback problem.
7. **If MPL-Hybrid is used** [CONDITIONAL]. `update_recipe_v1` changes amount, fees, and path with no timelock and "unconditionally overwrites `recipe.token` and `recipe.fee_location`" (hybrid-rarity §3.1). A compromised authority captures every NFT against a worthless mint, then points back and releases against real backing (T-HY-01). MPL-Hybrid itself is upgradeable by a Metaplex key (`mp14o4AQ…`) and "Audit Pending".
8. **Mint, freeze, and metadata authorities** [DOCUMENTED, good]. Mint and freeze are revoked and checked by `assert_launch_ready`. The Core collection update authority is a program PDA with no update or add-plugin instruction, and Permanent plugins are impossible because the program creates the collection. Keep this, and have `assert_launch_ready` verify it (R-03.1).

**Scenario (1).**
1. A signer set is phished, or insiders collude (threshold unknown).
2. They queue an upgrade adding `sweep(vault, dest)`, or upgrade immediately if there's no timelock.
3. They drain the backing of every hybrid collection. NFTs become unredeemable, and the attacker dumps the tokens into every pool.

**Cost to attacker.** About 0 for insiders. For outsiders, the cost of compromising threshold signers (estimate). Precedent: pump.fun, ~12,300 SOL (sourced).

**Fix.**
- **Upgrade authority behind a Squads timelock of ≥ 7 days**, published signers, threshold ≥ 3-of-5 with independent parties. Go `--final` after audit. `assert_launch_ready` refuses launch otherwise, and an on-chain monitor alerts on any buffer write or queued upgrade.
- **No pause instruction can block `release`** (or none at all). Pause may only block new captures and re-rolls, and it auto-expires.
- Fees: hard-coded caps, **only-decrease-to-a-floor or immutable per collection**, and a timelock ≥ 7 days with the pending value shown on the token page.
- Re-roll fee burned by a user-signed SPL `Burn` from the user's own ATA, so there's no fee account at all. Capture fees (if any) go to a destination fixed at init, shown on the page, with no change path.
- Q-H6: forbid out-of-pool allocations on-chain. `reveal_batch` mints into the vault only.
- Every platform fee (curve, migration, capture) is a program constant paid to a published PDA. Publish a reconciliation script (lesson #6), and no discretionary buybacks.

### R1-B-11: Commitment covers metadata JSON, not image bytes. **Medium**. [DOCUMENTED]

**Design as written.** `leaf_j = sha256("mintmark-trait-v1" ‖ j ‖ sha256(metadata_json_j))`, "where `metadata_json_j` includes the image URI (Arweave or another content-addressed store)". It's a recommendation, not an enforced check. The verify script checks URIs and JSON hashes, not image content.

**Scenario.**
1. The creator commits JSON whose `image` is `https://creator-cdn.example/42.png`.
2. After reveal and trading, the creator swaps the file behind the URL. The commitment still verifies.
3. The creator can make "their" NFTs look rare, deface a competitor's holdings, or rug the art. That undoes the design's promise "No one can 'upgrade' a rare later".

Pre-reveal leaks aren't the issue here. Under `hybrid_vault` the full list is public by design and selection is VRF, so peeking gives no edge.

**Cost to attacker.** Hosting (about 0) for the creator (estimate).

**Fix.**
- The launch gate rejects any `image`/`animation_url`/`uri` that isn't content-addressed (`ar://`, `ipfs://<CID>`), and the JSON includes `image_sha256`.
- The verify script downloads and hashes every image. Also hash the JSON **as bound on-chain** (the Core asset URI must point to an immutable copy of the same JSON).

### R1-B-12: Burn vs no-burn contradiction and its side effects. **Low**. [DOCUMENTED]

**Facts.** The BRIEF (2:51 PM) says the fee is paid in the token and burned, with supply copy "No one can mint more; re-roll burns can only reduce it". hybrid-rarity §4 says "Burning isn't recommended". DECISIONS Q-H3 says "Confirm no burn".

**Side effects of burn** (the current decision):
1. At max size (N×R = 1B), after the first burn not every NFT can be outside the vault at once. The last NFTs become permanently uncapturable, and the wizard copy "Up to {N×R} … can be held as NFTs" becomes false.
2. The burn must come from the payer's ATA via a user-signed `Burn`, and **never** from the vault ATA. QA must test that `vault_tokens ≥ R × nfts_outside` holds across burns (BRIEF already asks for this).
3. `capture_fee` destination is still unspecified.

**Cost to attacker.** N/A (correctness and disclosure).

**Fix.**
- Update hybrid-rarity §4, Q-H3, and the wizard copy to "Up to min(N×R, current supply)…".
- Decide whether the capture fee is also burned (simplest, since it leaves no custody).
- Add an invariant test: burns never touch the vault.

### R1-B-13: Creator can freeze captures and re-rolls by withholding the trait manifest. **Medium**. [DOCUMENTED]

**Design as written.**
- At first exit, "the caller supplies `metadata_json` hash + URI + Merkle proof … A bad proof fails the settle, and anyone can supply the correct one because the list is public".
- "A *fulfilled* request can only be settled", and `expire` applies only to unfulfilled requests.
- Settle is strictly FIFO.

**Scenario.**
1. The creator (or a failing host) makes the manifest unavailable. For example, it was uploaded to a mutable server, or the creator never uploads some entries.
2. The first settle that needs an unminted index whose leaf data nobody has can't produce a proof.
3. The head request is fulfilled, so it can't expire. The FIFO is **stuck forever**: no capture or re-roll can settle, and those users' tokens or NFTs stay locked in pending requests.

Release still works, so existing NFT holders can exit, but new holders can't enter. A creator could use this to trap requesters, or extort them.

**Cost to attacker.** About 0 for the creator (estimate).

**Fix.**
- Before capture opens, `hybrid_vault` must require either (a) all N leaves to be pre-bound on-chain (`reveal_batch` into the vault, with rent paid by the creator: ~1.6 SOL per 1,000 NFTs, Q-H5), or (b) the manifest on Arweave with its transaction ID stored in config, and our launch gate fetching and verifying every leaf.
- Add a liveness escape: if a fulfilled head can't settle for X slots, a permissionless `refund_stuck` returns principal but **not** fees. It never lets the requester choose, because the outcome is already fixed and unavailable to everyone.

### R1-B-14: Carry-overs and hygiene. **Info**. [DOCUMENTED]

- **B-12 → R1-B-05/07.** Honest-grinding economics re-run with the documented fee model.
- **B-16** (`[min, max)` off-by-one) applies only if MPL-Hybrid math is reused. `uniform_below(r, pool_len)` with rejection sampling is correct. Keep the reachability test.
- **B-15.** Pin the VRF program IDs and parse with the provider SDK. DECISIONS Q1 notes both SDKs currently fail to build with Anchor 1.2. A hand-rolled parser is a new bug surface and must be in audit scope.
- **B-10 → R1-B-03. B-11 → R1-B-08. B-01 → R1-B-01/02. B-03/B-05 → R1-B-10**, with the Token-2022 parts deferred.
- Queue compute: settle mints a Core asset and checks a Merkle proof (depth ~17 at 100k). Benchmark CU at max N before audit.

---

## Deferred / out of scope in round 1 (Token-2022 track shelved; kept on file, not deleted)

| Earlier finding | Status in round 1 | Why |
|---|---|---|
| B-02 lottery randomness | **Deferred** | No lottery in the SPL-404 scope. Its VRF-binding lessons are reused in R1-B-02 |
| B-04 Token-2022 fee authorities | **Deferred** | No Token-2022 mint. Its "no retained power" lesson is reused in R1-B-10 |
| B-06 Token-2022 fee bypass / escrow insolvency | **Deferred** | Classic SPL, no fee leak. The solvency invariant is kept (T-HV-08) |
| B-07 fee-funded NFT buys | **Deferred** | No tax-funded buys |
| B-08 flash snapshot, B-09 sybil tickets, B-13 holder-iteration DoS, B-14 draw liveness | **Deferred** | No holder lottery. `sim/lottery_sybil.py` and its output are kept unchanged for when the Rewards track returns |
| B-03 / B-05 (Token-2022 parts: extension allowlist, MintCloseAuthority, `transfer_checked`) | **Deferred** | The classic SPL parts (mint and freeze `None`, upgrade authority, pinned programs, derived destinations) remain in force via R1-B-10 |
| CR-00 ("SPL 404" repo name vs Token-2022 design) | **Resolved by the scope change** | The repo description now matches the scope |

## Not safe to launch until

1. **Swap engine decided and documented as `hybrid_vault`** (or an equivalent two-step VRF design). Stock MPL-Hybrid isn't in the swap path, and the BRIEF wording is corrected (R1-B-01).
2. **`expire` never refunds fees.** Switchboard freshness and not-revealed checks are enforced at request, third-party reveal is proven on devnet (or ORAO is chosen), and a redundant crank is live. A test shows that selective expiry can't change any later settle (R1-B-02).
3. **On-chain launch-window anti-sniping** on the curve: stored `open_slot`, plus per-wallet/per-slot caps or batch clearing. The dev buy is capped, disclosed, and vested (R1-B-03).
4. **Re-rolls locked, or traits unrevealed, until graduation or a fixed post-launch delay** (R1-B-04).
5. **Re-roll fee has an immutable minimum.** Live pool census and live odds are shown, and `reroll_ev_r1.py` is re-run with each launch's real census (R1-B-05).
6. **Minimum collection size / convertible share.** `expire` batch-clears stale heads (R1-B-06, R1-B-13).
7. **Graduation and migration are permissionless, atomic, price-continuous, and LP-burned,** with no operator signer or wallet in the path (R1-B-09).
8. **Upgrade authority is a published ≥ 3-of-5 Squads with a ≥ 7-day timelock** (then `--final`). No pause can block `release`. Fees are capped and only-decrease or immutable. There are no destination-change paths, no out-of-pool creator allocations, and every platform fee is an on-chain constant to a published PDA, with a public reconciliation script (R1-B-10).
9. **Images and metadata are content-addressed and hash-committed,** and the full manifest is pre-bound or verifiably stored before capture opens (R1-B-11, R1-B-13).
10. **Burn behaviour is reconciled across docs and copy,** with a test that burns never touch the vault (R1-B-12).
11. **Code exists and is re-reviewed** against this list (`code-review-<sha>.md`). Every T-HV test in docs/THREAT_MODEL.md passes, and fuzzing of the vault invariants is done.

## Professional audit requirement

This is a design-level review of documents. No code exists yet, so it is **not a security audit** and doesn't replace one. **A professional third-party audit of `hybrid_vault`, the bonding-curve and migration programs, the VRF integration, the authority and deployment configuration, and the economic parameters is required before any mainnet launch.** Any post-audit change needs a re-review. A bug bounty and a staged launch (small collections and caps first) are strongly recommended. No deployment was made, mainnet wasn't touched, and no real keys or funds were used in preparing this document.

## Files (round 1, Auditor B)

- `security/auditor-b/round1.md` (this file)
- `security/auditor-b/sim/reroll_ev_r1.py` (new; imports the confirmed MPL-Hybrid pick model from `reroll_ev.py`)
- `security/auditor-b/sim/reroll_ev_r1_output.txt` (new; output of the run above)
- Unchanged and reused: `threat-model.md`, `research-sources.md`, `design-requirements.md`, `code-review-no-commits.md`, `sim/reroll_ev.py`, `sim/reroll_ev_output.txt`, and `sim/lottery_sybil.py` + output (deferred)

## New sources read in round 1

- Bitquery Research, "Is StonkFun dumping on holders? We followed the fee wallet's $56 million" (figures verified 2026-09-22): https://bitquery.io/investigations/is-stonkfun-dumping-on-holders
- Switchboard, Randomness (Solana): https://docs.switchboard.xyz/docs-by-chain/solana-svm/randomness
- Switchboard, Randomness Tutorial (Solana): https://docs.switchboard.xyz/docs-by-chain/solana-svm/randomness/randomness-tutorial.md
- GitHub repo page (empty check): https://github.com/BartonBase/Launchpad1
- Everything else is cited from `research-sources.md` (MPL-Hybrid @ `aacf1a53`, Jito docs, MELT, IMC 2025, pump.fun coverage, Metaplex docs).
