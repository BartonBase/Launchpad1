# Armory Tax mode (Token-2022 tax + Hybrid conversion): post-hackathon roadmap

**Status: roadmap, not built.** Written 2026-10-02 (MT). Nothing in this note is built, deployed or live.
Tax mode shows as "Coming soon" in the app. The hackathon submission (deadline Mon Oct 12, 11:59 PM MT)
ships Launch and Hybrid only. The **full Tax build is planned after Oct 12**. Before then the only Tax work
is a script-only devnet spike (§8, results in `docs/tax-spike-results.md`) with no program changes.
The legal flags (§6) and security notes (§4) below still apply and need a lawyer's view and an external
audit before any mainnet release.

**Recommendation (Option A).**
- Our program creates the Token-2022 mint and the pool in **one transaction**:
  - 1B fixed supply;
  - mint authority and freeze authority both None;
  - tax rate fixed, and the authority that could change it set to **None**;
  - withdraw authority for the held-back tax is a **program-derived address (PDA)**.
- It deposits the whole supply into a **single-sided Meteora DAMM v2 pool** and **permanently locks** the liquidity position, which is owned by a program PDA.
- Meteora's bonding curve (DBC) can't be used: it creates the mint itself, and that mint can only have metadata or a temporary transfer hook, never a transfer fee.
- DAMM v2 accepts transfer-fee mints with no permission needed (Meteora `damm-v2` `utils/token.rs:223-262`).

## 1. What people experience

- **Creator.** Picks name, ticker, art, NFT ratio, collection size and a tax of 0.5–3% (0.5% steps). Signs one transaction. The token is tradeable at once; there's no curve and no graduation. The creator gets no allocation, no tax share and no powers.
- **Buyers and sellers.** Trade on the DAMM v2 pool through Meteora, Jupiter or any Token-2022 wallet. On every transfer (buy, sell, wallet-to-wallet), Token-2022 holds back the tax inside the receiving account. You receive the amount minus the tax.
- **Tax to NFT holders.**
  1. **Harvest (anyone).** Sweeps held-back tax from token accounts, including the pool vault, into the program's treasury. A public bot runs it on a timer.
  2. **Accumulator.** Each harvest adds `holders' share ÷ live NFTs` to a running "earned per NFT" number. Every NFT stores what it has already been paid up to.
  3. **Claim (pull).** Anyone can trigger a claim for an NFT. It always pays the NFT's **current owner**. Unclaimed earnings move with the NFT when it's sold. Each claim costs the same no matter how big the collection is.

## 2. Combining with Hybrid conversion

| Move | Token transfer? | Taxed? | Rule |
|---|---|---|---|
| Capture (tokens → NFT) | user → vault | **Yes** | User sends the **grossed-up** amount `g`, where `g − tax(g) = ratio`, so the vault receives exactly `ratio`. The user pays the tax on top. |
| Re-roll (NFT → other NFT) | none (NFTs are Metaplex Core, not Token-2022) | No | Unchanged: flat SOL fee plus Switchboard randomness. |
| Release / unwrap (NFT → tokens) | vault → user | **Yes** | Vault sends exactly `ratio`, and the user receives `ratio − tax`, disclosed up front. The vault can't top this up, because a subsidy would be drainable by repeatedly wrapping and unwrapping. |
| NFT sale on Tensor / Magic Eden | none | No | Core NFTs aren't taxed. |

- **No exemption is possible.** Token-2022 has no per-account fee exemption, so the vault can't be exempted. The only zero-fee case is a 0% rate. So we must gross up on the way in and disclose the haircut on the way out.
- **A full round trip** (capture + release) costs about 2× the tax plus the SOL fee. That also discourages cycling.
- **Existing bug to fix.** In the shelved `wrap_token22`, the transfer sends `ratio`, so the lock account receives `ratio − tax`, but the vault books `ratio` (`hybrid_vault/src/instructions/token22.rs:338-352, 411-414`). Gross-up fixes it. Backing invariant: `vault balance ≥ ratio × NFTs outside`, checked after every instruction.
- Any tax held back inside the vault's own token accounts is harvestable like any other account, and goes to holders.

## 3. Architecture

**Accounts.**
- `TaxLaunchConfig` (hybrid_launch, immutable): mint, pool, position, `tax_bps`, split, ratio, size, fee tier.
- `TaxVault` v2 (hybrid_vault):
  - token accumulator `acc_tok` and SOL accumulator `acc_sol` (u128, scaled);
  - `live_nfts`;
  - undistributed remainder;
  - totals harvested / paid / burned.
- `NftShare` per NFT: paid-up-to for tokens and for SOL, plus an active flag.
- PDAs:
  - `tax_authority` (tax withdraw authority, treasury owner, SOL holder);
  - `lp_owner` (owns the locked position);
  - `vault_authority` (owns the NFT backing, as today).

**Instructions.**

| Instruction | Program | Reused / new |
|---|---|---|
| `launch_tax_pool`: create T22 mint, CPI to DAMM v2 `initialize_customizable_pool` (single-sided preset from a compiled allowlist), `permanent_lock_position` | hybrid_launch | Mint code reused from `launch_token22` / `t22.rs`; pool CPI is new |
| `init_tax_vault`, then capture / re-roll / settle / release on Token-2022 | hybrid_vault | Mode 2 VRF engine reused; T22 transfers + gross-up are new |
| `harvest_tax` (batch of accounts) | hybrid_vault | Extends the existing single-account version |
| `claim_share` (tokens + SOL) | hybrid_vault | New (replaces the round-robin `claim_tax`) |
| `claim_lp_fees` (pool fees in SOL → SOL accumulator / platform cut) | hybrid_vault | New |
| `sweep_sol` (stray lamports → SOL accumulator) | hybrid_vault | New |
| `reclaim_dead_share` (burned NFT) | hybrid_vault | Reuses the `skip_dead_nft` "dead asset" check |

**Removed:**
- `buyback` and `buy_inventory`, plus the launch inventory (this removes the locked-SOL bug);
- round-robin position / round / pot-tier fields;
- the sequential Mode 4 wrap (rares could be sniped).

**Program decision.** Upgrade the two existing programs; don't add a third (see §5). Payouts have no arguments, and every destination is derived on-chain, never taken from the caller.

## 4. Security (top priority: nobody can drain or dump tax funds)

**Every way value can leave, and why it's safe:**

| Holder | What leaves | Only path | Why safe |
|---|---|---|---|
| Mint (held-back tax) | tokens | `harvest_tax` → treasury | Withdraw authority = `tax_authority` PDA. The destination is derived (`ATA(tax_authority, mint)`) and not a caller argument. |
| Treasury | tokens | `claim_share` → current NFT owner's derived account; optional fixed burn | Amount = `acc − paid_up_to`, bounded by treasury balance and `harvested − paid`. No arguments. |
| `tax_authority` SOL | lamports | `claim_share` (SOL part); optional fixed platform cut of pool fees only | Same accumulator bound. `sweep_sol` only moves SOL *into* the accumulator. |
| Locked LP position | liquidity | **none** (permanently locked) | Only pool fees can be claimed, and only by `lp_owner` via `claim_lp_fees` into fixed sinks. |
| Vault backing | tokens | `release` → NFT owner, exactly `ratio` | Existing hybrid invariant; NFT burned or returned in the same instruction. |
| Capture / re-roll fees | SOL | user → fixed platform fee wallet | Unchanged ADR-013 tier fee. |

**Rules enforced on-chain:**
- **The rate can't change.** The rate-change authority is None at creation, so Token-2022 itself rejects any change. It's re-checked at vault init and on every capture.
- **The withdraw authority can't move.** No instruction signs `SetAuthority` with the `tax_authority` seed. A source-scan test pins every signing use of that seed.
- **The program never sells.** It has no swap call at all; the only DAMM v2 calls are pool create, lock and fee claim. Holders may sell their own claimed tokens, but no single wallet ever holds the pot.
- **Cap.** `MAX_TAX_BPS = 300` (down from 1000). Only {50, 100, 150, 200, 250, 300} are accepted. `maximum_fee = supply`, so the percentage is exact.
- **Burned NFTs.** `reclaim_dead_share` returns the share to the pot and decrements `live_nfts`. It's rejected on a live NFT.
- **Stray SOL.** `sweep_sol` sends it to holders, so nothing gets stuck.
- **No admin, no pause, no creator powers.** Upgrades stay behind the planned 3-of-5 multisig plus 7-day timelock (that isn't set up yet; the devnet upgrade key is a throwaway).

**Known risks:**
- **Meteora operator.** It can disable swaps on, or change fees of, any DAMM v2 pool (`set_pool_status`, `update_pool_fees`, operator-gated). Harvest and claims don't depend on the pool, so funds already collected are unaffected. Disclose it.
- **Launch front-run.** Someone could create the token/SOL pool first at their own price. Mint and pool creation must be atomic, and we'll test it.
- **MEV / sandwich.**
  - DAMM v2 swaps can be sandwiched. The tax on both legs makes sandwiches costlier. The app sets tight slippage.
  - First-block sniping: use DAMM v2's time-decay or rate-limiter fee schedule in the preset.
  - Harvest timing: someone could capture an NFT just before a large harvest and share tax collected before they held it. Mitigations: frequent harvests, plus the capture's own tax + fee + randomness delay. Disclosed.
- **Rounding.** Remainders stay in the pot. A property test proves claims never exceed harvests.
- **Needs an external audit:**
  - the accumulator math and bounds;
  - gross-up / backing invariant on capture and release;
  - the DAMM v2 CPI accounts (pool, position, lock) and atomic launch;
  - PDA signing scope (`tax_authority`, `lp_owner`);
  - extension allowlist checks on the mint;
  - the reused VRF engine on Token-2022.

## 5. Cost and size

**Engineering (refined):**

| Area | Full scope | Reduced first release |
|---|---|---|
| Programs (launch_tax_pool, T22 capture/re-roll/release with gross-up, harvest, accumulators, LP fees, sweep, dead share; removals) | 12–15 days | 5–6 days |
| LiteSVM harness with real DAMM v2 + Token-2022, attack tests, devnet script | included above (≈4) | ≈2 of the above |
| App (wizard, token page tax/locked proofs, claims, harvest bot, reconciliation) | 4–6 days | 2–3 days |
| Review + test suites | 4–5 days | 2 days |

- Full scope is about 20–26 days of work. It doesn't fit before Oct 12 with review time, so it is scheduled after the hackathon.
- **Reduced first-release scope:**
  - taxed DAMM v2 launch (atomic, locked);
  - permissionless harvest;
  - **one-way capture** (Mode 2 randomness, Token-2022 gross-up);
  - per-NFT token accumulator plus pull claims;
  - `reclaim_dead_share`.
- **Deferred:** release/unwrap, re-roll on Token-2022, pool-fee claims and the SOL accumulator, burn split, platform cut.
- Claims need NFT holders, so *some* conversion (capture) has to stay in. Without it, only a taxed launch plus harvest into a locked treasury could be demoed.

**Devnet SOL and programs:**
- Upgrade the existing devnet programs instead of deploying new ones.
- Program accounts must grow to fit the new code:
  - vault: deployed 692 KB; today's build with tax + raffle code is 1.10 MB. A tax-only build is estimated at about 0.95 MB, so roughly 260 KB more ≈ **1.3 SOL** extra rent;
  - launch: 231 KB → ~350 KB, ≈ **0.6 SOL**.
  - Rent ≈ 5,080 lamports per byte (checked on devnet Oct 2). The upgrade buffer is refunded after deploy.
- A brand-new third program (~600 KB) would cost about **3.1 SOL** rent plus its own audit scope. Not recommended.
- Test runs need about 0.3–0.5 SOL per end-to-end launch (pool, accounts, buys, captures). Budget **1.5–2 SOL** for testing.
- **Total ≈ 3.5–4 SOL.** The throwaway deployer holds about 2.66 SOL (Oct 2); top up about 1.5 SOL from the devnet faucet (free).
- This upgrade changes the deployed devnet builds the web app uses today. That's a decision for Barton, see §7.

## 6. Legal flags (plain words; not legal advice, ask a lawyer)

- **Securities.** Paying holders an ongoing share of trading tax can look like "profit from others' efforts" (a security or investment contract in the US and similar tests elsewhere). Holding an NFT to earn makes it look more like yield.
- **Listings.** Many centralized exchanges, bridges and some wallets or bots refuse or mishandle transfer-fee tokens. Receivers always get less than was "sent". Jupiter does route them.
- **Jurisdiction and consumer law.** Rules on promoting yield to retail differ by country (US, UK, EU MiCA). Geo-blocking or disclaimers may be needed. Tax reporting for holders receiving tokens is their own obligation, but say so.
- **Recommendation.** Get a crypto lawyer's view before mainnet. For the hackathon: devnet only, fake SOL, clearly labelled as a demo.

## 7. Decisions needed from Barton

1. **Tax rates offered:** 0.5–3% in 0.5% steps (recommended), or fewer options.
2. **Split:** holders vs burn vs platform. Recommended: 100% of tax tokens to NFT holders (or 80/20 holders/burn). The platform takes SOL only (capture fees + optional share of pool fees). No creator share.
3. **When conversion opens:** at launch, or once the pool holds X SOL.
4. **Unwrap:** allowed with the tax haircut (recommended, disclosed), or one-way only.
5. **Meteora pool fee:** base fee (for example 1%), the anti-sniper start fee and decay time, and who gets our LP-fee share (holders, platform or a split).
6. **Devnet upgrade:** approve upgrading the deployed devnet programs (≈1.9 SOL extra rent, changes the live demo builds) and the reduced first-release scope (after Oct 12).

## 8. First step: 1-day devnet spike (script only, no program changes)

1. Script: create a 1% Token-2022 mint (rate authority None, withdraw authority = a throwaway key standing in for the PDA), 1B supply, mint authority revoked.
2. Create a single-sided DAMM v2 customizable pool with Meteora's SDK (`preparePoolCreationSingleSide`) and permanently lock the position.
3. Buy and sell from 2–3 throwaway wallets. Check the held-back amounts, the pool vault's held-back tax, and the Jupiter / Phantom display.
4. Harvest to the mint, withdraw to a treasury, and reconcile totals.
5. Write down the costs and compute units, and whether launch + pool + lock fits one transaction. That decides the atomic design.

Cost: about 0.3 SOL of devnet SOL. Output: a short results note, then go/no-go on the program work.
