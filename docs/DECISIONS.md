# Architecture decision records (on-chain)

Owner: Solana Program Engineer. Format: context → decision → consequences. Status values: Proposed / Accepted /
Superseded. Every decision here is **Proposed** until Barton signs off, except where marked **Accepted (decided by Barton)**.

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

**Status: Proposed. Partially superseded by ADR-008 (proposed)** for hybrid swaps. The no-custom-transfer-fee part still
stands.

- **Context.** Custom swap/fee code that holds pooled value is the biggest bug and drain surface, and each one needs
  its own audit.
- **Decision.** Hybrid swaps use **Metaplex MPL-Hybrid**. Transfer tax uses the **Token-2022 TransferFee extension**.
  Custom code is limited to `fee_treasury` and `holder_lottery`.
- **Consequences.** We inherit MPL-Hybrid's limits: classic SPL Token only, no timelock on escrow updates, SlotHashes
  rerolls, and **Audit Pending** (repo README at `aacf1a5`). All of those are documented in THREAT_MODEL.md T-HY-*.

## ADR-004: Transfer tax vs exact unwrap: hybrid and taxed are separate launch types

**Status: Accepted (decided by Barton, 2026-09-24 2:45 PM MT).** Hybrid = classic SPL Token, untaxed, convertible,
exact unwrap. Rewards = Token-2022 taxed, funds the lottery, not convertible.

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

- **Decision (proposed).**
  - `withdraw_withheld_authority` = `fee_treasury` PDA `["vault_authority", config]`.
  - `transfer_fee_config_authority` = `None` at launch, unless Barton wants adjustable tax (then it's a bounded,
    timelocked PDA, see Q4).
  - Admin = Squads multisig, with no withdraw power.
- **Consequences.** No person can take the tax. Tax rate is immutable by default.

## ADR-006: Lottery fairness primitives

- **Decision.**
  - `tickets = floor(balance / threshold)`, implemented and property-tested.
  - Time-weighted stake/registration with `min_holding_seconds` in 24h..90d (validated at init).
  - VRF only.
  - One-way draw state machine with pinned VRF account, permissionless settle, and pull-based claims.
- **Consequences.** Holders must opt in (register/stake) to earn tickets. That's a UX cost, but it avoids a transfer
  hook (Q2).

## ADR-007: Test strategy

- **Decision.** LiteSVM integration tests per program (`programs/*/tests/`), Rust unit tests for math, and a localnet
  smoke test (`tests/localnet-smoke`) that creates a real Token-2022 TransferFee mint and drives both programs through a
  real `solana-test-validator`. `anchor test --validator legacy` (Anchor 1.2 defaults to surfpool, which isn't
  installed).
- **Consequences.** Planned 🧪 tests in THREAT_MODEL.md become the backlog. Fuzzing (Trident or honggfuzz) and CU
  benchmarks come before audit.

## ADR-008: Cosmetic-rarity hybrid collections use a custom `hybrid_vault` (VRF selection), not MPL-Hybrid

**Status: Proposed.** Full design: [hybrid-rarity-and-assignment.md](hybrid-rarity-and-assignment.md).

- **Context.**
  - Barton chose cosmetic rarity: every NFT redeems for exactly `ratio` tokens.
  - Ratios are 10k/50k/100k/200k/1M, and collection size is a separate creator setting.
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
- **Decision (proposed).**
  1. **Supply check:** `collection_size × ratio ≤ 1B` via `checked_mul` in base units in `hybrid_vault::initialize`,
     mirrored in the frontend. Max sizes are 100,000 / 20,000 / 10,000 / 5,000 / 1,000. Ratio, mint and size are
     immutable.
  2. **Rarity assignment:** the creator commits a Merkle root of the full trait list before launch. A VRF seed drawn at
     lock keys a Feistel permutation from index to list position. Metadata is bound on-chain at first exit with a
     Merkle proof, the collection is created by the program with no metadata-update path, and a public verify script
     is published.
  3. **Selection:** two-step VRF. `request_capture`/`request_reroll` lock payment. Permissionless `settle` happens in
     strict FIFO order over a sequenced pool (only deposits with `seq < request.seq` are candidates). There's no cancel
     after fulfilment, and `expire` applies only to unfulfilled requests past the deadline. `release` is instant and
     exact.
  4. **Re-roll:** VRF-random *different* NFT (the hand-in is excluded from its own draw). The fee is a token bps of the
     ratio plus a SOL cost fee, and `capture_fee ≥ reroll_fee` is enforced. Fees are bounded and timelocked, and each
     request is charged the fee in force when it was made. The destination is fixed at init.
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

---

## Open questions for Barton

1. **VRF provider.** I lean toward **Switchboard On-Demand**: actively maintained, devnet queue available, slashing,
   and the `solana-v3` host build compiles against Anchor 1.2. It needs a strict commit–reveal design, and its crate
   currently fails the SBF build with Anchor 1.2 (a getrandom transitive dependency), so it needs a dependency spike or
   manual account parsing. **ORAO** is simpler (nodes fulfil, so there's no withheld reveal) at about 0.001 SOL per
   request, but its CPI crate pulls anchor-lang 0.32 and doesn't compile against Anchor 1.2, so it needs a hand-rolled
   CPI. Which do you prefer?
2. **Anti-sniping mechanism.** Is opt-in registration/stake with time-weighted tickets OK, versus a transfer hook
   that would break permissionless Meteora pools and cost CU on every transfer?
3. **Prize sourcing.** Should Rewards treasuries buy NFTs from our own Hybrid collections by default? (The launch-types
   question is resolved: ADR-004 was accepted by Barton on 2026-09-24.)
4. **Tax parameters.** Default bps and `maximum_fee`. Should the tax be immutable (`fee_config_authority = None`) or
   adjustable within a cap under a timelock?
5. **SBPF v2 vs v3.** OK to stay on v2 until we bump Rust and litesvm?
6. **Stonk.fun incident.** I couldn't find a credible public report of a bot draining tokens or liquidity via its
   distribution layer (Auditor B couldn't either). Can you share the source, X thread, or tx signatures?
7. **MPL-Hybrid audit.** It's marked "Audit Pending". Is it acceptable to rely on it if our audit scope includes our
   configuration, or should we ask Metaplex for audit status first?
8. **Multisig.** Squads v4 for admin, upgrade, and escrow authorities: who are the signers, and what's the threshold?

### New (hybrid rarity, ADR-008)

9. **Q-H1:** Accept replacing MPL-Hybrid with a custom `hybrid_vault` for hybrid launches? This reverses "no custom
   404" and adds it to audit scope.
10. **Q-H2:** Capture/re-roll fees as a % of the ratio in tokens (default 2%) plus a small SOL cost fee, instead of the
    mock's SOL-only 0.01/0.02?
11. **Q-H3:** Re-roll/capture fee destination: creator, platform, or a split (recommended)? Confirm no burn, which keeps
    "exactly 1B" literally true.
12. **Q-H4:** OK to require capture fee ≥ re-roll fee, with unwrap free of token fees?
13. **Q-H5:** Lazy minting (first capturer pays ~0.0016 SOL rent per NFT) or creator pre-mints (≈1.6 SOL per 1,000
    NFTs)?
14. **Q-H6:** Forbid creator/team NFT allocations outside the pool?
15. **Q-H7:** Show full pool contents/traits publicly, or only the census?
16. **Q-H8:** Is two-transaction capture/re-roll (a few seconds of "drawing") acceptable UX?
17. **Q-H9:** Keep the 100,000-NFT maximum (~2 SOL pool-account rent) or cap lower?
