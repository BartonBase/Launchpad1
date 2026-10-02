# Review of Barton's three files vs the current project (2026-10-01)

Prepared 2026-10-01, about 4:30 PM MT. **Read-only review.** Nothing on GitHub was changed and no project file was edited; this report is the only new file.

**Source of truth used:** `/workspace/launchpad` (docs/BRIEF.md through the 2026-09-25 Decisions, docs/DECISIONS.md ADR-001..019, docs/STATUS.md, graduation-design.md, marketplaces-and-ratios.md, audit-fixes-round1.md, security/merged/round1-merged.md, qa/FINDINGS_TRACKER.md, design/directions/a-obsidian/NOTE.md), plus a read-only clone of `BartonBase/Launchpad1` and `/workspace/migration/pack/claude-launchpad-history.md`. **NAMES.md doesn't exist.** STATUS.md is at `docs/STATUS.md`, not the repo root.

**Files reviewed**

| # | File | What it is |
|---|---|---|
| A | `fafe863d….pdf` (pdftotext OK, 152 lines) | "Frontend design brief & itinerary", written by Claude. Name/mascot = Fuze. |
| B | `8d527f01….pdf` (pdftotext OK, 383 lines) | "Handoff: hybrid coin/NFT launchpad", Claude's living HANDOFF.md, sessions 1–11. |
| C | `bac84343….md` (202 lines) | "Handoff for the next Grok — Fuze / Launchpad1", written 2026-10-01 by another Grok session. It describes Modes 1–5 code that isn't committed. |

All three files were readable. The attachments folder also holds 11 PNGs and one HTML file from Sep 30. Those weren't in scope and I didn't review them.

---

## 0. Repo check (GitHub vs local)

- **Verified:** the remote default branch `onchain/hybrid-launch` HEAD is `df0d1d9` (2026-09-26 8:04 AM MT), the same as local HEAD. The tracked file trees are identical. The other branch is `wip/hybrid-vault`, also at `df0d1d9`. There are no pull requests, open or closed. The last push was 2026-09-26 8:04 AM MT.
- **Commits by Barton since 2026-09-24: none.** All 19 commits in the repo were authored by "Solana Program Engineer (agent)" or "(Grok Bot)". The most recent one is `df0d1d9`.
- The local workspace is ahead of GitHub only in **untracked** files (docs/BRIEF.md, app/, design/, qa/, security/, some tests). **BRIEF.md has never been on GitHub.**
- **`.keys/` was never committed. Verified:** no `.keys`, `*keypair*.json` or `id.json` path appears in any commit on any branch, and `.gitignore` excludes them. This answers File B, open question 3.
- `programs/hybrid_escrow` and `site/hawk.html` (Claude's drafts, File B) **aren't in the repo on any branch**.
- **The Modes 1–5 code in File C isn't anywhere I can see.** It isn't on GitHub, and `/workspace/Launchpad1` doesn't exist on this box. Barton's Mac (`Mac.lan`) is registered but offline. No `raffle.rs`, `launch_token22.rs` or `permanent.rs` exists on the box. **Every code claim in File C is therefore UNVERIFIED.** Because that work is uncommitted on a machine we can't reach, it could also be lost.

---

## 1. File A: Frontend design brief & itinerary (Claude)

### Plain-English summary
Creative direction for the website, written after Claude saw devnet screenshots of our app (then called "Mintmark"). It says the name is **Fuze**, with **Hearth** as the backup. The mascot is a small round creature made of two fused halves, one half a coin and one half an NFT card, joined by a glowing seam. The mascot would appear in empty states, the wrap/unwrap animation, graduation, loading and errors. The brief praises the in-app trust/security sections, the launch wizard's live-math panel and the data density. It points out there's **no marketing landing page**. It proposes a signature accent colour (warm gold/bronze/burgundy, a "wax-seal" palette), a display typeface, launch-style icons, and three "signature moments" (graduation burst, convert split-and-snap, re-roll card flip). It closes with a seven-phase itinerary: brand, landing site, polish, motion, mobile/accessibility, microcopy, QA/performance.

### Sorted ideas

**Useful and new (adopt or consider)**
1. **A marketing/landing layer before wallet connect**: a one-screen hero and a plain-language "a coin that's also an NFT" explainer. The Obsidian home page exists as a mockup, but the explainer-diagram idea and a public trust page are new and good.
2. **A public version of the trust/security section.** This fits our main differentiator (the Stonk.fun lessons) well.
3. **A full mobile rework** (not just reflow), a **contrast/accessibility audit** and **keyboard navigation in the launch wizard**. None of these is tracked today.
4. **Designed empty and loading states**, and a **terminology/tone pass**.
5. **"Final pass once audit status moves off Pending"**: update every "Audit pending" badge and disclosure. Good hygiene. It ties to BRIEF hard requirement 7.
6. **Performance audit of data-heavy screens** (activity feed, holders, live chart).
7. **The rarity-toggle flag (section 2) is a real finding.** The Obsidian launch wizard (`design/directions/a-obsidian/launch.html:247`) offers *either* "Verifiable randomness (VRF)" *or* "Pre-committed, revealed later". The implemented design (ADR-008/016) is **both together**: the creator commits a Merkle root of the full trait list before launch, and Switchboard VRF assigns pieces at settle. The toggle misdescribes the product and implies a non-VRF option exists. **Fix the mockup copy** (design-only). See also the artist-rarity idea in File B.

**Already have it**
- The trust sections ("what nobody can change"), the 3-of-5 multisig + 7-day timelock disclosure, the live-math panel, the bonding-curve/graduation bar and the Tensor/Magic Eden buttons are all in the Obsidian direction (NOTE.md).
- Ratio presets 50k–5M: these are the current ratios (ADR-016). File A only notes them.
- Dark, near-black base with one saturated accent: Obsidian already does this (iris violet `#8F7DFF`).

**Conflicts with a current decision**
- **Accent colour:** the brief proposes warm gold/bronze/burgundy. The current approved direction is Obsidian iris violet (Barton approved Direction A on 2026-09-24). The wax-seal palette was tied to the retired "Mintmark" seal idea. The brief's own Fuze section says the accent should be the **mascot's seam-glow colour**, which isn't chosen yet. **Verdict:** neither option is clearly better. Pick the glow colour first, then retire or keep violet. It's Barton's taste call.
- **Name "Mintmark" → "Fuze":** BRIEF still says "Mintmark" (working name). Barton's choice of Fuze is newer and should replace it, but see the trademark warning below.
- **Launch-style icons for "plain / wrap / taxed wrap / burn mode":** BRIEF has only one launch type (SPL-404 hybrid). Token-2022/tax is deferred, and there is no burn mode. These icons assume File B's four styles or File C's five modes, which aren't in BRIEF yet.

**Not useful, wrong, or risky**
- **"Fuze came back clean" is wrong as stated.** My web search on 2026-10-01 found **Fuze Finance** (fuze.finance), a regulated digital-asset payments/custody/OTC company that supports Solana. That is a direct crypto-sector trademark risk. I also found a **FUZED** token on a PumpSwap pool created 2026-09-08. Claude's own earlier chat (claude-launchpad-history.md) had flagged Fuze Finance, 8x8 Fuze, Fuze Beverage and Fuse wallet. **Partly verified:** web search only, with no USPTO, domain or X-handle lookup. Do the formal check before any spend.
- **Re-roll "card flip/shuffle" animation:** fine, but it must not suggest the user can pick or see the "next" NFT. The result comes from VRF at settle, a few seconds later (two transactions). Copy and animation should match that. This is a design caution, not a rejection.

**Open questions for Barton**
1. Seam-glow/accent colour: keep Obsidian violet or switch?
2. Will you pay for a proper trademark/domain/X-handle check on Fuze (and Hearth) before branding work starts?
3. Is a separate marketing landing page a priority now, or after the programs are audited?

---

## 2. File B: Claude's HANDOFF.md (sessions 1–11)

### Plain-English summary
This is Claude's project memory. It describes **four launch styles**:
1. Plain SPL memecoin.
2. Reversible SPL hybrid (wrap a fixed number of coins into a Metaplex Core NFT and unwrap back).
3. A Token-2022 hybrid whose transfer tax goes to NFT holders (reversible, legally risky).
4. A burn mode (wrap burns the coins, no unwrap).

It says the creator uploads the **entire collection at launch** and that NFTs are "not minted on demand". It says **unwrapping always burns the NFT**. Its owner security rules are: no update instruction, every fund authority is a PDA, mint and freeze authority revoked, and **no AI agent or dev tool ever holds a mainnet key or the upgrade authority**. Open ideas include **artist-defined rarity** (hand-made pieces with print runs, sequential assignment, no randomness) and ways to differentiate Style 1. Claude's own unrequested suggestions are listed separately: Meteora vs Raydium LaunchLab routing, extra invariants, sniping concerns, avoiding mpl-hybrid, and process items (two audits, bug bounty, fuzzing, TVL cap). Branding: Fuze is picked, and a "bomb" mascot was rejected.

### Sorted ideas

**Useful and new (adopt or consider)**
1. **Owner rule 4: "No AI agent and no dev tool ever holds a mainnet key or the upgrade authority."** BRIEF has only "testnets and throwaway keys only". Writing rule 4 down explicitly also implies the 3-of-5 multisig signers must be humans (M-09 is still open). **Adopt into BRIEF.**
2. **Process items:** a **hard TVL cap plus an "unaudited beta" label** at mainnet, a **public bug bounty**, **fuzzing (Trident) against invariants**, and **two independent audits**. BRIEF requires one professional audit. ADR-007 mentions fuzzing "before audit", but no TVL cap or bounty is recorded. A TVL cap is a cheap way to limit worst-case loss in a first release. **Consider; Barton decides** (cost is a factor with no funding).
3. **Artist-defined rarity (hand-made pieces with print runs).** This is **compatible with the current design without dropping randomness.** Our creator already commits a Merkle list of every piece before the curve. That list can be hand-made images with print runs (say 1-of-1 up to 1-of-100). VRF then decides which piece a capture gets. Barton can support "artist mode" as a launch-wizard option or a curation badge without weakening fairness. (See below for why sequential assignment is risky.)
4. **"Vested creator" badge.** Today the full 1B goes to the DBC curve, with no creator allocation (N2, Q-H6/M-27 open). A badge only makes sense if a creator voluntarily locks tokens they bought, using an external lock. That would need a vetted lock program. Low priority; consider later.
5. **Legal review before any tax-to-holders style.** Correct and important. It applies even more strongly to File C's Mode 5 raffle.

**Already have it**
- Rules 1–3 (no update instruction, PDA authorities, mint and freeze revoked): ADR-009/010/014/015/017, plus tests such as `idl_has_no_cancel_refund_update_close_withdraw_or_burn_instruction`.
- **Avoiding mpl-hybrid:** ADR-008 (custom `hybrid_vault`; mpl-hybrid is reference only). Verified in DECISIONS.
- Most of the "extra invariants": collection update authority = vault PDA; immutable metadata URI plus a pre-curve Merkle root; only the launch program's `LaunchConfig` can open a vault; the fee goes only to the fixed `PLATFORM_FEE_RECIPIENT` (M-05); exact-ratio release; and the backing invariant `vault ≥ ratio × NFTs outside` with a property test (M-14). Per-epoch caps on fee sinks don't apply (no tax).
- **Sniping concern:** handled by two-step VRF with a program-chosen oracle (ADR-012), with no caller-chosen asset (M-02 closed). Grinding at the 2.5M/5M ratios is the remaining open item (M-19, pending Barton).
- Open question 1 ("Track B shelved, deliberate?"): **yes**. Barton chose this on 2026-09-24 at 2:51 PM MT (ADR-009). Open question 2 (remove the lottery): already shelved in BRIEF. **Note that File C reverses both.**
- Open question 3 (`.keys/`): **verified never committed** (see section 0).
- Ratios 50k–5M, min 100: current.

**Conflicts with a current decision**
| File B says | Current decision | Which looks better |
|---|---|---|
| **"Unwrapping always burns the NFT"** | Release returns the NFT to the vault, with no burn and no re-mint (ADR-016, `lazy-mint-interface.md`). File C also says unwrap returns it to inventory | **Current is better.** Returning keeps each NFT's identity and trade history, avoids paying the mint cost (~0.005 SOL) again, and keeps the "mint once" safety check (`minted_count ≤ N`). Burning would require re-minting a burned index, which is a new path that could duplicate pieces. |
| **"Entire collection uploaded at launch; NFTs not minted on demand"** | Lazy minting: the full trait/art list is committed (Merkle root) before the curve, and each NFT is minted on its first capture (ADR-016, Barton 2026-09-25 5:13 PM) | **Mostly compatible.** "All pieces and the exact count are known from launch" is already true via the committed list. "Not minted on demand" contradicts lazy minting. Pre-minting was dropped because minting 10,000 NFTs at graduation costs ~35–67 SOL and needs batching (graduation-design). **Keep lazy minting**, and confirm with Barton that "known and committed upfront" is what he meant. |
| **Four styles (plain, reversible, T22 tax-to-holders, burn)** | BRIEF: SPL-404 hybrid only; Token-2022/tax/lottery deferred | **Barton must choose.** File C says he authorised building these on 2026-10-01, but BRIEF hasn't been updated. |
| **Style 3 routed to Raydium LaunchLab** (Claude's suggestion) | Meteora DBC for all launches (ADR-014) | **Verified facts:** Meteora DBC has no transfer-fee extension support (docs.meteora.ag/core-products/dbc/token-2022-support: only metadata and transfer-hook paths). Raydium LaunchLab does support Token-2022 with a TransferFeeConfig (docs.raydium.io/products/launchlab). However, LaunchLab gives the fee-config authority to the launch authority before graduation and **hands both fee authorities to the platform key at graduation**. That retained power to change the tax is exactly what Stonk.fun had: our stonkfun-lessons.md notes that on 2,178 LaunchLab coins the reward wallet can still raise the tax to 100%. LaunchLab could only be safe if the platform key were one of our program PDAs that immediately sets the config authority to None. Prototype on devnet before deciding, as File B itself says. |
| **Sequential assignment for artist rarity** | VRF assignment (ADR-008/012) | **Current is better.** Sequential means everyone, and the creator first of all, knows which piece is next. Bots and insiders can wait for a rare piece's turn and capture it. That is a fairness and insider-sniping hole (M-02 class). Keep VRF even with hand-made art. |
| **Style 1 "add wrap/unwrap later if the community opts in"** | Frozen config; File C says a Mode 1 mint can't open a vault | **Worse as stated.** "Opt in later" needs someone with power to attach a collection after launch, which reintroduces a changeable setting. It could only work as a permissionless, fixed-rule add-on designed and audited separately. Park it. |

**Not useful, wrong, or risky**
- **"Cross-pollination" (NFT holders get early or priority allocation on new launches):** this needs an allowlist or presale gate on the curve. DBC has no native allowlist (only a transfer-hook path, per Meteora docs). That means extra custom code in the riskiest window (launch sniping, M-03), plus "insider" optics. Not worth it now.
- **"Rules 1 to 7" (open question 4)** refers to 7 rules, but the file lists only 4 (Claude moved the rest). It's an internal inconsistency, not a technical problem.
- **Branding: "Fuze and Hearth came back clean"**: see File A. Fuze collides with Fuze Finance (crypto) and a FUZED token. Unverified for trademark.

**Open questions for Barton**
1. On unwrap, do you want the NFT burned, or returned to the vault (current)? I recommend returned.
2. By "upload the entire collection at launch", did you mean "all art committed and known up front" (we do this) or "all NFTs minted up front" (dropped because of cost)?
3. Artist-defined rarity: support it as an option, **keeping VRF assignment**?
4. Do you want a TVL cap, a bug bounty and fuzzing written in as required pre-mainnet steps?
5. Should the 3-of-5 signers be named humans only (rule 4)?

---

## 3. File C: Handoff for the next Grok, Modes 1–5 (2026-10-01)

### Plain-English summary
Another Grok session says that **today (2026-10-01)** Barton asked for, and it built, **five launch modes**. They exist only as an **uncommitted local tree** (54 changed or new paths at `/workspace/Launchpad1`, on top of HEAD `df0d1d9`):
- **Mode 1, plain SPL coin** via Meteora DBC.
- **Mode 2, today's reversible hybrid**, unchanged.
- **Mode 3, burn hybrid:** wrap burns exactly the ratio and mints the *next* NFT in order. No unwrap, no VRF.
- **Mode 4, Token-2022 tax with an equal split:** tax set at launch (0.1–10%) and locked. Tax is harvested to a program PDA. A "buyback" spends SOL already on that PDA to buy the launch's own inventory at a locked price. The program pays every minted NFT an equal share in index order. No bonding curve.
- **Mode 5, raffle:** same as Mode 4, but when the pot reaches a tier, owners are snapshotted and one NFT wins the whole pot through Switchboard VRF.

It also records a **Pump.fun answer**: there is no drop-in, and the only sellable piece is Modes 4/5. That would need Pump to build the payout into its own fee path with a locked recipient. File C lists rules (no GitHub writes, no deploy, no audit, upgrade authority stays the throwaway deployer, Mode 2 untouched), build steps, test counts (83 vault tests green), and known gaps (Mode 5 can stall if the oracle never reveals; Modes 4/5 have no curve; unaudited; local-only).

### Sorted ideas

**Useful and new (adopt or consider)**
1. **Mode 1 (plain SPL via DBC, `PlainLaunchConfig`):** low risk in concept. It reuses the DBC allowlist and checks we already have, and the separate config type stops a Mode 1 mint from opening a hybrid vault. It's a sensible way to widen the product. *Unverified (code not available).*
2. **Pump.fun analysis.** The core conclusions hold:
   - **Verified:** program IDs `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P` (Pump) and `pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA` (PumpSwap) match Pump's public docs.
   - **Verified:** Pump creates the mint, and its global config sets the threshold and fees, so our DBC allowlist can't point at it.
   - **Partly correct:** "if the creator can change the fee-recipient list later, it's the Stonk hole." Pump's fee-sharing config can now be updated **once** and is then locked (`update_fee_shares_v2` revokes the admin, per pump-public-docs). Pump also limited creators to one fee-recipient change in March 2026 after "vamping". So the hole has been narrowed, though not removed before that one change.
   - The advice "don't build a Pump integration unless asked" is right.
3. **The design rules for any payout layer are the right Stonk.fun lessons:** program PDA as the tax withdraw authority, fee-config authority set to None (rate locked), no operator wallet, no selling into the pool, and no caller-chosen amount, recipient or order. If these ever get built for real, keep these rules. **Verified facts:** Token-2022 fees use ceiling division and cap at `maximum_fee`. Setting `transfer_fee_config_authority` to None makes `SetTransferFee` fail with `NoAuthorityExists`, so the rate is permanently locked (token-2022 v10 source).
4. **Mode 5's anti-sniping order** (snapshot owners **before** randomness exists, randomness tied to the round, rejection sampling, no second draw, the crank can't pick the winner) is the correct pattern for any draw. *Unverified in code.*
5. **The toolchain notes** (don't run a bare `anchor build`; pin v1.54; `test-mock-graduation` must never ship) match our STATUS.md and ADR-001/002. They're useful as a cross-check that this tree uses the same guardrails.

**Already have it**
- Mode 2 = the current product (ADR-008..019). The ratio set (50k–5M, 10k dropped, min 100), program IDs `9Loc4hQZ…` / `BEfL9dcc…`, the DBC allowlist, the `["dbc_buffer"]` PDA, Switchboard-only randomness, a failing `getrandom`, and no admin withdraw are all current. Verified against STATUS/DECISIONS.
- "Unwrap returns the NFT to inventory" matches ADR-016 (and disagrees with File B).
- "Upgrade authority stays the throwaway deployer" matches current devnet. It's fine for devnet only; mainnet still needs the 3-of-5 + 7-day plan (M-09 open).

**Conflicts with a current decision**
| File C says | Current decision (BRIEF/ADR) | Assessment |
|---|---|---|
| **Token-2022 tax modes (4, 5) built** | ADR-009 / BRIEF SCOPE CHANGE 2026-09-24: Token-2022 dropped and deferred | File C says Barton reversed this on 2026-10-01. **BRIEF/DECISIONS don't record it.** Barton needs to confirm, and an ADR is needed if he does. |
| **Mode 5 raffle (whole pot to one NFT)** | Lotteries deferred (BRIEF); File B open question 2 says "remove the lottery (gambling risk)" | **Riskier.** Legally, a lottery (pay in, chance to win a pot) is the highest-risk feature in either file. File B, Claude's history and BRIEF all flag gambling risk. Get a lawyer's opinion **before** treating this as part of the product. |
| **Mode 3 burn (supply shrinks)** | Supply fixed at 1B, "no burn" (ADR-013) | Allowed only as a separate mode with different copy ("supply can only go down"). The bigger issue is sequential assignment (see below). |
| **Mode 4 "per minted NFT, program pushes payouts in index order"** | File B's Style 3 used a pull-based "reward per share" accumulator, and was **reversible** | File C's Modes 4/5 appear to have **no unwrap** (the only `unwrap` in the listed IDL is Mode 2's), so wrapped tokens may be locked permanently. That contradicts File B's "Style 3 is reversible" and may conflict with Barton's "rejected burning" (locking forever has the same economic effect). *Unverified; needs the code.* |
| **Upgrade authority = throwaway deployer** | Devnet: same. Mainnet: 3-of-5 + timelock | No conflict for devnet. Make sure it never carries over to mainnet. |
| **The STATUS.md "top section updated for Modes 4/5"** | Local docs/STATUS.md is unchanged (still 2026-09-25/26) | This confirms the tree lives somewhere else. |

**Not useful, wrong, or risky** (security first)
1. **Mode 5 pot can be stuck forever.** File C says that if the chosen oracle never reveals, the round stalls, and recommit was "considered and not built". Mode 2 has recommit (≤ 3) plus `expire` refunds (ADR-012). Without a similar escape in Mode 5, a single non-revealing oracle could **permanently lock every SOL and token in that pot**, because there's no admin withdraw. It isn't a drain, but it is a loss of user funds by design. **This must be fixed before any deploy** (bounded recommit plus rollover of the pot into the next round), with Barton's OK.
2. **Mode 4/5 "market-cap tiers" don't behave like market-cap tiers.** File C values market cap at the **locked** buyback price. With a fixed 1B supply that number never changes, so each launch is stuck on one tier forever. At the example price the pot must reach **500 SOL** before any payout or draw. Small launches may never get there, which would leave tax sitting in the PDA indefinitely with no way out. *Arithmetic checked from File C's own figures; code unverified.*
3. **The buyback has no stated funding source.** It "spends SOL already on that same PDA", but the tax is collected in **tokens**, not SOL, and nothing says where SOL comes from. Modes 4/5 also have **no bonding curve and no DEX graduation** (DBC can't do transfer-fee mints, verified above). There's no clear way for the public to buy, trade and generate tax. Product viability is unclear. *Unverified.*
4. **Mode 3 sequential assignment.** The next NFT index is public, so the creator (who knows the art order) and bots can wait for rare pieces. This is the same flaw as sequential artist rarity. Use VRF or another blind assignment, as Mode 2 does.
5. **Tax range up to 10% (1..=1000 bps).** This is high. Claude's earlier design capped tax at 3% (claude-launchpad-history.md), and Stonk.fun's 1–3% tax drained up to 69% of supply from some charts (stonkfun-lessons.md). File C's design doesn't sell tax into the pool, which removes Stonk's main harm, but 10% still punishes every trader. Ask Barton for a lower hard cap.
6. **Mode 4 payout mechanics have unanswered costs and edge cases.** *(Unverified; needs the code.)*
   - Payouts go to the ATA of the **current** owner at pay time, so someone buying an NFT mid-round collects that round.
   - Payouts are strictly in order (`OutOfOrder`). If paying requires creating an ATA, either the crank pays about 0.002 SOL per holder (up to ~20 SOL per round at 10,000 NFTs), or one holder without an ATA could block everyone.
   - If the owner is a program, the tokens could become unreachable.
   - Each payout is itself a taxed Token-2022 transfer, a small haircut.
7. **Mode 5 snapshot cost.** One seat PDA per NFT per round, up to 10,000. The rent payer and whether rent is reclaimed aren't stated, and could be many SOL per round. *Unverified.*
8. **Big unaudited surface added in one day.** File C says nine new instruction files and 27 vault instructions (up from 13 on HEAD), with the launch suite not re-run after the last edit. None of the round-1 audit work (M-01..M-41) covered Modes 1, 3, 4 or 5. Under Barton's "security first" rule, none of this should go near mainnet without the same audit and QA process Mode 2 went through.
9. **Platform revenue is undefined for Modes 1, 3, 4 and 5.** Only Mode 2 has the tiered SOL fee. Barton's fee wallet gets nothing from the new modes unless DBC trade fees cover Mode 1/3. That's a business gap, not a security one.

**Open questions for Barton**
1. Did you approve Modes 1, 3, 4 and 5 (and the raffle) in another chat today? If yes, should they be written into BRIEF/DECISIONS as new decisions that reverse the 2026-09-24 Token-2022/lottery shelving?
2. Will you get a lawyer's view on Mode 5 (raffle) and Mode 4 (tax paid to holders) before any further work?
3. How should that uncommitted tree be preserved (for example, a zip archive, or later a separate non-default branch)? Right now it exists only on a machine we can't reach. This review doesn't change GitHub.
4. Should Mode 5 get recommit plus pot rollover so a stuck oracle can't lock funds? (Recommended: yes.)
5. Is a 10% maximum tax OK, or should it be capped at around 3%?
6. Should Mode 3 use VRF instead of sequential order?
7. Where does Mode 4/5 buyback SOL come from, and how do people buy Mode 4/5 tokens without a curve?
8. Which platform fee applies to Modes 1, 3, 4 and 5?

---

## 4. Cross-file contradictions (the three files disagree with each other)
| Topic | File A | File B | File C | Current project |
|---|---|---|---|---|
| Unwrap | animation only | burns the NFT | returns the NFT to inventory | returns to vault (ADR-016) |
| Number of launch types | icons for 4 styles | 4 styles | 5 modes | 1 (hybrid) |
| Lottery/raffle | — | "remove it, gambling risk" | built as Mode 5 | deferred |
| Style 3/Mode 4 reversible? | — | yes, reversible | appears to have no unwrap | n/a |
| NFT creation | — | all uploaded, not minted on demand | lazy via existing Mode 2 | lazy mint, committed list |
| Rarity | flags the VRF toggle | leaning artist-defined, sequential | Mode 3 sequential; Mode 2 VRF | committed list + VRF |
| Name | Fuze | Fuze | Fuze | "Mintmark" (working) |

---

## 5. Verification log
| Claim | Result | Source |
|---|---|---|
| Pump / PumpSwap program IDs | **Verified** | github.com/pump-fun/pump-public-docs |
| Pump fee-sharing recipients changeable | **Partly verified**: one update, then locked; one recipient change since 2026-03 | pump-public-docs CREATOR_FEE_SHARING.md; crypto.news |
| DBC can't run a transfer-fee Token-2022 mint | **Verified** (docs list metadata and transfer-hook paths only) | docs.meteora.ag DBC Token-2022 support |
| DBC creates the base mint | **Verified** (ADR-014 research test against the real program) | DECISIONS ADR-014 |
| LaunchLab supports Token-2022 transfer fee; hands fee authorities to the platform key at graduation | **Verified** (docs) | docs.raydium.io LaunchLab overview/accounts |
| Token-2022 fee = ceiling division; authority None locks the rate | **Verified** | solana-program/token-2022 v10 source |
| `.keys/` never committed | **Verified** | git history, all branches |
| Barton commits since 2026-09-24 | **None** (all commits agent-authored) | git log / GitHub API |
| Modes 1–5 code, 83 vault tests, IDL lists | **Unverified**: code not on box or GitHub; Mac offline | — |
| Fuze is "clean" | **Contradicted** by web search (Fuze Finance, FUZED token); no registry check done | web search 2026-10-01 |
| Metaplex Core / Tensor / Magic Eden | Not re-checked; already covered in marketplaces-and-ratios.md | — |

---

## 6. Recommended actions (not done; for Barton's approval)

**Bring into BRIEF.md (after Barton confirms)**
1. Name: **Fuze (pending trademark/domain/X check; Hearth backup)**. Note the Fuze Finance collision.
2. Owner rule: **no AI agent or dev tool ever holds a mainnet key or upgrade authority**; multisig signers are humans.
3. Pre-mainnet process: **TVL cap plus an "unaudited beta" label**, bug bounty, fuzzing (Barton decides which are mandatory).
4. **Artist-defined rarity as an option**, with **VRF assignment kept** (never sequential).
5. If Barton confirms File C's 2026-10-01 decisions: record them as a new dated Decisions block plus an ADR that reverses ADR-009's Token-2022/lottery shelving. Include the legal-review gate and the fixes in item 6 below as conditions.
6. Frontend backlog: marketing landing page, coin-is-also-an-NFT explainer, mobile rework, accessibility, empty/loading states, "audit pending" sweep.

**Fix (design/docs only, when allowed)**
- The Obsidian launch-wizard rarity toggle (VRF *vs* pre-committed). It should read "committed list + VRF".

**Drop or keep out**
- Burning the NFT on unwrap (keep return-to-vault).
- Pre-minting the whole collection (keep lazy mint; the art is already committed up front).
- Sequential assignment (in artist mode and Mode 3).
- "Opt-in later" wrap layer for plain launches, and holder early-access allowlists.
- Raydium LaunchLab routing, unless a devnet prototype proves the fee authorities end at a PDA or None.

**Before any of File C's code goes further**
- Preserve the uncommitted tree (Barton's call how; no GitHub change without his OK).
- Mode 5: add bounded recommit plus pot rollover so funds can never be stuck.
- Mode 4/5: answer the buyback SOL source, how tokens trade without a curve, the effectively fixed tier and the 500 SOL threshold, ATA creation/stall behaviour, seat rent, and whether Mode 4 has an unwrap.
- Lower the tax cap (for example 3%), and use VRF for Mode 3.
- Get a legal opinion on Modes 4/5.
- Run the full QA and audit process on every new mode (round 1 covered Mode 2 only).

**Still pending from before (unchanged by these files)**
- Anti-grind at 2.5M/5M (M-19), fee-wallet multisig/policy (M-17), launch fee and curve/DEX trade fees (placeholders), when to resume the auditors, the final name, and whether to create `main`.

---

## 7. GitHub update (re-checked 2026-10-01, about 4:30–5:00 PM MT)

**Read-only.** I cloned fresh into `/workspace/scratch/lp1-recheck`. Nothing was pushed, committed, commented or opened on GitHub, and no file under `/workspace/launchpad` changed except this report. Two proof-of-concept tests (described below) were added **only** to the scratch clone and haven't been committed anywhere.

### 7.1 What's new on GitHub
| Item | Detail |
|---|---|
| Branches checked | `onchain/hybrid-launch` (default), `wip/hybrid-vault`, **`onchain/modes-1-5` (NEW)**. Tag: `audit-baseline-r1` (unchanged). |
| `onchain/hybrid-launch` | Unchanged at `df0d1d9` (2026-09-26 8:04 AM MT). |
| `wip/hybrid-vault` | Unchanged at `df0d1d9`. |
| **`onchain/modes-1-5`** | Created 2026-10-01 **4:29 PM MT** by GitHub user `BartonBase`. One commit on top of `df0d1d9`. |
| New commit | `0a12471` at 2026-10-01 **4:31 PM MT**, author and committer "LAustinSauce" `<155275978+BartonBase@users.noreply.github.com>` (Barton's GitHub account). Message: "Add local Modes 1-5 launch and vault code. Not deployed. Not audited. Branch onchain/hybrid-launch stays at df0d1d9." **This is the first Barton-authored commit in the repo.** |
| Files | 57 files, +17,941 / −759 lines. **12 added:** `docs/GROK_HANDOFF.md` (byte-identical to the `.md` Barton sent), `t22.rs`, `plain.rs`, `launch_burn.rs`, `launch_token22.rs`, `register_plain_dbc.rs`, `register_burn_dbc.rs`, `permanent.rs`, `token22.rs`, `raffle.rs`, and `target/idl/hybrid_launch.json` + `hybrid_vault.json`. **45 modified** (state, constants, config, errors, lib.rs, tests, docs/STATUS.md, docs/ARCHITECTURE.md, Cargo files). |
| Pull requests | **None**, open or closed. |
| Secrets/keys | **None committed.** No `.keys`, `*keypair*`, `id.json`, private keys or byte-array secrets in the commit; `.gitignore` still excludes them. |

### 7.2 Does it build and do the tests pass? (all run on the box)
- **Build: yes.** The production SBF build (`scripts/build.sh`, platform-tools v1.54, separate `CARGO_TARGET_DIR`) succeeded. It produced `hybrid_launch.so` (442,120 B) and `hybrid_vault.so` (1,076,040 B). The build creates fresh throwaway program keypairs in my scratch target dir only, which is normal and not part of the repo.
- **Tests: all green, matching File C's claims.**
  - Host units: hybrid_launch **19**, hybrid_vault **15**.
  - LiteSVM: vault **83**, launch **30**, dbc_graduation **16**, real Switchboard **4**.
  - These suites read the IDL files committed under `target/idl`, which I copied in because `build.sh` didn't generate them.
- QA's `qa_launch` and `qa_regression` suites are commented out on this branch (their files were never committed), so QA's 93 regression tests aren't running against it.

### 7.3 Does it match the handoff (File C)?
- **Yes**, on everything I checked: 9 launch and 27 vault instructions (exact names match the IDL), tax 1..=1000 bps with the fee-config authority None, the tax PDA `["tax_authority", mint]` as withdraw authority, `buy_inventory` at the locked price, the pot tiers, cursor-ordered Mode 4 payouts, the Mode 5 snapshot → commit → reveal → settle order, and no recommit in Mode 5.
- **Mode 2 is effectively unchanged.** After normalizing formatting, `request`, `settle`, `unwrap`, `expire`, `randomness`, `selection`, `pool`, `merkle`, `invariants`, `launch` and `register_dbc` are **formatting-only** diffs. Real changes to shared files are small additions: `verify_recorded` for Mode 3 in `graduation.rs`, a user-pays Core mint helper, Mode 3/4 economics loaders in `config.rs`, and new constants.

### 7.4 Overlap with our programs
- It isn't a separate program. It **extends the same two programs, with the same IDs** (`hybrid_launch` 9Loc4hQZ…, `hybrid_vault` BEfL9dcc…). Deploying it would **upgrade the existing devnet programs in place**, swapping out the tested `df0d1d9` code.
- `hybrid_vault` grows from ~692 KB to ~1.08 MB. A devnet upgrade would need roughly 384 KB of extend (about 2.7 SOL more rent) plus a temporary buffer.
- New modes use **different account types** (`PlainLaunchConfig`, `BurnLaunchConfig`, `T22BurnLaunchConfig`, `TaxVault`, `PermanentVault`) under the same `["launch_config", mint]` seed. Anchor's discriminator checks stop a plain, burn or tax launch from opening a Mode 2 vault (tested).
- There's no duplicated logic of concern. It reuses Mode 2's oracle selection, Merkle leaf checks, DBC allowlist and graduation verifier.

### 7.5 Security findings (code-verified unless marked)

**Critical / high: funds can get stuck forever**
1. **One NFT owner can freeze all Mode 4 payouts and all Mode 5 raffles forever. PROVEN with a scratch PoC test.**
   - Payouts (Mode 4 `claim_tax`) and raffle snapshots (Mode 5 `snapshot_raffle`) must process NFT #0, #1, #2… strictly in order, and each step checks that the NFT still exists.
   - The NFTs are minted with no plugins, so any owner can burn theirs at any time. Metaplex Core allows owner burns by default and leaves a 1-byte dead account (verified in the Metaplex docs and mpl-core source).
   - After a burn, the cursor can never pass that index. The round stays open forever, and every later holder is never paid or entered.
   - New tax keeps piling up in `pending_base` but can never open a new round. There's no admin or escape path.
   - PoC tests `review_poc_burned_nft_freezes_mode4_payout_cursor_forever` and `review_poc_mode5_burn_blocks_snapshot_and_unrevealed_commit_has_no_exit` both pass, which means the freeze happens.
   - **Fix:** add a Core plugin that blocks burning (for example a permanent freeze/burn rule owned by the vault PDA), or skip missing or burned NFTs, forfeiting their share to the next round.
2. **The Mode 5 pot is stuck forever if the oracle never reveals. Confirmed in code.** The only way out of the COMMITTED phase is `reveal_raffle`. There's no recommit, no timeout and no rollover, and Mode 2's protections (`recommit_randomness` ≤ 3, `expire`) weren't carried over. **Fix:** bounded recommit plus a timeout that returns the round to SNAPSHOT/READY with a fresh draw, with the pot kept intact.
3. **Modes 4 and 5 can't work on mainnet at all: nobody can ever get tokens. Confirmed in code.**
   - `launch_token22`/`launch_raffle` mint all 1B into the launch inventory PDA.
   - The **only** instruction that can move tokens out is `buy_inventory`, and it sends them **only to the tax treasury**. The treasury pays **only NFT holders**, and getting an NFT requires wrapping tokens you can't get.
   - There's no curve, no DEX pool and no sale to the public.
   - **The tests hide this.** The harness gives test users tokens by directly editing account balances (`set_token_amount`), and funds the tax PDA with `airdrop`. Neither can happen on-chain.
4. **Any SOL sent to the Mode 4/5 tax PDA is effectively burned. Confirmed in code.** `buyback` spends whatever SOL is on the tax PDA. Nothing ever puts SOL there except voluntary transfers, and the SOL goes to the inventory PDA (`launch_vault`), which has **no instruction that can ever move lamports out**. It's locked permanently.

**High: who controls randomness**
5. **The creator picks the Switchboard queue.** Mode 5's `init_raffle_vault` takes `sb_queue` from the creator (it only checks Switchboard ownership). **The same gap already exists in Mode 2 at `df0d1d9`** (`init_vault` only checks it isn't empty), and the round-1 audit didn't list it.
   - A creator who runs their own Switchboard queue and oracle could at least withhold reveals. In Mode 5 that combines with finding 2 to freeze the pot. They might also be able to influence the value.
   - *Whether a custom-queue oracle can steer the value is UNVERIFIED.* Either way, pinning is cheap.
   - **Fix:** pin approved queues in a compile-time list, like `APPROVED_DBC_CONFIGS`.

**Medium**
6. **Mode 3 and Modes 4/5 assign NFTs in order.** The user must supply the Merkle leaf for index `minted_count`, so the full ordered trait list has to be public. Anyone (and the creator first of all) knows exactly when a rare piece comes up and can wrap at that moment. This is fair-launch sniping, not a fund drain. Use VRF as Mode 2 does, or explicitly accept it.
7. **The pot tier is fixed per launch.** "Market cap" is computed from the **locked** buyback price × 1B, so it never moves. At the test price (0.001 SOL/token) the pot must reach **500 SOL** before any payout or draw.
8. **The tax cap is 10%.** Unchanged from File C; recommend about 3%.
9. **Cost and liveness depend on volunteers.** Each Mode 4 payout (holder token account creation, ~0.002 SOL) and each Mode 5 seat (one new account per NFT per round, never closed) is paid by whoever runs the crank. Nothing rewards them, so up to roughly 15–20 SOL per round at 10,000 NFTs would have to be donated. *Rent figures estimated, not measured.*

**Checked and OK**
- No admin, update, withdraw or pause instructions. The tax rate is locked: config authority None, re-checked at vault init and on every wrap. The tax withdraw authority is a vault PDA.
- No caller-chosen amount, NFT or recipient in payouts or the raffle. Every token account is derived and checked. `buy_inventory` requires the tax PDA to sign, which only `hybrid_vault` can do.
- Owners are snapshotted before randomness; reveal once, no second draw; rejection sampling.
- Arithmetic is consistently checked (`checked_*`, with u128 for values). I found no overflow paths.
- Modes 1 and 3 copy Mode 2's DBC registration checks: allowlist, signer is the pool creator, fixed 1B pre/post, immutable, leftover goes to the buffer PDA.
- Mode 3 requires graduation before wraps and charges the tier SOL fee. Modes 4/5 also charge the tier SOL fee to `PLATFORM_FEE_RECIPIENT`.
- Upgrade authority is unchanged (devnet throwaway deployer). No key material in the repo.
- Minor: `init_raffle_vault` doesn't re-check `mint_authority_none` the way Mode 4 does (harmless, because our own launch created the mint). The `TaxShare` account is created per NFT and never used for payout math.

### 7.6 Sorted

| Category | Items |
|---|---|
| **Useful** | **Mode 1** (plain SPL via DBC). Low risk; reuses our checks. **Mode 3** (burn hybrid) mechanics, with its graduation gate and fee. The no-admin, locked-rate, PDA-only tax design. Mode 5's snapshot-before-randomness pattern. Discriminator separation between modes. |
| **Keep current** | Mode 2 code is untouched, so keep `onchain/hybrid-launch` @ `df0d1d9` as the default and audited baseline. Keep return-to-vault unwrap and VRF assignment. |
| **Risky (must fix before any deploy)** | Burned-NFT freeze (Modes 4/5). Mode 5 oracle stall. Creator-chosen Switchboard queue (Modes 2 and 5). Sequential assignment (Modes 3/4/5). |
| **Not useful as built** | **Modes 4 and 5 in their current form.** No way for the public to get tokens, SOL in the tax PDA is locked forever, and the pot tier is fixed. They need a real distribution path (and probably a different venue, since DBC can't do transfer-fee mints) before they're a product. Plus legal review for the raffle. |

### 7.7 Top fixes (none made; for Barton's approval)
1. **Don't merge or deploy `onchain/modes-1-5` yet.** Keep it as a separate branch. Leave the default branch at `df0d1d9`.
2. **Mode 4/5:** make NFTs non-burnable (or skip burned NFTs in the cursor), and add recommit plus a timeout for the raffle.
3. **Pin the Switchboard queue** to a platform allowlist in **both** Mode 2 and Mode 5. This one also applies to the current, tested product.
4. **Design a real token distribution path** for Modes 4/5, or shelve them. Remove or redirect the SOL-trap in `buy_inventory`, and make the tier use a real price or a fixed SOL target.
5. **Use VRF** (or accept and disclose sniping) for Modes 3/4/5. Lower the tax cap.
6. **Restore QA's suites** on this branch, add regression tests for findings 1–5, then send Modes 1/3 (and 4/5 if kept) through the same audit process as Mode 2.
7. **Record Barton's 2026-10-01 decisions** in BRIEF/DECISIONS (Token-2022 and lottery revived). Get the raffle legal review.
