# On-chain threat model (engineering view)

Owner: Solana Program Engineer. Status: v0.2 (2026-09-24, after Barton's SCOPE CHANGE, ADR-009). It complements, and
doesn't replace, the independent auditor models in `../security/auditor-a/00-threat-model.md` and
`../security/auditor-b/threat-model.md` (B-xx findings, R-xx requirements in `design-requirements.md`). Where a row maps
to an auditor requirement, it's cited.

> **Scope (ADR-009):** the product is **Track A only**: SPL-404 hybrid launches on a classic SPL Token mint.
> Token-2022, the transfer tax, the tax treasury and the holder lottery (**Track B**) are **SHELVED/DEFERRED**. Their
> threat rows are kept at the end of this file, marked SHELVED, so the work can resume later.

Test status legend:
- ✅ = implemented and passing in this repo today (test name given)
- 🧪 = planned test, specified here
- 🔍 = monitoring/process control
- 💤 = SHELVED with Track B

## Assets and trust boundaries

- **Pooled value (highest risk):** the hybrid engine's vault of backing tokens (`ratio × NFTs outside the vault`), the
  NFTs held in the pool, and escrowed request payments/fees (engine TBD, ADR-008). No fee pile exists: re-roll fees are
  burned (ADR-009).
- **Authorities:** mint (revoked in `hybrid_launch::launch`), freeze (never set), hybrid collection/vault authority
  (PDA), program upgrade authority (Squads multisig → frozen), the single admin power we may keep (pause new
  captures/re-rolls, see [admin-multisig-timelock.md](admin-multisig-timelock.md)).
- **Trust boundary:** everything the caller passes in is attacker-controlled: accounts, amounts, VRF accounts, and
  "program" accounts. Only PDAs we derive, program IDs we pin, and state we stored are trusted.

## Incident to design against: Stonk.fun (sourced)

**Source:** Bitquery Research, "Is StonkFun dumping on its own holders?", on-chain figures verified 2026-09-22,
https://bitquery.io/investigations/is-stonkfun-dumping-on-holders (provided by Barton; summary in
[stonkfun-lessons.md](stonkfun-lessons.md); I re-read the article on 2026-09-24). This replaces my earlier section,
which had no source and speculated about a "bot drain". That speculation (old S-1..S-5 table) is withdrawn.

**What happened, per Bitquery (23 Aug – 22 Sep 2026):**
- **Not an exploit.** The viral "STONK FEE DRAIN" wallet is Stonk.fun's own reward wallet (its API names it). Reward
  coins carry a 1% or 3% transfer tax, withheld in the recipient's account. That one wallet sweeps it, sells it into
  each coin's own pool, and pays holders in the pair asset: **$56.3M sold vs $56.2M paid out** in 30 days. The coins
  can't mint, so the "infinite supply" was just tax from tens of thousands of coins.
- **The design is the harm.** The tax sells into every chart. **160 coins lost more than half their supply** to tax
  (BUDDY 69%, ZCAT 61%, RAYCAT 91%). The median coin lost 2.5%.
- **The launch window is worst.** Snipers flip in the first seconds and every flip pays tax. 522 wallets traded 3.4× a
  coin's supply within a minute of one launch. **ZCAT lost 20% of supply in its first hour** (the 17.5% sold as tax
  fetched ~$18K and was worth ~$30M at peak).
- **Trust failures:**
  - **At least $1.41M** of reward money went in single transfers to Stonk.fun-linked wallets (STONK's creator, a
    treasury, a 1,000 SOL transfer), left out of its public reward ledger.
  - On **2,178 LaunchLab coins** the reward wallet **can still raise the tax to 100%** after a delay of a few days
    (never used). Older coins are locked for good.
  - Claimed buybacks were **$2.09M; on-chain it was $1.23M** (58% of that day's revenue).

**What we take from it (constraints, ADR-009 §C):**

| # | Stonk.fun failure | Our constraint | Where enforced |
|---|---|---|---|
| S-1 | Transfer tax sells into every chart | **No transfer tax.** Classic SPL Token only; Token-2022 is rejected at launch | ✅ `hybrid_launch::launch` uses `Program<Token>`; `attack_token_2022_program_substituted_is_rejected` |
| S-2 | A discretionary operator wallet sweeps, sells and routes value (incl. $1.41M off-ledger) | **No operator wallet custodies value.** Every flow is program-enforced; fees are burned, never swept | ✅ `fee_destination` must be BURN (`attack_fee_destination_other_than_burn_is_rejected`). 🧪 engine: no instruction takes a destination argument |
| S-3 | Retained power to raise the tax to 100% | **No mutable economics after launch.** Ratio, fees, fee destination and mint fixed at init; any remaining admin power is multisig + timelock and can only reduce risk | ✅ `LaunchConfig` has no update/close ix (`config_has_no_mutation_or_close_instruction_in_idl`). 🧪 engine config the same |
| S-4 | Ledger claims didn't match the chain | Fee destinations on-chain and equal to site copy; burns are visible as supply reduction; publish a reconciliation script | 🧪 INV-16 (QA). 🔍 public ledger |
| S-5 | Snipers dominate the first minutes | **Anti-sniping for the bonding-curve launch window** (open design item) | 🧪 once the curve venue is chosen; see admin-multisig-timelock.md §Anti-sniping |

## Threats, defenses, tests

### Token and authority layer (Track A)

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-TOK-01 | Extra minting after launch (supply ≠ 1B) | `hybrid_launch::launch` mints exactly 1B × 10^d once, then `SetAuthority(MintTokens → None)` in the same instruction, then re-reads the mint and fails unless supply, decimals and both authorities are right | ✅ `launch_mints_exactly_1b_revokes_mint_and_freeze_authority_and_records_immutable_config` |
| T-TOK-02 | Freeze authority used to trap holders | `initialize_mint2` with `freeze_authority = None` | ✅ same test |
| T-TOK-03 | Token-2022 mint (tax, PermanentDelegate, hooks…) sneaks into a hybrid launch | `token_program: Program<Token>` (classic only); the mint is created by the program, never onboarded | ✅ `attack_token_2022_program_substituted_is_rejected`, `attack_existing_token_2022_mint_cannot_be_onboarded` |
| T-TOK-04 | Supply accounting after burns | Supply starts at exactly 1B and only decreases (no mint authority). Invariant `circulating + vault + Σburned == 1B` | 🧪 engine: INV-02 (QA) |

### Launch program: `hybrid_launch` (built, not audited)

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-HL-01 | Existing mint onboarded (pre-minted supply, retained authorities, foreign program); pre-funded mint address griefs the launch (QA-HL-01) | The mint must be a fresh keypair that signs; an address with data or a non-system owner is rejected (`MintAccountInUse`); a pre-funded empty system account is topped up + allocated + assigned | ✅ `attack_existing_classic_mint_cannot_be_onboarded`, `attack_existing_token_2022_mint_cannot_be_onboarded`, `attack_mint_keypair_not_signing_is_rejected`, `attack_mint_address_with_data_or_program_owner_is_rejected`, `prefunded_mint_address_does_not_block_launch`, QA `qa_finding_prefunded_mint_address_should_not_block_launch` |
| T-HL-02 | Ratio outside the allowed set | `ALLOWED_RATIOS = {10k, 50k, 100k, 200k, 500k, 1M, 2.5M, 5M}`; `collection_size ≥ 100` | ✅ `attack_ratio_outside_allowed_set_is_rejected`, `attack_collection_below_minimum_100_is_rejected`, `every_ratio_launches_at_min_100_and_at_max_1b_over_ratio` |
| T-HL-03 | Collection bigger than supply / u64 wrap | `collection_size × ratio_base ≤ supply_base` via `checked_mul`; overflow rejects | ✅ `attack_collection_size_times_ratio_above_1b_is_rejected`, `attack_collection_size_overflow_is_rejected`, unit `attack_collection_size_overflow_is_rejected_not_wrapped`, `supply_table_max_collection_size_per_ratio` |
| T-HL-04 | Decimals overflow the 1B × 10^d math | `decimals ≤ 9` (1B × 10^9 < u64::MAX) | ✅ `attack_decimals_above_9_are_rejected` |
| T-HL-05 | Excessive fees / unwrap-rewrap cheaper than re-roll | `fee_bps ≤ 1000` (10% of ratio); `capture_fee_bps ≥ reroll_fee_bps` | ✅ `attack_fee_above_cap_is_rejected`, `attack_capture_fee_below_reroll_fee_is_rejected` |
| T-HL-06 | Fee routed to a wallet (Stonk.fun S-2) | `fee_destination` must equal BURN; no destination account is stored | ✅ `attack_fee_destination_other_than_burn_is_rejected` |
| T-HL-07 | Re-launch overwrites config | `init` on `["launch_config", mint]`; mint address can't be recreated | ✅ `attack_relaunch_of_same_mint_is_rejected` |
| T-HL-08 | Spoofed PDAs (attacker keeps mint authority / fake config) | `seeds` + `bump` on both PDAs | ✅ `attack_spoofed_mint_authority_pda_is_rejected`, `attack_spoofed_launch_config_pda_is_rejected` |
| T-HL-09 | Supply minted to an attacker-chosen account (QA-HL-02) | `launch_destination` must equal the classic ATA of the launch-vault PDA (seeds checked) | ✅ `attack_launch_destination_not_the_derived_ata_is_rejected`, `attack_caller_chosen_launch_vault_owner_is_rejected` |
| T-HL-10 | Config changed later | No update/close instruction exists | ✅ `config_has_no_mutation_or_close_instruction_in_idl` |
| T-HL-11 | Launch destination owner dumps the supply (Stonk.fun S-2, QA-HL-02) | Owner is a program PDA with no signing instruction; no person can withdraw. The future curve/distribution path arrives only via multisig + timelock upgrade | ✅ `supply_sits_in_launch_vault_pda_with_no_withdraw_instruction` |

### Burn path (re-roll fee = BURN, ADR-009)

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-BURN-01 | Burn reduces backing → vault insolvent, unwrap not exact | Fees are paid from the user's own balance **in addition to** the ratio. Re-roll moves no backing; capture moves exactly `ratio` into the vault plus a separate fee that is burned. Nothing ever burns from the vault | 🧪 engine: vault balance unchanged by re-roll; `vault ≥ ratio × NFTs outside` after every step (fuzz) |
| T-BURN-02 | Fee "burn" actually lands in a wallet (MPL-Hybrid `fee_location`) | hybrid_vault: inline `token::burn`. MPL-Hybrid: `fee_location` must be a program PDA with only a permissionless burn crank and no withdraw path. MPL-Hybrid's `BurnOnCapture`/`BurnOnRelease` paths are **forbidden** (they burn backing) | 🧪 per engine |
| T-BURN-03 | Copy promises "1B" after burns | Copy: "Fixed at 1,000,000,000 at launch. No one can mint more; re-roll burns can only reduce it." Site shows live supply and burned total | 🧪 INV-16 (QA) |

### Program/account-validation layer (all custom programs)

> ✅ rows below that cite "both programs" refer to the shelved `fee_treasury`/`holder_lottery` tests (💤). The
> `hybrid_launch` equivalents are T-HL-07/08.

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-ACC-01 | Re-initialization overwrites admin/config | Anchor `init` on PDA (fails if exists). No `init_if_needed` | ✅ `reinitialize fails, admin not overwritten` (both programs) |
| T-ACC-02 | Spoofed PDA (attacker-owned "vault") | `seeds` + `bump` constraints, bump stored at init | ✅ `spoofed vault PDA rejected (ConstraintSeeds)` (both) |
| T-ACC-03 | Invalid parameters (zero caps, absurd windows) | Validation in `initialize` (ZeroPurchaseCap, PurchaseCapExceedsWindowCap, InvalidSpendWindow, ZeroTicketThreshold, InvalidHoldingPeriod) | ✅ `invalid params rejected` (both) |
| T-ACC-04 | Malicious program substituted in CPI (fake Token-2022, fake MPL-Hybrid, fake VRF) | `Program<>`/`address =` pinning for every external program (R-05.1, R-15) | 🧪 substitute each program → fails |
| T-ACC-05 | Type cosplay / owner confusion | Anchor discriminators, `Account<>` owner checks, VRF account owner pinned | 🧪 foreign-owned account with the right bytes → fails |
| T-ACC-06 | Integer overflow/rounding abuse | `checked_*` math everywhere, `overflow-checks = true` in release profile, tickets via `checked_div` | ✅ math unit tests. 🧪 fuzz |
| T-ACC-07 | Upgrade authority rug | Upgrade authority = Squads multisig, then final. Checked by `assert_launch_ready` (R-03) | 🧪 / 🔍 |

### Hybrid layer (MPL-Hybrid configuration)

> **Status:** these rows apply only if MPL-Hybrid is used as the swap engine. ADR-008 (proposed) replaces it with
> `hybrid_vault` for cosmetic-rarity collections (see the T-HV section below and
> [hybrid-rarity-and-assignment.md](hybrid-rarity-and-assignment.md)). They're kept because they explain why.

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-HY-01 | **Escrow authority drain:** authority captures NFTs cheaply (`update_escrow`/`update_recipe` sets low `amount`), raises `amount`, releases to drain backing. There's no timelock in MPL-Hybrid (`update_escrow.rs:113`, `update_recipe.rs:115`). `update_recipe` also **unconditionally overwrites `recipe.token` and `fee_location`** with whatever is passed (`update_recipe.rs:91-93`), so the authority can swap in a worthless mint, capture everything, then swap back and release | Per-collection Squads multisig authority with timelock. Later a governor PDA with bounded, timelocked changes. 🔍 alert on any update ix | 🧪 localnet: document the attack as a regression test against our governor |
| T-HY-02 | Shared V2 escrow across collections (`["escrow", authority]`) | One dedicated authority per collection | 🧪 launch script asserts uniqueness |
| T-HY-03 | Escrow insolvency (not enough backing tokens for releases) | Invariant `escrow_balance ≥ R × NFTs outside escrow`. No Burn* paths (they burn backing; T-BURN-02). Classic SPL mint (no fee leak) | 🧪 capture/release loop keeps invariant. 🔍 monitor |
| T-HY-04 | Rarity cherry-pick / reroll gaming (caller-chosen asset `capture_v2.rs:52-54`, SlotHashes reroll `:187-203`) | **No MPL-Hybrid configuration prevents this** (hybrid-rarity-and-assignment.md §3.2), so it's replaced by `hybrid_vault` (T-HV-01..03) | 🧪 regression test documenting the MPL-Hybrid behaviour |
| T-HY-05 | Transfer tax breaks exact unwrap | Classic SPL only (ADR-009; Token-2022 dropped). MPL-Hybrid rejects Token-2022 anyway | ✅ `attack_token_2022_program_substituted_is_rejected` |
| T-HY-06 | MPL-Hybrid is unaudited ("Audit Pending") | Include it in the audit scope, pin the program version, 🔍 watch upgrades of `MPL4o4…` | 🔍 |

### Hybrid vault layer: `hybrid_vault` (cosmetic rarity, ADR-008, planned)

Design: [hybrid-rarity-and-assignment.md](hybrid-rarity-and-assignment.md). This layer holds pooled backing tokens and
NFTs, so it gets distribution-layer scrutiny.

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-HV-01 | **Cherry-picking** a known rare from the pool | The caller never names the asset. Selection is `uniform_below(vrf, pool_len)` at settle | 🧪 `request_capture` has no asset argument. 🧪 settle with a caller-supplied asset ≠ selected index fails |
| T-HV-02 | **Revert-until-rare** (CPI wrapper or simulation that aborts on a bad pick) | Two-step: payment locked in `request_*`, selection in a separate permissionless `settle`. No cancel once VRF is fulfilled | 🧪 a wrapper program that CPIs request and then reverts can't observe the pick. 🧪 cancel/expire after fulfilment fails |
| T-HV-03 | **Pool shifting after randomness is known** (release/deposit NFTs between VRF reveal and settle so `r mod len` lands on a rare) | Sequenced pool: FIFO settlement, and only deposits with `seq < request.seq` are merged before settle | 🧪 deposits after request `s` never appear in `s`'s candidate set. 🧪 settling out of order fails |
| T-HV-04 | Selective reveal / withholding (Switchboard requester sees the value first) | Pinned VRF account per request, permissionless reveal + settle cranked by us. `expire` only if the VRF is *unfulfilled* after `deadline_slot` | 🧪 expire on a fulfilled request fails |
| T-HV-05 | Predictable or biased randomness | VRF only (no SlotHashes/Clock/count). Rejection sampling (R-01.4). Pinned VRF program owner + seed check | 🧪 chi-square over 1M picks. 🧪 foreign/unfulfilled VRF account fails |
| T-HV-06 | **Creator steers traits** (arranges the list after seeing randomness, or later edits metadata) | `trait_root` + `leaf_count` committed before the permutation seed is requested. Feistel permutation keyed by VRF. Merkle proof checked at mint. PDA collection authority with no update/add-plugin instruction | 🧪 settle with the wrong leaf/proof fails. 🧪 seed request before root commit fails. 🧪 IDL has no metadata-update ix. Public verify script |
| T-HV-07 | Hostile Core plugins (Permanent Transfer/Burn/Freeze delegate, or a Royalties ruleset that blocks vault transfers) | `hybrid_vault` CPI-creates the collection with no Permanent delegates. Only Royalties with ruleset `None` | 🧪 initialize with an existing collection is rejected. 🧪 plugin census check |
| T-HV-08 | Escrow insolvency / accounting drift | Invariants: `vault_tokens ≥ ratio × nfts_outside`, and the NFT conservation equation (doc §3.3). Exact `ratio` amounts. Classic SPL Token (no transfer fee) | 🧪 fuzz random instruction sequences (≥1M) and assert the invariants after each |
| T-HV-09 | Supply/ratio misconfiguration or overflow | `initialize`: ratio in the allowed set, `checked_mul`, `collection_size × ratio_base ≤ supply_base`, mint supply == 1B and authorities `None`. Ratio/mint immutable | 🧪 every ratio at max and max+1. 🧪 decimals overflow fails cleanly |
| T-HV-10 | Admin drain via config (the MPL-Hybrid T-HY-01 class) | No instruction changes `ratio`, `mint`, `trait_root`, fees, fee destination (burn) or the vault token account (ADR-009: economics fixed at init). No admin withdraw. Fees capped at init (≤ 10% of R, ≤ 0.05 SOL SOL-cost fee) | 🧪 IDL review. ✅ fee over cap fails at launch (`attack_fee_above_cap_is_rejected`) |
| T-HV-11 | Fee changed under a pending request | Moot under ADR-009 (fees immutable); requests still record the fee charged for auditability | 🧪 settle charges exactly the recorded fee |
| T-HV-12 | Reroll farming / unwrap→rewrap bypass | Token fee as bps of R (scale-invariant). `capture_fee_bps ≥ reroll_fee_bps` enforced. Published pool census | 🧪 config with capture fee < reroll fee is rejected. 🔍 monitor rare outflow vs expected rate |
| T-HV-13 | Reroll returns the same NFT | Hand-in is tagged with the request's own `seq`, so it's excluded from its own candidate set | 🧪 pool of {own NFT, X} always yields X |
| T-HV-14 | Empty pool at settle / reservation exhaustion | Reservation check at request (`pool + incoming − pending ≥ 1`) | 🧪 request with no unreserved NFTs fails. 🧪 settle never hits an empty pool (fuzz) |
| T-HV-15 | Head-of-line DoS (stuck VRF blocks the FIFO) | `deadline_slot` + permissionless `expire` of unfulfilled heads. Requests lock capital + fees | 🧪 stuck head expires and the queue advances |
| T-HV-16 | Lazy-mint rent griefing (settle payer drained) | Worst-case asset rent is collected in the request's SOL fee. Settle pays from the request's escrowed lamports, not the cranker | 🧪 settle with a zero-balance cranker succeeds |
| T-HV-17 | Double settle / double release | Request account closed on settle. Asset ownership checked on release | 🧪 second settle fails. 🧪 releasing an asset not owned fails |

### Launch / market layer (summary; owned jointly with app)

| ID | Threat | Defense |
|---|---|---|
| T-MKT-01 | Snipers/bundlers in the bonding-curve launch window (Stonk.fun S-5) | **Open design item** (curve venue undecided): per-wallet caps, launch delay after pool creation, commit-reveal/batch buys, decaying fee in the first N slots. See [admin-multisig-timelock.md](admin-multisig-timelock.md) §Anti-sniping |
| T-MKT-02 | Sandwiching user swaps | `min_out`/`max_in` on every value-moving ix (R-11) |
| T-MKT-03 | Launch destination holder dumps supply | Supply goes to the curve vault PDA, not a person (open question) |

### Operational

| ID | Threat | Defense |
|---|---|---|
| T-OPS-01 | Key leakage from repo | `.keys/` gitignored, devnet-only throwaway keys, `*keypair*.json` ignored. No mainnet keys on the box |
| T-OPS-02 | Toolchain/dependency supply chain | Pinned Anchor 1.2.0 / Solana 4.1.2 / `Cargo.lock` committed, `--locked` tests (R-15) |
| T-OPS-03 | Shipping unaudited code | Professional third-party audit before mainnet (BRIEF #7). Mainnet deploy is out of scope for this workspace |

## Explicit non-goals / accepted risks (for now)

- Trusting Metaplex Core, the classic SPL Token program, and the chosen VRF network's liveness and honesty.
- The engine is undecided (ADR-008). If MPL-Hybrid is chosen, its unaudited code and T-HY-* rows are in scope.

---

## 💤 SHELVED: Track B threat rows (Token-2022 tax / treasury / lottery)

**SHELVED/DEFERRED per ADR-009 (Barton, 2026-09-24 2:51 PM MT).** Kept for when Track B returns; not maintained.
Code: branch `shelved/track-b-t22` and `shelved/programs/`. Note the Stonk.fun findings argue against bringing back a
transfer tax in this form at all.

### 💤 Token-2022 authority rows (old T-TOK-03..06)

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-TOK-01 | Extra minting after launch (supply ≠ 1B) | Mint the full 1B once, then `SetAuthority(MintTokens → None)`. `assert_launch_ready` checks `mint_authority == None` (R-03) | 🧪 launch script test: supply == 1e9×10^d and mint_authority None; `open_sale` fails otherwise |
| T-TOK-02 | Freeze authority used to trap holders | `freeze_authority = None` | 🧪 same |
| T-TOK-03 | Dangerous Token-2022 extensions (PermanentDelegate, DefaultAccountState=Frozen, NonTransferable, TransferHook) | Extension allowlist {TransferFeeConfig, MetadataPointer, TokenMetadata} checked on-chain (R-03.1). Phantom warns on permanent delegate | 🧪 mint with each forbidden extension → `open_sale` fails |
| T-TOK-04 | Tax raised to 100% by the fee-config authority | `transfer_fee_config_authority = None`, or a PDA with `bps ≤ cap`, `maximum_fee ≤ cap`, multisig and timelock ≥ 2 epochs + 72h (R-04.2). Token-2022 itself delays changes by 2 epochs | 🧪 `update_fee` above cap fails |
| T-TOK-05 | Withheld fees withdrawn by a person | `withdraw_withheld_authority` = `fee_treasury` `vault_authority` PDA. The only signer path withdraws into the fixed vault (R-04.1) | ✅ vault PDA derivation/seeds checked (`spoofed vault PDA rejected`). 🧪 withdraw to other destination fails |
| T-TOK-06 | Legacy/fake token program or non-Token-2022 mint passed to treasury/lottery | `owner = TOKEN_2022_PROGRAM_ID` constraint on mint. Pin all program IDs with `address =` (R-05.1) | ✅ `legacy-Token-owned mint rejected (MintNotToken2022)` in both programs |

### 💤 Distribution layer: `fee_treasury`

| ID | Threat | Defense | Test |
|---|---|---|---|
| T-DIST-01 | Admin or compromised key drains the vault | **No admin withdraw instruction exists.** Outflows only via `buy_prize` to pinned routes, with the NFT delivered straight to `holder_lottery` `prize_vault` | 🧪 no instruction moves vault funds to an arbitrary account (IDL review + negative tests) |
| T-DIST-02 | Caller-chosen destination on harvest/buy | Destinations derived (PDA/ATA) or stored, never arguments (R-05.3) | 🧪 passing a different destination ATA → fails |
| T-DIST-03 | Bot drains via repeated small buys / flash spending | `max_spend_per_purchase`, `max_spend_per_window` with rolling window (stored in config), pause | ✅ cap validation at init. 🧪 spend over window cap fails. 🧪 window roll-over |
| T-DIST-04 | Sandwich/price manipulation on prize buys or swaps | Hybrid route is a fixed-ratio `hybrid_vault` capture (no price on the NFT leg), but the Rewards-token → Hybrid-token swap needs TWAP `min_out`. Marketplace route needs an on-chain max price ≤ swap_rate × (1+10%). Any swap has TWAP `min_out` (R-07, R-11) | 🧪 buy above cap fails. 🧪 min_out violation reverts |
| T-DIST-05 | Harvest crank griefing / unbounded account list | Harvest is permissionless and paginated with bounded batch size. Anyone can harvest (the fee only moves to the mint and vault) | 🧪 CU bound at max batch. 🧪 garbage accounts ignored |
| T-DIST-06 | Cross-mint confusion (vault of mint A pays for mint B) | Config PDA seeded by `fee_mint`. Vault authority seeded by config | ✅ PDA derivation tests. 🧪 cross-config signing fails |
| T-DIST-07 | Treasury buys a fake/wrong NFT (worthless prize) | Allowlisted collections (Core collection address pinned per config). Verify `update_authority == Collection(x)` | 🧪 NFT from another collection → fails |

### 💤 Distribution layer: `holder_lottery`

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

