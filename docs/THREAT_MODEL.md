# On-chain threat model (engineering view)

Owner: Solana Program Engineer. Status: v0.1 for the scaffold. It complements, and doesn't replace, the independent
auditor models in `../security/auditor-a/00-threat-model.md` and `../security/auditor-b/threat-model.md` (B-xx findings,
R-xx requirements in `design-requirements.md`). Where a row maps to an auditor requirement, it's cited.

Test status legend:
- ✅ = implemented and passing in this repo today
- 🧪 = planned test, specified here
- 🔍 = monitoring/process control

## Assets and trust boundaries

- **Pooled value (highest risk):** fee vault (withheld Token-2022 fees), prize vault (NFTs), MPL-Hybrid escrow backing
  tokens, and any SOL/wSOL used to buy prizes.
- **Authorities:** mint/freeze (revoked), withdraw-withheld (PDA), fee-config (None/bounded PDA), MPL-Hybrid escrow
  authority, program upgrade authority, program admin.
- **Trust boundary:** everything the caller passes in is attacker-controlled: accounts, amounts, VRF accounts, and
  "program" accounts. Only PDAs we derive, program IDs we pin, and state we stored are trusted.

## Incident to design against: Stonk.fun distribution layer

**What the BRIEF says:** "a bot drained tokens and liquidity via Stonk.fun's distribution layer."

**What public sources support (searched 2026-09-24):** I found **no credible public report** of a Stonk.fun exploit,
hack, or bot drain of tokens or liquidity through its distribution layer. Auditor B independently reached the same
conclusion (`../security/auditor-b/research-sources.md` §1). Searches only turned up normal price moves, sniping
complaints, phishing domains, and reward-payout announcements. **Please send the original link or tx signatures for the
incident (DECISIONS Q6)** so we can model the exact mechanism.

**What *is* documented about Stonk.fun's distribution design** (this is how such a drain would plausibly work):

- Reward launches are Token-2022 transfer-fee tokens. After the fee is withheld, "everything after that is administered
  by StonkFun … Fees accumulate, StonkFun sweeps them, swaps them into the paired asset, and pays out to a list of
  eligible addresses". PDAs/locked holders were at first excluded from the list
  (https://stakepoint.app/blog/how-to-lock-stonkfun-tokens, a locker vendor with a commercial interest).
- ~3% transfer tax, graduation to Raydium (https://iq.wiki/wiki/stonk,
  https://www.cryptopolitan.com/why-is-stonk-crashing-competition-heated/). StonkFun admitted sniping and
  single-wallet-launch problems and moved to Raydium LaunchLab
  (https://thecoinomist.com/news/stonk-surges-250-to-140m-after-raydium-launchlab/, per Auditor B).
- Operator-discretion buybacks and payouts (https://fintrender.com/en/reports/a-case-for-stonk, an opinion piece).

**Failure modes we assume from that design, and our controls:**

| # | Plausible drain mechanism | Our control |
|---|---|---|
| S-1 | Hot operator wallet holds swept fees and the payout key, so a compromised or buggy bot empties it | No operator wallet in the path. Fees go withheld → `vault_authority` PDA → fixed vault. No instruction takes a free destination. Admin can't withdraw (T-DIST-01/02) |
| S-2 | Predictable sweep-and-swap sandwiched or front-run by bots (fee → paired asset → liquidity drained via price impact) | Prefer no token→SOL swap: buy prizes via fixed-rate MPL-Hybrid capture. Otherwise TWAP-derived on-chain `min_out`, per-purchase and per-window caps, randomized/permissionless timing (T-DIST-04) |
| S-3 | Off-chain eligibility list manipulated (sybil wallets, snapshot sniping, PDA exclusions) | On-chain tickets `floor(stake/threshold)`, time-weighted, min holding period, frozen `ticket_root` before VRF (T-LOT-01..03) |
| S-4 | Payout loop re-entrancy / double pay / unbounded loops | Pull-based one-shot claims (flag set before transfer), bounded paginated cranks (T-DIST-05, T-LOT-06) |
| S-5 | Liquidity drained by fake "reward" or phishing claim sites | 🔍 Official domain list, no signature-only claims, claim UI only calls our pinned program IDs |

## Threats, defenses, tests

### Token and authority layer

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-TOK-01 | Extra minting after launch (supply ≠ 1B) | Mint the full 1B once, then `SetAuthority(MintTokens → None)`. `assert_launch_ready` checks `mint_authority == None` (R-03) | 🧪 launch script test: supply == 1e9×10^d and mint_authority None; `open_sale` fails otherwise |
| T-TOK-02 | Freeze authority used to trap holders | `freeze_authority = None` | 🧪 same |
| T-TOK-03 | Dangerous Token-2022 extensions (PermanentDelegate, DefaultAccountState=Frozen, NonTransferable, TransferHook) | Extension allowlist {TransferFeeConfig, MetadataPointer, TokenMetadata} checked on-chain (R-03.1). Phantom warns on permanent delegate | 🧪 mint with each forbidden extension → `open_sale` fails |
| T-TOK-04 | Tax raised to 100% by the fee-config authority | `transfer_fee_config_authority = None`, or a PDA with `bps ≤ cap`, `maximum_fee ≤ cap`, multisig and timelock ≥ 2 epochs + 72h (R-04.2). Token-2022 itself delays changes by 2 epochs | 🧪 `update_fee` above cap fails |
| T-TOK-05 | Withheld fees withdrawn by a person | `withdraw_withheld_authority` = `fee_treasury` `vault_authority` PDA. The only signer path withdraws into the fixed vault (R-04.1) | ✅ vault PDA derivation/seeds checked (`spoofed vault PDA rejected`). 🧪 withdraw to other destination fails |
| T-TOK-06 | Legacy/fake token program or non-Token-2022 mint passed to treasury/lottery | `owner = TOKEN_2022_PROGRAM_ID` constraint on mint. Pin all program IDs with `address =` (R-05.1) | ✅ `legacy-Token-owned mint rejected (MintNotToken2022)` in both programs |

### Program/account-validation layer (both custom programs)

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-ACC-01 | Re-initialization overwrites admin/config | Anchor `init` on PDA (fails if exists). No `init_if_needed` | ✅ `reinitialize fails, admin not overwritten` (both programs) |
| T-ACC-02 | Spoofed PDA (attacker-owned "vault") | `seeds` + `bump` constraints, bump stored at init | ✅ `spoofed vault PDA rejected (ConstraintSeeds)` (both) |
| T-ACC-03 | Invalid parameters (zero caps, absurd windows) | Validation in `initialize` (ZeroPurchaseCap, PurchaseCapExceedsWindowCap, InvalidSpendWindow, ZeroTicketThreshold, InvalidHoldingPeriod) | ✅ `invalid params rejected` (both) |
| T-ACC-04 | Malicious program substituted in CPI (fake Token-2022, fake MPL-Hybrid, fake VRF) | `Program<>`/`address =` pinning for every external program (R-05.1, R-15) | 🧪 substitute each program → fails |
| T-ACC-05 | Type cosplay / owner confusion | Anchor discriminators, `Account<>` owner checks, VRF account owner pinned | 🧪 foreign-owned account with the right bytes → fails |
| T-ACC-06 | Integer overflow/rounding abuse | `checked_*` math everywhere, `overflow-checks = true` in release profile, tickets via `checked_div` | ✅ math unit tests. 🧪 fuzz |
| T-ACC-07 | Upgrade authority rug | Upgrade authority = Squads multisig, then final. Checked by `assert_launch_ready` (R-03) | 🧪 / 🔍 |

### Distribution layer: `fee_treasury` (heaviest scrutiny)

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-DIST-01 | Admin or compromised key drains the vault | **No admin withdraw instruction exists.** Outflows only via `buy_prize` to pinned routes, with the NFT delivered straight to `holder_lottery` `prize_vault` | 🧪 no instruction moves vault funds to an arbitrary account (IDL review + negative tests) |
| T-DIST-02 | Caller-chosen destination on harvest/buy | Destinations derived (PDA/ATA) or stored, never arguments (R-05.3) | 🧪 passing a different destination ATA → fails |
| T-DIST-03 | Bot drains via repeated small buys / flash spending | `max_spend_per_purchase`, `max_spend_per_window` with rolling window (stored in config), pause | ✅ cap validation at init. 🧪 spend over window cap fails. 🧪 window roll-over |
| T-DIST-04 | Sandwich/price manipulation on prize buys or swaps | Default route is fixed-rate MPL-Hybrid capture (no price). Marketplace route needs an on-chain max price ≤ swap_rate × (1+10%). Any swap has TWAP `min_out` (R-07, R-11) | 🧪 buy above cap fails. 🧪 min_out violation reverts |
| T-DIST-05 | Harvest crank griefing / unbounded account list | Harvest is permissionless and paginated with bounded batch size. Anyone can harvest (the fee only moves to the mint and vault) | 🧪 CU bound at max batch. 🧪 garbage accounts ignored |
| T-DIST-06 | Cross-mint confusion (vault of mint A pays for mint B) | Config PDA seeded by `fee_mint`. Vault authority seeded by config | ✅ PDA derivation tests. 🧪 cross-config signing fails |
| T-DIST-07 | Treasury buys a fake/wrong NFT (worthless prize) | Allowlisted collections (Core collection address pinned per config). Verify `update_authority == Collection(x)` | 🧪 NFT from another collection → fails |

### Distribution layer: `holder_lottery` (heaviest scrutiny)

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-LOT-01 | Sybil splitting to gain tickets | `tickets = floor(balance/threshold)`, linear, remainders discarded (R-09) | ✅ `sybil split never gains` property test in `math.rs` |
| T-LOT-02 | Snapshot sniping (buy right before the snapshot, sell right after) | Time-weighted stake/registration, `min_holding_seconds` (24h..90d, validated), no point-in-time balances (R-08) | ✅ holding-period bounds validated at init. 🧪 2-slot staker gets ≈0 tickets |
| T-LOT-03 | Ticket set changed after randomness is known | `ticket_root`/`total_tickets` frozen before request, seed includes root, one-way state machine (R-02) | 🧪 transitions backward fail. 🧪 second request fails |
| T-LOT-04 | Predictable randomness (SlotHashes, Clock, blockhash) | VRF only (Switchboard On-Demand or ORAO). Never slot/clock-derived | 🧪 grep/CI lint for `SlotHashes`/`Clock` in selection code. 🧪 foreign VRF account fails |
| T-LOT-05 | Selective reveal / reroll (Switchboard commit–reveal withheld if unfavourable, or CPI-and-revert "loss") | Pin VRF account at commit (`seed_slot == slot−1`), permissionless settle, no cancel after fulfill, deadline + roll-forward (R-02, R-14). With ORAO the nodes fulfil, so no requester reveal step | 🧪 revert-on-loss wrapper can't change recorded winner. 🧪 cancel after fulfill fails |
| T-LOT-06 | Double claim / re-entrancy on prize payout | Pull-based `Claim` PDA, flag set before transfer, prize moved to claim PDA at settle (R-05.3) | 🧪 double claim fails |
| T-LOT-07 | Modulo bias in winner selection | Rejection sampling (R-01.4) | 🧪 chi-square over 1M picks |
| T-LOT-08 | Compute DoS with many holders / dust accounts | O(log N) prefix-sum/merkle winner lookup, minimum stake ≥ threshold (R-13) | 🧪 CU benchmark at 1M holders |
| T-LOT-09 | Admin changes draw timing or params mid-round | Draw params fixed at creation. Admin changes are timelocked and only affect future rounds (R-02.4) | 🧪 change during round fails |

### Hybrid layer (MPL-Hybrid configuration)

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-HY-01 | **Escrow authority drain:** authority captures NFTs cheaply (`update_escrow`/`update_recipe` sets low `amount`), raises `amount`, releases to drain backing. There's no timelock in MPL-Hybrid (`update_escrow.rs:113`, `update_recipe.rs:115`) | Per-collection Squads multisig authority with timelock. Later a governor PDA with bounded, timelocked changes. 🔍 alert on any update ix | 🧪 localnet: document the attack as a regression test against our governor |
| T-HY-02 | Shared V2 escrow across collections (`["escrow", authority]`) | One dedicated authority per collection | 🧪 launch script asserts uniqueness |
| T-HY-03 | Escrow insolvency (not enough backing tokens for releases) | Invariant `escrow_balance ≥ R × NFTs outside escrow`. No Burn* paths. Classic SPL mint (no fee leak) | 🧪 capture/release loop keeps invariant. 🔍 monitor |
| T-HY-04 | Rarity cherry-pick / reroll gaming (caller-chosen asset `capture_v2.rs:52-54`, SlotHashes reroll `:95,190-199`) | `NoRerollMetadata`, value-uniform traits or disclosed (Auditor B B-01/B-12) | 🧪 config check |
| T-HY-05 | Transfer tax breaks exact unwrap | Hybrid mints are classic SPL (no tax). MPL-Hybrid rejects Token-2022 anyway (ADR-004, transfer-tax-vs-wrap.md) | 🧪 launch script refuses T22 mint for hybrid |
| T-HY-06 | MPL-Hybrid is unaudited ("Audit Pending") | Include it in the audit scope, pin the program version, 🔍 watch upgrades of `MPL4o4…` | 🔍 |

### Launch / market layer (summary; owned jointly with app)

| ID | Threat | Defense |
|---|---|---|
| T-MKT-01 | Snipers/bundlers at launch | Commit window + uniform clearing price or per-wallet/per-slot caps. Open slot stored at init (R-10) |
| T-MKT-02 | Sandwiching user swaps | `min_out`/`max_in` on every value-moving ix (R-11) |
| T-MKT-03 | DEX incompatibility traps liquidity | Only venues that support TransferFee mints (Raydium CPMM/CLMM, Meteora DAMM v2/DLMM, Orca V2). No transfer hook |

### Operational

| ID | Threat | Defense |
|---|---|---|
| T-OPS-01 | Key leakage from repo | `.keys/` gitignored, devnet-only throwaway keys, `*keypair*.json` ignored. No mainnet keys on the box |
| T-OPS-02 | Toolchain/dependency supply chain | Pinned Anchor 1.2.0 / Solana 4.1.2 / `Cargo.lock` committed, `--locked` tests (R-15) |
| T-OPS-03 | Shipping unaudited code | Professional third-party audit before mainnet (BRIEF #7). Mainnet deploy is out of scope for this workspace |

## Explicit non-goals / accepted risks (for now)

- Trusting Metaplex (Core, MPL-Hybrid), the Token-2022 program, and the chosen VRF network's liveness and honesty.
- MPL-Hybrid's own code isn't audited. The accepted interim state is devnet only.
