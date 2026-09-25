# Architecture decision records (on-chain)

Owner: Solana Program Engineer. Format: context → decision → consequences. Status values: Proposed / Accepted /
Superseded / Shelved. Every decision here is **Proposed** until Barton signs off, except where marked **Accepted (decided
by Barton)**.

> **Current scope (ADR-009, 2026-09-24 2:51 PM MT):** SPL-404 hybrid launches only (Track A). Token-2022, transfer tax,
> tax treasury and holder lottery (Track B) are **SHELVED/DEFERRED**. ADR-004 is superseded; ADR-005 and ADR-006 are
> shelved. New: ADR-009 (scope change + Stonk.fun constraints + burn), ADR-010 (`hybrid_launch`).

---

## ADR-001: Toolchain: Anchor 1.2.0 + Agave/Solana 4.1.2, pinned

- **Context.** Anchor 1.2.0 (released 2026-09-04) is current, and its release notes recommend Solana 4.1.2. The Anza
  stable channel is 4.2.2 (Agave 4.3.0 is tagged on GitHub but not yet on the stable installer channel). If Anchor.toml
  doesn't pin `solana_version`, Anchor 1.2 infers a version from `Cargo.lock` and tries to switch the global toolchain
  on the shared box.
- **Decision.** Pin `[toolchain] anchor_version = "1.2.0"`, `solana_version = "4.1.2"`, Rust `1.89.0` via
  `rust-toolchain.toml`, and commit `Cargo.lock`. Tests run with `--locked`.
- **Consequences.** Reproducible builds. Upgrading is a deliberate PR that changes all three pins together.

## ADR-002: Build SBPF v2 (not Anchor 1.2's default v3) while LiteSVM tests use litesvm 0.10

- **Context.** Anchor 1.2 builds SBPF v3 by default. Anchor's own litesvm test template pins `litesvm 0.10.0`, which
  can't load v3 ELFs (`add_program` → `InvalidAccountData`), and this reproduces on the pristine template.
  `litesvm 0.16` needs a newer rustc than 1.89.
- **Decision.** `ANCHOR_BUILD_SBF_ARCH=v2` (set in `scripts/env.sh`).
- **Consequences.** Works with the localnet validator and LiteSVM. Revisit when we bump Rust and litesvm (Q5). Devnet
  and mainnet both accept v2.

## ADR-003: No custom 404 program and no custom transfer-fee program

**Status: Proposed. Partially superseded by ADR-008 (Accepted)** for hybrid swaps. The transfer-fee half is moot
(Token-2022 shelved, ADR-009).

- **Context.** Custom swap/fee code that holds pooled value is the biggest bug and drain surface, and each one needs
  its own audit.
- **Decision.** Hybrid swaps use **Metaplex MPL-Hybrid**. Transfer tax uses the **Token-2022 TransferFee extension**.
  Custom code is limited to `fee_treasury` and `holder_lottery`.
- **Consequences.** We inherit MPL-Hybrid's limits: classic SPL Token only, no timelock on escrow updates, SlotHashes
  rerolls, and **Audit Pending** (repo README at `aacf1a5`). All of those are documented in THREAT_MODEL.md T-HY-*.

## ADR-004: Transfer tax vs exact unwrap: hybrid and taxed are separate launch types

**Status: SUPERSEDED by ADR-009 (Barton, 2026-09-24 2:51 PM MT).** It was Accepted at 2:45 PM MT. The Hybrid half
stands (classic SPL Token, untaxed, convertible, exact unwrap); the Rewards half (Token-2022 taxed, lottery) is
shelved/deferred. Kept for the record.

- **Context.** BRIEF open issue: Token-2022 fees apply to escrow transfers, which breaks "unwrap returns exactly
  1,000,000". Full analysis is in [transfer-tax-vs-wrap.md](transfer-tax-vs-wrap.md).
- **Finding (d).** **MPL-Hybrid doesn't support Token-2022 mints.** It uses `Program<'info, Token>` in every
  instruction (e.g. `capture_v2.rs:106`, `release_v2.rs:105`), legacy `token::transfer` (`capture_v2.rs:284`,
  `release_v2.rs:287`), and `validate_token_account` requires `spl_token::ID` (`utils.rs:36-41`). The Metaplex FAQ
  says it "only supports … SPL Token fungible assets".
- **Decision (option a + g).**
  - **Hybrid** launches use a classic SPL Token mint with **no tax**: 1B fixed supply, mint and freeze authority
    revoked, ratio divides 1B, exact unwrap. The only fees are disclosed per-swap MPL-Hybrid fees.
  - **Rewards** launches use a Token-2022 transfer fee → fee_treasury → NFT prizes → holder_lottery, and have **no
    converter**.
- **Rejected alternatives.**
  - (b) Custom gross-up escrow: it's a custom swap program, and the escrow either pays the unwrap fee (drainable by
    wrap/unwrap cycling) or unwrap isn't exact.
  - (c) Transfer hook allowlist: a hook can't collect a tax (read-only accounts, no signer), and it breaks
    permissionless Meteora pools and needs an Orca badge.
  - (e) Withheld-fee refund: it's a custom program, concentrates the mint-wide withdraw authority, leaks tax, and is
    unverified.
  - (f) SPL Token-Wrap twin: it creates a tax-free twin token that bypasses the tax, and the UX has three assets.
- **Consequences.**
  - The UI copy "Transfer tax on converts: None" is truthful only for Hybrid launches. The approved copy is in
    transfer-tax-vs-wrap.md §User-facing copy.
  - The launch wizard must force a type choice.
  - Rewards treasuries may buy NFTs from Hybrid collections as prizes.

## ADR-005: Fee authority custody

**Status: SHELVED (Track B, ADR-009).**


- **Decision (proposed).**
  - `withdraw_withheld_authority` = `fee_treasury` PDA `["vault_authority", config]`.
  - `transfer_fee_config_authority` = `None` at launch, unless Barton wants adjustable tax (then it's a bounded,
    timelocked PDA, see Q4).
  - Admin = Squads multisig, with no withdraw power.
- **Consequences.** No person can take the tax. Tax rate is immutable by default.

## ADR-006: Lottery fairness primitives

**Status: SHELVED (Track B, ADR-009).** Code on branch `shelved/track-b-t22` and in `shelved/programs/holder_lottery`.


- **Decision.**
  - `tickets = floor(balance / threshold)`, implemented and property-tested.
  - Time-weighted stake/registration with `min_holding_seconds` in 24h..90d (validated at init).
  - VRF only.
  - One-way draw state machine with pinned VRF account, permissionless settle, and pull-based claims.
- **Consequences.** Holders must opt in (register/stake) to earn tickets. That's a UX cost, but it avoids a transfer
  hook (Q2).

## ADR-007: Test strategy

- **Decision (updated for ADR-009).** Rust unit tests for math inside each program (`src/validation.rs`), LiteSVM
  integration tests that load the real SBF binary from `target/deploy`, grouped per track in `tests/track-a-hybrid/`
  (crate `track-a-hybrid-tests`, one `[[test]]` per instruction area, test names state the attack they prove), and
  `./scripts/test.sh` (isolated solana-test-validator on random free ports + unique ledger, then `anchor test
  --skip-local-validator`).
  The Token-2022 localnet smoke test is shelved with Track B.
- **Consequences.** Planned 🧪 tests in THREAT_MODEL.md become the backlog. Fuzzing (Trident or honggfuzz) and CU
  benchmarks come before audit.

## ADR-008: Cosmetic-rarity hybrid collections use a custom `hybrid_vault` (VRF selection), not MPL-Hybrid

**Status: ACCEPTED (2026-09-24). Engine = `hybrid_vault`; MPL-Hybrid is reference only.** Full design:
[hybrid-rarity-and-assignment.md](hybrid-rarity-and-assignment.md).

- **Who decided and why.** The Solana Program Engineer made this call on 2026-09-24. Barton did not answer Q-H1
  directly, but his stated requirements (blind assignment, VRF re-roll, burned fees, no mutable economics, multisig
  plus timelock) cannot be met by any MPL-Hybrid configuration (see Context). **Reversible if Barton objects**: the
  launch step (`hybrid_launch`) is engine-independent, and no mainnet deployment exists.

- **Context.**
  - Barton chose cosmetic rarity: every NFT redeems for exactly `ratio` tokens.
  - Ratios are 10k/50k/100k/200k/500k/1M/2.5M/5M (Barton, 2026-09-24), and collection size is a separate creator
    setting (min 100, max 1B / ratio).
  - Re-rolls cost a fee.
  - Wrapping must be "no choosing, no peeking".
  - Builds on ADR-004 (Accepted): Hybrid launches are classic SPL, untaxed, convertible.
  - MPL-Hybrid source (`aacf1a53`):
    - the capturer names the asset (`capture_v2.rs:52-54`);
    - "reroll" is a metadata rewrite from SlotHashes/timestamp/count (`capture_v2.rs:187-203`), which is predictable
      and CPI-revertible;
    - the authority check is skipped when `NoRerollMetadata` is set (`:183-185`);
    - `update_recipe` can change amount/fees/path with no timelock and overwrites `token`/`fee_location`
      (`update_recipe.rs:91-138`);
    - it's upgradeable by `mp14o4AQ…` and its audit is pending.
  - **No MPL-Hybrid configuration meets the requirement.**
- **Decision (accepted 2026-09-24).**
  1. **Supply check:** `collection_size × ratio ≤ 1B` via `checked_mul` in base units in `hybrid_vault::initialize`,
     mirrored in the frontend. Enforced in `hybrid_launch` (ADR-010). Max sizes are 100,000 / 20,000 / 10,000 /
     5,000 / 2,000 / 1,000 / 400 / 200; minimum 100 for every ratio. Ratio, mint and size are immutable.
  2. **Rarity assignment:** the creator commits a Merkle root of the full trait list before launch. A VRF seed drawn at
     lock keys a Feistel permutation from index to list position. Metadata is bound on-chain at first exit with a
     Merkle proof, the collection is created by the program with no metadata-update path, and a public verify script
     is published.
  3. **Selection:** two-step VRF. `request_capture`/`request_reroll` lock payment. Permissionless `settle` happens in
     strict FIFO order over a sequenced pool (only deposits with `seq < request.seq` are candidates). There's no cancel
     after fulfilment, and `expire` applies only to unfulfilled requests past the deadline. `release` is instant and
     exact.
  4. **Re-roll:** VRF-random *different* NFT (the hand-in is excluded from its own draw). The fee is a token bps of the
     ratio plus a SOL cost fee, and `capture_fee ≥ reroll_fee` is enforced. **Updated by ADR-009:** fees are capped and
     fixed at init (no `propose_fees`), and the token fee is **burned**.
- **Rejected.**
  - (i) Thin wrapper holding MPL-Hybrid's authority as a PDA (feasible via an atomic `BlockCapture` toggle): it keeps
    unaudited, externally upgradeable code under our backing and relies on fragile upstream behaviour.
  - (ii) Unrevealed-in-escrow metadata alone: it doesn't solve selection without the same two-step VRF, and NFTs lose a
    stable identity.
  - Pure MPL-Hybrid: cherry-picking or revert-until-rare in every configuration.
- **Consequences.**
  - Adds a custom program that holds pooled value, so it's in the pre-mainnet audit scope with the heaviest scrutiny.
  - Metaplex Core stays the NFT standard, and MPL-Hybrid leaves the swap path.
  - Capture and reroll are two transactions with a few seconds of VRF latency.

## ADR-009: Scope change: Token-2022 shelved; Stonk.fun design constraints; re-roll fee = BURN

**Status: Accepted (decided by Barton, 2026-09-24 2:51 PM MT, BRIEF "SCOPE CHANGE").** Recorded by the engineer.

- **Context.** Barton dropped Token-2022 for now and moved 100% of the focus to SPL-404 hybrid launches. Barton also
  supplied the real Stonk.fun source (Bitquery, figures verified 2026-09-22,
  https://bitquery.io/investigations/is-stonkfun-dumping-on-holders; summary in
  [stonkfun-lessons.md](stonkfun-lessons.md)). It was not a hack: Stonk.fun's own reward wallet swept a 1–3% transfer
  tax, sold it into each coin's pool and paid holders in the pair asset. 160 coins lost more than half their supply to
  tax, snipers made the first hour the worst, at least $1.41M went off-ledger to linked wallets, and on 2,178 coins the
  reward wallet can still raise the tax to 100%.
- **Decision A: scope.**
  - **Shelved/deferred (not dropped for good):** Token-2022, the transfer tax, tax-funded NFT buys, the tax treasury
    (`fee_treasury`, renamed `tax_treasury` on the shelved branch), the holder lottery (`holder_lottery`), BRIEF hard
    requirements #3 and #4, and [transfer-tax-vs-wrap.md](transfer-tax-vs-wrap.md) (kept as reference, marked SHELVED).
  - **ADR-004 superseded** (its Hybrid half stands). **ADR-005 and ADR-006 shelved.**
  - Code is preserved on local branch `shelved/track-b-t22` (commit `3916467`, WIP) and in `shelved/` on `main`,
    excluded from the Cargo workspace and `Anchor.toml`. See `shelved/README.md`.
- **Decision B: product.** Classic SPL Token, fixed 1B supply, mint + freeze authority revoked at launch (ADR-010);
  token↔NFT conversion with exact unwrap (engine: `hybrid_vault`, ADR-008 accepted); ratio in {10k, 50k, 100k, 200k,
  500k, 1M, 2.5M, 5M} (updated by Barton 2026-09-24); collection size 100 ≤ N ≤ 1B / ratio; cosmetic rarity; blind assignment; VRF re-roll.
- **Decision C: design constraints from Stonk.fun (binding on every Track A program):**
  1. **No transfer tax.** Classic SPL Token only; Token-2022 is rejected on-chain.
  2. **No operator wallet that custodies value.** No platform or creator wallet sweeps, holds or routes user funds.
     Every flow is program-enforced; no instruction takes a free destination argument.
  3. **No mutable economics after launch.** Ratio, collection size, fees, fee destination and mint are fixed at init.
     Any remaining admin power sits behind a Squads multisig **plus** a timelock and may only **reduce risk** (e.g.
     pause new captures/re-rolls). It can never block unwrap/release, move funds or change economics. See
     [admin-multisig-timelock.md](admin-multisig-timelock.md).
  4. **Fee destinations on-chain and matching the site.** Burned fees show up as supply reduction; publish a
     reconciliation script (lesson 6).
  5. **Anti-sniping for the bonding-curve launch window.** Open design item until the curve venue is chosen.
- **Decision D: re-roll fee = BURN.** Barton's decision (he skipped the question and burn became the default; the BRIEF
  records it). This **supersedes my earlier recommendation (Q-H3: creator/platform split, "no burn")**. Copy: "Fixed at
  1,000,000,000 at launch. No one can mint more; re-roll burns can only reduce it." Short form: "fixed 1B at launch;
  re-roll fees are burned". `hybrid_launch` only accepts `fee_destination = BURN`.
- **Burn safety check (engineering): confirmed, the reasoning holds.**
  1. Supply starts at exactly 1B × 10^d and can only decrease: there is no mint authority (✅ tested in `hybrid_launch`).
  2. The backing owed is `ratio × NFTs outside the vault`, held in the vault.
  3. Fees are paid from the user's own balance **on top of** the ratio. A re-roll moves no backing (NFT in, NFT out); a
     capture moves exactly `ratio` into the vault and burns a separate fee amount. Nothing burns from the vault.
  4. So `vault ≥ ratio × NFTs outside` holds after every instruction, and unwrap stays exact. Burning users' fee tokens
     can never make the vault insolvent.
  5. The invariant becomes `circulating + vault + Σburned == 1B × 10^d`. Once burns push supply below `N × R`, not
     every NFT can be out at once; the rest simply stay in the pool, and every circulating NFT is still fully backed.
     The copy "Up to N×R can be held as NFTs" must say "at launch" or show a live figure.
  6. **Engine caveat:** `hybrid_vault` burns inline. **MPL-Hybrid can't burn fees natively.** Fees go to
     `recipe.fee_location`, and its `BurnOnCapture`/`BurnOnRelease` paths burn the backing (forbidden). Under
     MPL-Hybrid, burning needs `fee_location` = a program PDA with only a permissionless burn crank (brief transient
     custody, no withdraw path), and `update_recipe` can still overwrite `fee_location` (T-HY-01).
- **Consequences.** THREAT_MODEL rewritten (sourced Stonk.fun section; Track B rows marked 💤). ARCHITECTURE and the
  hybrid-rarity doc updated. The SOL part of any fee (VRF, rent) can't be burned; it pays those costs only (N7).
  Fees under ADR-008 become immutable, which removes `propose_fees`/`execute_fees`.

## ADR-010: Engine-independent `hybrid_launch` program

**Status: Proposed (implemented and tested on localnet, not audited).** Program ID (localnet throwaway key):
`9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8`.

- **Context.** Whatever the engine (ADR-008), every hybrid launch needs the same first step: a classic mint with
  exactly 1B supply and no authorities, plus an on-chain record of economics nobody can change (ADR-009 C3).
- **Decision.** One instruction, `launch(params)`, in one transaction:
  1. Create the account for a **fresh mint keypair that must sign** (82 bytes, owner = classic SPL Token). If the
     address was pre-funded (system-owned, no data), top up to rent-exempt, then `allocate` and `assign`, the way the
     ATA program does, so 1 lamport can't grief a launch (**QA-HL-01, fixed**). Otherwise `create_account`. An address
     with data or any non-system owner is rejected (`MintAccountInUse`), so an existing mint (classic or Token-2022)
     can't be onboarded.
  2. `initialize_mint2(decimals, mint_authority = PDA ["mint_authority", config], freeze_authority = None)`.
  3. Create the classic ATA of the **launch-vault PDA** `["launch_vault", mint, launch_config]`, then `mint_to` exactly
     `1_000_000_000 × 10^decimals` into it. The owner is program-derived, **not caller-chosen**, and no instruction
     signs with the launch-vault seeds, so no person can withdraw (**QA-HL-02, fixed; N2**).
  4. `set_authority(MintTokens → None)`.
  5. Re-read the mint and the destination and fail unless owner, supply, decimals, both authorities, the destination
     owner (launch vault), its balance, and the absence of a delegate or close authority are all right.
  6. `init` an immutable `LaunchConfig` PDA `["launch_config", mint]`: creator, mint, launch destination, decimals,
     total supply, ratio (whole and base units), collection size, `max_tokens_in_nft_form`, capture/re-roll fee bps
     and exact amounts, `fee_destination = BURN`, `launched_at`, `launch_vault` and bumps.
  - Validation: `decimals ≤ 9`; ratio ∈ {10k, 50k, 100k, 200k, 500k, 1M, 2.5M, 5M}; `collection_size ≥ 100` and
    `collection_size × ratio_base ≤ supply_base` via `checked_mul` (overflow rejects); each fee ≤ 1,000 bps of the
    ratio; `capture_fee_bps ≥ reroll_fee_bps`; `fee_destination == BURN`. Fee amounts are exact because every ratio is
    a multiple of 10,000.
  - **Supply table** (whole tokens per NFT → max collection size = 1B / ratio; min 100 for all):

    | Ratio | 10k | 50k | 100k | 200k | 500k | 1M | 2.5M | 5M |
    |---|---|---|---|---|---|---|---|---|
    | Max NFTs | 100,000 | 20,000 | 10,000 | 5,000 | 2,000 | 1,000 | 400 | 200 |

  - `token_program: Program<Token>` rejects Token-2022 at the account-validation layer.
  - **No update, close or admin instruction exists.**
- **Out of scope (engine-specific):** SOL cost fees (VRF, rent), the vault, the engine's own config. The engine should
  read `LaunchConfig` instead of taking its own copies of ratio or fees.
- **Build note.** anchor-spl 1.2.0's `idl-build` references `token_interface` unconditionally, so the program's
  `idl-build` feature also enables `anchor-spl/token_2022`. That affects only IDL generation, not the on-chain program.
- **Consequences.** 5 unit tests and 25 engineer LiteSVM tests plus QA's `qa_launch` suite (`./scripts/test.sh`).
  N2 is decided (launch vault PDA). The curve/distribution mechanism that will move tokens out of the launch vault is
  TBD and must arrive through the upgrade process (multisig plus timelock), because today nothing can move them.
  Open: default decimals (N3).

---

## Open questions for Barton

Status after ADR-009. Obsolete items are struck through and kept for the record.

1. **VRF provider.** I lean toward **Switchboard On-Demand**: actively maintained, devnet queue available, slashing,
   and the `solana-v3` host build compiles against Anchor 1.2. It needs a strict commit–reveal design, and its crate
   currently fails the SBF build with Anchor 1.2 (a getrandom transitive dependency), so it needs a dependency spike or
   manual account parsing. **ORAO** is simpler (nodes fulfil, so there's no withheld reveal) at about 0.001 SOL per
   request, but its CPI crate pulls anchor-lang 0.32 and doesn't compile against Anchor 1.2, so it needs a hand-rolled
   CPI. Which do you prefer?
2. ~~Anti-sniping via lottery registration/stake~~ **Obsolete (lottery shelved).** Replaced by N6 (curve anti-sniping).
3. ~~Prize sourcing~~ **Obsolete (Track B shelved).**
4. ~~Tax parameters~~ **Obsolete (no tax).**
5. **SBPF v2 vs v3.** OK to stay on v2 until we bump Rust and litesvm?
6. ~~Stonk.fun incident source~~ **Resolved:** Bitquery investigation (ADR-009, THREAT_MODEL).
7. ~~**MPL-Hybrid audit.**~~ **Obsolete:** engine = `hybrid_vault` (ADR-008 accepted, 2026-09-24).
8. **Multisig.** Squads v4: who are the signers, what's the threshold, and what timelock length? (Proposed defaults in
   admin-multisig-timelock.md.)

### Hybrid rarity (ADR-008)

9. ~~**Q-H1:** Engine: MPL-Hybrid or the custom `hybrid_vault`?~~ **ACCEPTED: `hybrid_vault`** (decided 2026-09-24 by
   the Solana Program Engineer because MPL-Hybrid can't meet Barton's stated requirements; reversible if Barton
   objects; ADR-008).
10. **Q-H2:** Capture/re-roll fees as a % of the ratio in tokens (default 2%) plus a small SOL cost fee?
11. ~~**Q-H3:** Fee destination~~ **Resolved: BURN** (Barton, ADR-009). Supersedes my "no burn" recommendation.
12. **Q-H4:** OK to require capture fee ≥ re-roll fee, with unwrap free of token fees? (Enforced in `hybrid_launch`.)
13. **Q-H5:** Lazy minting or creator pre-mint?
14. **Q-H6:** Forbid creator/team NFT allocations outside the pool?
15. **Q-H7:** Show full pool contents/traits publicly, or only the census?
16. **Q-H8:** Is two-transaction capture/re-roll acceptable UX?
17. ~~**Q-H9:** Keep the 100,000-NFT maximum or cap lower?~~ **Resolved by Barton's ratio set** (2026-09-24): max =
    1B / ratio (100,000 at 10k), min 100.

### New (scope change, ADR-009/010)

- **N1 Capture fee burned too?** Engineering default: yes, same burn path, so no fee account exists anywhere.
  (`hybrid_launch` applies `fee_destination = BURN` to both fees.)
- **N2 Launch destination. DECIDED (engineering, security-driven, 2026-09-24; QA-HL-02):** 1B goes to a PDA launch
  vault; distribution mechanism (curve) TBD. The destination is the ATA of `["launch_vault", mint, launch_config]`,
  not caller-chosen, with no withdraw instruction. A creator allocation, if Barton wants one, would have to be a
  visible, capped, program-enforced rule (still a question for Barton).
- **N3 Decimals default.** 6 (pump.fun-style) or 9? `hybrid_launch` accepts 0–9.
- **N4 Pause at all?** Recommended: at most "pause new captures/re-rolls", multisig + timelock, never blocking
  release/settle/expire. Or no pause at all (simplest, most trustworthy)?
- **N5 Upgrade authority.** Squads multisig until audit + stabilization, then frozen (`--final`)? For how long?
- **N6 Bonding curve venue + anti-sniping mechanism.** Own curve vs an audited venue; then which mechanism (see
  admin-multisig-timelock.md §Anti-sniping).
- **N7 Fee burn timing. DECIDED (final, 2026-09-24): burn at SETTLE.** The token fee is escrowed in the request
  PDA's token account at request time, burned at settle, and refunded together with the locked tokens on expire.
  Still open: refund the SOL cost fee minus VRF cost on expire?
