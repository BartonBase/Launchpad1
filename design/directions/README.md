# Visual directions: Mintmark launchpad (working name)

Three distinct static HTML directions. Each folder has `home.html`, `token.html`, full-page 1440px-wide screenshots (`home.png`, `token.png`) rendered with headless Chrome via Playwright, and a `NOTE.md`. All pages use the same demo data: the sample hybrid collection **Nocturnes (NOCT)** with a 1B supply at launch, 1,000,000 NOCT = 1 NFT (1,000 NFTs), a price of 0.000000384 SOL (market cap 384 SOL = $57,600 at an assumed $150/SOL), 64% to graduation (54.4 of 85 SOL). (B and C still show the earlier 2% transfer tax / ticket model; A is current.)

| Direction | Path | One-line summary |
|---|---|---|
| **A: Obsidian** (approved) | `a-obsidian/` (home, token, launch) | Dark premium fintech: near-black glass surfaces, hairlines, one iris-violet accent, Geist type, wallet-style convert panel and portfolio card. |
| B: Gallery | `b-gallery/` | Light editorial auction house: warm paper, Instrument Serif display type with Inter, framed NFT "lots", and a token page laid out as a catalogue. |
| C: Terminal | `c-terminal/` | Pro trading desk: dense tables, candlesticks, IBM Plex Mono numbers, one lime signal color, and plain-language glossary/invariants for newcomers. |

**Status: A (Obsidian) approved by Barton on 2026-09-24; Mintmark kept as working name.** Recommendation was A. It best matches Barton's "clean and premium" pick. It has the polish of Phantom/Linear/Coinbase-level products, reads as trustworthy to newcomers, and still feels native to crypto. B's editorial warmth and C's data density are worth borrowing in places (e.g. B's "lot label" captions for NFT detail pages, C's invariants table for a docs/transparency page).

Wrap/unwrap is handled the same way in all three, following Barton's steer. There is no block grid and no "filling" metaphor. Instead there is a two-sided convert panel with exact amounts, an example fee and one primary button; a portfolio-style holdings card ("Tokens available to convert: N NFTs · value"); and the line "1,000,000 tokens converts to 1 NFT; convert back any time for exactly 1,000,000 tokens."

Other files: `NAMES.md` has three name options (Mintmark recommended; availability not checked). `_build/` has the Python generators (`common.py` holds the shared demo data and generative SVG art). Re-run `python3 _build/a_obsidian.py` (etc.) to regenerate the HTML.

## Caveats
- **Transfer tax:** resolved for now by dropping Token-2022. Every launch is a classic SPL token + MPL-Hybrid hybrid with no transfer tax, so the old "None*" footnote is gone from A. B and C still carry the older tax/lottery mockups and were not updated.
- Fonts (Geist, Instrument Serif, Inter, IBM Plex) load from Google Fonts and were confirmed loaded in the headless render. Each stack falls back to system fonts if offline.
- All figures are demo data. The fees (1% trade fee, 0.01 SOL convert fee) are labeled as examples. Audit is shown as "pending"; nothing claims to be safe, audited or guaranteed.

## Updates (2026-09-24, after approval)
- `a-obsidian/launch.html` and `launch.png`: a five-step create-a-collection page. Its JavaScript is interactive: the ratio control, collection-size validation, live allocation math, NFT price estimate, ticket example, re-roll fee choice and review summary all update live. The static default is 5,000 NFTs × 100,000 tokens (50% of supply can be in NFT form). The nav "Launch" link and the "Launch a collection" / "Start a launch" buttons on the A pages now point here.
- **Cosmetic rarity (Barton's decision) applied to Direction A only.** Changes:
  - Tier badges on the home cards, hero and thumbnails.
  - On the token page: a "Your NFTs" card with traits and a re-roll action, plus a "Floor and rarity" explainer (floor = ratio × token price), a re-roll row in the activity feed, and two new lines in "What nobody can change".
  - On the launch page: a traits and rarity step with example tiers and re-roll fee settings. The re-roll fee destination is marked "to be decided".
- Directions B and C were not updated with rarity or the launch page, because A is the approved direction.
- `NAMES.md` has 7 more name options and an unverified .com/.fun quick-check table.

## Update: hybrid-only model (2026-09-24, latest)
Barton changed the plan: **every launch is an SPL-404 hybrid** (classic SPL token + MPL-Hybrid converter). Token-2022 and support for existing collections are **deferred**, not dropped. The two-type (Hybrid/Rewards) work is archived in `../deferred/token-2022/`.
- **A pages are hybrid-only.** There is no transfer tax, treasury, tickets, lottery/raffle or type labels. The launch page is one four-step flow, and `launch.png` is the single screenshot.
- **Disclosures:** "No transfer tax"; Metaplex protocol fee of about 0.005 SOL per convert; creator fee as an example/configurable value; "Settings controlled by a multisig with a timelock (details to be published)".
- **Re-roll fees are paid in the collection's token and burned** (example: 20,000 NOCT). The supply copy is now: "Fixed at 1,000,000,000 at launch. No one can mint more; re-roll burns can only reduce it."
- **Trade maths:** 1 SOL buys 2,578,125 NOCT after the example 1% trade fee.

## Update: positioning, marketplaces and ratios (2026-09-24, latest)
- **Positioning:** A's copy no longer uses any "revive NFT culture" framing. It's built around SPL-404 (one asset, token or NFT, switch any time at a fixed rate) and showcasing the memes and collections. The new hero headline is "Trade the meme. Collect the art." (alternatives are in `a-obsidian/NOTE.md`). The home page adds a collection mosaic hero, a Featured collection spotlight and a bigger 3-column Trending collections grid, which now includes the 200-NFT Monolith collection.
- **Marketplaces:** the token page has a Trade NFTs card with external "Trade on Tensor" / "Trade on Magic Eden" links (text only, no logos) and a demo Listings preview. The copy says Mintmark handles curve buys and converting, while NFT listings happen on those marketplaces.
- **Ratios:** the launch page offers eight tokens-per-NFT options (10K to 5M), shown as two groups of tiles with max size and a per-NFT price estimate. Minimum collection size is 100.
- **Not updated:** Directions B and C still use the old "Revive NFT culture" hero and the older model.
