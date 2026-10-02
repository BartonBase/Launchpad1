# Fee-design stress test: 2% fee to one fixed address + SOL minimum (BRIEF FEE CHANGE, 2026-09-25 4:27 PM MT)

**Author:** Auditor B worker. **Date:** 2026-09-25, written 16:45 to 17:40 MT. **Status:** economic and design review, plus a
check of the engineer's **uncommitted working tree** as it stood at 16:39 MT. That tree is on branch `wip/hybrid-vault`: HEAD `977f8f2`
plus local edits to `programs/hybrid_launch` and `programs/hybrid_vault`, which are changing while I review. No deployment, no
devnet or mainnet writes, no real keys or funds. One read-only fetch of the Switchboard On-Demand program's public Anchor IDL
from mainnet RPC (for §(d)/M-04).

**Model:** `sim/fee_stress.py` → `sim/fee_stress_output.txt` (Parts A to J). Market inputs are **examples** and are labelled as
such in the script. The main ones: graduation FDV 100 / 410 / 1,500 SOL (410 ≈ the pump.fun-style curve from my round-1 sim), a
post-graduation CP pool of 206.9M tokens / 85 SOL, and rarity tiers of 1/1 = 1/N at 100×, legendary 0.1% at 20×, epic 1% at 5×,
rare 5% at 1×. **No Solana-404 premium dataset was read. Premiums are assumptions.**

**Design under test (BRIEF.md, 2026-09-25):**
- A 2% fee on capture and on every re-roll, paid in the collection token **on top of** N and sent directly to ONE fee address fixed at launch.
- The 2% rate is hard-capped in code.
- Release returns exactly N with no fee.
- Re-roll also pays a small fixed SOL minimum.
- Recommended: the fee address is a multisig.

**Label check:** the brief says this addresses "B-08", but no B-08 of mine is a fee finding. R1-B-08 is MEV around converts, and my
old threat-model B-08 is the flash-snapshot lottery item. A's round-1 "B-08" is the burn-from-user-balance item. I treat the
design as addressing **R1-B-04** (launch-window farming) and **R1-B-05** (fee sizing, no minimum), since the BRIEF's "(Auditor B
finding)" note on the SOL minimum points there. I also treat it as touching **R1-B-10** (operator powers, via the fee address)
and **R1-B-12** (burn, now superseded).

**What the working tree already implements (16:39 MT, uncommitted):**
- `hybrid_launch/src/constants.rs:47,50` `MAX_FEE_BPS = 200`, `MIN_FEE_BPS = 100`.
- `:56` `MIN_REROLL_SOL_FEE_LAMPORTS = 1_000_000`.
- `:58` `MAX_SOL_FEE_LAMPORTS = 50_000_000`.
- `:67` `PLATFORM_FEE_OWNER` constant, with a mainnet `compile_error!` until it's set.
- `validation.rs:50` enforces `capture_sol_fee_lamports ≥ reroll_sol_fee_lamports`.
- `validation.rs:31` computes the fee as `ceil(u128)`.
- `hybrid_vault/src/instructions/request.rs` pays the token fee to `vault.fee_account` (address-pinned, `create_idempotent`) and the SOL fee to `vault.fee_owner` **at request**.
- `randomness_ix.rs` adds a PDA-authority init/reveal/recommit.
- **No expire or refund path exists at all** (`constants.rs:33-36`, "ADR-012").

---

## Summary table

| ID | Item | Severity | Status in working tree |
|---|---|---|---|
| F-B1 | (a) Graduation-floor grinding: the token fee is ~0 in SOL at small ratios; the SOL minimum is the whole cost | Medium | Partly addressed (s ≥ 0.001 SOL; value TBD) |
| F-B2 | (a) No fixed s bounds **mid-tier** grinding at high floors, and none bounds the graduation-window growth bet | Medium (policy) | Open (needs Barton) |
| F-B3 | (d) Release→capture as a cheaper re-roll (bypasses the re-roll SOL minimum) | High (if capture s < re-roll s) | **Closed in working tree** (`validation.rs:50`). Keep the test |
| F-B4 | (d) Insider free-grind: the fee owner's own draws pay itself (token fee **and** SOL minimum) | High (insider) | **Open**: the SOL fee goes to `fee_owner` |
| F-B5 | (c) Sell pressure and trust from a single growing token position; no disclosure; Stonk.fun pattern | High (policy/trust) | Open |
| F-B6 | (c) Fee key as a single point of failure; "fixed at launch" doesn't help against key theft | High | Partly (the constant is meant to be a Squads vault) |
| F-B7 | (d) Fee-destination change path = the A-02 hijack class | Critical if a change path exists; closed if immutable | Constant today (good) |
| F-B8 | (d) Fee account address-pinned but owner not checked (classic SPL has no ImmutableOwner) → fees to a stranger, or a DoS | Medium | Open |
| F-B9 | (b) Escrow invariant: holds. Round trip leaks nothing; release is fee-free | Info (pass) | Pass in model; on-chain invariant is loose (F-B13) |
| F-B10 | (d) Fee on expired or never-revealed requests (R1-B-02 link) | High → Low | Committed `977f8f2` refunds the fee on expire (High). Working tree: no refund, but **no escape either** (liveness, see M-04 §) |
| F-B11 | (d) "2%, can never be raised, not even by the multisig" is false while the program is upgradeable; the working tree also lets creators pick 1% to 2% | Medium (disclosure) | Open |
| F-B12 | (d) Rounding to zero / small amounts | Low (pass) | Pass: exact for every allowed ratio and decimals value; s ≥ rent minimum |
| F-B13 | (b) On-chain solvency invariant ignores pending re-roll hand-ins | Low | Open (`invariants.rs:3,15-19`) |
| F-B14 | (d) Fee-free paths: secondary-market buys; the graduation mint; CPI wrappers | Info / Medium | See below |

---

## (a) Grinding cost at low token prices: what SOL minimum, and should it track the floor?

**Numbers** (`fee_stress_output.txt` Part A, FDV 410 SOL at graduation; draws exist only after graduation, per the GRADUATION MODEL):

| R | max N | floor F | 2% token fee (SOL) | cost/draw at s = 0.005 | share paid by s |
|---:|---:|---:|---:|---:|---:|
| 10k | 100,000 | 0.0041 | 0.000082 | 0.00518 | 96% |
| 100k | 10,000 | 0.041 | 0.00082 | 0.00592 | 84% |
| 1M | 1,000 | 0.41 | 0.0082 | 0.0133 | 38% |
| 5M | 200 | 2.05 | 0.041 | 0.0461 | 11% |

At FDV 100 and R = 10k the token fee is **0.00002 SOL per draw** (Part J). That's effectively zero. Without the SOL minimum, a draw costs
only network fees.

**Does a flat minimum make farming the TOP tier unprofitable?** (Part B; this settles disagreement D6 with A)
- For a 1-of-N, break-even premium is `m* = N × (0.02 + (s + tx)/F)` floors. It grows **linearly with N**, so a flat `s`
  already makes cost-per-hit scale with rarity. At s = 0.005 and FDV 410:
  - m* = **126,390×** (R = 10k)
  - **1,444×** (100k)
  - **411×** (200k)
  - **90×** (500k)
  - **32×** (1M)
  - **10×** (2.5M)
  - **4.5×** (5M)
- In my example band (1/1 at 10× to 1,000× floor), top-tier farming is unprofitable at steady state for R ≤ 100k, marginal at
  200k, and profitable only for R ≥ 500k when the 1/1 trades above ~4× to 90× floor.
- **At those large ratios the SOL minimum barely matters.** F is 0.2 to 2 SOL, so s is 2% to 20% of the per-draw cost, and m*
  converges to `0.02 × N` = 4× to 40×. Only the token fee sets it. No flat s changes that, and neither does my round-1
  "`s ≥ k / p_rarest`" scaling. That rule gives `s = k × N`, which is **largest for big N (where m* is already huge) and
  smallest for small N (where m* is small)**. It points the wrong way.
- **Verdict: A is right.** A flat constant is enough for the top tier, and my rarest-tier scaling should be withdrawn (see cross-review §7).

**Where a flat minimum does NOT work** (Parts C, D): tiers whose `p × m` exceeds 2%. In the example, epic 1% × 5× = 5% and rare 5% × 1× = 5%.
- For those tiers, grinding is +EV once `F × (p·m − 0.02) > s`. The required s is proportional to F:
  - 0.00002 SOL at R = 10k
  - 0.0122 SOL at 1M
  - 0.0614 SOL at 5M (FDV 410)
- A fixed s high enough for 5M is **500× the floor at R = 10k** (Part D). A fixed s low enough for 10k lets 5M epics be ground at +0.056 SOL per draw.
- Only a floor-tracking term `k × F` bounds this. Economically that's just a higher percentage fee (2% + k), and it collides with the "2% hard cap" copy.
- **Recommendation:** don't track the floor. Mid-tier grinding at `p·m > 2%` is a **market-priced lottery**. It's open to everyone,
  it's disclosed, and it's self-correcting: sustained grinding adds supply of those tiers until their premium falls to about
  `(0.02F + s)/p`. Treat it as disclosure. Show live odds and the expected cost per tier on the re-roll button.

**Graduation window** (Part E): farming at the graduation floor and selling after growth g.
- With s = 0.005 and FDV 410, a 1/1 is +EV at g = 10 if it's worth more than:
  - **144×** the graduation floor (R = 100k)
  - **3.2×** (R = 1M)
  - **0.4×** (R = 5M)
- That's a bet on growth, the same bet as buying the token, with a rarity kicker. No fee that is fair to honest users prices it out
  (Part C at g = 10 needs s = 117% to 618% of the floor).
- Tools that do work: (i) the curve-closed rule Barton already adopted (no draws at curve prices), and (ii) an optional **decaying graduation surcharge**.

**Recommended SOL minimum (formula and numbers):**
```
s_draw(t) = max( S_FLOOR , S_COST ) + k_g × F_g × max(0, 1 − (t − t_open) / T_g)      [optional 2nd term]
S_FLOOR   = 5,000,000 lamports (0.005 SOL), program constant, ≥ rent-exempt 890,880 lamports
S_COST    = measured Switchboard cost per commit+reveal (+ margin); today assumed ≤ 0.002 SOL
F_g       = R × final curve price, written on-chain once at graduation (no oracle, not manipulable)
k_g       = 0.10,  T_g = 72 h                                                           [Barton policy]
applied identically to capture AND re-roll (capture_sol ≥ reroll_sol), never refunded
```
- **Fixed vs tracking:** a fixed SOL minimum works across the whole price range **for the top tier and for anything with `p·m ≤ 2%`**.
  It doesn't need to track the NFT floor. It can't stop mid-tier grinding at high floors, and it shouldn't try: treat that as disclosed lottery pricing.
- **Values:** 0.005 SOL agrees with A's proposal. The working tree's floor of 0.001 SOL is too low for R ≤ 100k:
  - m* for a 1/1 at R = 100k drops from 1,444× to 468×.
  - At g = 10 the 1/1 is +EV above ~47× (Part B).
  - Raise `MIN_REROLL_SOL_FEE_LAMPORTS` to 5,000,000, or make the value a program constant rather than a creator range.
- **Graduation surcharge** (optional, Barton): at t = 0 it lifts the 1/1 break-even from 32× → 132× (R = 1M) and from 4.5× → 24.5× (R = 5M). It decays to 0 by 72 h.
  Because it applies to capture too, it's also an anti-flip speed bump. It must be disclosed on the token page as "launch
  surcharge, SOL, decays to 0 by …", and it must **not** go to the fee owner (see F-B4).

**Severity:**
- F-B1 **Medium.** Closed if S_FLOOR = 0.005 SOL is a constant applied to both capture and re-roll.
- F-B2 **Medium (policy).** Accept it with disclosure, or add the surcharge.
- **Owner:** Solana Program Engineer (constants), Barton (surcharge yes/no), Frontend Engineer (odds and cost display).
- **Test:** a unit test asserts `capture_sol == reroll_sol ≥ 5_000_000` for every launch. Also a property test that for every allowed
  ratio at FDV ∈ {100, 410, 1500}, the expected cost of a 1-of-N ≥ `N × S_FLOOR`.

---

## (b) Is the 2% really on top of N? Escrow invariant on capture, re-roll and release

- **Model** (Part H, 200k random ops for R = 1M, 10k for the other ratios, integer base units): with capture = user pays `R + fee` (R to the vault, fee to the fee address),
  re-roll = user pays `fee` (NFT in, NFT out), release = vault pays exactly R, and a principal-only expire, then
  **`vault == R × (NFTs outside + pending requests)` holds exactly, and `users + vault + fee_address == 1B` is conserved, for every ratio.**
- **Release pays no fee** (BRIEF; `hybrid_vault/src/instructions/unwrap.rs`: "Deliberately NO pause check and NO fee").
- **Round trip:** capture → release costs `0.02R + s` and returns R. **No value leaks** from the protocol. Re-roll → release is the
  same. The only arbitrage is market-side. The NFT floor sits in the band `[R·P, 1.02·R·P + s]` (my R1-B-07, unchanged).
- **Code check (working tree):** the fee transfer's source is `user_token` (`token::authority = user`, and the constraint forbids aliasing
  `vault_tokens`). The destination is `vault.fee_account` (address-pinned). Nothing moves the fee through the vault. Good.
- **F-B13 (Low, new):** the on-chain solvency check is `vault ≥ ratio × (assets_outside + pending_captures)` (`hybrid_vault/src/invariants.rs:3,15-19`),
  but `request_reroll` decrements `assets_outside` (`request.rs:209`). Each pending re-roll therefore leaves R of backing unaccounted.
  - My model's first version (which mirrored this) failed the **equality** check.
  - `≥` still holds, so it isn't exploitable today. But a future bug that paid out R during a pending re-roll would pass the check.
  - **Fix:** `owed = assets_outside + pending_captures + pending_rerolls`, and assert equality modulo donations (track `donations` separately, or compare against a stored `expected_vault_balance`).
  - **Owner:** Solana Program Engineer. **Test:** a fuzz test over interleaved request/settle/release that asserts `vault_tokens == ratio × (outside + pending) + donated`.

---

## (c) Sell pressure, the trust problem, and Stonk.fun

**The pattern we must not repeat** (stonkfun-lessons.md, quoting Bitquery):
> "The viral 'STONK FEE DRAIN' wallet is Stonk.fun's own reward wallet. … One wallet sweeps it, sells it into each coin's own
> pool, and pays holders in the pair asset. $56.3M sold vs $56.2M paid out over 30 days."
> "The harm is the design: the tax sells into every chart. 160 coins lost >50% of supply to tax (BUDDY 69%, ZCAT 61%)."
> "Trust failures: >=$1.41M of reward money went in single transfers to Stonk.fun-linked wallets … omitted from its public ledger."
> Lesson 2: "No discretionary platform wallet that custodies or routes user value." Lesson 4: "Every fee destination is on-chain,
> public, and matches what the site says." Lesson 6: "Publish a verifiable ledger."

**Differences:**
- The fee is charged per **convert/re-roll**, not per transfer, so ordinary trading generates none.
- Fees go to one address and aren't redistributed.

**Same risk:**
- One operator address accumulates the token of every collection and **may sell it into each coin's own pool**.
- Unlike Stonk.fun, it doesn't even pay anything back to holders.

**Magnitudes** (Part I; % of supply per draw = `0.02 × R / 1B`, identical for every ratio at max N):

| Scenario (per day, as a fraction of N) | fee tokens/day | 1-day dump impact | 30 days cumulative |
|---|---:|---:|---:|
| quiet (2% capture, 1% re-roll) | 0.06% of supply | −0.6% | 1.8% |
| normal (10% / 10%) | 0.40% | −3.8% | 12% |
| hot (30% / 50%) | 1.6% | −13.8% | 48% |
| farm frenzy (50% / 300%) | 7.0% | −44% | (exceeds float; only possible if the fee address sells and others re-buy) |

- One full farm of all 10 legendaries in a 10k pool (~29,290 draws) sends **5.86% of supply** to the fee address. Dumped at once, that's **−39%** on the example pool.
- Dumping 2% of supply in one trade costs **−16.9%**. Streamed over 30 tranches it's −0.64% per tranche (before arbitrage refills).

**Trust problems:**
1. A single operator address holds a growing position in every collection's token.
2. It can dump into thin post-graduation pools.
3. It has **insider information**: it knows about upcoming listings and features, and can watch its own inflows as grinding happens.
4. There's no on-chain disclosure of intent or sales.
5. "Fixed at launch" protects against the destination being re-pointed. It does nothing against theft of the key itself, and a stolen key dumps **every** collection at once.

**Mitigations (recommended; Barton decides policy):**
1. **Public address + dashboard:** the fee address, balance per collection, cumulative received and every outflow, shown on every token page and in the top-holder view. Publish a reconciliation script (lesson 6).
2. **PDA-held fee vault with enforced streaming** (preferred over a plain wallet):
   - Fees go to `ATA(FeeVault PDA, mint)`.
   - The only outflow is `release_fees(mint)`, which pays the Barton multisig at most `x%` of the vault balance per day (e.g. 1/30), after a `≥ 7-day` delay from receipt.
   - There's no other instruction.
   - This isn't a "discretionary wallet" (lesson 2): the program enforces the schedule and anyone can read it. It conflicts with the BRIEF line "No intermediate wallet". **Barton decision.** Wallet-and-policy (option 3) is the fallback.
   - Side benefit: it lets the program refund honest users on a provable oracle outage (see M-04), which a direct-to-wallet design can't do.
3. **Written sell policy**, if a plain wallet is kept: no sales in the coin's own pool for X days after graduation, a max per-day sell of y% of pool depth, OTC/TWAP only, and pre-announced windows. Or **programmatic buyback/none**: never sell, and only use fee tokens for disclosed purposes.
4. **Multisig:** the fee owner is a Squads vault, ≥ 2-of-3 with hardware keys held by independent people, and the signer set is published. The working tree's `PLATFORM_FEE_OWNER` compile-guard is the right hook. Assert on-chain at `init_vault` that the owner is **off-curve** (a PDA).
5. **Insider-trading policy:** the fee address and affiliated wallets never capture, re-roll or trade the collection tokens except through the published schedule. Monitor it publicly.

**Severity:** F-B5 **High (policy/trust)**, F-B6 **High**.
- **Owners:** Barton (policy, vault-vs-wallet), Solana Program Engineer (fee vault if chosen, off-curve assert), Frontend Engineer (dashboard).
- **Tests:**
  - `init_vault` rejects an on-curve `fee_owner`.
  - If a fee vault is adopted: `release_fees` can't exceed the schedule, and no other instruction moves vault fees.
  - A dashboard reconciliation test replays all fee transfers and matches balances.

---

## (d) Everything else

**F-B3 Release → capture bypass: High → closed in working tree.**
- **Scenario:** `unwrap` is free, so if capture carries only the 2% token fee, unwrap + capture is a re-roll without the SOL minimum. Part F discount: **96%** at R = 10k, 84% at 100k, 38% at 1M.
- **Cost to attacker:** tx fees plus 2% of F.
- **Fix:** `capture_sol ≥ reroll_sol`. Now enforced at `hybrid_launch/src/validation.rs:50`. Keep a negative test.
- **Owner:** QA Engineer. **Test:** `attack_capture_sol_below_reroll_sol_is_rejected`. Plus an economic test: the cost of unwrap + capture ≥ the cost of a re-roll for every ratio.

**F-B4 Insider free-grind: High (insider), open.**
- **Scenario:** the fee owner, or a wallet it funds, draws, and the 2% comes back to it. In the working tree the **SOL fee also goes to `fee_owner`** (`request.rs:226,274` `sol_fee(..., fee_owner, sol)`), so its marginal cost is tx + oracle cost ≈ **0.0011 SOL per draw vs 0.005 to 0.046 for the public** (Part F).
- **Cost to attacker:** ~0 for anyone with fee-owner access. It can farm every rare in every collection.
- **Fix:**
  - Send the SOL minimum to a sink nobody controls: the incinerator `1nc1nerator11111111111111111111111111111111`, or a program PDA with no withdraw, minus the actual oracle cost paid through.
  - Keep only the token fee going to the fee owner.
  - Disclose the conflict of interest.
  - Monitor fee-address outflows to wallets that later convert.
  - (Agrees with A F-04.)
- **Owner:** Barton (policy), Solana Program Engineer.
- **Test:** the SOL leg's destination == `INCINERATOR` (or sink PDA), and the fee owner's lamport delta == 0 on a draw.

**F-B7 Destination change path: Critical if it exists.**
- My B13 PoC (`poc-crosscheck/b13_token_swap_full_drain.out`) shows the upstream version: after an "empty" `update_recipe_v1`, a third party's 20,000-token fee goes to the attacker and the disclosed address gets +0.
- The BRIEF leaves "(or only via multisig + 7-day timelock, TBD)" open.
- **Recommendation:** **immutable**. `PLATFORM_FEE_OWNER` is a program constant copied into each `LaunchConfig`. If Barton insists on recoverability, use A's F-09(B) (a platform `FeeConfig` PDA with an in-program ≥ 7-day timelock and the pending value displayed), and never make it per-launch or creator-set.
- **Test:** no instruction in either IDL writes `fee_owner` or `fee_account` after init. An IDL diff check runs in CI.

**F-B8 Fee account owner not checked: Medium.**
- `request.rs` pins `fee_account` by **address** (`address = vault.fee_account`) and calls `create_idempotent`.
- Classic SPL Token accounts (ATAs included) can have their owner changed with `SetAuthority(AccountOwner)`. ImmutableOwner is a Token-2022 extension. So a compromised fee key can re-assign the ATA, and every later fee goes to the new owner at the same address.
- `create_idempotent` on an existing ATA with a changed owner errors, so draws then fail (DoS) or keep paying the stranger, depending on the order of checks.
- **Fix:** also deserialize and require `owner == vault.fee_owner && mint == vault.mint`. Fail closed. (Agrees with A F-06.)
- **Test:** re-assign the fee ATA's owner in LiteSVM. `request_capture` must fail with `FeeAccountOwnerMismatch`, and no tokens may move to the new owner.

**F-B10 Fee on expired or refunded requests: link to R1-B-02.**
- Committed `977f8f2` `expire.rs` refunds the escrowed fee: **High**. Part G: E[cost] per legendary falls from 13.30 SOL to 1.10 SOL for R = 1M at FDV 410.
- The working tree has **no refund and no expire**, and the fee is paid at request, so the free abort is closed. But see the M-04 analysis in cross-review §7: with no escape, an unrevealable request locks principal indefinitely.
- **Recommended final rule:** the fee (token and SOL) is paid at request and never refunded. Principal-only escape only after the recommit cap.
- **Owner:** Solana Program Engineer.
- **Test:** RR-08 rewritten. Withhold the reveal, run past all deadlines, and the requester recovers ≤ principal and 0 fee.

**F-B11 "Never raised" copy, and a creator-selectable rate: Medium (disclosure).**
- The 3-of-5 / 7-day upgrade path can change any constant until `--final`.
- The working tree also accepts `fee_bps ∈ [100, 200]` per creator. That's compatible with "capped at 2%", but it's not "2%". A 1% launch halves the token part of the grinding floor.
- **Fix:** A's F-08 copy. Store `fee_bps` in the immutable `LaunchConfig` and assert `≤ 200` in `hybrid_vault` too. Barton decides whether creators may pick 1%.
- **Test:** `hybrid_vault` rejects a `LaunchConfig` with `fee_bps > 200`.

**F-B12 Rounding: Low (pass).**
- Every allowed ratio × decimals {0, 6, 9} × bps {100, 200} gives an exact, non-zero fee (Part J, asserted). The minimum fee is 100 tokens.
- u64 has headroom.
- The SOL leg is ≥ the rent minimum (1,000,000 ≥ 890,880 lamports). A too-small SOL fee paid to an empty system account would fail (agrees with A F-12).

**F-B14 Fee-free paths:**
- **Secondary market:** buying an NFT on Tensor or Magic Eden and releasing it pays no protocol fee. That's legitimate: it's how the floor band works. **Info.**
- **Graduation mint:** the collection is minted into the vault. The working tree has new `mint_assets.rs` / `open_vault.rs` / `graduation.rs` files, which I haven't reviewed. Any path that sends NFTs anywhere except the vault is a fee-free *and* choice-enabling allocation (Q-H6). **Medium until reviewed.** **Test:** after graduation, `deposited == collection_size`, `assets_outside == 0`, and nobody but the vault owns any asset.
- **CPI wrappers:** a wrapper calling `request_*` still pays, because the fee is charged inside the instruction. **Info.**
- **Direct escrow interactions:** token donations to `vault_tokens` are harmless (≥ invariant). There's no instruction that moves NFTs except settle, unwrap, and the graduation mint.
- **Wash converts:** they cost everyone except the fee owner 2% + s, so they're pointless for outsiders. For insiders they're free and would inflate "volume" and fee stats. Disclose, and report volume net of fee-address-funded wallets. **Low.**
- **Does the SOL minimum go to the same address?** In the working tree, yes (`fee_owner`). I recommend against it (F-B4).

---

## Which round-1 findings the design closes, partially closes, or leaves open

| Finding | Effect of the 2026-09-25 fee design (+ working tree) |
|---|---|
| R1-B-04 launch-window farming | **Partially closed.** Mostly by the GRADUATION MODEL (no draws on the curve). The fee design adds the SOL floor. The graduation-window growth bet remains (F-B2; optional surcharge) |
| R1-B-05 fee sizing / no minimum | **Closed for the minimum** (bps floor 100, SOL floor ≥ 0.001; raise it to 0.005). Mid-tier pricing becomes disclosure (F-B2) |
| R1-B-02 expire refund | **Closed in the working tree** (fee at request, no refund). **Open in committed code and in docs** (N7, hybrid-rarity §3.3). A new liveness issue replaces it (M-04 review) |
| R1-B-12 burn contradiction | **Superseded.** Nothing is burned. Supply stays 1B. The burn copy in docs, design and tests must change (A's list) |
| R1-B-10 operator powers | **Worsened** in one way (a new operator-held value stream: F-B4, F-B5, F-B6, F-B7) and improved in another (the rate cap is in code). Stays **High** |
| R1-B-07 floor band / arbitrage | Unchanged. The band widens by 2% + s on the capture side |
| R1-B-01, -03, -06, -08, -09, -11, -13, -14 | Unaffected |

## Proving tests (collected)
1. `capture_sol == reroll_sol ≥ 5_000_000`, and the cost of unwrap + capture ≥ the cost of a re-roll (F-B1, F-B3).
2. The SOL leg goes to the incinerator/sink, and the fee owner's lamport delta is 0 on a draw (F-B4).
3. Fee ATA owner re-assigned → the request fails closed, and nothing is paid to the new owner (F-B8).
4. The IDL has no writer of `fee_owner`/`fee_account`/`fee_bps` after init (F-B7, F-B11).
5. Fuzz: `vault_tokens == ratio × (outside + pending_captures + pending_rerolls) + donated`; `Δfee_account == fee` on request, `0` on settle/unwrap/reveal/recommit (F-B9, F-B13).
6. Withhold the reveal past all deadlines: the requester recovers ≤ principal and 0 fee, and a later settle's candidate set is unchanged (F-B10).
7. Post-graduation: all N assets are vault-owned before converting opens (F-B14).
8. `init_vault` rejects an on-curve fee owner (F-B6).

A professional third-party audit is still required before mainnet. None of this replaces it.
