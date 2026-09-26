# Architecture decision records (on-chain)

Owner: Solana Program Engineer. Format: context → decision → consequences. Status values: Proposed / Accepted /
Superseded / Shelved. Every decision here is **Proposed** until Barton signs off, except where marked **Accepted (decided
by Barton)**.

> **Current scope (ADR-009, 2026-09-24 2:51 PM MT):** SPL-404 hybrid launches only (Track A). Token-2022, transfer tax,
> tax treasury and holder lottery (Track B) are **SHELVED/DEFERRED**. ADR-004 is superseded; ADR-005 and ADR-006 are
> shelved. New: ADR-009 (scope change + Stonk.fun constraints + burn), ADR-010 (`hybrid_launch`).
> **2026-09-25:** ADR-013 (flat SOL fee, release free; burn superseded), ADR-012 (VRF), ADR-015 (no pause), ADR-014
> (DBC graduation, not built), ADR-016 (**lazy minting**, supersedes batch pre-mint), ADR-017 (frozen LaunchConfig prefix); ADR-011 (2% token fee) superseded.

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

> **Partially superseded (QA-DOC-01).** The engine choice and the two-step VRF (§1–3) stand. Superseded parts:
> - **§4, the fee model** (token bps fee, SOL cost fee, `capture_fee ≥ reroll_fee`, burn): replaced by
>   **ADR-013**, one flat tiered SOL fee with no token fee and no burn, and release is free.
> - **The 10k ratio and its sizes** (Context and §1): dropped in **ADR-016**; N ≤ 10,000, 7 ratios.
> - **Metadata bound at first exit and pre-mint**: replaced by lazy mint at settle (**ADR-016**).
>
> The original text below is kept as the historical record.

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
  4. **Re-roll:** VRF-random *different* NFT (the hand-in is excluded from its own draw). ~~The fee is a token bps of the
     ratio plus a SOL cost fee, and `capture_fee ≥ reroll_fee` is enforced. **Updated by ADR-009:** fees are capped and
     fixed at init (no `propose_fees`), and the token fee is **burned**.~~ **SUPERSEDED by ADR-013:** the same flat
     tiered SOL fee as capture, no token fee, no burn (M-06 per ADR-018).
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

> Partially superseded: Decision D (burn) by ADR-013; the ratio set by ADR-016 (10k dropped, N ≤ 10,000); C3's "pause" allowance by ADR-015 (no pause).

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
- **Decision D: re-roll fee = BURN.** **SUPERSEDED by ADR-013 (flat tiered SOL fee; no burn, no token fee).** Barton's decision (he skipped the question and burn became the default; the BRIEF
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

> **Partially superseded (QA-DOC-01).** Steps 1–5 (fresh mint, authorities revoked, 1B into the launch-vault PDA)
> stand. Superseded parts:
> - **§6, the LaunchConfig fee fields and validation**: capture/re-roll fee bps, exact fee amounts,
>   `fee_destination = BURN`, the ≤ 1,000 bps cap and `capture ≥ reroll`. **ADR-013** replaced these with
>   `fee_lamports` (from the ratio tier table, cap 0.01 SOL) and `fee_recipient` = `PLATFORM_FEE_RECIPIENT`.
> - **LaunchConfig layout**: now v4, with the frozen prefix from **ADR-017** and `dbc_config`/`dbc_pool`
>   appended by **ADR-014**.
> - **The 10k ratio / 100,000 column** of the supply table: dropped in **ADR-016**; N ≤ 10,000 for every
>   ratio.
> - **"One instruction"**: `register_dbc_launch` (ADR-014) was added for the DBC curve. It only creates
>   configs.
>
> The original text below is kept as the historical record.

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
     total supply, ratio (whole and base units), collection size, `max_tokens_in_nft_form`, ~~capture/re-roll fee bps
     and exact amounts, `fee_destination = BURN`~~ (**superseded by ADR-013**: `fee_lamports`, `fee_recipient`),
     `launched_at`, `launch_vault` and bumps.
  - Validation (**superseded in part**: the 10k ratio was dropped by ADR-016; the fee rules were replaced by ADR-013): `decimals ≤ 9`; ratio ∈ {~~10k,~~ 50k, 100k, 200k, 500k, 1M, 2.5M, 5M}; `collection_size ≥ 100` and
    `collection_size × ratio_base ≤ supply_base` via `checked_mul` (overflow rejects); each fee ≤ 1,000 bps of the
    ratio; `capture_fee_bps ≥ reroll_fee_bps`; `fee_destination == BURN`. Fee amounts are exact because every ratio is
    a multiple of 10,000.
  - **Supply table** (whole tokens per NFT → max collection size = 1B / ratio; min 100 for all). **Superseded by
    ADR-016:** the 10k ratio is gone and every ratio is capped at N ≤ 10,000:

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

## ADR-011: 2% token fee on capture/re-roll

**Status: SUPERSEDED by ADR-013** (flat tiered SOL fee, 2026-09-25). Recorded only so the number stays reserved.

## ADR-012: VRF: Switchboard On-Demand, program-chosen oracle, heartbeat filter, bounded recommits, principal + escrow expire

**Status: Accepted (engineering; Auditor A v2 M-04). Implemented on localnet (mock Switchboard). CPIs proven against the REAL Switchboard program in LiteSVM. On devnet (2026-09-25): our deployed vault's `init_randomness` CPI into real Switchboard succeeded, and a gateway-signed reveal was settled by a third party (direct flow). The vault's own request/settle path isn't exercised on devnet, because it needs a graduated DBC pool.**

- **Real-program proof (2026-09-25).** `switchboard-on-demand` 0.13.0 (`default-features = false, features =
  ["solana-v3"]`) compiles for SBF with Anchor 1.2. `getrandom` 0.2 is satisfied by a `custom` backend that always
  fails, and nothing on-chain calls it. Our CPIs are hand-built: discriminators in `constants.rs`, with the account
  order taken from the SDK. The test suite `vault/real_switchboard.rs` loads the devnet program binary (`Aio4gaXj…`,
  dumped read-only), plus the devnet queue `EYiAmGSd…`, state, and oracles along with their `OracleRandomnessStats`
  PDAs, into LiteSVM. It then proves:
  - `init_randomness`, `request_capture` (commit) and `recommit` run through the real program. Switchboard records the
    new oracle after a recommit.
  - A forged reveal is rejected by Switchboard itself (`InvalidSecpSignature`, 6016). We delegate signature
    verification to Switchboard.
  - The reveal stats account is `["OracleRandomnessStats", oracle]`. It is not `["OracleStats", …]`: the harness had
    this wrong and the real program caught it. The vault passes the account through unchecked and Switchboard
    validates it.
  - LiteSVM doesn't maintain SlotHashes, so the harness fills the sysvar by hand (`refresh_slot_hashes`).

- The program picks the oracle from `vault.sb_queue` deterministically (vault, seq). A candidate is skipped only when
  the caller supplies that oracle's account as proof its heartbeat is older than `MAX_ORACLE_HEARTBEAT_AGE_SECS`
  (3,600 s); the chosen oracle must itself be fresh, otherwise `OracleStale`.
- `recommit_randomness` up to `MAX_RECOMMITS` = 3. After that, and after `REVEAL_TIMEOUT_SLOTS` + `EXPIRE_GRACE_SLOTS`,
  anyone can `expire_request`: it refunds the principal (tokens or the handed-in NFT) + the full mint escrow (ADR-016),
  **never the tier fee**. Batch expire: `expire_requests(count)` expires up to `MAX_EXPIRE_PER_CALL` = 7 consecutive heads in one
  instruction, all-or-nothing (bounded by the 64-account lock limit; K > 3 needs v0 + ALT).
- **VRF cost (M-37): the requester pays at cost, separately (Auditor A decision).** **Measured on devnet 2026-09-25
  (real Switchboard On-Demand `Aio4gaXj…`, default devnet queue `EYiAmGSd…`, gateway-signed reveal settled by a third
  party; details in CD Q2):** commit 5,000 lamports (1 signature), reveal 10,000 lamports (2 signatures: the third-party
  fee payer plus the randomness authority), **oracle fee 0 lamports** (no SOL or wSOL moved in the reveal). Mainnet queue
  reward read via RPC = 5 lamports (not exercised). Per draw ≈ 15,000 lamports + settle 5,000 ≈ 0.00002 SOL before
  priority fees, far below the 0.002 minimum tier. The randomness account (480 bytes) is a one-off, reusable setup.

## ADR-013: Flat tiered SOL fee on capture/re-roll; release free (supersedes the burn and the 2% token fee)

**Status: Accepted (Barton, 2026-09-25; release free confirmed 5:04 PM MT). Implemented.**

- **Decision.** One flat SOL fee per ratio tier, charged at **request** on `request_capture` and `request_reroll`
  (the same amount for both, so re-roll ≤ capture holds) and **never refunded** (not on expire either). Paid to the
  code constant `hybrid_launch::PLATFORM_FEE_RECIPIENT` (account constraint), not a launch-time copy.
  Release (`unwrap`) is **free**: it returns exactly `ratio` tokens per NFT, and the user pays only the tx fee.
- **Tiers** (`needs_barton::FEE_TIERS`): 50k → 0.002 SOL; 100k, 200k → 0.005; 500k, 1M, 2.5M, 5M → 0.01.
  `MAX_FEE_LAMPORTS` = 0.01 SOL hard cap.
- **Fail-safe charging (M-08).** The vault charges `min(stored, tier(ratio) or u64::MAX, MAX_FEE)`, so it never
  reverts on a mismatch and never overcharges; the LaunchConfig version must be 3 (fail closed).
- **Removed:** the token fee (2%/bps), the burn path, the FeeVault, `sweep_fees`, `fee_vault_bump` and
  `FEE_VAULT_SEED` (the error became `ReservedNothingToSweep`, 6049).
- **Copy:** "Supply fixed at 1,000,000,000; nobody can mint more. Capture/re-roll cost a flat SOL fee; release is free
  and returns exactly N tokens."
- **Metaplex Core fees (reference):** Create 0.0015 SOL, Execute 0.00004872 SOL, Transfer free.

## ADR-014: Graduation via Meteora DBC; 25% buffer in a locked PDA

**Status: Accepted (Barton, 2026-09-25). IMPLEMENTED on localnet against the REAL DBC program (mainnet dump) on
LiteSVM. The DAMM v2 migration step is simulated (see Limits).** The curve is Meteora DBC (graduation-design §3).

**Research: who creates the base mint?** In DBC 0.2.1, `initialize_virtual_pool_with_spl_token` declares `base_mint`
as `#[account(init, signer, mint::authority = pool_authority, …)]`. So the mint address **must sign** that
transaction.
- A plain keypair works; that's the SDK flow.
- A **PDA also works**, but only if our own program CPIs into DBC and signs for the PDA with `invoke_signed`. The
  signer privilege carries into DBC's nested `create_account`.
- A non-signing address can never be the base mint.

DBC then mints `pre_migration_token_supply` into its base vault (owner = DBC `pool_authority`, a PDA) and revokes
the mint authority in the same instruction. It never sets a freeze authority. The test
`research_dbc_mint_is_created_by_dbc_with_authorities_revoked` confirms this against the real program.

**Decision: register and verify, don't wrap.** Wrapping DBC's create in our CPI would couple us to DBC's full
account list and CU budget, and it buys nothing: whatever DBC creates, we verify on-chain. So the creator creates
the pool with DBC's SDK (keypair mint), then calls `hybrid_launch::register_dbc_launch(ratio, collection_size)`.
- **Platform allowlist (checked first):** `dbc_config` must be in `hybrid_launch::APPROVED_DBC_CONFIGS`, a
  compile-time constant list in `needs_barton.rs`. The platform, not the creator, picks the curve shape, fees,
  partner/fee claimer and migration settings. No instruction, key or account can edit the list; changing it
  takes a program upgrade (3-of-5 multisig + 7-day timelock, admin-multisig-timelock.md). Rejection is
  `DbcConfigNotApproved` (6023). Listed configs must still pass every structural check below.
  - Devnet/localnet list = `5L1MfYm4yqPySiVddKugruYoGyN6an6viSkHzL7MK1Y`, the real devnet config used as the test
    template. On devnet itself its `leftover_receiver` isn't our buffer PDA, so a real devnet registration against
    it would still fail `DbcConfigRejected`. A platform-created devnet config naming `["dbc_buffer"]` is needed
    before a devnet end-to-end run.
  - **Mainnet list: NEEDS BARTON.** It's empty, and a `mainnet` build refuses to compile until at least one config
    key is set (a const assert).
  - Test: `register_rejects_config_not_on_platform_allowlist` (a byte-identical valid config at an unlisted address
    is rejected, and no LaunchConfig is created).
- **Pool checks:** owner DBC, VirtualPool discriminator and length 424, `base_mint == mint`, `config == dbc_config`,
  SPL pool type, **signer == pool.creator** (nobody can register someone else's token with other parameters), not
  yet migrated.
- **Config checks:** owner DBC, PoolConfig discriminator and length 1048, quote = wSOL, SPL token type,
  `token_decimal` == the mint's decimals, **fixed supply with pre == post == 1B × 10^dec** (DBC burns nothing at
  migration, so supply stays exactly 1B), `token_update_authority == Immutable`, **`leftover_receiver` == the
  `["dbc_buffer"]` PDA**, and `migration_quote_threshold` inside our 10–100,000 SOL bounds (it's recorded as the
  graduation threshold).
- **Mint checks:** classic Token program (typed `Account<Mint>`; Token-2022 is rejected), mint authority None,
  freeze authority None, supply exactly 1B × 10^dec. Ratio and collection size are validated as for `launch`.
- **Records:** LaunchConfig **v4**, which appends `dbc_config` and `dbc_pool`. The frozen exit-path prefix (ADR-017)
  is unchanged. `launch_destination` = DBC's base vault and `launch_vault` = DBC's pool authority. `init` means
  one registration per mint.
- **Buffer ATA:** created idempotently as ATA(buffer PDA, mint).
- Offsets come from DBC 0.2.1 source (`hybrid_launch/src/dbc.rs`), were checked against real devnet accounts, and
  are pinned by the unit test `layout_is_consistent_with_dbc_0_2_1`.

**Graduation check** (`hybrid_vault::graduation::verify`, compiled into every build):
- The proof must be the recorded `dbc_pool`, owned by DBC with the right discriminator and length.
- It must name this mint and config.
- It must have `is_migrated == 1` **and** `migration_progress == CreatedPool`. DBC sets both in the instruction that
  creates the DAMM pool.

A native `launch` records no pool, so its vault can never open in production. The TEST-ONLY mock is additionally
accepted only under `test-mock-graduation`, which is still barred from mainnet builds by a compile_error and from
devnet deploys by the marker and allowlist guards.

**25% buffer (T-GRAD-03).** DBC requires `pre_supply ≥ swap_base × 1.25 + migration_base`. After migration, DBC's
**permissionless** `withdraw_leftover` (fixed-supply configs only; it requires CreatedPool and pays once) moves the
unsold remainder to ATA(`leftover_receiver`, mint). Because every accepted config names our `["dbc_buffer"]` PDA and
hybrid_launch has **no instruction that signs with that seed**, those tokens are locked forever. They're still
counted in the 1B, since nothing burns.

Tests (`vault/dbc_graduation.rs`, 9 tests, real DBC program plus the real devnet config as a template):
- the real DBC `withdraw_leftover` fills the buffer;
- DBC refuses a different receiver;
- transfer, approve, burn, set_authority and close by a thief or by the creator all fail;
- DBC won't pay the leftover twice;
- supply stays at 1B;
- a source scan pins the seed to 3 files, none of them signing;
- registration rejects a wrong signer, a fake-owner pool, another mint's pool, a bad ratio, a repeat registration,
  and patched configs (another leftover receiver, post < pre, creator-held metadata authority, threshold < 10 SOL);
- `open_vault` rejects the pool before migration, another migrated pool, a fake-owner copy, and `is_migrated`
  without CreatedPool, then opens and serves a capture.

**Limits:**
1. **Migration is simulated.** The two pool bytes are flipped, because a real DAMM v2 migration (swaps to the
   threshold plus the DAMM v2 program and accounts) isn't loaded. A real migrated devnet pool
   (`DGtaRQ9E…`) is parsed as graduated to confirm the fields.
2. In the leftover test, nothing was sold, so the whole 1B lands in the buffer. That shows the path, not a
   realistic amount.
3. The DBC offsets are pinned to 0.2.1. A DBC upgrade that changed its layout would make `load_pool` fail closed
   (discriminator and length).
4. `hybrid_launch` now has two instructions, so three QA IDL guards in `qa_launch` need QA's review (audit-fixes).
5. ~~We don't restrict which DBC config is used.~~ **Resolved:** there's a platform allowlist,
   `APPROVED_DBC_CONFIGS` (see above).

## ADR-015: No pause

**Status: Accepted (Barton, 2026-09-25). Implemented.** No instruction, key or multisig action can pause or halt any
vault instruction. Test `no_pause_path_exists_no_key_can_halt_any_instruction` scans the IDL. Resolves N4.

## ADR-016: LAZY MINTING (supersedes batch pre-mint and the launch-time affordability rule)

**Status: Accepted (Barton, 2026-09-25 5:13 PM MT). Implemented and tested on localnet.** Resolves **Q-H5**.
Interface: [lazy-mint-interface.md](lazy-mint-interface.md). Design: graduation-design §2.5/§4.4.

- **No pre-mint.** `mint_assets` (crank), the graduation fund and the affordability check are removed.
  `graduation_slice_pct` is always 0; `GraduationUnfundable` became `ReservedGraduationUnfundable` (6015);
  `CollectionNotFullyMinted` (6035) and `GraduationFundShortfall` (6052) are reserved. The 100 ≤ N ≤ 10,000 bounds stay;
  the 10k ratio is dropped.
- **Gate:** graduated AND the Core collection exists (update authority = vault_authority, sizes consistent) AND pool
  capacity == N. There's no minted == N requirement.
- **Request escrow:** capture and re-roll move `MINT_ESCROW_LAMPORTS` = (3,570,480 rent + 1,500,000 Core fee) × 125% =
  6,338,100 lamports into the system-owned PDA `["mint_escrow", vault, seq]`, alongside the principal; the fee is charged
  at request. A live-rent check (`MintCostConstantStale`) makes an underfunded request impossible. It's a separate PDA
  because Core needs a data-less system payer and the program can't debit its own Request inside a CPI-ing ix.
- **Uniform pick** over all indices the vault holds (unminted + returned): lazy Fisher-Yates pool of 0..N−1.
- **Mint-once:** a minted bitmap in the pool account; `minted_count ≤ N` is asserted every ix.
- **Settle (permissionless):** transfer if the pick is minted; else verify the settler-supplied leaf + proof against the
  pre-curve root and Core-create (payer = escrow PDA, owner = user, collection set + verified, immutable URI). The
  remaining escrow always goes to the user; the settler pays only its tx fee (tip 0, T-HV-16). Settle can't strand funds.
- **Re-roll** returns the handed-in asset to the vault: no re-mint, no burn.
- **Expire:** principal + full escrow back, never the fee.
- **Cost:** the pool account (64 + 16N + N/8 bytes) is ~1.12 SOL rent at N = 10k, creator-funded at vault creation.
  A u16 packing could cut this later. Settle-with-mint is ~1.2 KB at 10k, so it needs v0 + ALT.
- **Accepted downside:** marketplaces show only minted assets until they're captured.

## ADR-017: Frozen LaunchConfig prefix for exit paths (M-41); M-16 accepted

**Status: Accepted, implemented.** release, settle and expire read LaunchConfig raw through `config::exit_view`: owner ==
hybrid_launch, discriminator, length ≥ `LC_STABLE_PREFIX_LEN` (221), and only `ratio_base` (offset 157) and
`collection_size` (165); the mint is checked against `vault.mint`. The offsets are frozen in
`hybrid_launch::stable_layout` and guarded by tests (`offsets_are_frozen`, `appended_field_keeps_prefix`), so a
LaunchConfig upgrade can never lock users out of exits. Capture still fails closed on an unknown version.
**M-16 (upgrade authority is a trust assumption until freeze) is accepted** and documented in THREAT_MODEL.

## ADR-018: QA-FEE-04: first-mint cost comes from the deposit; M-06 holds on average, not per draw

**Status: Accepted (Barton, 2026-09-25 5:20 PM MT). Accepted trade-off.** Refines ADR-013/ADR-016.

- **Decision.** A capture or re-roll whose draw lands on a **never-minted** index pays that index's first-mint cost
  (asset rent + Metaplex Core create fee) out of the mint deposit it escrowed at request. There's **no flat surcharge**:
  the flat tier fee is the same for every draw. The unspent part of the deposit is refunded **exactly** to the requester
  in the same settle instruction.
- **Consequence for M-06 ("re-roll ≤ release + capture").** The *fee* comparison holds per draw (re-roll fee ==
  capture fee, release free). The *all-in* cost differs per draw by the first-mint delta: a draw that mints costs
  ~0.0031–0.0044 SOL more than one that transfers an already-minted asset, whether it's a capture or a re-roll. The
  expected first-mint delta is *almost* the same for a re-roll and for release + capture, because both draw uniformly
  from nearly the same pool. So M-06 holds **on average when many NFTs are left to draw**; see the QA-FEE-05 bound
  below. The per-draw check therefore **excludes the first-mint delta**. Tests assert the fee
  equality (`reroll_charges_same_flat_sol_fee_…`) and the exact deposit refund
  (`first_mint_cost_breakdown_is_exact_and_unspent_deposit_is_refunded`, `returned_asset_is_transferred_again_never_reminted`).
- **Why accepted.** A flat surcharge would overcharge the draws that hit minted assets. Charging the actual cost keeps
  fees flat and honest, and nobody can pick whether their draw mints (VRF picks uniformly over minted + unminted).
- **Disclosure copy (suggestion):** "If your NFT is being minted for the first time, about 0.003–0.004 SOL of your
  deposit covers its on-chain rent and the Metaplex fee; the rest of the deposit comes back automatically."
- **QA-FEE-05 (disclosure; QA finding, engineering-verified): the average holds only with about 613 or more NFTs
  left to draw.**
  - **Setup:** let P = NFTs left in the pool, u = how many of them are still unminted, c = first-mint cost (3,066,000
    lamports for our shape; up to ~4.35M at the maximum URI).
  - **Re-roll** draws from the P NFTs; the handed-in NFT is excluded from its own draw. Its expected mint cost is
    c·u/P.
  - **Release, then capture** draws from P + 1: the released NFT goes back in, and it's already minted. The expected
    mint cost is c·u/(P+1), but it pays one extra transaction fee (5,000 lamports base).
  - **Difference:** re-roll − (release + capture) = c·u/(P(P+1)) − 5,000. It's worst when nearly everything is
    unminted (u ≈ P), where it becomes c/(P+1) − 5,000.
  - **Break-even:** P + 1 = c / 5,000 ≈ 613 (for c = 3,066,000).
  - **Examples:** at P = 100 a re-roll averages about **25,000 lamports (0.000025 SOL) more** (3,066,000/101 −
    5,000 ≈ 25,356). At P = 1,000 it averages ~1,940 lamports *less*. At the pool floor (the minimum P is
    max(5, 2% N) + 1) the gap is largest: ~433,000 lamports (≈0.00043 SOL) at P = 6 (3,066,000/7 − 5,000).
  - **Per-draw fees are unaffected:** re-roll fee == capture fee and release is free. The gap is only the
    expected first-mint cost, which the requester's own escrow pays (ADR-018). It shrinks as more of the collection
    is minted (u/P → 0) and disappears once everything is minted.
  - **Not fixed on purpose:** a subsidy or a surcharge would bring back the flat-overcharge problem ADR-018
    rejected.
  - **Disclosure wording for the frontend / Creative Director (exact):**
    > "Re-rolling costs the same flat fee as capturing, and releasing is free. When fewer than about 600 NFTs are left
    > in the pool, a re-roll can cost slightly more on average than releasing and capturing again (for example, about
    > 0.000025 SOL more with 100 left), because a re-roll can't draw the NFT you hand in, and that NFT is already
    > minted. Any first-mint cost comes out of your refundable deposit; the rest is returned automatically."

## Creative Director questions (answered 2026-09-25; figures measured on LiteSVM unless marked ESTIMATE)

### CD Q1: Is the ~0.005 SOL mint cost refunded when the draw lands on an already-minted NFT?

**Yes, in full, in the same settle instruction.** Every capture or re-roll escrows a fixed deposit of **6,338,100
lamports (0.0063381 SOL)** in its own escrow account, `["mint_escrow", vault, seq]`. At settle:

- **Draw lands on an already-minted NFT** (returned to the vault earlier): no mint happens. The NFT is transferred and
  **all 6,338,100 lamports** go back to the requester in that same instruction.
- **Draw lands on a never-minted NFT:** the program mints it straight to the requester, paid from the deposit. The
  **unspent margin is returned in that same instruction**, exactly (the test asserts spend + refund == deposit):
  - Core create fee: **1,500,000** lamports (0.0015 SOL; Metaplex's fee, held in the asset account).
  - Asset rent: `(128 + asset_bytes) × 6,960` lamports. It's **recoverable by nobody**; it stays with the asset. The size
    depends on the URI length:
    - measured with an 18-char test URI (97 bytes): rent **1,566,000**, total spend **3,066,000**, refund **3,272,100**.
    - typical IPFS CIDv1 URI (~66 chars, ~148 bytes): rent ≈ 1,920,960, spend ≈ 3,420,960, refund ≈ 2,917,140 (computed from the same formula).
    - worst case (`#9999`, 200-char URI, ~282 bytes): rent ≈ 2,853,600, spend ≈ 4,353,600, refund ≈ 1,984,500 (computed).
  - So the real first-mint cost is **~0.0031–0.0044 SOL** (not ~0.005; the 0.00509 figure was for assets with an
    Attributes plugin, which we don't add). The deposit is sized for a 385-byte worst case plus 25% margin.
- Separately, the request account's rent (**3,006,720**) and the randomness-lock rent (**1,231,920**) are also returned to
  the requester at settle (or expire). The flat tier fee is the only thing never refunded.
- If the request expires instead (randomness never arrives), the **whole deposit** is refunded along with the principal.

### CD Q2: What does randomness actually cost per request?

| Item | Amount | Who pays | Refunded? | Status |
|---|---|---|---|---|
| Switchboard oracle fee | **0 lamports on devnet** (measured: the gateway-signed reveal moved no SOL and no wSOL; the reward escrow stayed at 0). Mainnet queue reward read via RPC: 5 lamports | the requester, at cost, separately from the tier fee (Auditor A decision, ADR-012) | no | **measured on devnet** (mainnet 5 lamports read, not exercised) |
| Commit tx (arms the randomness for this draw) | **5,000** lamports (1 signature, 15,081 CU) | requester | no | **measured on devnet** |
| Reveal tx (submits the oracle's signed value) | **10,000** lamports when a third party pays (2 signatures: fee payer + randomness authority, 49,675 CU); 5,000 if the payer is the authority. Plus any priority fee | whoever cranks it: the requester's client, or any third party | no | **measured on devnet** |
| Settle tx | 5,000 lamports base + priority | the settler (anyone) | no | exact base; priority varies |
| Request tx signature | 5,000 lamports base + priority (paid by the requester anyway) | requester | no | exact base |
| One-off `init_randomness` setup | **8,229,760 lamports at mainnet rent** (LiteSVM against the real program, test `real_switchboard_init_cost_breakdown`) = randomness account 480 bytes, rent 4,231,680 + wSOL reward-escrow ATA 2,039,280 + address lookup table 1,948,800 + tx fees 10,000. **On devnet: 6,009,480** = 3,088,640 + 1,488,440 + 1,422,400 + 10,000 fee, the same three accounts at devnet's lower rent rate (×0.7299; for example a 165-byte token account costs 1,488,440 there). Measured twice on devnet: directly, and through our vault's permissionless `init_randomness` CPI | whoever runs `init_randomness` | **not refunded, but reusable**: one account serves any number of sequential requests (each commit re-arms it; `rand_lock` stops concurrent use). The vault has no close instruction today, so these lamports stay locked | measured (LiteSVM and devnet) |
| Randomness lock + request rent | 1,231,920 + 3,006,720 | requester | **yes**, at settle/expire | measured |

**Per-request marginal randomness cost, measured on devnet 2026-09-25: oracle 0 + commit 5,000 + third-party reveal
10,000 = 15,000 lamports**, plus settle 5,000 (base fee) = **≈ 20,000 lamports ≈ 0.00002 SOL** before priority fees.
With mainnet's 5-lamport queue reward it's 20,005. That's about **1% of the lowest 0.002 SOL tier**, so it fits
comfortably; the tier fee isn't used to fund it (the requester pays randomness at cost, separately). The one-off
randomness setup (≈ 0.0082 SOL at mainnet rent, 0.0060 on devnet) is amortised over every request that reuses the
account. (An earlier draft said 408 bytes / 3,730,560: that was the mock's size.)

Devnet evidence (throwaway keys; the requester and the third-party settler are different keypairs):

- Direct Switchboard flow: `randomness_init` `5yvm5QeS…`, commit `2uwwVLZk…` (requester `5JkYLKC5…`), gateway-signed
  reveal `2DWhZjXY…` paid and submitted by the third party `8Eo6ioy5…`. Randomness account `5DZo4yC9…`, revealed at slot
  504225284.
- A forged reveal (oracle signature bytes tampered) was rejected in simulation with `SecpRecoverFailure` (6033). Replaying
  the already-used reveal was rejected with `ConstraintHasOne` (2001).
- Our deployed programs (`hybrid_launch` `9Loc4hQZ…`, `hybrid_vault` `BEfL9dcc…`): native `launch` `2fgof3f7…`,
  `init_vault` (N = 100, Core collection) `s5gotumM…`, and the vault's permissionless `init_randomness` CPI into real
  Switchboard, paid by the third party, `4HTiwsxB…`. The created randomness account `bPfZTsW1…` is owned by Switchboard
  with authority = our `randomness_authority` PDA. `open_vault` on that native launch fails closed with
  `GraduationCheckUnavailable` (6037, simulated).
- **Not measured on devnet: the vault's own request → reveal → settle-with-mint.** `request_capture` needs an open vault.
  `open_vault` needs a migrated DBC pool registered under an allowlisted config whose leftover receiver is our
  `["dbc_buffer"]` PDA (threshold ≥ 10 SOL of real buys). The settle-with-mint figure is therefore still the LiteSVM
  measurement against the real mpl_core program (CD Q1): **3,066,000 lamports** for a 97-byte asset (rent 1,566,000 +
  Core fee 1,500,000), with 3,272,100 of the 6,338,100 escrow refunded. Settle tx base fee 5,000.

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
10. ~~**Q-H2:** Capture/re-roll fees as a % of the ratio in tokens~~ **Resolved: flat tiered SOL fee (ADR-013).**
11. ~~**Q-H3:** Fee destination~~ **Resolved: BURN** (Barton, ADR-009). Supersedes my "no burn" recommendation.
12. ~~**Q-H4:**~~ **Resolved:** re-roll fee == capture fee (one tier fee), release free (ADR-013).
13. ~~**Q-H5:** Lazy minting or creator pre-mint?~~ **Resolved: LAZY MINTING (Barton 2026-09-25 5:13 PM MT, ADR-016).**
14. **Q-H6:** Forbid creator/team NFT allocations outside the pool?
15. **Q-H7:** Show full pool contents/traits publicly, or only the census?
16. **Q-H8:** Is two-transaction capture/re-roll acceptable UX?
- **OPEN (HOLD for Barton):** re-roll fee sink / excluding the fee wallet from capture.
17. ~~**Q-H9:** Keep the 100,000-NFT maximum or cap lower?~~ **Resolved by Barton's ratio set** (2026-09-24): max =
    1B / ratio (100,000 at 10k), min 100.

### New (scope change, ADR-009/010)

- ~~**N1 Capture fee burned too?**~~ **Obsolete: no burn (ADR-013).** Engineering default: yes, same burn path, so no fee account exists anywhere.
  (`hybrid_launch` applies `fee_destination = BURN` to both fees.)
- **N2 Launch destination. DECIDED (engineering, security-driven, 2026-09-24; QA-HL-02):** 1B goes to a PDA launch
  vault; distribution mechanism (curve) TBD. The destination is the ATA of `["launch_vault", mint, launch_config]`,
  not caller-chosen, with no withdraw instruction. A creator allocation, if Barton wants one, would have to be a
  visible, capped, program-enforced rule (still a question for Barton).
- **N3 Decimals default.** 6 (pump.fun-style) or 9? `hybrid_launch` accepts 0–9.
- ~~**N4 Pause at all?**~~ **Resolved: no pause (ADR-015).** Recommended: at most "pause new captures/re-rolls", multisig + timelock, never blocking
  release/settle/expire. Or no pause at all (simplest, most trustworthy)?
- **N5 Upgrade authority.** Squads multisig until audit + stabilization, then frozen (`--final`)? For how long?
- **N6 Bonding curve venue + anti-sniping mechanism.** Own curve vs an audited venue; then which mechanism (see
  admin-multisig-timelock.md §Anti-sniping).
- ~~**N7 Fee burn timing.**~~ **Superseded (ADR-013/012/016): the SOL fee is charged at request and never refunded; expire refunds the principal + the mint escrow.** Old text: DECIDED (final, 2026-09-24): burn at SETTLE.** The token fee is escrowed in the request
  PDA's token account at request time, burned at settle, and refunded together with the locked tokens on expire.
  Still open: refund the SOL cost fee minus VRF cost on expire?
