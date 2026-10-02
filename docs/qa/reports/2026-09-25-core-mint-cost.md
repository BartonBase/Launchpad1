# Metaplex Core mint-cost benchmark (QA), 2026-09-25

Run: 2026-09-25 ~4:55 PM MT. LiteSVM 0.10.0 with the **mainnet** Metaplex Core binary, dumped read-only on 2026-09-24
15:25 MT (`tests/bench/fixtures/mpl_core_mainnet.so`, 848,320 B, sha256 `96fa631a…a8ba5`). No transactions touched any network.
Script: `tests/bench/` (`cd tests/bench && cargo run --release --offline`). SDK: `mpl-core` 0.12.1 (Rust).

## Setup
- Collection: `CreateCollectionV2`, 32-char name, 200-char URI, no plugins (design: program-created, no plugins).
- Asset: `CreateV2` into the collection, owner = vault-authority stand-in, update authority = collection authority. Variants:
  - `none`: 32-char name + 200-char URI, no plugins (what the wip `hybrid_vault` CPI does);
  - `royalty`: + asset Royalties (500 bps, 1 creator, ruleset None) (design rule);
  - `roy+attr`: + Attributes with 8 traits;
  - `min-short` / `attr8-short`: the engineer's shapes from graduation-design §1.2 (15-byte name, 63-byte arweave URI; no plugins / 8 attributes).
- Assets per tx: packed keypair-signed creates until the 1,232-byte packet or 1.4M CU limit breaks, verified by executing the tx.
- Tx fee: 5,000 lamports per signature (payer + one asset keypair per asset); **no priority fee** in these numbers.

## Results

collection: 281 bytes, rent 2846640 lamports (0.002847 SOL), 5258 CU, tx fee 2 sigs = 10000 lamports

| plugins | asset bytes | rent-exempt min (SOL) | lamports above min = Core create fee (SOL) | lamports in asset (SOL) | CU/asset (1 per tx) | tx bytes (1 asset) | assets/tx | limit | CU (full tx) | tx fee/asset (SOL) | total/asset (SOL) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| none | 307 | 0.003028 | 0.001500 | 0.004528 | 12073 | 618 | 2 | packet (1232 B) | 24146 | 0.000008 | 0.004535 |
| royalty | 376 | 0.003508 | 0.001500 | 0.005008 | 20042 | 660 | 2 | packet (1232 B) | 40084 | 0.000008 | 0.005015 |
| roy+attr | 615 | 0.005171 | 0.001500 | 0.006671 | 29403 | 890 | 1 | packet (1232 B) | 29403 | 0.000010 | 0.006681 |
| min-short | 154 | 0.001963 | 0.001500 | 0.003463 | 12125 | 464 | 4 | packet (1232 B) | 48500 | 0.000006 | 0.003469 |
| attr8-short | 411 | 0.003751 | 0.001500 | 0.005251 | 25088 | 695 | 2 | packet (1232 B) | 50176 | 0.000008 | 0.005259 |

| plugins | N | txs | lamports into assets (SOL) | tx fees (SOL) | total incl. collection (SOL) |
|---|---|---|---|---|---|
| none | 100 | 50 | 0.453 | 0.0008 | 0.456 |
| none | 1000 | 500 | 4.528 | 0.0075 | 4.538 |
| none | 10000 | 5000 | 45.276 | 0.0750 | 45.354 |
| none | 100000 | 50000 | 452.760 | 0.7500 | 453.513 |
| royalty | 100 | 50 | 0.501 | 0.0008 | 0.504 |
| royalty | 1000 | 500 | 5.008 | 0.0075 | 5.018 |
| royalty | 10000 | 5000 | 50.078 | 0.0750 | 50.156 |
| royalty | 100000 | 50000 | 500.784 | 0.7500 | 501.537 |
| roy+attr | 100 | 100 | 0.667 | 0.0010 | 0.671 |
| roy+attr | 1000 | 1000 | 6.671 | 0.0100 | 6.684 |
| roy+attr | 10000 | 10000 | 66.713 | 0.1000 | 66.816 |
| roy+attr | 100000 | 100000 | 667.128 | 1.0000 | 668.131 |
| min-short | 100 | 25 | 0.346 | 0.0006 | 0.350 |
| min-short | 1000 | 250 | 3.463 | 0.0063 | 3.472 |
| min-short | 10000 | 2500 | 34.627 | 0.0625 | 34.693 |
| min-short | 100000 | 25000 | 346.272 | 0.6250 | 346.900 |
| attr8-short | 100 | 50 | 0.525 | 0.0008 | 0.529 |
| attr8-short | 1000 | 500 | 5.251 | 0.0075 | 5.262 |
| attr8-short | 10000 | 5000 | 52.514 | 0.0750 | 52.592 |
| attr8-short | 100000 | 50000 | 525.144 | 0.7500 | 525.897 |



Notes:
- "lamports above min = Core create fee": every asset account holds rent-exempt minimum **+ 1,500,000 lamports**, the
  Metaplex Core protocol create fee (0.0015 SOL, documented at metaplex-foundation/disclosures protocol-fees.md and the Core
  FAQ). It's collected by Metaplex later and is not recoverable by us. The collection account held exactly its rent-exempt
  minimum in our run (no create fee visible on `CreateCollectionV2`); graduation-design §1.4 books 0.0015 SOL for the
  collection too — immaterial (one-time).
- CU per create: 12.1k (no plugins) to 29.4k (royalty + 8 attributes). CU never binds; the packet size does
  (2 assets/tx with a 200-char URI, 4/tx with a 63-byte URI and no plugins, 1/tx with royalty+attributes+200-char URI).
- Program-derived (PDA) assets created by CPI from `hybrid_vault` need no asset signature, so the tx fee drops to
  5,000 lamports per tx, but each create then carries a Merkle leaf + proof (graduation-design §1.3: ~2 verified creates/tx).
  Tx fees are < 1% of the total either way; **rent + the Core fee are ~99%**.
- URI length matters: each byte costs 6,960 lamports per asset. 200 → 63 bytes saves ≈ 0.00095 SOL per asset (≈ 95 SOL at 100k).

## Totals (asset lamports + tx fees + collection)

| Collection size | lean (min-short) | engineer "our setup" (attr8-short) | design w/ 200-char URI + royalty | royalty + 8 attrs + 200-char URI |
|---|---|---|---|---|
| 100 | 0.35 SOL | 0.53 SOL | 0.50 SOL | 0.67 SOL |
| 1,000 | 3.47 | 5.26 | 5.02 | 6.68 |
| 10,000 | 34.7 | 52.6 | 50.2 | 66.8 |
| 100,000 | 346.9 | 525.9 | 501.5 | 668.1 |

## Comparison with engineer estimates
- **docs/hybrid-rarity-and-assignment.md §Q-H5 / DECISIONS**: "~0.0016 SOL rent per asset (~180 bytes), ≈1.6 SOL per 1,000,
  ≈156 SOL at 100k". **Stale and too low by 2.2–4.2×**: it omits the 0.0015 SOL Core create fee and underestimates size.
  → tracker QA-GRAD-01 (doc bug, M-33 family).
- **docs/graduation-design.md §1** (engineer, 2026-09-25, localnet, devnet Core dump, JS SDK): our setup 385 B, 3,570,480 lamports
  rent + 1.5M fee = 0.00507 SOL; minimal 153 B. QA measured 411 B / 0.00525 SOL for the attr8-short shape (longer trait values
  in our fixture) and 154 B / 0.00346 SOL minimal: **agrees within ~4%** on an independent (mainnet) binary and SDK. CU agree
  within 1% (12,146 vs 12,125; 29,199 vs 29,403 for the royalty+attr shape).
- Economics: at the ~500 SOL graduation cap, a 100k collection (10k ratio) would need 347–668 SOL just to mint — more than the
  whole cap. Supports the pending 10,000-NFT cap / dropping the 10k (and 50k) ratios (graduation-design §4.2, BRIEF pending Barton).

## Lazy minting: cost per first capture (added 2026-09-25 ~5:30 PM MT)

Barton, 5:13 PM MT: nothing is pre-minted. Asset i is minted inside settle the first time VRF picks it, and the capturer pays
(T-HV-16: from the request's escrowed lamports, never from the cranker). These numbers reuse the measurements above; nothing
new was run. The first-mint cost m = asset rent + the 0.0015 SOL Core create fee. The settle transaction's fee is paid by
whoever sends it (crank or capturer) and is not included.

| Variant | m (SOL, first capture) | Extra CU in settle | + 50k tier (0.002) | + 100k/200k tier (0.005) | + 500k–5M tier (0.01) |
|---|---|---|---|---|---|
| min-short (63-byte URI) | 0.003463 | ~12.1k | 0.005463 | 0.008463 | 0.013463 |
| none (200-char URI) | 0.004528 | ~12.1k | 0.006528 | 0.009528 | 0.014528 |
| royalty | 0.005008 | ~20k | 0.007008 | 0.010008 | 0.015008 |
| attr8-short | 0.005251 | ~25.1k | 0.007251 | 0.010251 | 0.015251 |
| royalty + attributes | 0.006671 | ~29.4k | 0.008671 | 0.011671 | 0.016671 |

- **Capture or re-roll of an already-minted index:** tier fee only (the asset is transferred, not re-minted). **Release:** free (tx fee only).
- **Whole collection, paid by first capturers (protocol pays 0):** 100 NFTs = 0.35–0.67 SOL; 1,000 = 3.46–6.67; 10,000 = 34.6–66.7 SOL.
  So a 10,000-NFT collection at the 85 SOL default needs nothing from the raise.
- **Fit:** settle + Core create adds 12–29k CU, plus about 450–600 B of Merkle proof per graduation-design §2.5. Within the default
  compute limit. Tx size needs a v0 tx + ALT (per the design doc; not measured here).
- **M-06 per outcome (QA-FEE-04):** a re-roll that triggers a first mint costs m − tx (0.00346–0.00667 SOL) more than a release +
  capture that lands on a minted index. Equal in expectation; flagged as a design question.
- Versus the NFT's token value: at the 50k ratio, m is 1.7–3.3× the 0.002 SOL tier fee. The per-NFT token value depends on the
  curve price, which isn't measured here.
