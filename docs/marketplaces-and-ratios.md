# Marketplaces (Tensor, Magic Eden) for Core collections, and ratio edge cases

- **Status:** research only. All sources were checked **2026-09-25**. Where a claim comes from on-chain reads or our own fetches, that is stated. Unverified items are marked **unverified**.
- **Author:** executor agent, 2026-09-25 (MT).
- **Scratch:** `/workspace/scratch/graduation/` holds on-chain reads, fetched pages, and cloned marketplace sources.
- **Companion doc:** `docs/graduation-design.md`.

> **UPDATE 2026-09-25 (implemented):** the **10k ratio is dropped**; the allowed ratios are {50k, 100k, 200k, 500k, 1M, 2.5M, 5M}
> (`hybrid_launch::ALLOWED_RATIOS`). The collection size is 100 ≤ C ≤ min(1B/N, 10,000). Minting is **lazy** (ADR-016), so
> marketplaces only see assets once they're first captured. Fees are a flat tiered SOL fee at request (ADR-015), not
> 2% in tokens; release is free. The 10k and bps/2% material in Part B is kept for the record.
- **Fee assumption:** capture and re-roll pay 2% of N in tokens to one fixed fee address. The fee is charged at request time and never refunded.

## PART A: Marketplace support for Metaplex Core

### A.1 Reference Core collections (verified on-chain: account owner = Core `CoREENxT…`, update-authority type Collection)

| Collection | Collection address | Sample asset |
|---|---|---|
| Metardians | `DF7WFTiK86GbxvwGTo5sBav8UnHTYzQHbtvNjVPRF1dK` | `AVrJEMsyJi6BCkuNHRjL9dHdqgQqNjWHhZCjtcSnDc7G` |
| BLOKS | `BTye657E6uSYLgUFcFQi62tUBTGdjbKo4RRhFmkJ6r6T` | BLOK #55 `Edg61iXh3NpLU5mtgCzBFxPmQbJzb5NhSRJLLogsyFen` |
| Hegends (memecoin-linked, a precedent for us) | `EQV2wUwZ3EmMFfkAvU3KhjzstoUez9XQsKZmurRqScgj` | `8qsvWVg8WyVWK6wkdjKEyL7tRDaDw3kDUUEGF3Mt56Qf` |
| Liminals | `HgT3fbwuVcbeKgd8x7gP5UmeVmzYDD46HxuZbZWosS2y` | `E1ZSM1F4aePjvCt4fGYTKrbdWjf55kj3NDR2VpmbsTLb` |

### A.2 Feature matrix

| Feature | Tensor | Magic Eden |
|---|---|---|
| Core listing | **Yes.** Tensor Marketplace program `TCMPhJdwDryooaGtiocG1u3xcYbRpiJzb283XfCZsDp` has `list_core` / `buy_core` (source: https://github.com/tensor-foundation/marketplace @8be7f2c). Live Core collections trade at tensor.trade/trade/… (fetched). | **Yes.** Live Core listings found on-chain for metardians, bloks, hegends and liminals via M2 (`M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K`) and MMM pools (`mmm3XBJg5gk8XJxEKBvdgptZz6SgK4tXvn36sodowMc`). ME API `/v2/collections/<symbol>/listings` returns them. |
| Instant buy | Yes (`buy_core`) | Yes (M2 buy; MMM pools sell instantly) |
| Collection bids | Yes. Bids API: https://dev.tensor.trade (reference; checked) | Yes: collection offers (MMM pools) |
| Trait bids | Yes. Trait bids in the Tensor bids API (dev.tensor.trade) | Yes. Collection offers with "Add Traits" supports Core: https://help.magiceden.io/en/articles/6721452 (read via search snippet; page is behind Cloudflare) |
| Trait display and filtering | Yes (live collection pages show attributes) | Yes. `/v2/collections/<symbol>/attributes` and `/v2/tokens/<asset>` return JSON attributes for Core assets (fetched) |
| Rarity | HowRare and MoonRank methods: https://docs.tensor.trade/trade/troubleshooting/rarity | API returns `rarity` with howrare, moonrank and meInstant (fetched) |
| Royalties (Core Royalties plugin) | Enforced in full from the **asset** plugin if present, otherwise the **collection** plugin; 0 if neither (TCMP source, `buy_core`). Docs: 2% taker fee, full royalties on enforced collections: https://docs.tensor.trade/trade/fees-and-royalties | API returns `sellerFeeBasisPoints` for Core assets (fetched). Enforcement is on-chain in M2/MMM for Core (MMM source reads the Royalties plugin). |
| Custody while listed | **Escrow.** `list_core` transfers the asset to the `list_state` PDA (source) | **Escrow.** M2-listed Core assets are owned by `1BWutmTvYPwDtmw9abTkS4Ssr8no61spGAvW1X6NDix` (an M2-owned account, seen on-chain on metardians `BJEd1wme…`, `3iK5baiP…`). MMM-listed assets are owned by the pool PDA. |
| Devnet | TCMP deployed on devnet (read on-chain). No public devnet UI or indexer found (`devnet.tensor.trade` doesn't resolve): **unverified** | M2, MMM and M3 deployed on devnet (on-chain). ME devnet API returned 404. No devnet UI found: **unverified** |

**Consequence for `hybrid_vault`.** A listed NFT is not owned by the user. It's owned by the marketplace escrow. `release` and `request_reroll` require the user to own the asset, so **the user must delist first**. The UI should detect this (owner ∈ {TCMP list_state, M2 escrow, MMM pool}) and offer "Delist & release".

### A.3 Plugins to avoid (they break or restrict listing)
- **Magic Eden MMM** (open-source https://github.com/magiceden-oss/mmm @0a054d2) has `CORE_DENY_LIST`:
  - FreezeDelegate, allowed **only** if its authority is Owner. Live M2-listed metardians carry FreezeDelegate(Owner), not frozen.
  - BurnDelegate
  - PermanentTransferDelegate
  - PermanentBurnDelegate
- **Tensor** (`tensor-foundation/toolbox::validate_core_asset`) has no on-chain plugin denylist in the public source, but any freeze or permanent delegate can make transfers fail or be seized. The deployed binary may differ from the repo (**unverified**).
- **Also avoid:**
  - TransferDelegate, PermanentFreezeDelegate
  - the **Oracle** external plugin (it can reject transfers)
  - any Royalties **ruleset** other than None. A ProgramAllowList that omits TCMP, M2, MMM or M3 blocks that marketplace; a DenyList can do the same.
- **Our setup (measured in graduation-design.md):**
  - Attributes plugin with authority None;
  - collection-level Royalties with ruleset None, only if the creator wants royalties;
  - nothing else.
  - The hybrid_vault PDA owns vault NFTs directly, so no delegates are needed.

### A.4 Indexing and verification
- **Magic Eden:** "Solana collections will be listed automatically" (https://help.magiceden.us/en/articles/10362331-how-to-list-your-nft-collection-on-magic-eden-a-guide-for-creators). The manual creator application asks for the Core collection (MCC) address. The verified badge is a separate application. **How long indexing takes is not documented: unverified.**
- **Tensor:** new collections are verified through the Creator Portal (https://docs.tensor.trade/work-with-us/creator-portal/verifying-a-new-nft-collection):
  - search by collection ID (MCC, FVC or hashlist);
  - connect the creator's X account;
  - optionally sign with the update-authority wallet;
  - staff review.
  - Unverified collections may still be tradeable by address. Review time isn't documented: **unverified**.
  - **Our update authority is a PDA, so it can't sign.** Verify through X plus a pointer to the on-chain `LaunchConfig`.

### A.5 "Trade on X" URL patterns
- **Tensor (fetched with curl, checked by page `<title>`):**
  - Collection: `https://www.tensor.trade/trade/<slug>`. Worked for `metardians`, `hegends`, `bloks`.
  - **By on-chain collection address:** `https://www.tensor.trade/trade/<collection address>` resolved to the right collection for all four reference collections. BLOKS failed once transiently, then worked. **So we can link by address before a slug exists.**
  - Item: `https://www.tensor.trade/item/<asset address>`.
  - Caveat: unknown slugs or addresses still return HTTP 200 with a generic title, so a link checker must test the title, not the status.
- **Magic Eden:**
  - Collection: `https://magiceden.io/marketplace/<symbol>`. Item: `https://magiceden.io/item-details/<asset address>`.
  - **Unverified by fetch.** magiceden.io web pages returned Cloudflare challenges to curl, WebFetch and headless Chrome. We did not try to bypass them.
  - The public API works:
    - `https://api-mainnet.magiceden.dev/v2/tokens/<asset>` returns `collection` = the symbol (bloks, metardians, hegends).
    - `/v2/collections/<symbol>/stats` works.
    - `/v2/collections/<collection address>/stats` returned `listedCount 0`: the **address is not mapped to a symbol**.
  - **Fallback:**
    - Before a symbol is known, link `item-details/<asset>` for a vault-held asset, or hide the ME button.
    - After graduation, resolve the symbol with `/v2/tokens/<any minted asset>` (rate limit ~1 req/s; cache it).
    - Store it off-chain in our indexer.

### A.6 Traits source
Both marketplaces display JSON-metadata attributes. Whether either prefers the on-chain Attributes plugin over the JSON is **unverified**. Keep the plugin and the JSON identical (graduation-design §5 enforces this at mint).

---

## PART B: Ratio edge cases (N ∈ {10k, 50k, 100k, 200k, 500k, 1M, 2.5M, 5M}; collection size 100 ≤ C ≤ 1B/N)

### B.1 Escrow cornering in small collections (Auditor B R1-B-06 / merged M-20)
**The problem.** One actor captures most or all of a 100-NFT collection. At N = 10k that takes only 1M tokens (0.1% of supply) + 2% fees + SOL minimums. Once the pool is empty, nobody else can capture or re-roll. With lazy minting the same applies to the virtual pool.

| Option | Assessment |
|---|---|
| Per-wallet caps | **Sybil-able** (new wallets are free), and they add state. Rejected. |
| Disclosure only | Needed anyway (live pool size, top-holder concentration), but doesn't keep the mechanism usable. |
| **Re-roll reserve / pool floor** | Captures succeed only while `pool_available − pending > r`, where `r = max(5, ceil(2% × C))`. Re-rolls may draw into the reserve because they hand an NFT back, so the pool size doesn't change. Effects: a cornerer can take at most C − r; re-rolls always have ≥ r candidates; the draw never degenerates to a known single asset (B.2). Sybil-proof, because it's a pool-level rule. |

**Recommendation:** implement the **pool floor/reserve** `r = max(5, ceil(2% × C))`, plus the live pool census and top-holder disclosure.

### B.2 Re-roll with a pool of 0 or 1 other assets; reservation accounting
- **Pool of 0 other assets:** re-roll must fail (`PoolTooThin`) **before** charging the fee. The fee is never refunded, so the check comes first.
- **Pool of 1:** the outcome is **deterministic and known**. The pool is public, so a re-roll becomes "swap my NFT for that known one". That breaks blindness. The same applies to a capture from a pool of 1.
- **Reservation rule at request:** let `candidates(s)` = virtual/pool entries plus real deposits with `seq < s`, excluding the caller's own hand-in. Require `candidates(s) − pending_ahead(s) ≥ r + 1` for captures and `≥ r` for re-rolls, with r from B.1 and at least 2. `pending_ahead` counts earlier unsettled requests, each of which reserves one candidate FIFO. Invariant, tested by fuzzing: every pending request has a guaranteed candidate at settle.

**Recommendation:** enforce a minimum candidate set (≥2, in practice ≥ r) at request time, before taking the fee.

### B.3 Pool-content timing
- **What's visible:** the pool contents are public on-chain. Candidates are fixed at request time (FIFO `seq < s`), and randomness is unknown until the VRF reveal.
- **What an observer can do:** time a request for a moment when rares are in the pool, e.g. just after someone releases a legendary. Once earlier requests have been revealed, they can compute the exact pool their own request will face.
- **What they can't do:** know or influence their own draw. There's no cancel after commit, and the fee is non-refundable (M-04).
- **So timing buys better public odds, not a known outcome.** Bots have a speed advantage over humans.

**Recommendation:** accept and disclose it. Show the live pool census and odds to everyone (which removes the information edge; merged M-24). The pool floor from B.1 prevents the degenerate small-pool case.

### B.4 u64 arithmetic (u64 max = 18,446,744,073,709,551,615 ≈ 1.84e19)
- `5M × 10^9 = 5e15`: fine.
- `1B × 10^9 = 1e18`: fine. Decimals 10 → 1e19 still fits; decimals 11 overflows. **Keep decimals ≤ 9** (hybrid_launch already does).
- `C × N_base ≤ 1e18`: fine.
- **Trap found:** `N_base × 10,000` (any bps-style multiply by the denominator first) **overflows u64** at 2.5M and 5M with 9 decimals (2.5e16 × 1e4 = 2.5e20). `N_base × 200` stays under 1e18 and is fine. Any fee or bps computation over total supply (`1e18 × 200`) overflows too.
- Price × amount must always use u128.

**Recommendation:** compute every multiply-then-divide in u128 with `checked_*`, then `u64::try_from`. Add unit tests at `N = 5M, d = 9` and `supply = 1e18` (merged M-29).

### B.5 Fee rounding: `fee = ceil(N_base × 200 / 10,000)`

| Ratio N (tokens) | Fee (tokens) | Fee base units, d=6 | Fee base units, d=9 | Exact? |
|---|---|---|---|---|
| 10,000 | 200 | 200,000,000 | 200,000,000,000 | exact |
| 50,000 | 1,000 | 1,000,000,000 | 1,000,000,000,000 | exact |
| 100,000 | 2,000 | 2,000,000,000 | 2,000,000,000,000 | exact |
| 200,000 | 4,000 | 4,000,000,000 | 4,000,000,000,000 | exact |
| 500,000 | 10,000 | 10,000,000,000 | 10,000,000,000,000 | exact |
| 1,000,000 | 20,000 | 20,000,000,000 | 20,000,000,000,000 | exact |
| 2,500,000 | 50,000 | 50,000,000,000 | 50,000,000,000,000 | exact |
| 5,000,000 | 100,000 | 100,000,000,000 | 100,000,000,000,000 | exact |

- Every ratio is a multiple of 50 whole tokens, so the 2% fee divides exactly at any decimals ≥ 0. `ceil` never rounds and `fee > 0` always holds.
- Capture debit = 1.02 N (e.g. 5,100,000 tokens at 5M). Release = exactly N.
- The fee is taken at request and never refunded, so a failed or expired request loses exactly this amount.

**Recommendation:** keep `ceil` + `require!(fee > 0)` for robustness, compute it once at launch, and store it.

### B.6 Liquidity and UX at N = 5M with 200 NFTs
- One NFT = 5M tokens = 0.5% of supply. At an illustrative 400 SOL migration market cap (**assumption**) that's ~2 SOL per NFT, and the 200 NFTs together back 10% of supply.
- Capturing means buying 5.1M tokens. Against a DAMM v2 pool holding, say, ~200M tokens, that's about a 2.5% price impact (constant-product estimate; depends on the curve config).
- **Marketplace liquidity is thin:** 200 items with a high unit price means few bids and wide spreads.
- **Cornering is expensive in token terms:** taking all 200 means buying about 1.02B tokens × 10%, roughly 102M tokens or 10.2% of supply. Whales can still take most of the pool. The floor from B.1 (r = 5) applies.
- A 100-NFT collection at 5M backs 50% of supply. That's allowed, but flag it as heavily NFT-weighted.

**Recommendation:** keep 5M and 2.5M, but show "1 NFT = 0.5% of supply ≈ X SOL" and enforce slippage bounds on composite buys (M-25). Barton may drop 5M if he wants NFT prices under ~1 SOL.

### B.7 One-line recommendations
- **Cornering:** a pool floor `r = max(5, 2% × C)` plus live concentration disclosure. Not per-wallet caps.
- **Pool of 0/1:** reject re-roll and capture before charging when the candidate set (minus reservations) < r, and never < 2.
- **Timing:** accept and disclose. Show the live pool census and odds.
- **u64:** u128 intermediates everywhere. Decimals ≤ 9. Never multiply `N_base` by 10,000 in u64.
- **Fee rounding:** exact for all ratios. Keep `ceil` and `> 0`, stored at launch.
- **N=5M / 200 NFTs:** keep, with a price-per-NFT and slippage UX. Optionally drop 5M. (Separately, graduation-design §4 recommends dropping 10k.)

## Unverified
- Magic Eden web URL patterns (Cloudflare blocked every fetch; the API confirms the symbol mapping).
- ME and Tensor indexing and verification turnaround times.
- Devnet UIs and indexers for both marketplaces.
- Which trait source (plugin vs JSON) the marketplaces prefer.
- Deployed TCMP and M3 binaries vs public source (M3 is closed source).
- The 400 SOL market-cap and DAMM v2 depth figures (illustrative).
