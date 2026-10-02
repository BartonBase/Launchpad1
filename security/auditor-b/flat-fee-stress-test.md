# Flat SOL fee stress test (BRIEF "FEE CHANGE v2" 4:49 PM + "FEE REFINEMENT" 4:52 PM MT, 2026-09-25)

**Author:** Auditor B worker. **Date:** 2026-09-25, written from 16:50 MT onward. **Scope:** SPL-404 hybrid launches only.

**Status:** economic and design review, plus a read-only look at the engineer's uncommitted working tree (WT), branch
`wip/hybrid-vault` (HEAD `977f8f2`). The WT changed while I was reviewing it. At ~16:50 it still had the superseded 2% token fee. By
16:54 MT it had the v3 flat, tiered SOL fee. **Everything below refers to the 16:54 state.** No deployment, no devnet or mainnet
writes, no real keys or funds.

**Model:** `sim/flat_fee.py` → `sim/flat_fee_output.txt`.
- Parts A–F test a flat 0.01 SOL fee on every action (the 4:49 PM proposal).
- **Part G tests Barton's 4:52 PM tiered schedule.**
- Market inputs are **examples** and are labelled as such in the script. **No Solana-404 premium dataset was read. Rarity premiums
  are assumptions.**

## 0. What the BRIEF says now

**FEE CHANGE v2** (`docs/BRIEF.md:79-86`, 4:49 PM):
- **No token fee.** A flat SOL fee goes to **Barton's fee wallet** on every capture, release and re-roll, "proposed 0.01 SOL each".
- "Re-roll (0.01) < release + capture (0.02), closing the M-06 bypass. Flat per-attempt fee is the anti-grind floor (M-19)."
- Release returns exactly N tokens.
- The fee is fixed per launch in LaunchConfig, with a hard cap in code (M-08). "Can't be raised" may be used only with the beta
  caveat.
- The fee wallet is a multisig and never converts or re-rolls (M-16). The fee account is checked against LaunchConfig (F-06).
- "Charging a SOL fee on release must never be able to block release (M-26)."
- Open questions: VRF cost (paid from the fee or separately?) and any Metaplex protocol fee.

**FEE REFINEMENT** (`docs/BRIEF.md:89`, **4:52 PM**, the latest):
- The fee is "**TIERED BY RATIO**, fixed per collection at launch in LaunchConfig, immutable, **hard cap 0.01 SOL in code**".
- "Tiers: 10k/50k = **0.002** SOL; 100k/200k = **0.005** SOL; 500k/1M/2.5M/5M = **0.01** SOL."
- "Not tied to live market cap (rejected: gameable, needs an oracle)."
- "Engineering to confirm 0.002 SOL covers per-request VRF cost and that re-roll ≤ release + capture in every tier. Dropping 10k
  still undecided."

| Question I was asked to check | Answer (BRIEF + WT at 16:54) |
|---|---|
| Cap value | **0.01 SOL, hard-coded.** `MAX_FEE_LAMPORTS = 10_000_000`, with a compile-time assert that it is exactly 0.01 SOL (`programs/hybrid_launch/src/needs_barton.rs`). The floor is `MIN_FEE_LAMPORTS = 2_000_000` (≥ the 890,880 rent minimum). `hybrid_vault/src/config.rs` re-asserts both bounds **and** `fee == fee_for_ratio(ratio)` on every use (`FeeAboveHardCap`) |
| Can the creator set the fee? | **No.** `LaunchParams` has "no fee field of any kind". The fee is `fee_for_ratio(ratio)` from the constant `FEE_TIERS` (`validation.rs:9-14, 28-40`). The creator only picks the ratio, which picks the tier |
| Fee wallet: per collection or the platform's? | **The platform's**: one `PLATFORM_FEE_RECIPIENT` program constant for every launch, with a mainnet `compile_error!` until it's set. Capture and re-roll fees go **directly** to it (`request.rs:52, 105, 227, 274`). Release fees go to a **per-vault `FeeVault` PDA** (program-owned, never closed) and are swept permissionlessly to the recipient only (`unwrap.rs`, `sweep_fees.rs`) |
| Same fee on all three actions? | **Yes.** `econ()` sets capture = release = re-roll = the tier fee (`config.rs`) |
| Allowed ratios | 10k, 50k, 100k, 200k, 500k, 1M, 2.5M, 5M tokens per NFT. Dropping 10k is "still undecided" |
| Collection sizes | 100 ≤ C ≤ min(1B / ratio, **10,000**). The WT enforces `MAX_COLLECTION_SIZE = 10_000` in `hybrid_launch` (`validation.rs:39`), still marked "NEEDS BARTON" |

**Other WT facts used below:**
- The fee is paid at request and never refunded. `expire` returns principal only, after 1 + `MAX_RECOMMITS` (3) unrevealed commits
  plus ~1 day (`expire.rs:1-6, 78-79`). **So the old expire loophole (M-04) is closed in the WT.**
- The pool-floor check runs before any fee is taken (`request.rs`, `check_can_request`).
- The guardian pause blocks only `request_capture`/`request_reroll`, never unwrap, settle, reveal or recommit (`admin.rs:1-3`,
  `unwrap.rs` "Deliberately NO pause check"). The guardian is a creator-supplied key (`init_vault.rs:18-19, 130`).
- `Request` and `RandLock` rent is returned to the user at settle (`settle.rs:33, 42`).

**The old "ratio" axis, restated.** The earlier figures (1/1 break-even 126,390× / 1,444× / 90× / 32× / 4.5× at the "10k / 100k /
500k / 1M / 5M ratio") varied **R = tokens per NFT**. For each R they set the collection size to its **maximum, C = 1B / R**, at FDV
410. Each row therefore changed the ratio, the collection size (1/1 odds = 1/C) **and** the floor (F = R × 0.41e-6 SOL) together.
"126,390× at a 10k ratio" means R = 10,000, C = 100,000 and F = 0.0041 SOL. The multiple comes mostly from the 1-in-100,000 odds and
the tiny floor. Under a 10,000 cap that row can't exist. Sim Part A reproduces those numbers (for example 1,443.9× at R = 100k).
Below, collection size and floor vary independently.

## Summary

**Recommendation (§5):**
- Keep Barton's ratio tiers for **capture = re-roll**: 0.005 SOL for 100k/200k, 0.01 SOL for 500k–5M.
- Raise the 50k tier from 0.002 to **0.003**, or have the requester pay the oracle. 0.002 doesn't cover ~0.002 of VRF plus the
  crank's txs.
- Set **release to 0**.
- **Drop the 10k ratio.**
- **Keep the 0.01 SOL hard cap.**
- **Cap collections at 10,000.**

**Findings:**

| ID | Item | Severity | Status (WT 16:54 MT) |
|---|---|---|---|
| FF-01 | Release fee is an exit tax. Dust (underwater only after an −87% or worse drawdown) for R ≥ 50k under the tiers; 52.5% of V at graduation for 10k | Low; **Medium at 10k** (High under a flat 0.01) | Design. Fix: release 0 / drop 10k |
| FF-02 | Release fee could block release | Low | **Closed in WT** (FeeVault PDA + permissionless sweep) |
| FF-03 | Insider free re-rolls + exclusive band arbitrage: the fee wallet's true cost is 21% of the public's in the 0.01 tier | **High (insider)** | Open (policy; sink split optional) |
| FF-04 | Mid tiers (F ≥ ~0.25) and every tier at 2.5M/5M are market-priced lotteries | Medium (disclosure) | Open (UI) |
| FF-05 | Pool depletion: odds from a thin pool make farming up to 50× cheaper than the census suggests | Low | Open (UI) |
| FF-06 | Creator-set fee / capture 0 (reopened M-06 at ~16:50) | High → **closed** | Closed at 16:54 (tier table, vault re-assert) |
| FF-07 | "Hard cap in code" is upgradeable (M-08) | High (disclosure) | Open (copy, freeze, `min(stored, tier)`) |
| FF-08 | One platform fee key; Stonk.fun trust. No token pile any more | Medium (custody) / Low (sell pressure) | Open (Squads, ledger) |
| FF-09 | Fee kept on failed or expired draws | Low | Accept + disclose (loophole closed) |
| FF-10 | Sybil/DoS at 0.002–0.01 | Low | Pool floor is the real defence |
| FF-11 | Docs still describe token fee, burn, refund, 2%; ADR-013 missing | Medium (docs) | Open |
| FF-12 | Rent | Low (pass) | Tiers ≥ 2,000,000 lamports |
| FF-13 | 0.002 tier doesn't cover VRF + crank | Low | Open (engineering question in BRIEF:89) |
| **FF-14** | **Fee validation (`econ()`: tier equality, recipient, version) runs inside `unwrap`: any tier, recipient or version change bricks release in every collection** | **Medium (new, M-26 class)** | Open |

**Key numbers:**
- **Anti-grind:** a flat SOL fee fixes the low-price grind (every draw costs at least the fee at any token price). Its strength falls
  as 1/floor.
  - At graduation (FDV 400), 1/1 break-even: 1,051×–5,251× at 10k/50k, 1,276× at 100k, 320× at 200k, 102× at 500k, 26× at 1M,
    **5× at 2.5M, 2× at 5M**.
  - Epics: 54× at 10k down to **1.5× at 5M**. After a 5× run, epics are ≤ 3.6× at every ratio ≥ 50k.
- **Release trap:** underwater FDV = 212 SOL (10k), 42 (50k), 52 (100k), 26 (200k), 20 (500k), 10 (1M). **It's a dust floor, not a
  trap, except at 10k.** The fee wallet's maximum crash take = C × fee (e.g. 50 SOL at 100k/10k NFTs).
- **Small ratios:** capture as % of V at graduation = 52.5% (10k), 10.5% (50k), 12.8% (100k), 6.4% (200k), ≤ 5% (500k+).
  - **Drop 10k: agree.**
  - **Drop 50k: not needed under the tiers** (it was needed under a flat 0.01, when capture was 51% of V).
  - **Cap C at 10,000: agree** (neutral to positive for security). Don't cap lower: 1/1 protection scales with C.
- **Old "ratio" axis:** it meant R tokens per NFT with C = 1B/R (max size) and F = R × 0.41e-6. For example, "126,390× at 10k" =
  R 10k, C 100k, F 0.0041 SOL.

**Files:** this document; `sim/flat_fee.py`; `sim/flat_fee_output.txt`.

---

## 1. Is the flat SOL fee enough of an anti-grind floor for rare traits?

### 1.1 Model
- A farmer holds a floor NFT and re-rolls until they hit the target tier. Each draw costs `c` and hits with probability `p`, so the
  expected cost per hit is `c / p`.
- **Break-even premium** (the rare's price ÷ floor): **`m* = 1 + c / (p × F)`**. Farming is +EV only if the rare sells above m* × F.
- Outsider `c = fee + tx` (tx ≈ 0.0001 SOL per attempt), if Barton's crank pays VRF out of the fee. In the "VRF sep" case the
  requester pays VRF separately: + ~0.002.
- **Fair VRF, loophole closed** (WT): the fee is paid at request and never refunded, and a revealed draw can't be discarded.
- **Loophole open** (committed `977f8f2` and the old docs, N7): the fee was refunded on expire, and a requester could withhold a bad
  reveal, so a bad draw cost only tx + VRF ≈ 0.0021.
- Tiers (examples):
  - **1/1** = 1/C;
  - **legendary** = 0.1% (1 / 5 / 10 NFTs at C = 1k / 5k / 10k; at 1k it's the same as a 1/1);
  - **epic** = 1%.
- Example premium bands for the verdicts (assumptions): 1/1 10×–1,000×, legendary 5×–50×, epic 2×–10×.
  - **Protected**: m* is above the band.
  - **Market-dependent**: m* is inside it.
  - **Lottery**: m* is below it.

### 1.2 Break-even m* (× floor) at a flat 0.01 SOL, loophole closed (sim Part B)

| Tier | C | F=0.01 | 0.05 | 0.1 | 0.25 | 0.5 | 1 | 2 | 5 SOL |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1/1 | 1,000 | 1,011 | 203 | 102 | 41 | 21 | 11 | **6.1** | **3.0** |
| 1/1 | 5,000 | 5,051 | 1,011 | 506 | 203 | 102 | 51 | 26 | 11 |
| 1/1 | 10,000 | 10,101 | 2,021 | 1,011 | 405 | 203 | 102 | 51 | 21 |
| legendary 0.1% | any | 1,011 | 203 | 102 | 41 | 21 | 11 | 6.1 | **3.0** |
| epic 1% | any | 102 | 21 | 11 | 5.0 | 3.0 | **2.0** | **1.5** | **1.2** |

- **Loophole open:** the same cells fall 4× to 5× for rare tiers. The 1/1 at C = 10k and F = 1 drops from 102× to 22×. Epic at
  F = 0.25 drops from 5.0× to 1.9× (a lottery). **Keep it closed.** The WT does; the docs still describe a refund (M-33).
- **Expected SOL per hit** (0.01, outsider): 1/1 = 10.1 SOL at C = 1k, 50.5 at 5k and 101 at 10k. Legendary = 10.1. Epic = 1.01.
- **Floor at which a tier turns +EV** at example premiums (1/1 = 100×, legendary = 20×, epic = 5×):
  - 1/1: 0.10 SOL (C = 1k), 0.51 (5k), 1.02 (10k);
  - legendary: 0.53 at any C;
  - epic: 0.25 at any C.
- Sensitivity (Part B3): m* − 1 scales linearly with the fee. At 0.005 every figure halves; at 0.02 it doubles.

### 1.3 What a flat SOL fee fixes, and what it doesn't
- **It fixes the low-price grind.** Under the 2% token fee, a draw cost 0.00002 SOL at R = 10k and FDV 100. A SOL fee doesn't depend
  on the token price, so every draw costs at least the fee however far the token falls. The cost per hit is `fee / p`, so it scales
  with rarity on its own (1/1 cost ∝ C). The flat-constant conclusion from D6 still holds.
- **It weakens as the floor rises.** m* − 1 = fee / (p F) falls as 1/F. A fixed fee can only protect tiers where
  `p × (m − 1) × F < fee`.
- Compared with 2% + 0.005: a flat 0.01 is stronger below F = 0.5 SOL and weaker above it. At R = 1M and FDV 410 the 1/1 m* goes
  from 32× to 26×; at R = 5M, from 4.5× to 2.0× (Part A).

### 1.4 Barton's tiered schedule (4:52 PM) at plausible floors (sim Part G)

C = min(1B / R, 10,000). "grad" = FDV 400 SOL (graduation-design, illustrative). "5×" = FDV 2,000.

| R | Fee | C | F at grad | 1/1 m* | legendary m* | epic m* | F at 5× | 1/1 | legendary | epic |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 10k | 0.002 | 10,000 | 0.004 | 5,251 | 526 | 54 | 0.02 | 1,051 | 106 | 11.5 |
| 50k | 0.002 | 10,000 | 0.02 | 1,051 | 106 | 11.5 | 0.1 | 211 | 22 | **3.1** |
| 100k | 0.005 | 10,000 | 0.04 | 1,276 | 129 | 13.8 | 0.2 | 256 | 27 | **3.6** |
| 200k | 0.005 | 5,000 | 0.08 | 320 | 65 | 7.4 | 0.4 | 65 | 14 | **2.3** |
| 500k | 0.01 | 2,000 | 0.2 | 102 | 52 | 6.1 | 1.0 | 21 | 11 | **2.0** |
| 1M | 0.01 | 1,000 | 0.4 | 26 | 26 | **3.5** | 2.0 | **6** | **6** | **1.5** |
| 2.5M | 0.01 | 400 | 1.0 | **5** | 11 | **2.0** | 5.0 | **2** | **3** | **1.2** |
| 5M | 0.01 | 200 | 2.0 | **2** | **6** | **1.5** | 10 | **1** | **2** | **1.1** |

Bold = the tier is a lottery or close to one (m* at or below the low-to-mid part of its example band).

Tiering by ratio roughly tracks the floor without an oracle, because F = R × price and a bigger R gets a bigger fee. That's sound, and
it's the right call over a live-market-cap fee (which is gameable and needs an oracle). But the tiers top out at the **0.01 cap**, so
the fee stops growing exactly where floors are largest.

### 1.5 Verdict: which tiers are protected
- **Protected at graduation** (m* above the example band):
  - **1/1s at R ≤ 200k** (320× to 5,251×);
  - **legendaries at R ≤ 100k** (≥ 106×);
  - **epics at R ≤ 100k** (≥ 11.5×).
  - This holds while floors stay under about 0.1 SOL.
- **Market-dependent at graduation:** 1/1s at 500k–1M (26×–102×), legendaries at 200k–2.5M (11×–65×), epics at 200k–500k
  (6×–7×).
- **Market-priced lottery at graduation:** at **2.5M and 5M everything is**. The 1/1 m* is 5× and 2×, the epic 2.0× and 1.5×.
  At 1M, epics are one too (3.5×).
- **After a 5× run (FDV 2,000):** epics are a lottery at **every ratio ≥ 50k** (m* ≤ 3.6×). 1/1s and legendaries are a lottery at
  1M and above (m* ≤ 6×).
- **Plainly:** the fee protects **big-collection 1/1s and every tier of low-floor collections**. **Mid tiers become a lottery as soon
  as floors pass ~0.25 SOL, and everything at 2.5M/5M is a lottery from day one.**
- This is **not an exploit**:
  - odds are public and equal for everyone;
  - grinding pays the fee;
  - sustained grinding adds supply of the tier until its premium falls to about m*.
- It must be **disclosed**. Show live pool odds and the expected SOL per hit (`fee / p`) on the re-roll button.
- **Raising the cap doesn't rescue it.** Even 0.05 at 5M gives the 1/1 only m* ≈ 6× at graduation (1 + 200 × 0.0501 / 2), and epics
  would need ≈ 0.08 SOL at F = 2 (Part B3). **Keep the 0.01 cap.** A cap that holders can trust is worth more than protection that
  a fee can't buy anyway.

### 1.6 Pool depletion makes farming cheaper than the census implies (sim Part B4). **FF-05, Low**
- Draws come from the **current pool**, not the full collection. If most NFTs are outside the vault and the 1/1 is still inside, the
  odds are 1 / pool size.
- At the pool floor `r = max(5, 2% × C)`:
  - the 1/1 at C = 10k and F = 0.25 falls from 405× to **9.1×** (at 0.01);
  - at C = 1k and F = 0.1, from 102× to **3.0×**.
- Each draw is still fair and the odds are public. **The protection figures above assume a full pool.**
- **Fix:** display live pool odds, not the launch census (as in M-24). **Attacker cost:** none (it's public information).

### 1.7 VRF cost: does the fee cover it? Does 0.002 cover it?
- **Official Switchboard docs publish no per-request price** (`docs.switchboard.xyz/docs-by-chain/solana-svm/randomness`, fetched
  2026-09-25).
- Third-party figures:
  - QuickNode's 2026 Switchboard On-Demand guide: "about **0.002 SOL per request**", excluding tx and priority fees.
  - Switchboard's own older VRF post: "just under 0.002 SOL".
  - ORAO: 0.001 SOL (co-founder, via Solana StackExchange 2024).
- **Unverified on devnet.** Measure the exact lamport delta in the M-04 devnet reveal test.
- **Covered:** the 0.01 tier has a margin of ~+0.0078 SOL per draw after ~0.002 VRF + ~0.0002 of crank txs; the 0.005 tier ~+0.0028.
- **Not covered, 0.002 tier (10k/50k): margin ≈ −0.0002 SOL per draw** if Barton's crank pays the oracle (Part G). Barton subsidises
  every draw.
  - A spammer costs Barton ~0.0002 per draw at a cost to itself of 0.0021, so it's a weak grief, not an exploit.
  - It still means **the 0.002 tier doesn't meet the engineering condition in BRIEF:89**.
- **Who pays VRF in the WT is unclear:** reveal takes `sb_reward_escrow` plus a payer (`randomness_ix.rs:35, 116`).
- **Fix:** either
  - (a) the requester funds the oracle reward inside `request_*` (then 0.002 is pure fee and the public pays ~0.0041 per draw), or
  - (b) raise the lowest tier to **0.003 SOL**.
- **Metaplex:** the 0.0015 SOL Core Create fee applies once per asset at the graduation mint (graduation-design §1). No create
  happens on capture, release or re-roll. Whether Core's ~0.00005 SOL "Execute" fee applies to our transfers is **unverified**, and
  it's negligible either way.

---

## 2. Can the release fee trap holders during a crash?

### 2.1 When the fee exceeds the NFT's token value (sim Parts C and G)

The exit path is NFT → `release` (fee) → N tokens → sell into the AMM (example 1% fee plus slippage) → SOL. A holder is
**underwater on release** when `V × 0.99 < fee + tx`, where V = N × token price.

| R | Fee (tier) | Token price where release is underwater | Same, as FDV | Drawdown from grad FDV 400 needed | Flat 0.01 for comparison: FDV |
|---:|---:|---:|---:|---:|---:|
| 10k | 0.002 | 2.1e-7 SOL | **212 SOL** | **−47%** | 1,020 (underwater at graduation) |
| 50k | 0.002 | 4.2e-8 | 42 | −89% | 204 |
| 100k | 0.005 | 5.2e-8 | 52 | −87% | 102 |
| 200k | 0.005 | 2.6e-8 | 26 | −94% | 51 |
| 500k | 0.01 | 2.0e-8 | 20 | −95% | 20 |
| 1M | 0.01 | 1.0e-8 | 10 | −97.5% | 10 |
| 2.5M | 0.01 | 4.1e-9 | 4 | −99% | 4 |
| 5M | 0.01 | 2.0e-9 | 2 | −99.5% | 2 |

*(Price column = underwater FDV / 1e9.)*

**Release fee as % of V at graduation (FDV 400):** 10k **52.5%**, 50k 10.5%, 100k 12.8%, 200k 6.4%, 500k 5%, 1M 2.5%, 2.5M 1%,
5M 0.5%. Under the flat 0.01 proposal the figures would have been 253% / 51% / 25% / 13% / 5% / 2.5% / 1% / 0.5%.

### 2.2 Real trap or dust floor? Verdict
- **Under the tiered schedule, it's a dust floor for every ratio except 10k.**
  - At 50k and above, an NFT is underwater on release only after a −87% or worse drawdown from graduation.
  - The maximum loss per floor NFT is `min(V, fee)` ≤ 0.002–0.01 SOL.
  - Holding is free, and the NFT can also be sold on Tensor or Magic Eden.
- **At 10k it's a real exit tax:** 52.5% of the NFT's value at graduation, and underwater after a −47% move, which is ordinary for a
  memecoin.
- **Under the flat-0.01 proposal it would have been a real trap at 10k–100k.** Tiering fixed most of it.
- **What remains in every tier:**
  - The fee is priced into the NFT. An arbitrageur bids at most `V − fee − tx` for an NFT they plan to release, so the bid-side floor
    sits one fee below the token value. When V < fee, the NFT **detaches from its backing**: "converts back to exactly N tokens" is
    true, but economically worthless.
  - **Holders with no SOL can't release** until they get at least the fee plus tx. This is 400–2,000× the network fee and affects
    gifted, airdropped and marketplace-bought NFTs.
  - The fee bites hardest in a crash, when holders most want out.

### 2.3 Can the fee destination profit from a crash?
- **Yes, from panic volume, but it's bounded.** Every release pays the fee wallet, and release volume spikes in a crash.
  - Maximum take if everyone releases = C × fee. For example: 10k × 0.005 = 50 SOL at R = 100k, 1k × 0.01 = 10 SOL at R = 1M.
  - At R = 100k and FDV 100, 50 SOL is 50% of the value exiting (100 SOL of tokens). Under a flat 0.01 it would have been 100%.
  - Rational holders stop releasing once they're underwater, so the real take is lower. The income is still driven by volatility and
    panic, and largest relative to holder value when holders are worst off.
- **Insider band arbitrage** (part of FF-03):
  - Outsiders won't buy and release an NFT unless it's listed more than fee + tx below V.
  - The fee wallet, or a wallet it funds, releases at a true cost of tx only, because the fee comes back to it.
  - So in a crash it can buy every NFT listed between V − fee and V and redeem it for up to one fee per NFT **that no outsider can
    earn**. The capture side works the same way (capture at V, sell at up to V + fee).
  - This is small per NFT (≤ 0.01) but exclusive.

### 2.4 Do a changeable fee wallet or a paused release make it worse?
- **Pause: no, in the WT.** The guardian pause can't block unwrap (`unwrap.rs`: "Deliberately NO pause check"). Keep it that way
  (M-26).
- **Fee-wallet state can't block release in the WT. That's good design.**
  - The release fee goes into a per-vault **`FeeVault` PDA**, which is program-owned, rent-exempt, non-executable and never closed. A
    system transfer can always credit it.
  - `sweep_fees` (permissionless) moves the surplus to the fixed recipient. If the recipient can't be credited, only the sweep fails.
  - "The only way a release fails on the fee is the user's own SOL balance" (`unwrap.rs` header). **That closes FF-02 in the WT.**
  - Keep this pattern if a release fee survives. Never pay the release fee straight to the recipient.
- **Capture and re-roll fees go straight to the recipient.** If it were made uncreditable (for example, an executable account),
  captures and re-rolls fail closed. That's acceptable: it's a request-side DoS, not an exit block. It's reachable only through a
  program upgrade, because the recipient is a constant.
- **Changeable wallet:** if Barton ever adopts M-17 option B (a `FeeConfig` owner change with a timelock), the new owner must be
  validated as system-owned, zero-data and non-executable (or the known Squads vault PDA). That keeps capture and re-roll live.
- **Changeable amount:** the fee is immutable per LaunchConfig and re-asserted in the vault. The only change path is a program
  upgrade (3-of-5 + 7 days until `--final`). Raising a release fee after launch would be the Stonk.fun "retained power to raise the
  tax" on the exit (stonkfun-lessons, lesson 3). See FF-07.

### 2.5 Recommended fixes
1. **Release fee = 0 (preferred).**
   - Release is the redemption guarantee, and a fee there **adds nothing to anti-grind**: grinding uses re-roll or
     capture-after-release, never release on its own.
   - Setting it to 0 removes the 10k exit tax, the insider exit arbitrage, crash income for the fee wallet and the no-SOL lock.
   - M-06 still holds, because the bypass costs 0 + capture ≥ re-roll. Every release already needs a paid capture first, so release
     spam is priced.
2. **If Barton keeps release revenue:** keep **release ≤ capture** (today they're equal), keep the **FeeVault** path, and **drop the
   10k ratio**. That leaves a dust floor (underwater only after an −87% or worse drawdown) plus disclosure.
3. **A fee capped at a % of the NFT's value:** not recommended. It needs an on-chain price. Spot AMM prices can be manipulated within
   a transaction, a TWAP adds an oracle and a failure mode (release must never fail), and a graduation snapshot goes stale in exactly
   the crash case. BRIEF:89 already rejected market-linked fees for the same reason.
4. **No pause on release:** already true. Keep the `release_succeeds_while_paused` test.

**FF-01. The release fee is an exit tax that detaches the NFT from its backing in a crash.**
- **Severity:** Low under the tiers for R ≥ 50k (dust); **Medium at R = 10k** (52.5% of V at graduation, underwater after −47%);
  it would have been High under a flat 0.01 at 10k/50k.
- **Scenario:** the token falls 50% to 90% after graduation. The release fee reaches or exceeds V, the NFT floor stops tracking its
  backing, and holders without SOL can't exit.
- **Attacker cost:** none. It's a design harm. The insider earns up to one fee per arbitraged NFT.
- **Fix:** release fee 0. Or drop 10k and keep release ≤ capture through the FeeVault.
- **Test:** `release_charges_zero_lamports` if the fee is 0. Otherwise `release_fee_le_capture_fee_every_tier` plus
  `release_succeeds_when_recipient_uncreditable` (exists in spirit; point the recipient at an executable account in LiteSVM).

**FF-02. The release fee is a new way to block release (M-26 class).**
- **Severity:** Low, **closed in the WT** by the FeeVault. It would be Medium if the fee were paid straight to a changeable recipient.
- **Scenario:** the recipient becomes uncreditable, so every release fails across every collection.
- **Fix:** already implemented (FeeVault + sweep). Keep it.
- **Test:** `release_succeeds_when_recipient_uncreditable`, and `sweep_fails_but_release_succeeds`.

---

## 3. Small ratios, and the CD/engineer proposal (drop 10k, maybe 50k; cap C at 10,000)

### 3.1 The fee as a % of the NFT's value (V = R × FDV / 1e9; sim Parts D and G)

| R | Max C (1B/R) → with 10k cap | Supply convertible under the cap | Tier fee | Capture % of V: FDV 100 | 400 (grad) | 1,000 | Round trip at FDV 400 (capture + release) | Round trip, release = 0 | Flat 0.01 capture % of V at 400 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 10k | 100,000 → 10,000 | **10%** | 0.002 | 210% | **52.5%** | 21% | **105%** | 55% | 253% |
| 50k | 20,000 → 10,000 | **50%** | 0.002 | 42% | 10.5% | 4.2% | 21% | 11% | 51% |
| 100k | 10,000 | 100% | 0.005 | 51% | 12.8% | 5.1% | 25.5% | 13% | 25% |
| 200k | 5,000 | 100% | 0.005 | 26% | 6.4% | 2.6% | 12.8% | 6.5% | 13% |
| 500k | 2,000 | 100% | 0.01 | 20% | 5.0% | 2.0% | 10.1% | 5.1% | 5% |
| 1M | 1,000 | 100% | 0.01 | 10% | 2.5% | 1.0% | 5.0% | 2.6% | 2.5% |
| 2.5M | 400 | 100% | 0.01 | 4% | 1.0% | 0.4% | 2.0% | 1.0% | 1% |
| 5M | 200 | 100% | 0.01 | 2% | 0.5% | 0.2% | 1.0% | 0.5% | 0.5% |

### 3.2 Arbitrage band, wash converts, rare farming
- **Band:** the outsider NFT price sits in `[V − fee − tx, V + fee + tx]`, two fees wide. At graduation that's 105% of V at 10k,
  21% at 50k, 25% at 100k, 13% at 200k and ≤ 10% at 500k and above.
  - At 10k the band is wider than the backing, so conversion doesn't pin the NFT to its tokens.
  - From 50k up the band is workable (≤ ~25%) and narrows as the token rises.
- **Wash converts:** an outsider pays 2 × fee per round trip (1 × fee with a free release), which is pointless at every tier. The
  insider pays 2 × tx, i.e. free (FF-03). Report volume net of wallets linked to the fee wallet.
- **Rare farming at small ratios:** these launches are the **best protected** (1/1 m* 1,051×–5,251× at graduation for 10k/50k,
  Part G). Farming is **not** a reason to drop small ratios. Usability is.
- **Mint cost:** the per-NFT Core mint cost is 0.00509 SOL (graduation-design §1, measured). At 10k that exceeds V (0.004 at FDV 400).
  At 50k it's 25% of V.

### 3.3 Verdict on the proposal (with the tiered fees)
- **Drop 10k: agree.** At a graduation FDV of 400:
  - an NFT is backed by 0.004 SOL, while capture costs 52.5% of it, the round trip 105% and the mint 127% of it;
  - release is underwater below FDV 212 (−47%);
  - under the 10k cap only 10% of supply can ever be in NFT form.
  - The 0.002 tier also doesn't cover VRF plus crank (§1.7). There's no configuration of 10k in which conversion is economic at
    launch-level prices.
- **Drop 50k: not necessary under the tiers.** This changes my view from the flat-0.01 analysis, where 50k was at 51% capture and
  101% round trip.
  - With 0.002: capture is 10.5% of V at graduation, the round trip 21% (11% with a free release), and release is underwater only
    after −89%.
  - Only 50% of supply can be NFTs under the 10k cap. That's a design choice to disclose ("up to 500M $TICKER can be held as NFTs"),
    not a flaw.
  - **Condition:** fix the lowest tier's VRF coverage (requester pays the oracle, or raise it to 0.003). At 0.003, capture = 15% of V
    at graduation and release is underwater after −84%. Still fine.
  - If Barton wants the smallest NFT to be worth at least ~0.05 SOL at graduation, drop 50k for product reasons, not security ones.
- **Cap C at 10,000: agree.**
  - The security effect is **neutral to positive**. It makes graduation mint funding and the crank (M-10/M-22) tractable and shrinks
    the pool-array rent and the batch-mint stall surface.
  - It only removes collections above 10k, which exist only at 10k/50k.
  - Farming protection scales with C (1/1 m* ∝ C), so **don't cap lower**. At C = 1k the 1/1 is protected only while F ≤ ~0.1 SOL.
  - The WT enforces it in `hybrid_launch` (`validation.rs:39`). Add the same assert in `hybrid_vault::init_vault`, since the vault
    reads the LaunchConfig and shouldn't trust it blindly.
- **Net:** ratios **{50k, 100k, 200k, 500k, 1M, 2.5M, 5M}**, C ≤ 10,000, the lowest tier at 0.003 (or the requester pays VRF),
  release fee 0.
  - Worst case at graduation: capture = 15% of V (50k at 0.003) or 12.8% (100k). Best case: 0.5% (5M).
  - At 2.5M/5M every rarity tier is a lottery from day one (§1.5). Disclose it.

---

## 4. Other issues

### FF-03. Insider free re-rolls and exclusive band arbitrage (fee wallet owner). **High (insider)**. Supersedes F-B4; merged M-16 has it at Medium
- **Scenario:** 100% of every fee now goes to Barton's fee wallet, so the old token half (burned or kept) no longer dilutes it.
  - Whoever controls that wallet, or any wallet it funds from off-chain (a CEX withdrawal is indistinguishable), re-rolls and
    captures for a true cost of tx + VRF ≈ 0.0021 SOL. The public pays fee + tx.
  - The same party is the only zero-friction arbitrageur on both edges of the NFT band (§2.3).
- **Numbers (Part E/G):**
  - The insider pays **21%** of the public's cost per draw in the 0.01 tier (4.8× cheaper) and **41%** in the 0.005 tier (2.4×).
    There is no discount in the 0.002 tier, but only because that fee barely covers VRF.
  - The 1/1 at C = 10k and F = 1 SOL: public m* 102×, **insider 22×**.
  - Epic at F = 0.25: public 5.0×, **insider 1.8×**.
  - So the insider can profitably farm every epic and most legendaries in every 0.005/0.01 collection, and in a crash can capture
    the exit spread nobody else can.
- **Attacker cost:** ~0.0021 SOL per draw, plus access to the fee wallet (Barton, a signer, or anyone who compromises it).
- **Why the published policy isn't enough:** "the fee wallet never converts or re-rolls" (BRIEF:83) can't be checked, because
  funding can come from off-chain.
- **Fixes:**
  - (a) **Structural:** route a fixed share of each draw fee to a sink nobody controls (incinerator
    `1nc1nerator11111111111111111111111111111111`). With 0.005 of 0.01 sunk, the insider pays 0.0071, 70% of the public's cost. This
    halves Barton's revenue. It's Barton's call, and he skipped M-16's sink before.
  - (b) If no sink: the fee wallet is a **Squads vault** with published signers, every outflow goes only to a published treasury,
    there's a written insider policy, and a disclosure line on the token page: "Mintmark earns every capture, release and re-roll fee.
    Wallets we control could re-roll at cost; our policy is that they never do."
  - (c) A public monitor that flags fee-wallet outflows to addresses that later convert. It's weak (off-chain funding bypasses it)
    but cheap.
  - (d) Release fee 0 removes the exit-side arbitrage half.
- **Owner:** Barton (policy, sink yes/no), engineer.
- **Tests:** `fee_split_sink_share_reaches_incinerator` (if adopted); otherwise a QA check that the disclosure string appears on the
  token page.

### FF-04. Mid tiers and every tier at 2.5M/5M are market-priced lotteries. **Medium (policy/disclosure)**. Answers M-19's value question
- **Scenario:** at graduation, epics at 1M+ (m* ≤ 3.5×) and everything at 2.5M/5M (1/1 5× and 2×) are +EV to grind at any plausible
  premium. After a 5× run, epics are a lottery at every ratio ≥ 50k.
- **Attacker cost:** fee / p per hit, the same for everyone (epic ≈ 1 SOL at 0.01; a 1/1 at C = 200 ≈ 2 SOL).
- **Fix:** disclosure, not a fee. The re-roll button shows live pool odds per tier and "expected cost to hit: X SOL".
  - Optionally drop 5M, or cap it at C ≤ 200 with a warning that its rares are effectively lottery-priced.
  - A higher cap doesn't fix it (§1.5).
- **Test:** a frontend check that the displayed expected cost = fee / (pool count / pool size) for each tier.

### FF-05. Pool depletion. **Low** (§1.6)

### FF-06. Who sets the fee. **Closed in the WT (16:54)**
- The creator can't set the fee. It's `fee_for_ratio(ratio)` from a constant table, stored immutably, and `hybrid_vault` re-checks
  `fee == fee_for_ratio(ratio)` and the bounds on every use.
- (At ~16:50 the WT still had creator-set SOL fees with capture allowed to be 0, which **reopened M-06**. The 16:54 WT fixed that.)
- **Tests to keep:** `launch_params_have_no_fee_field` (IDL), `vault_rejects_config_with_fee_not_equal_tier` (forge a LaunchConfig
  in LiteSVM with fee = 0.02 or fee = 0.005 at R = 1M; expect `FeeAboveHardCap`).

### FF-07. "Hard cap in code" is upgradeable. **High (disclosure), carries M-08 unchanged**
- **Scenario:** `MAX_FEE_LAMPORTS` and `FEE_TIERS` are constants, but the program is upgradeable by a 3-of-5 multisig with a 7-day
  delay until `--final`. One upgrade can raise every tier for **every existing collection**, because `econ()` re-derives the
  expected fee from the new constants. The per-launch `LaunchConfig.fee_lamports` is immutable, but the vault would then reject old
  configs, or a migration could rewrite them.
- **Attacker cost:** 3 signer keys, plus 7 days of public warning.
- **Fix:**
  - Copy: "The fee is X SOL and no setting can raise it. Until the program is frozen after the audit, a 3-of-5 multisig upgrade with
    a public 7-day delay could change the program."
  - Freeze (`--final`) after audit (Barton sets the date, N5).
  - Monitor buffer writes and queued upgrades.
  - Make `econ()` charge `min(stored fee_lamports, current tier, MAX_FEE_LAMPORTS)` instead of requiring equality. Then an upgrade
    can only **lower** what an existing collection pays, never raise it, and a table change can't brick old vaults (see FF-14).
- **Test:** `upgrade_cannot_raise_fee_for_existing_launch` (a unit test on `econ()` with a raised tier table: it charges the stored,
  lower fee).

### FF-08. Fee wallet as one platform key; trust and disclosure vs Stonk.fun. **Medium (custody) / Low (sell pressure)**. Updates M-17/M-18
- **Much better than the 2% token design.**
  - The fee wallet **no longer accumulates any collection's token**, so there's nothing to sell into the coin's own pool. The
    Stonk.fun harm ("sweeps it, sells it into each coin's own pool", 160 coins lost more than half their supply) is structurally gone.
  - **M-18 drops from Medium to Low:** only SOL revenue, which still needs disclosure.
- **Still true:**
  - One platform key receives SOL from every collection, directly (capture and re-roll) or through the FeeVault sweeps.
  - A stolen key steals **accumulated revenue only**, never holder principal. The escrow and the release path don't depend on it.
    Holder safety doesn't need a recovery path, so "never changeable" (M-17 option A, the WT's constant) is now the better choice.
  - Stonk.fun lessons 4 and 6 still apply. Publish the recipient address and the per-vault FeeVault address, cumulative fees
    (`vault.total_fee_lamports`, `fee_vault.total_accrued/total_swept` are already on-chain) and all outflows, with a reconciliation
    script. Lesson 2 (no discretionary wallet routing user value): fees are revenue, not user value, so that's acceptable if
    disclosed.
- **Fix:**
  - A Squads vault (≥ 2-of-3 hardware keys, signers published).
  - Assert off-curve at launch (`PLATFORM_FEE_RECIPIENT` is a PDA).
  - A ledger page.
- **Test:** `ledger_reconciles_fee_recipient` (the sum of on-chain fee transfers plus sweeps = the site figure).

### FF-09. Fees on failed and expired draws. **Low (accept + disclose)**
- **WT:**
  - A request that can't be served fails **before** the fee (pool floor).
  - After commit, the fee is kept whatever happens.
  - If the oracle never reveals, after 3 recommits plus ~1 day, `expire` returns principal only.
- An honest user loses one fee (≤ 0.01) per outage-stuck request. That's small and acceptable.
- **Don't reintroduce a refund.** Part B's "open" rows show it makes rare farming 4–5× cheaper.
- If Barton wants goodwill refunds on a proven Switchboard outage: the FeeVault pattern would allow a governance-gated refund, but
  only for fees paid **into** it. Today only release fees go there, so capture and re-roll fees can't be refunded. Not recommended.
- **Test:** `withheld_reveal_past_all_recommits_returns_principal_only` (as M-04).

### FF-10. Sybil and DoS griefing at 0.002–0.01. **Low**
- **Queue spam:** each request pays the fee (non-refundable) and locks R tokens or an NFT until settle. 1,000 junk requests cost
  2–10 SOL. Settle and reveal are permissionless and cranked, and stuck heads expire.
- **Crank drain at the 0.002 tier:** ~0.0002 SOL per draw lost to VRF if the crank pays (§1.7). A 10:1 cost ratio for the griefer.
  Fix it with requester-paid VRF or a 0.003 floor.
- **Cornering** (Part E, FDV 400): capturing 980 of 1,000 NFTs at R = 100k costs 39 SOL of tokens plus 4.9 SOL of fees (0.005 tier).
  At C = 200 and R = 5M: 390 SOL plus 1.95. The fee adds only 0.5%–13% to the token cost. The pool floor `r = max(5, 2% C)` is the real
  defence, and it is in the WT (`pool_floor`).
- **Incoming backlog:** more than 32 incoming entries → `MergeBacklog`, cleared by the permissionless `merge_incoming` crank. Each
  entry costs a paid capture first.
- Per-wallet limits remain pointless (sybil).

### FF-11. Doc and code drift. **Medium (reopens M-07/M-33 as docs)**
- The code is now v3. But `docs/DECISIONS.md` (ADR-008/009/010, N1/N7), `docs/hybrid-rarity-and-assignment.md` (§Summary row 4,
  §3.3, §4 "token fee … burned", "refunded on expire", "≤ 10% of R token fee, ≤ 0.05 SOL") and `docs/marketplaces-and-ratios.md`
  (fee assumption "2% of N in tokens", §B.5) still describe token fees, burns, refunds or the 2% design.
- The WT refers to an ADR-013 that isn't in DECISIONS.md yet.
- **Fix:** write ADR-013 and mark every token-fee, burn and refund passage SUPERSEDED.
- **Test:** `rg -n -i "2%|bps|burn|refund" docs design qa`. Every hit is historical or an LP burn.

### FF-12. Rent. **Low (pass)**
- Every tier is ≥ 2,000,000 lamports, above the 890,880 rent-exempt minimum (compile-time assert in `needs_barton.rs`). A first
  transfer to an empty recipient succeeds (M-30 closed).
- `sol_fee` returns early on 0 (`vault_token_ops.rs`), so a release fee of 0 is safe.
- If Barton ever chooses a dust release fee below 890,880 lamports, it must go to the FeeVault (already rent-exempt), never to the
  recipient.
- `Request`/`RandLock` rent goes back to the user. The Switchboard randomness-account rent owner is unverified.

### FF-13. VRF cost coverage. **Low**. Engineering's question in BRIEF:89
- The 0.002 tier doesn't cover ~0.002 of VRF plus crank txs. The 0.005 and 0.01 tiers do (§1.7).
- **Fix:** the requester funds the oracle reward in `request_*`, or the floor tier goes to 0.003.
- **Test:** a devnet measurement of the lamports the oracle charges per request, recorded in DECISIONS.

### FF-14. Fee validation sits on the release path: a config or tier mismatch bricks release. **Medium (new; M-26 class)**
- **Evidence:** `unwrap::handle_unwrap` calls `config::econ(&launch_config)?` before paying out. `econ()` fails unless all of these
  hold (`config.rs`):
  - `cfg.version == 3`;
  - `MIN_FEE ≤ fee ≤ MAX_FEE`;
  - **`fee == fee_for_ratio(ratio)`**;
  - `fee_recipient == PLATFORM_FEE_RECIPIENT`.
- **Scenario:** any future upgrade that changes a tier (even **lowering** it), changes `MIN`/`MAX_FEE_LAMPORTS`, rotates
  `PLATFORM_FEE_RECIPIENT` (M-17 option A's recovery path is exactly "an upgrade"), or bumps the LaunchConfig version makes `econ()`
  reject **every existing LaunchConfig**. That halts **release** (plus capture, re-roll and sweep) in every live collection until
  someone ships another upgrade.
  - These checks are right for requests. On release they turn a routine fee or recipient change into a holder lockout. BRIEF:84 says
    a SOL fee on release "must never be able to block release".
- **Attacker cost:** no attacker needed. One well-meant upgrade does it (3-of-5 + 7 days). A malicious upgrade could do it on
  purpose, but a malicious upgrade can do anything (M-09).
- **Fix:**
  - `unwrap` must use a **release-only reader** that checks only the mint and the ratio. It takes the release fee as
    `min(cfg.fee_lamports, MAX_FEE_LAMPORTS)` (or 0), with no tier-equality or recipient check, because the release fee goes to the
    per-vault FeeVault anyway.
  - `sweep_fees` alone checks the recipient.
  - Keep the strict `econ()` for `request_*`.
- **Owner:** engineer. **Tests:**
  - `release_survives_fee_tier_change`: build with a modified `FEE_TIERS` in a test feature and release against an old LaunchConfig.
    Expect success, exactly N back, and a fee ≤ the stored fee.
  - `release_survives_recipient_rotation`: same, with a different `PLATFORM_FEE_RECIPIENT`.

---

## 5. Recommended fee for Barton

**Recommendation: keep Barton's ratio tiers for capture and re-roll, set release to 0, raise the lowest tier to 0.003 (or make the
requester pay VRF), keep the 0.01 hard cap, and drop 10k.**

| Ratio | Capture | Re-roll | Release | Why |
|---|---:|---:|---:|---|
| 10k | **drop the ratio** | – | – | Capture = 52.5% of V at graduation, mint cost > V, only 10% of supply convertible, fee < VRF |
| 50k | **0.003** (0.002 if the requester pays VRF) | same | **0** | Capture 15% of V at graduation (10.5% at 0.002). Covers VRF + crank. Every tier is protected at graduation (1/1 ≈ 1,550×, legendary ≈ 156×, epic ≈ 16.5× at 0.003) |
| 100k / 200k | **0.005** | 0.005 | **0** | Capture 12.8% / 6.4% of V at graduation. Protected: 1/1 1,276× / 320×, epic 13.8× / 7.4× |
| 500k / 1M / 2.5M / 5M | **0.01** (= cap) | 0.01 | **0** | Capture ≤ 5% of V. The top tiers are market-priced lotteries at 1M+; disclose it (FF-04) |

- **Hard cap:** `MAX_FEE_LAMPORTS = 10_000_000` (0.01 SOL). Keep it exactly as the WT has it.
  - Raising it would buy little. At 5M even 0.05 gives the 1/1 only m* ≈ 6× at graduation.
  - A low, fixed cap is the strongest trust signal against the Stonk.fun "retained power to raise the tax".
  - Floor: `MIN_FEE_LAMPORTS = 3_000_000`, or keep 2,000,000 only if the requester pays the oracle separately.
  - Release: hard-code **0**. If Barton wants release revenue, `release ≤ capture`, paid into the FeeVault, and never validated on
    the release path (FF-14).
- **Should capture, release and re-roll differ?**
  - **Capture = re-roll.** Capture must be ≥ re-roll, or release + capture becomes a cheaper re-roll (M-06). Equal is simplest and
    keeps "every draw costs the same".
  - **Release should differ: 0.** It adds nothing to anti-grind (§2.5), and it's the only fee that can tax a holder's exit in a
    crash, feed the fee wallet on panic, or block users who hold no SOL.
  - With release 0, the M-06 condition is `re-roll ≤ capture + 0` → equal, which holds.
- **Why not a flat 0.01 on every action (the 4:49 PM proposal)?**
  - At 10k–100k, 0.01 is 25%–253% of an NFT's value at graduation.
  - Release would be underwater after only a −75% move at 100k.
  - The 4:52 PM tiers fix both while giving up protection only where floors are tiny and the fee already dominates. For example, the
    1/1 at 100k and graduation goes from 2,526× to 1,276×, still far above any plausible premium.
- **Revenue note** (not a security point): dropping the release fee removes about one third of fee events. If Barton wants that back,
  raise the capture fee within the cap rather than taxing exits.
- **Disclosure copy** (for the CD): "Converting and re-rolling cost a flat X SOL (fixed for this collection, max 0.01 SOL).
  Converting back to tokens is free. Odds: [live pool census]. Expected cost to hit a [tier]: ~Y SOL."

## 6. Which earlier findings this design closes, reopens or changes

| Finding | Effect of the flat tiered SOL fee + WT at 16:54 |
|---|---|
| **M-06** release + capture bypass | **Closed.** Fee equal on all three actions, so re-roll ≤ capture + release. Stays closed with release = 0 (re-roll = capture). It was briefly **reopened** in the ~16:50 WT (creator capture fee could be 0) and fixed at 16:54 |
| **M-19** grinding floor / SOL-minimum value | **Resolved in form** (a flat per-attempt SOL fee, independent of token price). Value answered here: protected at low floors, lottery for mid tiers at F ≥ 0.25 and for everything at 2.5M/5M. It becomes **FF-04 (disclosure)** |
| **M-05** fee-account substitution | **Mostly closed.** No token ATA. The SOL recipient is address-pinned (`FeeRecipientMismatch`) and re-checked against the platform constant |
| **M-15** fee ATA missing/closed DoS | **Moot** (no fee ATA) |
| **M-29** fee rounding (bps) | **Moot** (no bps) |
| **M-30** SOL minimum below rent-exempt | **Closed** (every tier ≥ 2,000,000; compile-time assert) |
| **M-07** `hybrid_launch` vs fee decision | **Closed in code** (no fee params, tier table). **Reopened as docs drift:** FF-11 |
| **M-33** doc bugs | **Reopened and extended:** the 2% token-fee text is now wrong too (FF-11) |
| **M-14** fee never backing / release no fee | Backing part: **closed** (the fee is SOL, it never touches the vault). The "release carries no fee" part is **reversed by design**. Recommendation: restore it (release = 0) |
| **M-26** pause/fee must never block release | Pause: **OK** in the WT. SOL fee via FeeVault: **OK** (FF-02 closed). **New gap: FF-14** (fee validation on the release path) |
| **M-04** expire loophole | **Closed in the WT** (fee at request, never refunded; principal-only expire after 3 recommits). Sim: reopening it makes rare farming 4–5× cheaper |
| **M-08** "never raised" vs upgradeable | **Unchanged: High (disclosure).** Now quantified as a 0.01 cap. FF-07 adds `min(stored, tier)` |
| **M-16** insider discount | **Worsened in relative terms:** 100% of the fee returns to the insider (21% of the public's cost at the 0.01 tier). B rates it **High (insider)**. See FF-03 |
| **M-17** fee owner custody / change path | **Improved:** a stolen key steals only revenue, never principal, so "never changeable" (option A) is now preferable. Remains Medium (custody) |
| **M-18** fee tokens → sell pressure (Stonk.fun) | **Largely closed:** no token accrues to the fee wallet. **Downgrade to Low** (SOL revenue disclosure only) |
| **M-20** queue DoS / cornering | Unchanged. The fee is now a flat SOL cost per request, independent of token price. The pool floor is the real defence (FF-10) |
| **M-24** band / listing arbitrage | Changed: the band is two fees wide and SOL-denominated (105% of V at 10k, ≤ 25% from 50k at graduation). The insider holds the zero-friction edge (FF-03) |
| **M-10 / M-22** graduation mint | Helped by the 10,000 cap (§3.3) |
| F-B1 / F-B2 (old fee doc) | F-B1 closed (flat SOL floor). F-B2 → FF-04 |
| F-B5 / F-B6 / F-B7 / F-B8 | F-B5 largely closed (no token pile). F-B6 → FF-08. F-B7 closed while the recipient is a constant. F-B8 moot (no token account) |
| F-B10 (fee on expire) | Closed (WT). Honest-user cost ≤ one fee per stuck request (FF-09) |
| F-B13 solvency invariant ignores pending re-rolls | **Closed in the WT:** `invariants.rs:3,18-19` now counts `assets_outside + pending_captures + pending_rerolls` (still `≥`, not equality modulo donations) |
| R1-B-12 burn | Still superseded (nothing is burned; supply stays 1B) |

## 7. Proving tests (collected)
1. `launch_params_have_no_fee_field` + `vault_rejects_config_with_fee_not_equal_tier` (FF-06, M-08).
2. `reroll_le_capture_every_tier` and `unwrap_rewrap_never_cheaper_than_reroll` (M-06).
3. `release_charges_zero_lamports` (if release = 0), or `release_fee_le_capture_fee_every_tier` (FF-01).
4. `release_succeeds_when_recipient_uncreditable` + `sweep_fails_but_release_succeeds` (FF-02).
5. `release_survives_fee_tier_change` + `release_survives_recipient_rotation` (FF-14).
6. `upgrade_cannot_raise_fee_for_existing_launch` (FF-07).
7. `withheld_reveal_past_all_recommits_returns_principal_only`: fee kept, principal returned (FF-09, M-04).
8. Devnet: measured Switchboard lamports per request vs the lowest tier (FF-13).
9. `init_vault_rejects_collection_above_cap` in `hybrid_vault` as well as `hybrid_launch` (§3.3).
10. Frontend: the expected cost per tier = fee / live pool odds (FF-04, FF-05). Ledger reconciliation of fee transfers and sweeps
    (FF-08).

A professional third-party audit is still required before mainnet. None of this replaces it.
