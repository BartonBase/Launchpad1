# Solana NFT Launchpad: Design-Level Adversarial Threat Model (Auditor B)

**Prepared for:** Barton
**Date:** 2026-09-24 (rev. 2, same day: sourced from `research-sources.md`)
**Status:** Design review only. No program code exists yet (the public repo `BartonBase/Launchpad1` was checked on 2026-09-24 at 14:29 MDT and is empty, with no commits; see `code-review-no-commits.md`), so every finding is conditional: "if the design does X, then Y". Findings have to be checked again against real code.
**Method:** Rev. 1 was written from the reviewer's own knowledge. Rev. 2 updates it with the primary sources recorded in `research-sources.md` (MPL-Hybrid source at commit `aacf1a53`, Token-2022 source at commit `8867f751`, Solana/SPL/Jito docs, the pump.fun post-mortem coverage, and the MELT and IMC 2025 sandwich datasets). Claims that are still unsourced are marked **[needs source]**; everything research-sources.md covers has been resolved and cited inline. All attacker costs are **estimates** that use the example parameters from the sims. They are not measurements.

## System under review (as described)

1. **Swap layer:** Anchor programs built on Metaplex **MPL-Hybrid**. It escrows NFTs and swaps fungible tokens for NFTs ("capture") and NFTs for tokens ("release"). Metadata can optionally be rerolled on swap.
2. **Token:** a **Token-2022** mint with the **transfer-fee extension**. Collected (withheld) fees fund NFT purchases.
3. **Holder lottery:** ticket-based, using **VRF randomness** (Switchboard or ORAO).

## Simulation evidence used

| Sim | File | Key numbers |
|---|---|---|
| Reroll EV + predictability | `sim/reroll_ev_output.txt` | Honest reroll cycle = **0.0501 SOL** (classic SPL) / **0.0701 SOL** (100 bps T22 fork). Honest legendary grind (p=0.1%, 60 SOL value) EV = **+8.90 SOL** (profitable) with no T22 fee, **-11.10 SOL** with a 100 bps fee. Breakeven premium is 50.10 / 70.10 SOL. **With the predictable pick, a conditional bot needs about 994 slots (~6.6 min) and one landed cycle (~0.05 SOL) per legendary. That is about 1000x cheaper, with EV about +58.95 SOL per grab.** Pick range 1024/4096 gives **1 distinct pick** over 5,000 random slot hashes, so the hash contributes nothing. Range 10,000 gives only **625** distinct picks. |
| Lottery sybil + flash snapshot | `sim/lottery_sybil_output.txt` | Attacker holds 0.5% of supply. With **flat-per-wallet tickets, a split into k=1000 wallets gives 820.9x the share (17.93% win chance vs 0.0218%)**. log10 gives 465.2x, sqrt 29.5x, capped-linear 4.9x, and linear 1.0x (neutral). Splitting into 1000 wallets costs about 0.05 SOL in fees plus 2.16 SOL of rent, which comes back when the accounts are closed. **A flash position (50M tokens held for 2 slots) gets a 7.69% share at a point snapshot vs 0.000077% under TWAB.** The point-snapshot attack breaks even when the prize is above about 13 SOL. |

Sim caveat: `reroll_ev.py` was truncated. It had no return value, no Part B, and no `main`, so it printed nothing. Auditor B finished it minimally (the original is kept as `sim/reroll_ev.py.orig`). Part B models the MPL-Hybrid capture-time metadata pick (SlotHashes bytes 12..20, unix timestamp, and `escrow.count`). **Rev. 2: this model is now confirmed against the upstream source**, [capture.rs L167-183 at commit `aacf1a53`](https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture.rs#L167-L183). The expression matches line for line, and an independent u64 re-implementation reproduces the sim's 1 / 625 distinct-pick counts (research-sources.md §5). Caveat: the sim holds `count` fixed across slots; in reality every capture or release by anyone increments it, which just adds another public input. A local-validator PoC is still recommended as a regression test, but the reading is no longer an assumption.

## Severity scale

- **Critical:** direct theft of funds or NFTs, a fully rigged lottery or rarity outcome, or a single key that can rug users.
- **High:** a large economic advantage for attackers, or loss of funds under realistic conditions.
- **Medium:** fairness or availability harm that can be bounded, or a loss that needs extra conditions.
- **Low / Info:** hardening, operational or process issues.

## Findings summary

| ID | Severity | Title |
|---|---|---|
| B-01 | Critical (confirmed from source) | Predictable reroll / capture metadata pick enables reroll-until-rare at ~1000x lower cost |
| B-02 | Critical | Lottery randomness manipulation (slot-hash RNG, reveal-and-abort, unbound request/fulfil, draw trigger) |
| B-03 | Critical | Upgrade-authority, mint-authority, freeze and metadata-authority rug |
| B-04 | Critical | Token-2022 fee authority abuse (withdraw-withheld and fee-config authorities) |
| B-05 | High | Treasury / escrow drain through weak account validation or accounting |
| B-06 | High | Token-2022 fee bypass and escrow insolvency on wrap/unwrap paths |
| B-07 | High | Price manipulation of fee-funded NFT purchases |
| B-08 | High | Flash-buy / borrow before a point-in-time lottery snapshot |
| B-09 | High | Sybil splitting under per-wallet or concave ticket formulas |
| B-10 | High | Launch sniping and front-running (Jito bundles, first-slot buys) |
| B-11 | Medium | Sandwich / MEV on swaps and on protocol-side buys |
| B-12 | Medium | Wrap/unwrap arbitrage, inventory sniping and honest reroll grinding |
| B-13 | Medium | Griefing / DoS: unbounded holder iteration, dust accounts, compute limits |
| B-14 | Low | Lottery liveness: stuck draws and unrecoverable prize escrow |
| B-15 | Info | Dependency pinning and program-ID verification for MPL-Hybrid, VRF and Token-2022 |
| B-16 | Low / Info | MPL-Hybrid reroll range is `[min, max)`: index `max` is never picked, so following the docs orphans the last metadata file |

---

## Findings

### B-01: Predictable reroll / capture metadata pick enables reroll-until-rare. Severity: **Critical**

**Condition:** The design uses MPL-Hybrid's built-in capture-time metadata pick, or any pick derived from `SlotHashes`, `Clock` (slot / unix_timestamp), `escrow.count`, a recent blockhash, or other values that are public before the transaction executes.

**Status: Critical, confirmed from source.** MPL-Hybrid computes the reroll index in [capture.rs L167-183 at commit `aacf1a53`](https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture.rs#L167-L183) (the latest `main` commit when checked on 2026-09-24):

```rust
let most_recent = array_ref![data, 12, 8];          // SlotHashes sysvar bytes 12..20
let seed = u64::from_le_bytes(*most_recent)
    .saturating_sub(clock.unix_timestamp as u64)
    .wrapping_mul(escrow.count);
let remainder = seed.checked_rem(escrow.max - escrow.min)
    .ok_or(MplHybridError::RandomnessError)? + escrow.min;
```

In words: SlotHashes minus `unix_timestamp`, times `count`, mod `(max - min)`, plus `min`. The account named `recent_blockhashes` is really the SlotHashes sysvar (capture.rs L83-86, constants.rs L7). `escrow.count` increments on every capture (L293) **and** release (release.rs L273), starting at 1 (init_escrow.rs L119). [capture_v2.rs L188-204](https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture_v2.rs#L188-L204) uses the same logic with `recipe.count`. `CaptureV1Ctx` has no instructions-sysvar account and no caller check; `owner` is only a `Signer`, so a wrapper program can CPI into capture. Owner and asset keys are not part of the seed. The Anza sysvar docs confirm SlotHashes "contains the most recent hashes of the slot's parent banks", so every input is fixed and readable before the capture instruction runs (research-sources.md §5).

**Why it matters:** `seed = u64_le(SlotHashes[12..20]).saturating_sub(unix_ts).wrapping_mul(count); index = seed % (max-min) + min`. The newest SlotHashes entry is the *parent* slot, so a transaction sees it before it executes. Bytes 12..20 hold the high 4 bytes of the slot number, which are zero, plus 4 bytes of hash, so the pick has **at most 32 bits of hash entropy**. If `(max - min)` divides 2^32 (for example 1024 or 4096), **the hash term cancels out completely** (sim: 1 distinct pick). For 10,000 items only 625 residues can be reached per (timestamp, count) pair.

**Attack scenario:**
1. The attacker reads the public metadata URI range and ranks rarity by index. The URIs are enumerable because MPL-Hybrid stores a URI template plus a min/max range.
2. The attacker deploys a small wrapper program. Its instruction reads SlotHashes, Clock and the escrow account, computes the index MPL-Hybrid is about to pick, and **aborts unless the index is rare**. If it is rare, it CPIs into release+capture, or capture only.
3. The attacker sends this wrapper every slot inside a Jito bundle. Jito documents bundles as "up to 5 transactions that execute sequentially and atomically" within one slot, and "if any transaction in a bundle fails, none of the transactions in the bundle will be committed to the chain" (https://docs.jito.wtf/lowlatencytxnsend/). So an aborted attempt never lands and pays no on-chain fee or tip. (The Jito docs read do not say whether a failed bundle costs anything off-chain; assume roughly zero.) Without Jito, a failed attempt costs only the base fee.
4. After about 1/p slots (sim: about 994 slots, ~6.6 min for p=0.1%), the pick is legendary and the swap lands.
5. The attacker repeats until the rare set is drained, then sells the rares on secondary markets.

**Estimated attacker cost (estimate):** about 0.05 SOL per legendary (one swap cycle plus a small tip and wrapper compute), compared with about 50.10 SOL expected for honest grinding. The sim puts EV at about **+58.95 SOL per legendary** at 60 SOL value. One-off build cost is a few engineer-days.

**Fix (program-level):**
- **Do not use MPL-Hybrid's built-in pick for any rarity-bearing reroll.** Set the escrow to the no-reroll path and do reroll inside our own program.
- Use a **two-phase commit/reveal with VRF**. Phase 1 (`request_reroll`) locks the NFT and the swap payment in a per-request PDA, records `request_slot`, the VRF request account, and the user. Phase 2 (`settle_reroll`) can run only after the VRF result is fulfilled for **that exact request**. It is **callable by anyone**, and the outcome is applied unconditionally. The user cannot cancel after the randomness exists.
- Bind the randomness to the request: `seed = hash(vrf_output || request_pda || nft_mint || nonce)`. Reject any VRF account whose seed or owner doesn't match the one stored at request time.
- Reject CPI callers on reroll instructions. Check that the instruction is top-level with the instructions sysvar, or allowlist callers. **Defense in depth only; the commit/reveal is the real fix.** A CPI check does not stop predict-and-abort: a *later top-level instruction* in the same transaction can read the written URI and revert. That is exactly the variant described in the (unverified) Candy Machine rarity-exploit write-up, and Candy Machine v2 had a CPI-caller restriction yet still used the same `array_ref![data, 12, 8]` SlotHashes read (research-sources.md §4).
- Make the index space uniform. Use rejection sampling instead of `% n` on low-entropy input, and use an inclusive range that matches the metadata files (see B-16).

**Precedent:** Metaplex's own Candy Machine docs say the slot-hash-based "random" mint order "is not entirely unpredictable and can be influenced by sufficient resources and malicious intent", and its anti-bot guide lists sequential `0.json, 1.json …` URIs as a "Predictable URI Vulnerability". MPL-Hybrid requires exactly that `<base><index>.json` layout (research-sources.md §4). No independently reported Solana incident of a randomness drain was found, so this rests on the source code, not on a loss event.

### B-02: Lottery randomness manipulation. Severity: **Critical**

**Condition:** The lottery draw does any of these: (a) takes randomness from slot hashes, blockhash, clock, or VRF output that is readable before the draw is committed; (b) lets the requester see the randomness and then decide whether to finalize; (c) does not bind the VRF request to one specific draw; (d) lets a privileged or arbitrary party choose *when* to request randomness, *which* snapshot to use, or re-request after an unfavorable result.

**Attack scenarios:**
- **Slot-hash RNG (a):** the same technique as B-01. A validator or bundle searcher computes the winner before the draw lands and only lands `draw()` when an attacker wallet wins. A leader can also choose to skip or reorder. Cost is about 0 beyond tips (estimate).
- **Reveal-and-abort (b):** with Switchboard-style commit/reveal randomness, the reveal and the consume can end up in one transaction. Then the attacker calls `reveal + settle` inside a wrapper that reverts unless the attacker wins. With an ORAO-style request/fulfil where anyone can read the result, a draw that needs a second "finalize" transaction from a privileged operator lets the operator refuse to finalize and re-request.
  1. Operator or attacker calls `request_draw`.
  2. The oracle fulfils. The result is now public in the randomness account.
  3. The attacker simulates `settle_draw` off-chain. If they lose, they never call it, or call `cancel/re-request`.
  4. They repeat until they win.
- **Unbound request/fulfil (c):** `settle_draw` accepts *any* fulfilled randomness account owned by the VRF program. The attacker creates their own VRF requests (they cost the attacker a small fee) and waits for one whose output makes them win. Then they pass that account to `settle_draw`.
- **Draw trigger and snapshot timing (d):** if the draw can be called any time after an epoch ends, the caller times it with B-08 (flash snapshot). If the admin can pick the snapshot slot after seeing balances, the admin can pick one that favors insiders.

**Supporting evidence:** Metaplex documents that its slot-hash pick is influenceable (research-sources.md §4), and B-01 shows the same pattern in MPL-Hybrid. No independently reported Solana lottery or raffle randomness exploit was found, so severity rests on mechanism, not an incident.

**Estimated attacker cost (estimate):** from about 0.001–0.01 SOL per try (VRF request fee plus tx fees) up to 0 for a malicious operator. The winning rate approaches 100% given enough tries.

**Fix (program-level):**
- Store a `Draw` PDA state machine: `Open → SnapshotFrozen → RandomnessRequested → Fulfilled → Settled`, one-way only. Include a `Cancelled/Refund` path that never re-requests randomness for the same draw.
- Freeze the ticket set (a merkle root or a finalized ticket account set, see B-13) **before** requesting randomness. Store `ticket_root` and `total_tickets` in the Draw PDA.
- At request time store the exact VRF account pubkey and seed (`seed = hash(draw_pda || draw_id || ticket_root)`). In `settle`, require `randomness_account == draw.vrf_account`, owner == pinned VRF program ID, fulfilled == true. For Switchboard commit/reveal, also require `reveal_slot > commit_slot` and that the commit happened after the freeze [needs source for the current Switchboard On-Demand API].
- `settle_draw` must be **permissionless**, and must **always** apply the result. There is no branch where the outcome can be declined. The winner is computed on-chain from the stored randomness, and the prize is moved into a winner-claimable PDA in the same instruction.
- The draw trigger is permissionless after a fixed `draw_slot` written at draw creation. The admin cannot change the snapshot window or draw slot after the draw is Open.
- Liveness: if the VRF is not fulfilled by `deadline_slot`, anyone can move the draw to `Cancelled`. Tickets roll over to the next draw. No re-roll of the same draw.

### B-03: Upgrade-authority, mint-authority, freeze and metadata-authority rug. Severity: **Critical**

**Condition:** At launch, any of these is held by a single hot key or an unconstrained team wallet: program upgrade authority, Token-2022 mint authority, freeze authority, permanent-delegate extension (if enabled), NFT collection or metadata update authority, MPL-Hybrid escrow authority.

**Attack scenario (malicious insider or compromised key):**
1. The attacker deploys a program upgrade that adds `withdraw_all` for the treasury, escrow and prize PDAs. PDAs sign for whatever code the program contains, so an upgrade can move everything the program controls.
2. Or: mint unlimited tokens, capture every NFT from escrow, and dump the tokens on the pool.
3. Or: use freeze authority to freeze holders' token accounts, or a permanent delegate to transfer or burn user tokens directly.
4. Or: change NFT metadata or rarity after sale with the collection update authority. Or change the escrow's `amount/fee` so swaps become confiscatory.

**Precedent: pump.fun, 2024-05-16.** Per pump.fun's post-mortem (quoted by The Block, https://www.theblock.co/post/295029/pump-fun-post-mortem), "a former employee, having illegitimately taken access of the withdraw authority using their privileged position at the company, used flash loans on a Solana lending protocol" at 15:21 UTC (09:21 MDT). The borrowed SOL pushed many bonding curves to 100% so the privileged withdraw/migrate path could be triggered on them. About **12,300 SOL (~$1.9M)** was taken out of ~$45M in the curves (The Block; Cointelegraph, https://cointelegraph.com/news/solana-memecoin-tool-pumpfun-claims-ex-employee-exploiter). On-chain analysis quoted by Quadriga (https://quadrigainitiative.com/casestudy/pumpfuninsiderflashloanexploit.php) says the service account "itself initiated the withdrawal of all liquidity" and co-signed the attacker's transactions. The root cause was a single privileged key, not a math bug ("the pump.fun contracts are safe"). The insider was sentenced to six years on 2025-12-18 (Decrypt, https://decrypt.co/352876/former-pump-fun-dev-sentenced-six-years-prison-2-million-solana-fraud). The flash-loan venue is inconsistent across sources and is treated as unconfirmed. Lesson: graduation/migration and treasury paths must be permissionless and pay only PDA-derived destinations, with no hot operator cosigner.

**Estimated attacker cost (estimate):** about 0 SOL for the insider. Compromising a single hot key is the usual cost of a phishing or malware campaign.

**Fix (program-level + deployment):**
- Before launch, move upgrade authority to a **multisig (e.g. Squads) with a timelock**, or make the program immutable once stable. The program should check `ProgramData.upgrade_authority` in an `assert_launch_ready` gate that the launch instruction calls.
- **Mint authority:** mint the fixed supply, then set mint authority to `None`. The launch-gate instruction checks `mint.mint_authority == None` (or a program PDA that has no mint instruction).
- **Freeze authority:** `None`. **Do not enable PermanentDelegate, DefaultAccountState=Frozen, or MintCloseAuthority.** MintCloseAuthority is forbidden because the SPL docs warn that "if a mint is closed and reinitialized with the transfer fee extension, there may be existing token accounts without the transfer fee extension, allowing them to bypass fee checks" (https://www.solana-program.com/docs/token-2022/extensions). The launch gate checks the mint's extension list against an allowlist that contains only TransferFeeConfig and metadata pointer/metadata if used.
- Escrow config (`amount`, `fee_amount`, `sol_fee_amount`, URI range): only changeable by the multisig through a timelocked instruction that emits an event, within hard-coded bounds.
- NFT collection update authority goes to the multisig. Where the standard allows, mark metadata immutable after reveal.

### B-04: Token-2022 fee authority abuse. Severity: **Critical**

**Condition:** `transfer_fee_config_authority` or `withdraw_withheld_authority` on the mint is a single team key (or any key that is not a program PDA), or fee changes have no bounds.

**Background (confirmed from Token-2022 source at commit `8867f751` and Solana/SPL docs, research-sources.md §6):**
- The fee is withheld in the *recipient's* token account. `HarvestWithheldTokensToMint` is permissionless. Only the `withdraw_withheld_authority` can move withheld tokens, and SPL docs say it "can move these tokens wherever they wish".
- `transfer_fee_config_authority` can set `transfer_fee_basis_points` up to `MAX_FEE_BASIS_POINTS = 10_000` (**100%**, mod.rs L26, enforced in processor.rs L91-93). `maximum_fee` is a plain `u64` with **no protocol cap**, so it can be `u64::MAX` (no cap in practice). A config authority can therefore take 100% of every transfer.
- **Timing:** `SetTransferFee` schedules the new fee for `epoch.saturating_add(2)` ("set two epochs ahead to avoid rug pulls at the end of an epoch", [processor.rs L101-110](https://github.com/solana-program/token-2022/blob/8867f751c0f69367ba03af4f85510b5611989491/program/src/extension/transfer_fee/processor.rs)). A change sent in epoch N takes effect at the **start of epoch N+2**. If it is sent at the very end of epoch N, the **minimum real notice is just over one epoch** (an epoch is roughly two days at typical slot times; estimate), not two full epochs. A pending fee can be overwritten, and each overwrite re-schedules it to current epoch + 2.
- Either authority can be rotated or removed with `SetAuthority`. Setting it to `None` is permanent.

**Attack scenarios:**
1. **Fee skim:** the withdraw-withheld authority is a team wallet. The "fees fund NFT buys" promise isn't enforced on-chain, and the key holder sends the harvested fees to themselves. Cost about 0 (estimate).
2. **Fee hike rug:** the config authority raises the fee to 10,000 bps (100%) with `maximum_fee = u64::MAX`. From the start of epoch N+2 (as little as just over one epoch after the change), every transfer, including swaps into the MPL-Hybrid escrow and sells to the AMM, sends its value to the fee pot, which the withdraw authority then takes. Holders can see it coming on-chain but have to exit through the same fee path. Cost about 0 (estimate).
3. **Authority as program PDA, but with an unconstrained instruction:** `set_fee(bps)` or `withdraw_fees(dest)` exists with only an `admin` signer check, which is the same risk as (1)/(2) behind one extra step.

**Fix (program-level):**
- Set `withdraw_withheld_authority` to a **program PDA** (`["fee_vault_auth"]`). The only instruction that uses it is `sweep_fees`, which harvests and then withdraws **only into the program's purchase-vault PDA**. The destination is a PDA constraint, not an argument.
- Set `transfer_fee_config_authority` to `None` at launch (fee fixed forever), **or** to a program PDA whose `update_fee` instruction enforces a hard-coded `MAX_BPS` (e.g. ≤ 300 bps) and a hard-coded `maximum_fee`. It can only be called by the multisig, behind a program-level timelock (e.g. ≥ 72h) that runs **before** `SetTransferFee` is sent, and it emits an event. Do not count the Token-2022 epoch+2 schedule as the notice period: it guarantees only just over one epoch.
- The launch gate (see B-03) checks both authorities on-chain before the sale opens.

**Precedent:** the pump.fun 2024-05 incident (see B-03) was a misused privileged withdraw authority on a launch curve, ~12,300 SOL lost. A withdraw-withheld authority held by a team key is the same trust model. StonkFun's Token-2022 reward launches are also described as fees swept and paid out by the operator off-chain (research-sources.md §1); see the StonkFun note below.

### B-05: Treasury / escrow drain through weak account validation or accounting. Severity: **High**

**Condition:** Any program instruction that moves value from a PDA vault (fee vault, purchase vault, prize vault, MPL-Hybrid escrow) takes destination, mint, token program, marketplace program, or vault accounts as unchecked inputs, or trusts caller-supplied amounts.

**Attack scenario (typical Anchor mistakes):**
1. The attacker calls `buy_nft_with_fees(listing, marketplace_program, seller_ata, ...)`, passing their **own** program as `marketplace_program`. The purchase vault PDA signs a CPI (`invoke_signed`) into the attacker's program, which forwards the signer authority to a token transfer that drains the vault.
2. Or: the attacker passes a fake `mint` or a token account owned by the classic Token program where Token-2022 is expected. A missing `token_program` check lets the attacker's program impersonate the token program.
3. Or: `claim_prize` checks `winner == signer` but not `draw.settled && !draw.claimed`, so the same prize can be claimed twice.
4. Or: vaults use one shared PDA authority (`["vault"]`) for all pots, so an instruction meant for pot A can sign transfers out of pot B.

**Estimated attacker cost (estimate):** below 0.01 SOL in fees. The loss is the whole vault balance.

**Fix (program-level):**
- Pin every external program ID with an Anchor `address =` or `Program<'info, T>` constraint: Token-2022, Associated Token, MPL-Hybrid, MPL Core/Token Metadata, marketplace (if any), VRF program, System.
- Use `InterfaceAccount<Mint>` / `InterfaceAccount<TokenAccount>`, with `mint::token_program = token_program` and a constraint that `token_program.key() == spl_token_2022::ID` for the fungible mint.
- Use a **separate PDA authority per pot** (`["fee_vault", config]`, `["purchase_vault", config]`, `["prize_vault", draw_id]`). Never pass a signer-bearing PDA into an unpinned CPI.
- Destinations of outflows are derived (a PDA or a stored winner address), never free inputs. Every claim uses a one-shot state flag set *before* the transfer (checks-effects-interactions).
- Add an invariant test: sum of vault balances ≥ sum of recorded liabilities after every instruction (fuzzed).

### B-06: Token-2022 fee bypass and escrow insolvency on wrap/unwrap paths. Severity: **High**

**Condition:** The design forks MPL-Hybrid (or writes its own swap) to accept the Token-2022 fee mint, and does one of the following: credits the user with the pre-fee amount; pays out a fixed `escrow.amount` on release while capture received `amount - fee`; uses burn/mint instead of transfer on one leg; or keeps a fee-exempt path (e.g. a program "internal transfer" that doesn't use `transfer_checked`).

**Rev. 2: the fork is mandatory, not hypothetical.** Stock MPL-Hybrid supports **classic SPL Token only**. Every swap context declares `token_program: Program<'info, Token>` and `token: Account<'info, Mint>` (capture.rs L95, capture_v2.rs L106 at `aacf1a53`), it calls unchecked `token::transfer`, and the Metaplex FAQ says it "only supports Metaplex Core NFT assets and SPL Token fungible assets" (https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/faq). The SPL docs say transfers on a fee mint must use `transfer_checked` or `transfer_checked_with_fee`, "otherwise, the transfer will fail". So a Token-2022 fee mint **needs a fork** that switches to the token interface (`Interface<TokenInterface>`, `InterfaceAccount<Mint>`) **and** to `transfer_checked`. Also, MPL-Hybrid V2 has `BurnOnCapture` / `BurnOnRelease` paths (state/path.rs; capture_v2.rs ~L261-272 burns `recipe.amount` instead of transferring). **V2 `BurnOnCapture` must not be used with a fee mint**: a burn is not a transfer, so it skips the fee (scenario 2) and removes backing from the escrow.

**Attack scenarios:**
1. **Insolvency / bank run:** capture sends `amount` = 1,000,000 tokens, but the escrow only receives 990,000 (100 bps withheld). Release still pays out a fixed 1,000,000 from the escrow. Each capture→release round trip leaves the escrow 10,000 tokens short (`amount × fee`). The shortfall is the fee, which ends up withheld in the recipient ATA and then the fee pot. The escrow reserve no longer backs every NFT, and at some point the last NFT holders can't release. Anyone with an incentive (a competitor, or someone who wants to run the bank early) loops round trips to speed this up, then releases their own NFTs first.
2. **Fee bypass:** if unwrap pays out by **minting** (program holds mint authority) and wrap **burns**, no transfer fee is charged on either leg, because burn and mint are not transfers. Traders then route through wrap/unwrap to move value without paying the fee that is supposed to fund buys. (NFT → tokens by mint, then the NFT goes to a friend, etc.)
3. **Stuck withheld fees:** fees withheld in the escrow's own token account never get harvested into the fee pot. That's an accounting leak, not theft.

**Estimated attacker cost (estimate):** about 0.05 SOL of swap SOL fees plus 1% token fee per round trip (sim cycle cost 0.0701 SOL for a T22 fork). Each round trip pulls about 0.01 SOL worth of tokens out of the reserve at the example parameters, so draining 1% of a 1,000-NFT reserve takes about 1,000 cycles, roughly 70 SOL (estimate). That's griefing, not profit. Fee bypass (scenario 2) has a cost of about 0 and saves the attacker the full fee on every transfer they route.

**Fix (program-level):**
- Use only `transfer_checked` with the Token-2022 fee mint (plain `transfer` fails). Compute the net received amount by **reading the vault balance before and after** (preferred), or from the current epoch's fee via `TransferFeeConfig::get_epoch_fee(epoch)` (Token-2022 mod.rs L152-153), which switches to the newer fee once `epoch >= newer_transfer_fee.epoch`. Credit only the net amount.
- On release, pay out `escrow.amount` and **document that the user receives `amount - fee`**. Never top up from reserves. Assert `vault_balance_after >= outstanding_nft_backing` in both instructions.
- No mint/burn legs in the swap. With mint authority revoked (B-03) mint legs are impossible. Burn legs are not blocked by that, so the fork must remove or hard-disable `BurnOnCapture` / `BurnOnRelease` for the fee mint.
- Add a `harvest_withheld(escrow_ata)` crank to sweep fees withheld on protocol-owned accounts into the fee pot.

### B-07: Price manipulation of fee-funded NFT purchases. Severity: **High**

**Condition:** The program (or an operator crank) spends the fee pot on NFTs and does one of these: prices the buy from a spot AMM price or the current floor listing; accepts any listing up to a caller-supplied max price; swaps fee tokens to SOL at spot with no slippage bound; or lets the caller choose which listing to buy.

**Attack scenarios:**
1. **Self-dealing listing:** the attacker lists their own NFT at 10× floor and then calls the permissionless `buy_with_fees(listing)` crank. The pot pays the attacker. Cost is the marketplace fee (estimate: 2–5% of price) against a gain of the listing price minus the NFT value.
2. **Floor pump:** the pot buys "the floor NFT" priced by a floor oracle or the last sale. The attacker wash-trades two of their own NFTs at high prices to lift the reference, then lists near the new reference.
3. **Token→SOL conversion sandwich:** if the pot sells fee tokens for SOL before buying, the attacker sandwiches that swap (see B-11). With a 1% transfer fee on each attacker leg, the sandwich pays when the pot's price impact is above about 2% plus the AMM fee (estimate).
4. **Timing:** a predictable crank time lets the attacker pre-position listings or liquidity.

**Estimated attacker cost (estimate):** about 2–5% marketplace fees plus tx fees. The gain is bounded by the pot size per buy.

**Fix (program-level):**
- Buy NFTs **only from the MPL-Hybrid escrow at the fixed swap rate (capture)**, never from open-market listings chosen by the caller. The swap rate is a hard-coded, non-manipulable price. Or, if open-market buys are required, buy only from an allowlisted marketplace program with a per-buy price cap tied to the fixed swap rate × (1 + max premium, e.g. 10%).
- If token→SOL conversion is required, use a TWAP from a manipulation-resistant oracle (at least a 30-minute window) with `min_out` enforced on-chain, and split into small tranches with a per-epoch spend cap.
- Limit spend per epoch, and give the crank a randomized or VRF-chosen execution slot inside a window.

### B-08: Flash-buy / borrow before a point-in-time lottery snapshot. Severity: **High**

**Condition:** Tickets come from token balances read at a single slot (a point snapshot), or at draw time, or from a balance the holder self-reports during a short registration window.

**Attack scenario (sim numbers):**
1. The attacker watches for the snapshot slot, which is predictable or triggered in a transaction the attacker can bundle behind.
2. In a Jito bundle: buy (or borrow, from a lending market that lists the token) **50M tokens**, trigger or be included in the snapshot, sell or repay in the next slot.
3. Sim result: **7.69% of all tickets at a point snapshot vs 0.000077% under a 216,000-slot TWAB.**
4. Round-trip cost is the transfer fee twice, **2% of position ≈ 1.00 SOL at the example price**, plus AMM spread and price impact. **The attack breaks even when the prize is above about 13 SOL** (sim, ignoring price impact).

**Estimated attacker cost (estimate):** about 1 SOL plus price impact per snapshot, with no capital at risk beyond 2 slots. Borrowing removes the capital need.

**Fix (program-level):**
- Use **time-weighted average balance (TWAB)** over the full draw period, using a per-holder checkpoint account the program updates. Because Token-2022 transfers don't call our program, updates come from a **Transfer Hook extension** or opt-in staking.
- Recommended: **explicit staking/lock** into a program vault for a minimum period (e.g. ≥ 1 epoch before the draw) to be eligible. Tickets = stake × time. Unstaking forfeits the current draw.
- If a snapshot is used anyway, choose the snapshot slot **retroactively with VRF** from inside the past period, after the period ends, and require that the balance was held for N consecutive checkpoints.
- Note: adding a Transfer Hook changes the mint's extension set and has its own risks (hook program upgrade = censorship of transfers). If used, the hook program must be immutable or behind a multisig and timelock (see B-03).

### B-09: Sybil splitting under per-wallet or concave ticket formulas. Severity: **High**

**Condition:** Tickets are flat per wallet, log-, sqrt- or tiered-per-wallet, capped per wallet, or any formula that isn't linear in balance (f(a)+f(b) > f(a+b)).

**Attack scenario (sim numbers, attacker = 0.5% of supply):**
1. The attacker splits 5M tokens across k wallets, each above the minimum balance.
2. Win probability for a single winner at k=1000:
   - flat per wallet: **17.93% vs 0.0218% (820.9×)**, and 46.6% at k=4000
   - log10: **15.68% (465.2×)**
   - sqrt: **6.42% (29.5×)**
   - capped linear (1M cap): **1.22% (4.9×)**. Splitting just below the cap is enough.
   - linear: **0.82% (1.0×), neutral**
3. Cost at k=1000: **~0.05 SOL in transfer fees + 2.16 SOL of ATA rent (recoverable on close) + 0.005 SOL of tx fees.** The rent isn't sunk, so the real cost is about 0.055 SOL plus the time value of 2.16 SOL.

**Estimated attacker cost (estimate):** about 0.06 SOL real cost per 1,000 wallets. Scripted with any wallet tool.

**Fix (program-level):**
- **Tickets must be linear in (time-weighted) balance or stake**: `tickets = floor(twab / TICKET_UNIT)`. This is the only formula in the sim that splitting doesn't help. Choose `TICKET_UNIT` so rounding loss per wallet is negligible (the tiered formula in the sim shows coarse flooring *penalizes* splitting but also penalizes small holders).
- No per-wallet caps unless combined with identity or proof-of-personhood, which is out of scope. Caps are sybil-bypassable, as the 4.9× result shows.
- A minimum eligible balance is fine as dust-spam protection (B-13), but it doesn't stop sybils.
- If the product wants "one wallet, one chance" fairness, say publicly that it can't be enforced on-chain.

### B-10: Launch sniping and front-running (Jito bundles, first-slot buys). Severity: **High**

**Condition:** At launch: the pool or escrow opens at a known slot or time; the first captures (NFTs) or first token buys are at a fixed low price; there's no per-wallet cap, allowlist, or commit phase; liquidity is added in a transaction that bots can see or bundle behind.

**Attack scenario:**
1. Bots watch the program and the launch config account. When `open_slot` arrives, or the "add liquidity / enable trading" transaction appears, they send Jito bundles `[launch_tx, buy_1 … buy_n]` with high tips, or spam first-slot buys from many wallets.
2. They capture most of the cheap initial supply or the rarest escrow NFTs (combined with B-01), then dump on retail in the following minutes.
3. With a transfer fee, the bots pay 1% per leg. That isn't enough to deter them when launch price moves are 2–10×.

**Market data (research-sources.md §3):** the MELT dataset (Hu et al., Georgia Tech, arXiv 2602.13480, https://arxiv.org/abs/2602.13480) covers 41,470 migrated pump.fun launches from 2024-12-01 to 2025-03-01. It finds **"36.5% of the total token supply is held by bundled accounts at the point of migration"**, "98.7% of 'create' events co-occur with developer buys in the same transaction", and 60.26% of launches fell below 0.2× the migration price within 20 minutes. A separate single-author preprint (Kamat, arXiv 2607.02795, not peer-reviewed) finds 1,012 persistent sniper cohorts across 166,098 launches, with 7.0% of cohort launches having zero non-cohort buyers in the first 30 minutes. StonkFun publicly acknowledged sniping and single-wallet launch problems in September 2026 (see the StonkFun note below).

**Estimated attacker cost (estimate):** Jito tips from 0.01 to a few SOL per bundle, plus infrastructure. The profit multiple per launch is an estimate; the prevalence of bundled first buys is measured (above). Treat anti-snipe design as a launch blocker.

**Fix (program-level):**
- **Fair-launch phase:** a commit window where users deposit SOL into a program vault (per-wallet cap), then a **batch clearing at a single price**, with allocation pro rata or by VRF lottery after the window closes. No first-come advantage.
- Or a Dutch auction or a bonding curve with an **anti-snipe decay**: a per-wallet max and a max buy per slot during the first N slots, enforced on-chain with a per-slot counter PDA.
- Escrow NFT captures during launch come in randomized order via B-01's VRF commit/reveal, so rarity can't be sniped.
- Trading opens at a **program-enforced** slot. Avoid "admin sends enable-trading tx" patterns that bots can bundle behind. Liquidity is seeded inside the same instruction that opens trading.
- Sybils can get around per-wallet caps (B-09). Treat caps as speed bumps and combine them with batch clearing.

### B-11: Sandwich / MEV on swaps and protocol-side buys. Severity: **Medium**

**Condition:** User-facing token↔SOL trades (AMM) and any protocol-side conversion have no `min_out`/`max_in`. Or escrow parameters (swap rate, fees) can change between the user signing and execution. Or the protocol's own buys go out at predictable times without bounds.

**Attack scenario:**
1. A searcher sees a large user buy or protocol conversion. Jito shut down its public mempool in March 2024 (CoinDesk, https://www.coindesk.com/business/2024/03/08/solana-client-developer-jito-announces-end-of-mempool-function), but that "pushed it underground" into private mempools; the Solana Foundation removed 32 validators from its delegation program for taking part in sandwich-enabling mempools (CoinDesk, 2024-06-10, https://www.coindesk.com/business/2024/06/10/solana-heavyweights-wage-war-against-private-mempool-operators).
2. The bundle is: front-run buy → victim buy at a worse price → back-run sell.
3. The Token-2022 fee costs the attacker 1% per leg (≈2% round trip), so only trades with price impact above about 2% plus the AMM fee can be sandwiched profitably. Large protocol conversions are the main target.
4. Parameter race: if the admin can change `escrow.amount` or fees, a front-run config change makes a user capture at a worse rate (this overlaps B-03).

**Measured data:** Gerzon et al., "Quantifying the Threat of Sandwiching MEV on Jito" (ACM IMC 2025, https://ben-weintraub.com/files/solanamev.pdf) found **521,903 sandwich attacks** in Jito bundles from 2025-02-09 to 2025-06-09, with victims losing **≥ $7,712,138** (a lower bound). Median loss per victim was about $5, sandwich bundles tipped a median of > 2,000,000 lamports, and sandwiches were only 0.038% of bundles. Medium stays appropriate for small user trades, but large, predictable protocol conversions are exactly the high-impact targets. Jito's `jitodontfront` account marker lets a transaction reject being placed anywhere but index 0 of a bundle; the frontend can add it as a cheap user-side mitigation.

**Estimated attacker cost (estimate):** Jito tip plus about 2% transfer fee plus AMM fees. Profit scales with victim size × impact.

**Fix (program-level):**
- Every value-moving instruction takes `min_out`/`max_in` (and `expected_escrow_amount`, `expected_fee_bps`) as arguments and reverts when they're violated.
- Protocol-side conversions: TWAP-bounded `min_out`, small tranches, and a per-epoch cap (as in B-07).
- Config changes are timelocked (B-03), so a user's signed expectations stay valid.

### B-12: Wrap/unwrap arbitrage, inventory sniping and honest reroll grinding. Severity: **Medium**

**Condition:** The escrow swap rate is fixed, rerolls are allowed (even with honest randomness), and the rarity premium over floor is large compared with the reroll cycle cost.

**Attack scenario (sim numbers):**
1. **Honest grinding** (randomness fixed per B-01): cycle cost **0.0501 SOL** (classic SPL assumptions). Legendary p=0.1%, value 60 SOL, so **EV = +8.90 SOL, profitable**. The breakeven premium is 50.10 SOL. With a 100 bps T22 fee the cycle is **0.0701 SOL** and EV is **-11.10 SOL**, breakeven 70.10 SOL. Epic and rare sit about at breakeven (≈ -0.01 SOL). So the attack depends entirely on parameters: if legendary secondary prices rise above about 50–70 SOL, rational grinders drain legendaries from the escrow.
2. **Floor arbitrage:** when the NFT floor is below the swap rate, arbs buy NFTs on the market and release them for tokens. When it's above, they capture from the escrow and list. This pins the floor to the swap rate minus fees. That's expected behavior, but it means **escrow inventory composition** (which NFTs sit in escrow) can be drained of rares if release is allowed to pick specific NFTs.
3. **Inventory sniping:** if capture lets the caller choose *which* NFT to take out of escrow, or the next NFT is deterministic, rares released by others are captured immediately by bots.

**Sourced inputs:** the sim's Part A cost model matches MPL-Hybrid V1 code: a 0.005 SOL protocol fee on **both** capture and release (`get_protocol_fee()` = Rent-derived 5,000,000 lamports, constants.rs L10-18), and the project `sol_fee_amount` is also charged on both legs in V1 (research-sources.md §5). Metaplex says it "may change the fee amounts over time". MPL-Hybrid's mandatory sequential `<base><index>.json` URIs are the "Predictable URI" anti-pattern Metaplex warns about, so the rarity of every index can be scraped ahead of time.

**Estimated attacker cost (estimate):** about 50 SOL of expected spend per legendary under honest randomness (sim). High variance: P(≥1 legendary in 693 tries) = 50%, in 3000 tries = 95%.

**Fix (program-level):**
- Set reroll cost from the sim formula so that `cycle_cost / p_rarest > expected premium × safety factor (≥ 2×)`. Enforce it with an on-chain **reroll surcharge** that scales with rarity count remaining, or a per-NFT **reroll cooldown** (e.g. one reroll per NFT per epoch).
- Capture always gets a **VRF-chosen** NFT from escrow inventory (B-01 commit/reveal). Never let the caller choose it.
- Publish the rarity distribution and the escrow's live composition so the economics are clear to everyone.

### B-13: Griefing / DoS: unbounded holder iteration, dust accounts, compute limits. Severity: **Medium**

**Condition:** The lottery or fee distribution iterates over all holders or ticket accounts in one instruction; `remaining_accounts` lists of arbitrary length decide the winner; any holder with more than 0 balance is eligible; or fee harvest has to touch every token account.

**Attack scenarios:**
1. **Compute exhaustion:** a draw that loops over N ticket accounts can't finish once N exceeds what fits in one transaction. Limits: ~1.4M CU per transaction and a per-transaction account and size limit (about 64 accounts with legacy transactions, more with address lookup tables) [needs source for current limits]. The attacker opens thousands of dust eligible accounts (rent ~0.00216 SOL each, recoverable) so the draw can never settle, which freezes the prize.
2. **Selective omission:** if the settler passes ticket accounts in `remaining_accounts` and the program sums whatever it's given, the settler leaves out other holders' accounts and wins more often.
3. **Dust accounts that block fee sweeps:** withheld fees scattered across many dust accounts mean harvesting is paginated. An attacker keeps creating dust accounts to make sweeping expensive. It's a nuisance, not theft, because harvesting is permissionless and batched.
4. **Account-close grief:** holders can't close Token-2022 accounts that hold withheld fees until the fees are harvested, which is user-facing friction. Confirmed by the SPL docs: "it is impossible to close an account that holds any tokens, including withheld ones" (research-sources.md §6).

**Estimated attacker cost (estimate):** 1,000 dust accounts ≈ 2.16 SOL of recoverable rent plus about 0.05 SOL in fees (sybil sim numbers). That's cheap enough to grief permanently.

**Fix (program-level):**
- **No holder iteration at draw time.** Use either (a) opt-in ticket accounts with a **cumulative-ticket prefix** kept in a program-owned structure (e.g. a Fenwick/segment tree across paginated PDAs), so winner lookup is O(log N) with a binary search over a bounded number of accounts; or (b) an off-chain snapshot committed as a **merkle root** of `(owner, cumulative_start, tickets)` before randomness is requested (B-02), with the winner claiming by proof. The merkle root needs a challenge window or multisig attestation, because the root publisher is a trust point.
- **Minimum eligible stake** (e.g. ≥ TICKET_UNIT), with staking into a program vault, so dust can't create tickets.
- Paginate every crank (harvest, checkpoint updates) with a stored cursor, a bounded batch size, and no correctness dependence on completing all pages in one slot.
- Settling a draw must never depend on caller-supplied account lists being complete, unless completeness is proven (cumulative totals checked against `draw.total_tickets`).

### B-14: Lottery liveness: stuck draws and unrecoverable prize escrow. Severity: **Low**

**Condition:** The draw has no deadline or cancel path; the prize vault can only be released by `settle_draw`; the VRF oracle or queue can stop fulfilling (outage, unfunded queue, deprecated program version); or unclaimed prizes have no expiry.

**Attack scenario:**
1. The VRF provider has an outage, or the request account was underfunded. The draw stays in `RandomnessRequested` forever.
2. The prize SOL or NFTs are locked in a PDA with no exit. A careless fix, "admin can re-request", brings back B-02 reveal-and-abort.
3. A winner who never claims leaves the prize stuck forever.

**Estimated attacker cost (estimate):** about 0 for an oracle outage (no attacker needed). A deliberate grief needs a way to make fulfilment fail, which may not exist with a properly pinned VRF.

**Fix (program-level):**
- `deadline_slot` on every randomness request. After it passes, a permissionless `cancel_draw` rolls the prize and tickets into the next draw. **The same draw is never re-requested.**
- Claim window (e.g. 30 days). After it, a permissionless `sweep_unclaimed` sends the prize back to the prize pool, not to a team wallet.
- Monitoring and alerting on VRF queue balance and fulfilment latency (operational).

### B-15: Dependency pinning and program-ID verification. Severity: **Info**

**Condition:** The design relies on MPL-Hybrid, MPL Core/Token Metadata, Switchboard/ORAO, and Token-2022 without pinning program IDs, crate versions, or reviewed commits.

**Risk:** An upstream upgrade (or a spoofed program passed as an account) changes semantics. For example, MPL-Hybrid's randomness or fee constants change, or a VRF account layout changes so the program reads the wrong bytes. The MPL-Hybrid reading is verified at commit `aacf1a53c8a43bf395db7b562c02cf016ce604c5` (latest `main` on 2026-09-24). Pin that commit, or re-verify whichever commit the fork is based on. The protocol fee is computed from the Rent sysvar (constants.rs L10-18), so it changes if rent parameters change. For any fork, also fix capture.rs L236-237 (`let _transfer_nft_result = transfer_nft_ix.invoke_signed(...)` discards the NFT-transfer result; V2 uses `?`). A failing CPI is believed to abort the whole transaction, which would make it harmless, but that was not verified; add a test.

**Estimated attacker cost (estimate):** N/A (supply-chain / integration risk).

**Fix:** Pin crate versions and git commits in `Cargo.lock`. Hard-code program IDs with `address =` constraints. Deserialize VRF accounts with the provider's official SDK at a pinned version and check the account owner. Add a CI test that fails if upstream program IDs or layouts change. Re-review after any dependency bump.

### B-16: MPL-Hybrid reroll range excludes `max`, so the docs' advice orphans the last metadata file. Severity: **Low / Info**

**Condition:** The escrow uses MPL-Hybrid reroll (or a fork that keeps its index math), and `max` is set to the last metadata file index, as the Metaplex create-escrow docs instruct ("min: 0, max: 4999" for files `0.json … 4999.json`, https://www.metaplex.com/docs/smart-contracts/mpl-hybrid/create-escrow).

**Why it matters:** The pick is `seed.checked_rem(max - min) + min` (the `checked_rem` line in [capture.rs L167-183 at `aacf1a53`](https://github.com/metaplex-foundation/mpl-hybrid/blob/aacf1a53c8a43bf395db7b562c02cf016ce604c5/programs/mpl-hybrid/src/instructions/capture.rs#L167-L183)), which ranges over **`[min, max)`**. Index `max` can never be selected. With min 0 / max 4999 the highest reachable index is 4998 (checked numerically, research-sources.md §5). The file `4999.json` is silently orphaned: it is never assigned on capture. If that file holds a rare or a promised 1/1, holders were sold a distribution that can't occur (a disclosure and fairness problem, not theft). Separately, `max - min == 0` makes `checked_rem` fail with `RandomnessError`, so a single-file range bricks capture.

**Scenario:** A team follows the docs, puts a legendary at the top index, and advertises it. No reroll ever produces it. A collector who ground rerolls for it (B-12) spends money chasing an outcome with probability 0.

**Estimated attacker cost (estimate):** N/A. This is a correctness and disclosure issue, not an attack.

**Fix:** In our own selection code (B-01 fix), use an inclusive, explicitly documented range: `index = min + uniform_below(max - min + 1)` with rejection sampling. Add a unit test that every index in `[min, max]` is reachable. If any MPL-Hybrid escrow is used as-is, set `max = last_index + 1` and document the deviation from the docs. Consider reporting the docs/code mismatch upstream to Metaplex.

## External precedent notes (from research-sources.md)

- **pump.fun, 2024-05-16 (B-03, B-04, B-05):** an insider misused a privileged bonding-curve withdraw authority, with flash-loaned SOL, and took ~12,300 SOL. See B-03 for citations.
- **MELT dataset (B-10):** 36.5% of supply held by bundled accounts at migration across 41,470 pump.fun launches. See B-10.
- **Jito sandwich study, IMC 2025 (B-11):** 521,903 sandwiches, ≥ $7.7M victim losses in four months of 2025. See B-11.
- **StonkFun (stonkfun.xyz):** **No credible exploit, hack, or platform-rug report was found.** What is documented: sniping and single-wallet-dominated launches, which StonkFun itself acknowledged in a 2026-09-02 post (reported second-hand by The Coinomist, 2026-09-06, https://thecoinomist.com/news/stonk-surges-250-to-140m-after-raydium-launchlab/) and answered by raising dev-buy limits and moving launches to Raydium LaunchLab. Its ~60% buyback is at the operator's "off-chain discretion" with no DAO or named operator (Fintrender, 2026-09-08, https://fintrender.com/en/reports/a-case-for-stonk, an opinion piece), and Token-2022 transfer fees are swept, swapped and paid out off-chain by the operator (StakePoint blog, https://stakepoint.app/blog/how-to-lock-stonkfun-tokens). Brand-impersonation phishing domains exist (e.g. claim-stonkfun.xyz) but are not a platform compromise. Relevance: supports B-10 (sniping is real even on a live competitor) and illustrates the discretionary, off-chain fee-pot trust model that B-04 and B-07 tell us to avoid. It gives no reason to raise or lower any severity.
- **Candy Machine (B-01, B-02, B-12):** Metaplex documents that its slot-hash pick is influenceable and that sequential URIs let bots target indexes. See B-01.

---

## Not safe to launch until

1. **B-01:** Reroll and capture selection uses VRF commit/reveal bound to a per-request PDA. The MPL-Hybrid built-in pick is disabled for anything rarity-bearing, and a PoC test shows a predict-and-abort wrapper cannot beat the base rate.
2. **B-02 / B-14:** The lottery is a one-way state machine: freeze tickets, request VRF bound to the draw, permissionless unconditional settle, deadline and cancel with no re-request.
3. **B-03:** Upgrade authority is on a multisig with a timelock (or the program is immutable). Mint and freeze authorities are `None`. No PermanentDelegate. The launch-gate instruction verifies all of this on-chain.
4. **B-04:** Withdraw-withheld authority is a program PDA that can only pay the purchase vault. Fee config authority is `None`, or a PDA with a hard bps cap and timelock.
5. **B-05 / B-06:** Every external program ID is pinned, there's one PDA authority per pot, net-of-fee accounting is used, there are no mint/burn swap legs, and fuzzed solvency invariants pass.
6. **B-07:** Fee-funded buys happen only at the fixed escrow rate, or from allowlisted venues with an on-chain price cap and a per-epoch spend cap.
7. **B-08 / B-09 / B-13:** Tickets are linear in time-weighted stake, with a minimum stake. There's no holder iteration at draw time.
8. **B-10 / B-11:** The launch uses batch clearing or on-chain anti-snipe limits, and every value-moving instruction has `min_out`/`max_in` and expected-parameter arguments.
9. **B-16:** Our selection uses an inclusive range with a reachability test for every metadata index.
10. Every remaining **[needs source]** item has been checked against primary docs. Rev. 2 resolved the MPL-Hybrid pick logic, Token-2022 support in MPL-Hybrid, Token-2022 fee-change timing and caps, Jito bundle atomicity, launch-sniping and sandwich data, and mempool visibility. **Still open:** the current Switchboard On-Demand / ORAO APIs (B-02) and current Solana compute/account limits (B-13). Also unverified: whether failed Jito bundles cost anything off-chain, and whether the ignored CPI result at MPL-Hybrid capture.rs L236-237 is ever reachable (B-15).

## Professional audit requirement

This is a design-level threat model written before any code exists. It is **not a security audit** and doesn't replace one. **A professional third-party audit of the final Anchor programs, the deployment and authority configuration, and the economic parameters is required before mainnet launch.** It should be followed by a re-review after any post-audit change. A bug bounty and a staged launch (caps on escrow and pot sizes at first) are strongly recommended. No deployment was made, and no real keys were used, in preparing this document.
