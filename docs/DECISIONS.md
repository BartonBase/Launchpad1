# Architecture decision records (on-chain)

Owner: Solana Program Engineer. Format: context → decision → consequences. Status values: Proposed / Accepted /
Superseded. Every decision here is **Proposed** until Barton signs off.

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

- **Context.** Custom swap/fee code that holds pooled value is the biggest bug and drain surface, and each one needs
  its own audit.
- **Decision.** Hybrid swaps use **Metaplex MPL-Hybrid**. Transfer tax uses the **Token-2022 TransferFee extension**.
  Custom code is limited to `fee_treasury` and `holder_lottery`.
- **Consequences.** We inherit MPL-Hybrid's limits: classic SPL Token only, no timelock on escrow updates, SlotHashes
  rerolls, and **Audit Pending** (repo README at `aacf1a5`). All of those are documented in THREAT_MODEL.md T-HY-*.

## ADR-004: Transfer tax vs exact unwrap: hybrid and taxed are separate launch types

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
3. **Launch types.** Do you accept ADR-004 (Hybrid = no tax, Rewards = tax with no converter)? Do you want Rewards
   treasuries to buy from our own Hybrid collections by default?
4. **Tax parameters.** Default bps and `maximum_fee`. Should the tax be immutable (`fee_config_authority = None`) or
   adjustable within a cap under a timelock?
5. **SBPF v2 vs v3.** OK to stay on v2 until we bump Rust and litesvm?
6. **Stonk.fun incident.** I couldn't find a credible public report of a bot draining tokens or liquidity via its
   distribution layer (Auditor B couldn't either). Can you share the source, X thread, or tx signatures?
7. **MPL-Hybrid audit.** It's marked "Audit Pending". Is it acceptable to rely on it if our audit scope includes our
   configuration, or should we ask Metaplex for audit status first?
8. **Multisig.** Squads v4 for admin, upgrade, and escrow authorities: who are the signers, and what's the threshold?
