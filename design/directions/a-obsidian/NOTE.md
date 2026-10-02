# Direction A: Obsidian (recommended flagship)

Obsidian treats the launchpad like a premium financial product: near-black surfaces, hairline borders, soft glass panels and one confident accent (iris violet, #8F7DFF) kept for primary actions and bonding-curve progress, so the page stays calm and the important thing stands out. Type is Geist for UI and headlines, with tight tracking and tabular numerals. Geist Mono handles labels and addresses, which gives the precise Linear/Vercel/Phantom feel without looking like a terminal. Wrap/unwrap is shown as a two-sided **Convert** panel, like a wallet swap: tokens on one side, NFTs on the other, exact amounts, an example fee, one primary button, and the explainer line "1,000,000 NOCT converts to 1 NFT. Convert back any time for exactly 1,000,000 NOCT". The home hero previews the same panel. Holdings appear as a portfolio card: total value, token balance, NFTs held with thumbnails, and a highlighted "Tokens available to convert: 3 NFTs · 1.152 SOL" row. Nothing fills up or gets gamified. The direction appeals most to the crypto-native but quality-sensitive crowd who use Phantom, Jupiter and Coinbase Advanced, and it still reads as trustworthy and simple to newcomers.

**Current model (2026-09-24, latest): hybrid-only.** Every launch is an SPL-404 hybrid: a classic SPL token plus the Metaplex MPL-Hybrid converter. Token-2022, transfer tax, treasury, tickets and prize draws are out of scope for now; that work is parked in `../../deferred/token-2022/`.

- **Pages:** `home.html`, `token.html` (Nocturnes on the bonding curve, 64%), `token-graduated.html` (Nocturnes after graduation) and `launch.html`, each with a full-page 1440px PNG.
- **Launch page:** one flow in four steps:
  1. Basics.
  2. Supply and conversion: a seven-option ratio control (50K to 5M tokens per NFT, each showing its max collection size and fee), a collection-size input validated to 100 to min(10,000, 1B ÷ ratio), a graduation target field (default 85 SOL, chain minimum 10 SOL, no reserve maths), a plain-language lazy-minting line, and a live math panel with a thin allocation bar and an NFT price estimate.
  3. Traits and rarity: cosmetic tiers and re-roll settings.
  4. Review and launch on devnet.

  The fixed-facts row reads: "Fixed at 1,000,000,000. No one can mint more.", mint authority revoked, conversion rate set at launch (beta: 3-of-5 multisig, public 7-day delay), and no transfer tax.
- **Rarity is cosmetic only.** NFTs carry subtle tier badges (Common, Uncommon, Rare, Legendary) in muted tints: slate, sage, iris and champagne. The floor explainer (graduated page only) reads floor = ratio × token price (1,000,000 × 0.000000612 SOL = 0.612 SOL). Every Nocturne converts back for exactly 1,000,000 NOCT whatever its rarity, and any premium is set by buyers and sellers, not guaranteed. Converting and re-rolling pick at random with verifiable randomness from Switchboard.
- **Fees and disclosures:** superseded, see *Fees* below. "No transfer tax" stays. Trade fee on the curve: 1% (example). Audit shown as pending.
- **Trade maths (Nocturnes, no tax):** 1 SOL − 1% example trade fee = 0.99 SOL ÷ 0.000000384 = **2,578,125 NOCT**. Sells use tokens × price × 0.99.

**Update (2026-09-24): positioning, marketplaces and wider ratios.**

*Positioning.* The copy assumes NFT culture is alive and well, and never mentions a decline. It centres on two ideas:
1. SPL-404: one asset you can hold as a token or as an NFT, switching any time at a fixed rate.
2. Showcasing the memes and their collections.

Every "revive / bring back" line is gone.

Hero headline options:
1. **"Trade the meme. Collect the art."** (used). It's short and confident, and it names both forms of the asset.
2. "One asset. Token or NFT, your call." This is the most literal SPL-404 statement.
3. "Every meme comes with a collection." This leads with the showcase idea.

Subhead: "On Mintmark, a meme token and its NFT collection are one SPL-404 asset. Hold it as tokens or as NFTs, and switch any time at a fixed rate: 1,000,000 NOCT is one Nocturne, and one Nocturne is always 1,000,000 NOCT."

*Collections are the visual star.*
- The hero is now a mosaic of six collections with traits on the large tile, plus a compact "Switch any time" card.
- A new **Featured collection** spotlight (Nocturnes) shows large art, stats, trait values, rarity tiers, five NFTs with tier badges, and CTAs.
- **Trending collections** is a 3-column grid with bigger 4:3 art and a four-NFT strip per card. It has nine cards, including Monolith, a small collection: 200 NFTs at 5,000,000 MONO each, price 0.000000206 SOL, market cap 206 SOL, floor 5,000,000 × 0.000000206 = 1.03 SOL.
- How it works is now "One asset, two forms": buy the token, convert to an NFT, collect/show/list it, switch back.
- The guarantees intro, CTA band, footer blurb, footer links and a `<meta name="description">` on every page were updated to match.

*Marketplaces.* The token page has a **Trade NFTs** card:
- "Trade on Tensor" and "Trade on Magic Eden" buttons: plain text with a generic external-link icon, no logos. They're labelled as external links and open in a new tab. The collection-page URLs are placeholders.
- The line "Mintmark handles buying on the curve and converting. NFT listings and sales happen on Tensor and Magic Eden."
- A **Listings** preview of four Nocturnes, tagged "Demo data · via marketplace".

*Launch page ratios.* The eight options are split into two labelled groups of 2×2 tiles:
- **Large collections:** 50K / 100K / 200K (10K dropped 2026-09-25).
- **Small, scarce collections:** 500K / 1M / 2.5M / 5M.

Each tile shows the max collection size and "≈ X SOL per NFT (est.)" at the example launch price of 0.000000028 SOL. Collection size can be anything from 100 up to min(10,000, 1B ÷ ratio) (cap added 2026-09-25). Validation, live maths, status, helper copy and the review step all reflect the range, and a scripted check covered 5M@99/100/200, and 2.5M@400/401 (the original 10K case no longer applies).

**Update (2026-09-25): single graduation, flat SOL fees, lazy minting (5:13 PM MT).**

## Graduation (lazy minting, Barton 2026-09-25 5:13 PM MT)

Lazy minting replaces the 4:55 PM batch pre-mint.
- Every launch starts on a bonding curve. The token trades as a plain token there, converting is closed, and no NFTs exist yet. Copy says "when the curve fills" (the target, 85 SOL by default, is shown on the curve card).
- **Nothing is minted at graduation.** Two things happen:
  1. Liquidity moves to a DEX.
  2. Converting opens, and the art is revealed.
- **Each NFT is minted the first time a capture (token → NFT) picks it.** Each capture or re-roll puts up a 0.0063 SOL mint deposit on top of the platform fee; about 0.003–0.004 SOL of it is kept only if the pick is minted for the first time (Solana rent + Metaplex Core fee), and the rest is refunded at settle. Later captures of the same NFT (after a release) get the whole deposit back.
- Tradeoff, stated in the copy: marketplaces only show NFTs that have been captured so far. The collection page fills in over time.
- During the curve, traits and rarity odds can be previewed.
- `token.html` (curve state):
  - no mint-reserve line;
  - graduation steps: 01 liquidity to a DEX, 02 converting opens (art revealed, each Nocturne minted on first capture), 03 NFT trading follows;
  - a locked Convert panel showing the capture cost once open and "Converting back: No platform fee";
  - a Capture cost breakdown in the Fees card, and "Capture cost ≈ 0.015 SOL + randomness" in the facts card (superseded 10/1: deposit wording, no randomness fee).
- `token-graduated.html` (after graduation):
  - "Graduated at 85 SOL · 412 of 1,000 captured so far", tagged Example;
  - the Convert panel itemizes the capture cost;
  - the Re-roll panel shows a possible mint cost (see below);
  - holdings minis read "Captured so far 412" and "Not yet minted 588";
  - the Trade NFTs card explains that marketplaces show captured Nocturnes only.
- `home.html`: step 01 now ends "the token graduates and converting opens". The "collection is minted" wording is gone. There was no Glasshouse target text on the page. Glasshouse (200K / 5,000 NFTs) is now valid at 85 SOL anyway.
- `launch.html`: the mint-reserve readout, the "too large for target" error and its quick fixes, the `?ratio/size/target` demo params, and the reserve/10% review lines are all removed. `launch-too-large.png` is deleted. The only size validation left is 100 to min(10,000, 1B ÷ ratio). The graduation target field stays (minimum 85 SOL at the time; 10 SOL since the 10/1 chain alignment). Step 2 and review carry: "NFTs are minted one at a time, the first time someone captures them. The collector pays the small mint cost." Step 2 has a "Mint cost per NFT: about 0.005 SOL, once per NFT" box. Review shows "NFT minting: On first capture", and the Fees box lists the mint cost and randomness. (All superseded 10/1: 0.0063 SOL refundable deposit, no randomness fee.)

### Capture cost formula (for the Frontend Engineer)

```
platform_fee(ratio) = 0.002 SOL if ratio == 50K
                      0.005 SOL if ratio in {100K, 200K}
                      0.01  SOL if ratio >= 500K          (fixed in LaunchConfig, hard cap 0.01)
mint_deposit        = 0.0063381 SOL per capture/re-roll request    (hybrid_vault MINT_ESCROW_LAMPORTS = 6,338,100)
mint_spend          = Core asset rent + Core create fee, only if the pick was never minted (measured 0.0031–0.0044 SOL); refund = deposit − spend
randomness          = no separate fee on chain (BRIEF 9/25 said "at cost": flagged)

capture  = platform_fee + mint_deposit up front; net = platform_fee + (picked NFT never minted ? mint_spend : 0)
re-roll  = same as capture (same tier fee, same deposit)
burn     = platform_fee + mint_spend paid directly (no deposit)
release  = 0 platform fee, 0 mint cost (NFT already exists); network fee only; returns exactly N tokens
```

- The pick is blind, so the UI can't know up front whether the mint cost applies. It shows **Paid now** (fee + 0.0063 SOL deposit, e.g. Nocturnes (1M): 0.0163 SOL) and **Net after refund** (0.01 to ≈ 0.014 SOL).
- BRIEF says the mint cost is "escrowed at request". Whether it is refunded when the pick is an already-minted NFT still needs **engineering to confirm**. The copy says "only the first time an NFT is captured" and doesn't promise a refund.
- Disclosure used next to every itemized total: "The mint cost is Solana rent plus the Metaplex Core fee, not a Mintmark fee."
- **No randomness fee.** The chain charges none; don't show a randomness line in totals (2026-10-01 chain alignment).

### Re-roll
A re-roll can land on an NFT that hasn't been minted yet. In graduation-design.md §2.5, unminted indices are part of the VRF candidate set, and §4.4 says a re-roll can "trigger a first mint that is still paid by the requester". So the re-roll panel shows the 0.0063 SOL mint deposit (refunded except ≈ 0.003–0.004 SOL if the new pick is minted for the first time), paid now 0.0163 SOL, net 0.01 to ≈ 0.014 SOL.

## Fees

- A flat SOL **platform fee** applies to every capture (token → NFT) and every re-roll. **Release (NFT → tokens) is free** (BRIEF, RELEASE IS FREE, 5:04 PM MT). The fee is fixed per collection at launch and set by the ratio:
  - 50K tokens per NFT: **0.002 SOL**
  - 100K and 200K: **0.005 SOL**
  - 500K, 1M, 2.5M and 5M: **0.01 SOL** (Nocturnes, Monolith)
- No tokens are ever taken. 1,000,000 NOCT converts to exactly one NFT and back to exactly 1,000,000 NOCT.
- A re-roll costs the same platform fee as a capture (re-roll fee = capture fee), plus the refundable 0.0063 SOL mint deposit (about 0.003–0.004 SOL kept if the new pick is minted for the first time). The old "half the cost of converting back and again" line is removed, because release is now free and it no longer holds.
- Removed: token-denominated re-roll fees and burns, creator convert fees, and "current supply / burned" facts. The mint-deposit disclosure (0.0063 SOL, mostly refunded) is back under lazy minting, as a Solana/Metaplex cost and not a Mintmark fee.
- Kept: "Network fees apply" (a small Solana transaction fee) and "No transfer tax".
- Disclosure copy: "Platform fee: <X> SOL per capture (tokens to NFT) or re-roll, set for this collection at launch based on its ratio. Converting back is free. No setting can raise it. During beta, the program can only be changed by a 3-of-5 multisig with a public 7-day delay; it will be permanently frozen after the final audit."
- Fee wallet shown as "Fee address: 7xKp…3fQa (example)". A "Beta" tag sits in the header.
- The launch page shows each ratio tile's fee. Review, the re-roll settings and the fee disclosure update live with the chosen ratio.
- Still shown as examples: launch fee 0.02 SOL and curve trade fee 1%.

**Update (2026-09-25, Barton approved 4:55 PM MT; reserve removed 5:13 PM): bonding curve, seven ratios.** (Barton's 9/25 decision named Meteora DBC; see the 2026-10-01 chain-alignment entry.)

## Ratios and collection size
- There are seven ratios: 50K, 100K, 200K, 500K, 1M, 2.5M and 5M. **10K is dropped everywhere.**
- The launch page groups them:
  - Large collections: 50K to 200K (three tiles);
  - Small, scarce collections: 500K to 5M (2×2).
- Collection size runs from 100 up to min(10,000, 1B ÷ ratio). Max per ratio:

  | Ratio | 50K | 100K | 200K | 500K | 1M | 2.5M | 5M |
  |---|---|---|---|---|---|---|---|
  | Max NFTs | 10,000 | 10,000 | 5,000 | 2,000 | 1,000 | 400 | 200 |

- Fee tiers: 50K = 0.002 SOL; 100K and 200K = 0.005 SOL; 500K and up = 0.01 SOL.
- Home demo ratios were moved onto approved values: Glasshouse is now 200K / 5,000 NFTs, and Field Notes 2.5M / 400 NFTs.

## Graduation target (reserve removed 2026-09-25 5:13 PM MT)
- The curve: a bonding curve (see the chain-alignment entry). The graduation target defaults to **85 SOL**; the chain minimum is **10 SOL** (0.1 SOL on the devnet test build). Updated 2026-10-01.
- The mint reserve, the 10% rule, the refuse-unfundable-launch error and `launch-too-large.png` were **removed** with lazy minting. Any collection size from 100 to min(10,000, 1B ÷ ratio) is valid at any target.
- `token.html`: "Bonding curve", "54.4 of 85 SOL raised" at 64%, "200M NOCT set aside for the DEX pool".
- `token-graduated.html`: "Graduated at 85 SOL · 412 of 1,000 captured so far" (Example).
- Market cap figures are tagged "Example figures".
- DBC's 25% unsold buffer is locked in a program-owned account. Footnote on the token pages' facts card: "Unsold curve tokens are locked by the program, not burned; supply stays 1,000,000,000."
- Unchanged, still tagged as examples: launch fee 0.02 SOL, 1% curve trade fee, 0.25% DEX pool fee.

**Update (2026-10-01): Fuze rename, Ember accent, bomb mascot (still images only).**
- Visible brand is now **Fuze** (working name) on all Obsidian pages, through a local `BRAND` override in `a_obsidian.py` (common.py untouched).
- Accent: Ember **#FF6A2B** (from the mascot's fuse spark) replaces violet #8F7DFF everywhere in Obsidian. Filled accent buttons use dark text #120805 for contrast. Error/down colour is now #F25C7A. Details: `../../mascot/palette.md`.
- Headlines use **Bricolage Grotesque 800** (condensed); UI stays Geist and Geist Mono.
- Progress bars are now a static fuse: braided track, ember trail and a still spark at the tip. No animation anywhere; the card hover transition on home was removed.
- Mascot (static PNGs from `../../mascot/`): home hero front pose beside the headline; token.html shows the loading pose at the end of the curve fuse bar, the boom pose beside "What happens at graduation", and the convert pose in the locked Convert panel.
- Home "How it works" now has a small launch-styles row: Hybrid (in this preview), Plain and Burn (planned, definitions to be confirmed; not defined in BRIEF).
- token-graduated and launch pick up the accent, font token and brand automatically. Their layouts are unchanged and their PNGs were regenerated.

**Update (2026-10-01, Barton 5:13–5:14 PM MT): Armory, no mascot art, design tokens.** Supersedes the Fuze entry above.
- The brand is **Armory** (final). The "Working name" tag and "(working name)" footer text are removed. The ARMS platform ticker is not shown on any page.
- The nav and footer mark is a plain-text **Armory** wordmark in Bricolage Grotesque 800. The circle-and-square mark is removed.
- Mascot art is pending from Barton (Higgsfield). The bomb poses are replaced with dashed "Mascot, coming from Higgsfield" placeholders of the same footprint (home hero; token curve, graduation and Convert panel). The bomb art is archived in `../../mascot/archive-bomb/`.
- Progress bars are a clean track with an Ember fill and a still accent dot at the tip, with no fuse styling. No animation anywhere; the launch allocation-bar transition is removed.
- Design tokens extracted from this CSS: `../../system/tokens.json`.

**Update (2026-10-01, itinerary steps 3–6): launch types, new pages, logic spec.**
Sources:
- ADR-020 (`docs/DECISIONS.md` line 577; commits 81d0b8d 17:04 MT and c43be58 17:16 MT) decides what's live: Plain and Burn kept, Tax split and Raffle shelved.
- Branch `fix/modes-1-5`, commit 0a12471 (16:31 MT, read only), defines the mechanics: `docs/GROK_HANDOFF.md`, `programs/hybrid_launch/src/instructions/plain.rs`, `launch_burn.rs` and `programs/hybrid_vault/src/instructions/permanent.rs`.
- The two don't conflict on Plain or Burn.

### Launch types
| | Plain (Mode 1) | Hybrid (Mode 2) | Burn (Mode 3) | Tax split (4) | Raffle (5) |
|---|---|---|---|---|---|
| Status | Live | Live | Live | Coming soon | Coming soon |
| Token | Classic SPL, 1B fixed | Classic SPL, 1B fixed | Classic SPL, starts at 1B, burns down | Token-2022, transfer fee | Token-2022, transfer fee |
| Authorities | Mint + freeze revoked, no update ix | Mint + freeze revoked | Mint + freeze revoked | — | — |
| Curve | Bonding curve → DEX at graduation | same | same, vault opens after graduation | — | — |
| NFTs | None (no ratio, collection, wrap, unwrap, platform fee) | Reversible wrap/unwrap | One-way: burn exactly the ratio → mint next NFT | Lock ratio → next NFT | Lock ratio → next NFT |
| Which NFT | — | Switchboard VRF (committed art) | `minted_count` index, collection order, Merkle-verified leaf, no VRF | sequential | sequential |
| SOL fee | none | tier fee on capture + re-roll; release free | tier fee per burn → platform fee wallet | — | — |
| Mint cost | — | 0.0063 SOL deposit per capture/re-roll, refunded except ≈ 0.003–0.004 SOL if the NFT is minted new | ≈ 0.003–0.004 SOL every burn, paid directly | — | — |
| Back to tokens / re-roll | — | Yes, exact, free / yes | No / no (no unwrap, re-roll or expire) | — | — |

Tier fee (Hybrid and Burn): 50K = 0.002, 100K/200K = 0.005, 500K–5M = 0.01 SOL. Ratios: 50K, 100K, 200K, 500K, 1M, 2.5M, 5M. Size: 100 to min(10,000, 1B ÷ ratio).

### Wizard math (`launch.html`; `launch-plain.html` and `launch-burn.html` are the same page with another type preselected)
- **Common:** `T` = graduation target, valid if `T ≥ 10` (chain minimum; default 85; on the curve path T must match a platform-approved curve config). Supply = 1,000,000,000. Coming soon types can't be selected.
- **Plain:** valid iff `T ≥ 10`. No ratio, size or art step; steps renumber 1–4.
  - Aside shows supply 1B and an Example split of 800M sold on the curve and 200M set aside for the DEX pool (set by the approved DBC config).
  - Per-NFT fees: none.
- **Hybrid:**
  - `max = min(10,000, 1B / ratio)`; valid iff `100 ≤ size ≤ max` and `T ≥ 10`.
  - `nftTok = size × ratio`; `pct = nftTok / 1B`. Aside reads "can be in NFT form at once", plus `1B − nftTok` that always stays tokens.
  - Holder pays per NFT: `fee(ratio)` + 0.0063 SOL mint deposit (mostly refunded).
  - NFT price estimate = ratio × example curve price.
- **Burn:** same validity and `max` as Hybrid.
  - Aside: "if every NFT is minted, `nftTok` (pct) burned for good, supply ends at `1B − nftTok`".
  - Holder pays per burn: `fee(ratio)` + ≈ 0.003–0.004 SOL mint, paid directly.
  - Review adds "Most that can be burned", "NFT order: collection order, public" and "Converting back: not possible".
- **Randomness is no longer a toggle (`#assign` removed).** Step 4 shows fixed facts:
  - Hybrid = art and traits committed before converting opens, at vault creation after launch (Merkle root) **and** Switchboard VRF picks which NFT you get, at every capture and re-roll.
  - Burn = committed art + collection order, plus an honest note that the next piece is public.
- Default size is 500 at 1M (50%), so the bar shows a real split.
- Deposit cap + "Unaudited beta" slot (`cap_slot()`) sits above the acknowledgement. The acknowledgement now includes "unaudited beta".

### Plain token page (`token-plain.html`, Salt Flats, on curve 88%)
- Curve card shows the full lifecycle: curve → DEX, nothing else.
- "What a Plain launch is" facts card and token-only holdings.
- Trade fees only; type-specific locks (no update instruction, unsold curve tokens locked, no NFT side ever).
- No convert, NFT, re-roll, marketplace or capture-cost sections.

### Burn token page (`token-burn.html`, Ferro, graduated)
- Burn-to-mint panel: you burn 1,000,000 FERRO and receive **the next NFT in collection order** (#0312).
  - Itemized: 0.01 SOL fee + ≈ 0.003–0.004 mint, total ≈ 0.013–0.014 SOL.
  - "There's no way back" note.
- Holdings: Burn NFTs are **not valued** (no token backing, so no floor = ratio × price).
- Collection order card: next five pieces, minted 312 of 500 (Example).
- Supply card: 1B − 312M = 688M (Example), cosmetic rarity, and a cost-to-mint breakdown.
- Ferro demo changed from 1,000 to 500 NFTs so the most that can ever burn is 500M (50%), not the whole supply.

### Hybrid updates
- `token.html` and `token-graduated.html`:
  - Hybrid type chip; facts read "Launch type: Hybrid · SPL-404" and "Randomness: Committed art + Switchboard VRF".
  - Audit shows "Not yet · unaudited beta"; deposit cap slot in the trade panel.
- State pages, rendered as element crops with `_build/crop.js`:
  - `token-capture.html/.png`: review capture. You lock 1,000,000 NOCT (held, not spent) and get 1 random Nocturne.
    - Itemized: platform fee 0.01, mint deposit 0.0063 (refunded except ≈ 0.003–0.004 if the NFT is minted new), network fee; paid now 0.0163.
    - Steps: sign → randomness → reveal.
  - `token-release.html/.png`: NFT → tokens. Pick #0142 → exactly 1,000,000 NOCT. No platform fee, mint cost or randomness. Release is free.
  - `token-reroll.html/.png`: re-roll confirm with the same cost rows and 3 steps.

### Site-wide
- Beta banner (`BETA_BAR`, a step 7 hook) on every page: Unaudited beta, devnet, deposit cap (Example), link to `trust.html` (not built).
- Nav: Explore, Launch, Portfolio, Trust, FAQ.
- Deposit cap is `DEP_CAP = "10 SOL per wallet"`, **always tagged Example**: no value exists in BRIEF, docs or the branch.
- Home:
  - "A coin that's also an NFT" explainer with a coin ⇄ NFT diagram.
  - 5-type comparison table replaces the "to be confirmed" row.
  - Trust section adds unaudited, authorities, multisig and deposit cap, plus a link to trust.
  - Copy no longer says every launch is SPL-404.
- New pages:
  - `explore.html`: type and phase filters; Soon tabs for Tax split and Raffle.
  - `portfolio.html`: summary, positions table, NFTs by collection as a list, history. No tile grids.
- Design system: `../../system/components.html`, `icons/*.svg`, tokens changelog in `../../system/README.md`.

**Update (2026-10-01, Barton 5:30–5:40 PM MT): itinerary steps 7–8 (safety UI, trust/bounty/FAQ, mobile, accessibility, wording) and the knight placeholder.**

*Step 7: safety UI*
- "Unaudited beta" chip, consistently: the site-wide banner above the nav; next to the deposit-cap slot in every trade panel and in the wizard's review/acknowledgement; and as a `UA_LINE` ("Unaudited beta · only use amounts you can afford to lose") on every convert panel (graduated, on-curve, capture, release, re-roll) and the Burn panel.
- Deposit cap: `DEP_CAP` "10 SOL per wallet", always tagged **Example** (no real value is decided). Shown in trade panels, the wizard, portfolio, trust and the FAQ.
- New "Locked authorities" card on every token page (Plain, Hybrid, Burn), placed before the renamed "Rules for this token" card. Facts and sources (read-only, branch `fix/modes-1-5`):
  - Mint authority revoked and freeze authority never set, on all three types (`register_dbc`, `register_burn_dbc` and `register_plain_dbc` require None).
  - Token metadata immutable on all three (`DBC_TOKEN_AUTHORITY_IMMUTABLE`).
  - Launch settings: no update or close instruction exists.
  - Hybrid collection: PDA update authority, no update instruction once the trait root is committed (admin-multisig-timelock.md §2, T-HV-06). Burn collection: PDA authority, Merkle-checked per NFT; **immutability is not claimed for Burn**.
  - Fee recipient: a code constant (`PLATFORM_FEE_RECIPIENT`).
  - Pause: no key can halt a vault instruction (ADR-015). For Plain: "none in Armory's programs; third-party curve/DEX not ours".
  - **Program upgrades: shown as "Not locked yet" (amber).**
- **Upgrade-policy correction:** the brief's "3-of-5 multisig, 7-day delay" was presented as current on home, token fees, the wizard and the footer. But audit-fixes M-09 is still OPEN ("one upgrade key per program today"), and DECISIONS Q8 (signers) and N5 (freeze timing) are open. All copy now says: today, one development key per program on devnet; **planned** for mainnet, a 3-of-5 multisig with a 7-day public delay; frozen after the audit plus a stabilization period (the admin doc proposes 3 months; not decided). Constants: `UPG_SHORT`, `UPG_LONG`.
- New pages (`_doc()` layout with a sticky section index):
  - `trust.html`: audit status (two internal reviews, 41 merged findings, regression tests per fix, M-09 open; planned: paid third-party audit, live bounty, cap on, human-only upgrade keys, verifiable build hash); authorities table across types; upgrades timeline; key policy; randomness (committed Merkle root, Switchboard VRF, pinned queue `APPROVED_SB_QUEUES`, Burn order public); fees; deposit cap; known limitations; program-address placeholder; report CTA.
  - `bug-bounty.html`: "not live yet"; scope marked **Proposed**; how to report with a **labelled placeholder** contact; ground rules; "Rewards to be announced". No amounts and no real address.
  - `faq.html`: Basics, Hybrid, Burn and Plain, Fees, Safety, Coming soon. Every answer comes from BRIEF / DECISIONS / ADR / branch facts.
- Key rule: no exact wording exists in BRIEF or DECISIONS (ITINERARY: "a published rule that no AI holds mainnet keys"; admin doc: "No mainnet keys on the box"). Written plainly as **"No AI agent holds mainnet keys. Mainnet keys are held by humans only."** plus "This applies to every mainnet key: program upgrades, the multisig, and the platform fee address" (my extension). **Needs Barton's sign-off.**
- Footer links now point to the real anchors (trust#audit, #keys, #randomness; bug-bounty.html; faq.html#curve).

*Step 8: mobile*
- `RESP_CSS` adds desktop-first media queries from the tokens.json breakpoints (max-width 1199.98 / 899.98 / 639.98). Grids collapse to one column, the nav gets a menu button, banner details hide, wide tables scroll inside their card, and `main` clips stray decorative overflow. Mobile renders at 390px: `home-mobile.png`, `token-mobile.png` (Hybrid on curve), `launch-mobile.png`, `portfolio-mobile.png`; `scrollWidth` = 390 on all four.

*Step 8: accessibility* (full results in `../../system/a11y.md`)
- Dim text `#5d626c` → `#858a94` (3.25 → 5.75:1 on bg; Coming soon chip 3.11 → 4.70). Muted `#8b909a` → `#a0a5ae`. Static `:focus-visible` outline, skip link, `<main>` landmark, labelled inputs and icon buttons, decorative SVGs hidden, chart labelled. Heading order fixed on token pages and portfolio (card titles h3 → h2), footer and checklist headings too. Checker: `_build/a11y_check.py`.

*Step 8: wording sweep (fixes)*
- Jargon explained on first use: home lead (bonding curve + graduation), home comparison (VRF = verifiable randomness), explore lead (graduation + curve), a glossary line under every token header (bonding curve, graduation, plus SPL-404 on Hybrid), token facts "Hybrid · SPL-404 (token + NFTs)", fee row "Switchboard VRF (verifiable random function)", launch lead (curve + graduation) and launch VRF fact. Trust and FAQ already define them.
- "Transfer tax" now appears only as "No transfer tax". The Tax split and Raffle descriptions say "A fee on every transfer, fixed at launch, …", and the home comparison row is "Fee on transfers".
- FAQ "Are rare NFTs worth more?" → "Does rarity change what I get back?" ("No. Rarity is cosmetic…").
- Banned words: no "jackpot", "valuable" or "guarantee" claims (the only "not guaranteed" is a negation in the launch art help). "Upgrade" appears only for **program upgrades / upgrade authority** (needed for step 7), never about NFTs or rarity.
- Stale names: none in pages, system files or the sitemap. Mintmark and Fuze remain only in the dated, superseded entries of this NOTE (history).
- Fee consistency: "0.0049" → "≈ 0.005" and "0.0098" → "≈ 0.01"; static fee placeholders now show 0.01 (the default 1M ratio). Tiers everywhere: 50K = 0.002, 100K/200K = 0.005, 500K–5M = 0.01 SOL; mint deposit 0.0063 SOL, mostly refunded (superseded 10/1, see below); release free.
- "Pending" audit wording replaced by "Not yet · unaudited beta" / "Not audited yet".

*Placeholder mascot (Barton 5:40 PM, override of the shield idea; no shield files were made)*
- Every mascot box now shows a **temporary comic-ink knight** (`../../placeholder/knight-placeholder.svg`, code-drawn with `../../placeholder/_build/knight.py`) at the same footprint, with alt "Placeholder art, mascot coming soon" and a tiny muted "Placeholder art" caption on boxes 120px or taller. Ember is used only on the plume and the shield emblem. The pages have no favicon or logo from it; the nav keeps the text wordmark. To replace it with the Higgsfield art, change `MPH` in `_build/a_obsidian.py`.

**Update (2026-10-01, ~6:00 PM MT): chain alignment (Frontend Engineer devnet findings, verified read-only against `fix/modes-1-5`, BRIEF and DECISIONS).** Rule: published copy never claims what the chain doesn't do; where the chain contradicts a Barton decision, the page states what's true now and the conflict is flagged.
1. **Deploy status.** Only Hybrid is deployed on devnet (STATUS.md: deployment 2026-09-25; Modes 1 and 3 "Not deployed. Local LiteSVM"). Plain and Burn now show **Pending deploy** (`STATUS` map + `status_chip()`): home comparison and heading ("One is live on devnet"), explore tabs and lead, wizard picker (the launch button is disabled for pending types), Plain/Burn token headers, trust authorities note, FAQ, components, sitemap.
2. **Graduation target.** Default 85 SOL (constants.rs `DEFAULT_GRADUATION_THRESHOLD_LAMPORTS`, DECIDED). Minimum is now the chain's **10 SOL** (`MIN_GRADUATION_THRESHOLD_LAMPORTS`; 0.1 SOL on the devnet-e2e build), not 85. Help text adds: "It must match one of the platform's approved curve settings", because on the curve path the threshold is read from the allowlisted curve config, not from the creator (register_dbc.rs: "threshold from DBC's config"). **Flag:** Barton's 85 SOL minimum dates from the mint-reserve rule, which was dropped with lazy minting.
3. **Mint deposit.** Hybrid capture and re-roll: a 0.0063 SOL deposit (`MINT_ESCROW_LAMPORTS` 6,338,100), refunded at settle except about 0.003–0.004 SOL when the pick is minted for the first time (DECISIONS CD Q1, measured 0.0031–0.0044). Paid now: 0.0163 SOL at the 1M ratio; net 0.01 to ≈ 0.014 SOL. Burn: no deposit, ≈ 0.003–0.004 SOL paid directly (`create_asset_user_pays`). Updated: capture, re-roll and graduated panels; fees card; facts; wizard lazybox, review and live math; home footnote; portfolio history; trust table and limitations; FAQ; components.
4. **No randomness fee.** All "At cost" / "+ randomness" lines are removed; VRF and committed-art explanations stay. **Flag:** BRIEF 9/25 5:04 PM says "VRF cost paid by the requester at cost"; the vault charges no such fee.
5. **Re-roll settings removed** (no on-chain switch or fee input). Re-rolls are built in and cost `min(stored, current tier, 0.01)`, the capture fee (request.rs header), plus the deposit.
6. **Art step restructured.** There's no upload on chain. The real inputs are collection name (≤ 32), a content-addressed collection URI (`ipfs://` or `ar://`), `trait_root` and `trait_schema_hash`, committed at vault init (init_vault.rs `InitVaultParams`). Each NFT is minted from a leaf (8 trait values, salt, image and JSON SHA-256, metadata URI) plus a Merkle proof (asset_source.rs). The wizard now says plainly that art is prepared off-chain. The token-image drop zone became a "Token metadata URI" field, and the rarity tier editor was removed. **Flag:** no tooling for building leaves or hosting the leaf list is designed, and who hosts it is open.
7. **Curve wording is neutral** ("bonding curve", "Platform-approved curve settings") on every page and in this NOTE. **Flag:** the branch's only curve program is Meteora DBC (dbc.rs; `APPROVED_DBC_CONFIGS`). The app's "native" launch is native `launch` with **no curve at all** (the vault can never open, error 6037), and its Meteora DBC path is simulate-only on devnet. The chain therefore doesn't contradict Barton's 9/25 Meteora DBC decision; neutral wording was applied as requested.
8. **Defaults changed** (wizard): minimum target 85 → 10 SOL; launch fee "0.02 SOL (Example)" → "None on chain today" (no launch-fee transfer exists; launch.rs only tops up mint rent); curve split 80/20 (Example) → 55% curve / 20% DEX pool / 25% locked buffer (devnet curve settings, DECISIONS); curve trade fee 1% and DEX pool 0.25% are now labelled as devnet settings; decimals 6 shown in review; re-roll toggle default removed. Unchanged: ratio 1M, collection size 500 (the build's default), target default 85.
- Also: no per-wallet deposit cap exists in the branch programs (app armory.ts: "No cap exists on-chain yet"). The beta bar now reads "Deposit cap not enforced yet"; cap slots tag the 10 SOL value "Example · not enforced yet"; trust and the FAQ say it's planned and not enforced, and the mascot alt text is "Placeholder art, mascot coming soon". The mobile wallet button is already neutral white (`.btn.p`), so no change was needed there.
- Commitment timing corrected: art is committed at vault creation, which needs the existing launch config (init_vault.rs; Burn: init_permanent_vault in permanent.rs), so copy now says "before converting opens" or "before burning opens" instead of "before launch".

## 2026-10-01 7:03 PM: Switchboard approved
Barton approved naming Switchboard publicly as the randomness provider. The pages already said "Switchboard VRF", so nothing on the pages changed.
