# Token-2022 transfer tax vs. exact NFT unwrap

Status: **RESOLVED (proposed), pending Barton sign-off and review by Auditors A and B.**
Owner: Solana Program Engineer. Date: 2026-09-24.
Resolves the BRIEF open issue "Token-2022 transfer fee vs. exact unwrap".

## TL;DR

- **(d) MPL-Hybrid does not support Token-2022 mints at all.** Metaplex's FAQ says so, and the program hardcodes the
  classic SPL Token program in every instruction. A taxed Token-2022 mint can't be put into an MPL-Hybrid escrow, so
  the "tax on converts" problem can't happen with MPL-Hybrid as it ships today. What remains is a product question.
- **Recommendation: option (a).** Hybrid (token ⇄ NFT) launches use a classic SPL Token mint with **no transfer tax**:
  1B fixed supply, mint and freeze authority revoked, ratio divides 1B, unwrap returns exactly N. **Taxed launches
  (Token-2022 transfer fee → fee_treasury → NFT buys → holder_lottery) are a separate launch type with no converter.**
  The two types can still be linked: a taxed launch's treasury can buy NFTs from hybrid collections as lottery prizes.
- Why each other option loses:
  - **(b) custom gross-up escrow:** it's a custom 404/swap program (needs its own audit), and whoever pays the unwrap
    fee is the problem. If the escrow pays, every wrap/unwrap round trip drains backing. If the user pays, unwrap isn't
    "exact".
  - **(c) transfer hook with an exempt-escrow allowlist:** a transfer hook **cannot collect a tax**. It gets read-only
    accounts and no signer or owner authority, and it can't move the user's tokens. On top of that it breaks
    permissionless Meteora pools and needs an Orca TokenBadge.
  - **(e) refund the fee from withheld balances:** still a custom program. It also concentrates the mint-wide
    withdraw-withheld authority in a hot swap path, and the same-transaction refund is unverified.
  - **(f) SPL Token-Wrap twin (taxed T22 → untaxed wrapped token → MPL-Hybrid):** technically exact on the NFT leg and
    audited, but it creates a **tax-free twin token** that anyone can trade instead of the taxed one. That dissolves the
    tax base that funds the lottery. It also has three-asset UX, and Token-Wrap isn't deployed on devnet at its
    documented ID.
- Truthful converter copy (hybrid launches only) is in [User-facing copy](#user-facing-copy). The mockup line
  `Transfer tax on converts: None*` may ship **only** on hybrid launches, with the asterisk resolved into real text.

---

## (d) Does MPL-Hybrid support Token-2022? No.

Source checked: https://github.com/metaplex-foundation/mpl-hybrid at commit
`aacf1a53c8a43bf395db7b562c02cf016ce604c5` (2026-05-27, latest on `main` when cloned 2026-09-24). Paths are relative to
`programs/mpl-hybrid/src/`.

| Evidence | Location |
|---|---|
| Every instruction takes `token_program: Program<'info, Token>`. Anchor's `anchor_spl::token::Token` type only accepts the classic program `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`, so passing Token-2022 fails account validation | `instructions/capture.rs:95`, `capture_v2.rs:106`, `release.rs:94`, `release_v2.rs:105`, `init_escrow.rs:60`, `init_recipe.rs:62`, `migrate_tokens_v1.rs:66` |
| The fungible mint is typed `Account<'info, Mint>` from `anchor_spl::token`. That type checks the owner is classic SPL Token, not `token_interface::Mint` | e.g. `capture_v2.rs:75`, `migrate_tokens_v1.rs` (`token: Account<'info, Mint>`) |
| Transfers use legacy `token::transfer`, not `transfer_checked`. Token-2022 rejects plain `Transfer` for mints with the TransferFeeConfig extension | `capture.rs:250,261`; `capture_v2.rs:284,296`; `release.rs:241`; `release_v2.rs:287,311`; `migrate_tokens_v1.rs:127`; burn `capture_v2.rs:271` |
| ATA creation is hardcoded to `&spl_token::ID` | `utils.rs:18-22` |
| `validate_token_account` rejects any token account not owned by `spl_token::ID` | `utils.rs:36-41` (`if account.owner != &spl_token::ID { return Err(InvalidTokenAccount) }`) |
| `spl-token-2022` appears in `Cargo.toml` but isn't used by any instruction | `programs/mpl-hybrid/Cargo.toml:32` |

Official docs agree. Metaplex's MPL-Hybrid FAQ says MPL-404 "only supports Metaplex Core NFT assets and SPL Token
fungible assets", and support for Token Extensions is future work:
https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/faq. The repo README also says **"Security: Audit Pending"**.
The idea that MPL-Hybrid is "audited" is not correct as of this commit.

Other MPL-Hybrid facts that matter for this decision (same commit):

- **The escrow authority can change the swap amount at any time.** See `update_escrow.rs:113` (`escrow.amount = amount`),
  `update_recipe.rs:115`, and `update_new_data.rs:103`. There's no timelock. "Exactly 1,000,000" is only true while the
  escrow authority is unable or unwilling to change it. If the authority is compromised, it can capture NFTs cheaply,
  raise `amount`, then release to drain the escrow's backing tokens. This is the MPL-Hybrid version of the
  Stonk.fun-style distribution-layer risk. See [THREAT_MODEL.md](THREAT_MODEL.md) T-HY-01.
- **V2 escrows are seeded by authority (`["escrow", authority]`), not by collection.** One authority running several
  recipes that use the same token shares one token pool. Use one dedicated authority per hybrid collection.
- **The capturer chooses which escrowed NFT they get.** In `capture_v2.rs:52-54` the `asset` is a caller-supplied
  account. Rerolls use SlotHashes plus a timestamp (`capture_v2.rs:95,190-199`), which is predictable, and Auditor B's
  B-01/R-01 already flags this. Hybrid collections should either set the `NoRerollMetadata` path and have traits that
  don't differ in value, or disclose that rarer NFTs can be cherry-picked.
- **Protocol fee** is about 0.005 SOL per capture and per release, sent to Metaplex
  (https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/protocol-fees). The project can add its own token fee
  (`fee_amount_capture/release`) and SOL fee.
- Program `MPL4o4wMzndgh8T1NVDxELQCj5UQfYTYEkabX3wNKtb` is deployed and executable on devnet (checked read-only via
  `getAccountInfo` on 2026-09-24).

**Conclusion for (d):** with MPL-Hybrid, a taxed Token-2022 hybrid is impossible without forking MPL-Hybrid. A fork is
a custom swap program, which is option (b) with extra steps.

---

## Background: how the Token-2022 transfer fee behaves

Sources: https://solana.com/docs/tokens/extensions/transfer-fees, https://www.solana-program.com/docs/token-2022/extensions,
and `spl-token-2022-interface` 2.1.0 `src/extension/transfer_fee/mod.rs`.

- `fee = min(ceil(amount × bps / 10_000), maximum_fee)` (`calculate_fee`, ceiling division). Every
  `transfer_checked` pays it. There's **no per-account exemption**. The fee is withheld *in the recipient's token
  account*, so the recipient receives `amount − fee`.
- Anyone can move withheld amounts to the mint (`HarvestWithheldTokensToMint`). Only
  `withdraw_withheld_authority` can withdraw them (`WithdrawWithheldTokensFromMint` / `FromAccounts`).
- `SetTransferFee` (signed by `transfer_fee_config_authority`) takes effect **two epochs later**
  (`get_epoch_fee(epoch)` picks `older` vs `newer_transfer_fee`), so a transaction always sees one deterministic fee.
- `transfer_checked_with_fee` makes the caller state the expected fee. It fails if the expected fee doesn't match, so a
  stale quote reverts rather than delivering a different amount.
- Helpers exist for inverse math: `calculate_pre_fee_amount`, `calculate_inverse_fee`, and
  `calculate_inverse_epoch_fee` (mod.rs:87-160).

So any design where the user receives **exactly N** of a taxed mint in one transfer requires someone to send
`pre_fee_amount(N) > N`. The whole question is **who funds the difference.**

---

## Option (a): hybrid collections use a non-taxed mint. Taxed T22 is a separate launch type. **RECOMMENDED**

**Design**

- *Hybrid launch type*: classic SPL Token mint with 1,000,000,000 × 10^decimals supply, all minted once at launch, then
  `SetAuthority(MintTokens → None)` and `SetAuthority(FreezeAccount → None)`. It gets a Metaplex Core collection and an
  MPL-Hybrid escrow/recipe. The ratio `R` must satisfy `1_000_000_000 % R == 0` (e.g. R = 1,000,000 gives at most
  1,000 NFTs). Don't use the `BurnOnCapture`/`BurnOnRelease` paths, because burning would take supply below 1B and
  break the backing invariant.
- *Taxed launch type*: Token-2022 mint with TransferFeeConfig (plus MetadataPointer/TokenMetadata), the same fixed 1B
  supply, and mint and freeze authority revoked. `withdraw_withheld_authority` = fee_treasury PDA.
  `transfer_fee_config_authority` = `None`, or a PDA with hard bounds and a timelock (Auditor B R-04). **No converter.**
  Tax funds NFT purchases (including from our hybrid collections), which go to holders through holder_lottery.

**Why it wins**

- It needs **zero custom swap code.** Hybrid uses MPL-Hybrid as-is, and taxed uses stock Token-2022. Our custom
  surface stays at fee_treasury + holder_lottery, which is where the BRIEF wants the most scrutiny anyway.
- "Exact unwrap" is literally true, because classic SPL `transfer` of N delivers N.
- It keeps the escrow solvent by construction. MPL-Hybrid moves the same `amount` in both directions with no fee leak.
- It works with every DEX. Classic SPL tokens are supported by every Raydium pool type, Meteora, Orca, and Jupiter.

**Trade-offs and UX**

- A hybrid token has no built-in tax, so hybrid launches can't self-fund a lottery from transfers. If a hybrid creator
  wants revenue, the options are MPL-Hybrid's own swap fees (`fee_amount_*` / `sol_fee_amount_*`, shown before signing)
  or bonding-curve/LP fees. Those are disclosed per swap, not hidden in every transfer.
- The launch wizard must make users pick a type up front: "Hybrid (token ⇄ NFT, no tax)" vs "Rewards (transfer tax
  funds NFT lottery, no converter)". A token can't switch types after launch.

**Exploit surface (what remains)**

- MPL-Hybrid escrow authority: it can change `amount`/fees with no timelock, so it's an admin drain vector (T-HY-01).
  Mitigation: the authority is a per-collection Squads multisig with a timelock. Later, the authority becomes a PDA of a
  tiny audited "escrow-governor" program that only exposes timelocked, bounded updates. Also run a monitor that alerts
  on any `UpdateEscrow`/`UpdateRecipe`.
- Backing: before the first release, the escrow must hold `R × (NFTs in circulation)` tokens. The invariant
  `escrow_token_balance ≥ R × nfts_outside_escrow` is checked in tests and monitored.
- Reroll and cherry-pick randomness is weak (SlotHashes, caller-chosen asset). Mitigation: `NoRerollMetadata` and
  value-uniform NFTs, or disclose.
- MPL-Hybrid is **audit pending**. Our pre-mainnet audit scope must include our MPL-Hybrid configuration, and ideally
  Metaplex publishes an audit first.

## Option (b): custom escrow/wrap program that grosses up the fee

**Design sketch.** A fork or rewrite of MPL-Hybrid that uses `token_interface` and `transfer_checked_with_fee`. On
unwrap it sends `G = calculate_pre_fee_amount(N)` at the current epoch's fee, so the user nets exactly N. On wrap it
either requires the user to send `pre_fee_amount(R)` so the escrow nets exactly R, or it credits whatever arrived.

**Who pays the fee?** There are only three payers, and each one breaks something:

1. **The escrow pays** (sends G, but recorded backing per NFT is R). Each release leaks `G − R`. Cycling
   wrap → unwrap → wrap costs the attacker their own wrap-leg tax, but the escrow pays the unwrap-leg tax every time.
   Across `k` cycles the escrow loses `k × fee(G)`. Once it's under-collateralized, **the last NFT holders can't
   unwrap** (bank run). A griefer with enough capital can accelerate this, and a bot can drain it whenever the
   treasury "tops up" the escrow. This is exactly the drain pattern the BRIEF warns about.
2. **The user pays on wrap** (a wrap costs `pre_fee(R) + pre_fee(pre_fee(R))`-style prepayment to pre-fund the future
   unwrap tax). This is solvent, but the wrap price then isn't "1,000,000 tokens". It also depends on the fee **at
   unwrap time**, which the admin can change (after the 2-epoch delay) between wrap and unwrap, so prepaid tax can be
   wrong in either direction. Over-collection is a hidden fee, and under-collection is insolvency.
3. **The user pays on unwrap** (they receive N − fee). That's honest, but it isn't "exact". It's what vanilla Token-2022
   does anyway, so building a custom program for it isn't justified.

**Rounding and cap edge cases**

- `calculate_pre_fee_amount` has a non-unique inverse. Several pre-fee amounts can map to the same post-fee amount
  because of ceiling rounding, and the program must pick the minimum. At `bps = 10_000`, the inverse is only defined
  through `maximum_fee`. When the cap binds, `G = N + maximum_fee` exactly. When `maximum_fee = 0` or `bps = 0`,
  `G = N`.
- The fee must be read from the mint **inside the instruction** for `Clock::epoch` (`calculate_inverse_epoch_fee`),
  never taken from the client. A quote built in epoch E and landing in E+1 after a scheduled fee change would otherwise
  deliver the wrong amount. `transfer_checked_with_fee` turns that into a revert, which is safe but surprising for
  users.
- `u64` overflow on `amount × bps` is handled internally with u128, but any custom accounting around it needs checked
  math.

**Exploit surface.** A new custom program holding pooled backing is a new place to hide a drain:
fee-accounting mistakes, PDA confusion across collections, stale-epoch math, and admin fee changes that make the
escrow leak. It needs its own audit, fuzzing (Auditor B R-06: 10,000 cycles at 100 bps), and monitoring.

**Verdict: reject.** It violates the "no custom 404/swap program" decision (ADR-003), and every payer choice either
leaks escrow backing (a drain/bank run) or makes "exact" untrue.

## Option (c): transfer hook with an exempt-escrow allowlist instead of the fee extension

**Can a transfer hook collect a tax? No.** Per the Solana transfer-hook docs
(https://solana.com/docs/tokens/extensions/transfer-hook), Token-2022 CPIs into the hook with every transfer account
**de-escalated to read-only**, and the sender's signer privileges **don't extend** to the hook. The hook gets the
source, mint, destination, and owner as read-only, plus whatever extra accounts are in its ExtraAccountMetaList. It
therefore:

- can't debit the source token account (no owner/delegate signature, and the accounts are read-only);
- can't CPI back into Token-2022 to move the *same* token (re-entrancy into the token program during a transfer isn't
  allowed);
- can't change the transfer amount.

The only fee pattern in the official docs collects a **separate** token (wSOL) from a delegate the sender approved
beforehand. That's a side payment the user must set up, not a tax on the token. A hook can **allow/deny** transfers
(e.g. an allowlist, or blocking transfers during a sale window) and **record** things (e.g. holding-time checkpoints
for the lottery). It can't tax.

So "hook with exempt-escrow allowlist instead of the fee extension" means the token has **no tax**. That collapses to
option (a) with extra cost and risk. "Hook plus fee extension" doesn't help either, because the fee extension still
charges the escrow, and a hook can't refund it.

**Compatibility and cost, even as a non-tax hook**

- DEXs:
  - Meteora DAMM v2/DLMM allow permissionless pools for Token-2022 only if the transfer-hook program and authority are
    unset, and DAMM v2 doesn't forward hook accounts
    (https://docs.meteora.ag/core-products/damm-v2/token-2022-support, https://docs.meteora.ag/core-products/dlmm/token-2022-support).
  - Orca Whirlpools require a TokenBadge for TransferHook mints
    (https://docs.orca.so/developers/architecture/token-extensions).
  - Raydium AMM v4 and LaunchLab base mints don't support Token-2022 at all
    (https://docs.raydium.io/algorithms/token-2022-transfer-fees).
- Wallets and aggregators must resolve extra accounts for every transfer. Failures show up as "transaction failed", and
  the hook adds CPI compute units to **every** transfer, including DEX swaps.
- The hook isn't called on self-transfers. A hook-controlled allowlist is also an admin kill switch, because the hook
  authority can freeze trading. Auditor B R-03 requires no such authority.
- MPL-Hybrid doesn't pass hook extra accounts, and it doesn't support Token-2022 anyway.

**Verdict: reject.** It can't implement a tax, and it costs DEX compatibility, compute, and an admin kill switch.

## Option (e): refund the fee out of withheld balances (fee "withheld at destination and harvested back")

**Idea.** The fee is withheld in the *recipient's* account. A custom swap program that holds
`withdraw_withheld_authority` could, in the same transaction as the escrow → user unwrap transfer, call
`WithdrawWithheldTokensFromAccounts` on the user's ATA and send the withheld fee back to the user. The net would be
exactly N.

**Problems**

- It's still a custom swap program, and it must hold the **mint-wide** withdraw-withheld authority. That authority can
  withdraw fees withheld in *any* holder account. Putting it in a user-facing, high-frequency path is the concentration
  Auditor B B-04/R-04 warns about. It would also compete with fee_treasury, which should be the only holder.
- The user's ATA may already hold withheld fees from earlier trades. Withdrawing "all withheld" refunds those too, which
  leaks tax. Withdrawing an exact amount isn't supported (`FromAccounts` withdraws the account's full withheld balance),
  so the program would have to harvest first, which is permissionless and can be front-run.
- The fee is refunded *to the destination*. Whether withdraw-to-the-same-account works in one instruction is
  **unverified**, and no reference implementation exists.
- The wrap leg still needs the escrow to receive exactly R. That means refunding into the escrow's own account (same
  issues) or a user gross-up.

**Verdict: reject.** It's a custom program plus concentrated fee authority, it leaks tax, and it's unverified.

## Option (f): SPL Token-Wrap twin (taxed Token-2022 → untaxed wrapped classic token → MPL-Hybrid)

**How it works.** SPL Token-Wrap (`TwRapQCDhWkZRrDaHfZGuHxkZ91gHDRkyuzNqeU5MgR`, https://github.com/solana-program/token-wrap,
checked at `a7cedd3`, 2026-09-21) creates a deterministic wrapped mint for any token. It's audited by Zellic
(2025-05-16) and Runtime Verification (2025-06-11 and 2025-10-30), per its README. `Wrap` sends the taxed token into a
PDA escrow with `transfer_checked_with_fee` and mints **`amount − fee`** wrapped tokens (`program/src/processor.rs:256-279`),
so it's always 1:1 backed by what actually arrived. `Unwrap` burns N wrapped and sends N of the taxed token back, and
the user receives `N − fee`. The wrapped token can be a classic SPL Token mint, which MPL-Hybrid accepts, so
NFT ⇄ 1,000,000 wTOKEN would be exact.

**Why it loses**

- **Tax bypass.** The wrapped token has no tax and is freely transferable and tradable. Anyone can wrap once (paying
  tax once) and then trade or LP the untaxed wTOKEN forever. Liquidity migrates to the untaxed twin, and the lottery's
  funding source shrinks toward zero. That defeats the purpose of the taxed launch type.
- **UX.** Three assets (TOKEN, wTOKEN, NFT) with a taxed hop between the first two. "Exact" only holds on the
  NFT ⇄ wTOKEN leg, which is hard to explain honestly.
- **Supply.** wTOKEN supply floats, so "1B fixed supply" becomes a statement about TOKEN only.
- **Deployment.** The program ID is **not deployed on devnet** (read-only `getAccountInfo` returned null on
  2026-09-24), so we'd have to deploy our own instance for testing.

**Verdict: don't adopt for v1.** Keep it as a documented future opt-in if Barton explicitly accepts the tax-bypass
trade-off for a specific collection.

## Option (g): "fee on wrap only" / charge the conversion fee explicitly

This means a non-taxed hybrid mint where the only fee is an explicit, disclosed swap fee charged by MPL-Hybrid
(`fee_amount_capture` in tokens and/or `sol_fee_amount_capture` in SOL, sent to a treasury). It's a **variant of (a)**,
not a separate architecture. It gives hybrid creators a revenue stream without a transfer tax, and it keeps unwrap
exact. Fees are shown in the confirmation step, but they're changeable by the escrow authority (see T-HY-01), so the
authority must be timelocked. Recommended as the monetization knob for hybrid launches.

---

## Recommendation

**Adopt (a) + (g).**

1. Two launch types, chosen at creation and immutable:
   - **Hybrid**: classic SPL Token, 1B fixed supply, no transfer tax, and MPL-Hybrid converts at a ratio that divides
     1B. Unwrap returns exactly R tokens. There's an optional disclosed per-swap token/SOL fee, and a mandatory Metaplex
     protocol fee of about 0.005 SOL.
   - **Rewards (taxed)**: Token-2022 transfer fee, 1B fixed supply, fee → fee_treasury → NFT purchases (hybrid
     collections are eligible prize sources) → holder_lottery. **No token ⇄ NFT converter.**
2. MPL-Hybrid escrow authority per hybrid collection is a dedicated Squads multisig, with a timelock if one is available.
   Longer term, it moves to a PDA of a minimal audited governor with bounded, timelocked `amount`/fee changes. A
   monitor alerts on escrow updates.
3. Revisit if Metaplex ships Token-2022 support in MPL-Hybrid. Even then, the fee-leak math in (b) still applies, so
   exact unwrap for a taxed mint would still need a payer.

## User-facing copy

Hybrid launch, convert panel (replaces `Transfer tax on converts: None*`):

> **Transfer tax on converts: None.** This is a standard Solana token with no transfer tax.
> Converting is exact: 1 NFT ⇄ 1,000,000 $TICKER, both ways.
> You pay Solana network fees and Metaplex's protocol fee (about 0.005 SOL per convert)
> {if fee configured:}, plus this project's convert fee of {X $TICKER / Y SOL}, shown before you sign.
> Convert amounts and fees are set by the collection's escrow authority ({multisig address}); changes are
> {timelocked for N hours / announced on-chain}.

(If the creator fee is 0, drop that clause. Don't claim "exact" or "None" for any other launch type.)

Rewards (taxed) launch, where the convert panel would be:

> **This token has a {B}% transfer tax** (max {M} per transfer), set at launch and paid on every transfer, including
> buys and sells. The tax funds NFT prizes for holders. Rewards tokens can't be converted into NFTs.

Rewards launch, tooltip if the tax authority isn't revoked:

> The tax rate can be changed by {authority}, capped at {cap}%. Changes take effect at least 2 Solana epochs
> (~4 days) after they're announced on-chain.

## References

- MPL-Hybrid source: https://github.com/metaplex-foundation/mpl-hybrid (commit aacf1a53c8a43bf395db7b562c02cf016ce604c5)
- MPL-Hybrid FAQ: https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/faq
- MPL-Hybrid create escrow: https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/create-escrow
- MPL-Hybrid protocol fees: https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/protocol-fees
- Transfer fees: https://solana.com/docs/tokens/extensions/transfer-fees, https://www.solana-program.com/docs/token-2022/extensions
- Transfer hook: https://solana.com/docs/tokens/extensions/transfer-hook
- SPL Token-Wrap: https://github.com/solana-program/token-wrap, https://www.solana-program.com/docs/token-wrap
- DEX support: https://docs.raydium.io/algorithms/token-2022-transfer-fees, https://docs.meteora.ag/core-products/damm-v2/token-2022-support,
  https://docs.meteora.ag/core-products/dlmm/token-2022-support, https://docs.orca.so/developers/architecture/token-extensions
- Wallets: https://docs.phantom.com/developer-powertools/solana-token-extensions-token22 (Phantom shows transfer fees; Solflare fee display unverified)
- Auditor B requirements: ../security/auditor-b/design-requirements.md (R-01, R-03, R-04, R-06)
