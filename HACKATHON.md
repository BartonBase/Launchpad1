# Armory: Meteora DBC hackathon submission

**Track:** Best use of Meteora DBC (Superteam Earn). **Status:** working demo on Solana **devnet**,
**unaudited**. Test SOL only, nothing here has real value.

## What Armory is

Armory is a Solana meme-coin launchpad. Each coin is priced by a **Meteora Dynamic Bonding Curve
(DBC)** and, when the curve fills, its liquidity moves to a **Meteora DAMM v2** pool. There are two
launch types:

- **Launch** (live on devnet; called "Plain" in code as `plain`): just the coin. A fixed 1,000,000,000-token SPL coin on a DBC curve
  that graduates to DAMM v2. No NFTs and no platform fee per action. It launches straight from the
  web app through the DBC SDK, with no Armory program in the path.
- **Hybrid** (Armory's own programs): the coin plus an NFT collection. After graduation, a fixed
  number of tokens converts into one random Metaplex Core NFT, and the NFT converts back for exactly
  those tokens. The NFT vault only opens once Armory's program has checked **on-chain** that the
  DBC pool really migrated. Burn, Tax and Raffle launch types are shown as "Coming soon" and can't
  be selected.

## How it uses Meteora

| Step | What happens | Meteora piece |
| --- | --- | --- |
| 1. Launch | Launch type: `creator.createPool` on Armory's platform DBC config. Hybrid: `hybrid_launch::register_dbc_launch` creates the pool on the same config and records it. | DBC |
| 2. Trade on the curve | Buy and sell in the app with a live quote, slippage setting (0.5/1/3/5%), minimum received, trading fee and Meteora protocol fee (`swapQuote2` + `swap2`, exact-in). | DBC SDK |
| 3. Progress | A progress bar shows SOL raised against the config's migration threshold, with three steps: curve, graduation, DAMM v2. | DBC state |
| 4. Graduation | At the threshold, DBC migrates the liquidity to DAMM v2. LP is permanently locked by the config. | DBC to DAMM v2 |
| 5. Trade after graduation | The same trade panel switches to the DAMM v2 pool, with quote, price impact and minimum received (`getQuote2` + `swap2`). | DAMM v2 (cp-amm) SDK |
| 6. NFT layer (Hybrid only) | `hybrid_vault::open_vault` verifies the recorded DBC pool has `is_migrated == 1` and `migration_progress == CreatedPool` before converting is allowed. | DBC account checks |

The in-app page **`/meteora`** ("How it works", linked from the site footer) shows this flow, live
config facts read from chain, every address and the code paths below.

### Platform DBC config (devnet) `DuQYHUCToW6uHkWngXFiU4uGVSjcVKKCTJwViEb87Em9`

Read from chain with the SDK:

- Quote token: SOL (wrapped). Trading fee: 1% (`cliffFeeNumerator` 10,000,000 / 1e9, which covers
  both the trading fee and the Meteora protocol fee).
- Migration threshold: **0.1 SOL** (devnet test value). Migration option: **DAMM v2**, migrated pool
  fee 0.25%.
- Fixed supply 1B, 6 decimals. Mint authority revoked. Metadata immutable.
- Partner LP is 100% permanently locked. Leftover tokens go to `3GjqFEgvYKQ1jFr4L1RMxrykf2aE2VqGXsXmtQc2g27C`,
  the `hybrid_launch` `["dbc_buffer"]` PDA (locked, not burned).

### Addresses (devnet)

| What | Address |
| --- | --- |
| Meteora DBC program | `dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN` |
| Meteora DAMM v2 program | `cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG` |
| Armory platform DBC config | `DuQYHUCToW6uHkWngXFiU4uGVSjcVKKCTJwViEb87Em9` |
| DAMM v2 migration config (0.25%) | `7F6dnUcRuyM2TwR8myT1dYypFXpPSxqwKNSFNkxyNESd` |
| Armory `hybrid_launch` program | `9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8` |
| Armory `hybrid_vault` program | `BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG` |

### Devnet proof (explorer: `https://explorer.solana.com/tx/<signature>?cluster=devnet`)

**Plain launch made in the web app UI** ("Armory Hackathon Plain", AHACK, still on its curve):

- Mint `FN1d6Z15FcxywjKRNaw26x3oYDc5eRjACzZDgVXLRoP4`
- Launch `4iDJHJ9MnTLNx2Skyuch2XhoodzE21uZaM9629FxvJgtmNQw61gfb4BfmdadVAZBGx81TbRg5NcBsyrhoueH5SyE`

**Full lifecycle of a plain launch** ("Armory Plain Demo", APLN, mint `AQ9p3TKktkTGPs1BjJkrvy6MAcRXZvgmMkJb7rqjGd63`,
DBC pool `5CbQAed4Zpq6bfhtGhwepQWzsGYsFqrFKUb7p24wJYiX`, now DAMM v2 pool `GisVVbiEdwTbGA43jCeppXVTYWTou3Tc6NCwnAJd6Yfw`):

| Action | Where | Signature |
| --- | --- | --- |
| Launch on DBC | SDK script | `4awRSUS5gQ3VnpKnP1D5VPQ1iX7FPc3A7veAKVuzUhJpWy1wP2MtowoSKyrx7kyfBSN9zvbi6AxQGYNsH8iTXkdk` |
| Buy 0.01 SOL on the curve | SDK script | `F31NFAf6MjCqpcdu6mN3hFgd1mz8UegFgPMWBKfRhre2fi3UVsNrWfMvyjWNWd9UiDGbmyULvPpxEmfq4NTw5su` |
| Sell on the curve | SDK script | `tXVXFGjw58CPpcxvyo5EdXQXBwhMMDkvC5ffzgzGxoU6y13AiuZssUTmjY58iyMgVUu4impaugLsFt5Vq6xYTc7` |
| Buy 0.005 SOL on the curve | **web app UI** | `2VrpCTZs4MpKmgrrHDQPQ7YF2gD3Mz9h9MBXaKxqCzdhVjpvr23epES4q3LRMoedoGnQ9MZG5Dskq9A99v85bL6F` |
| Sell 30M APLN on the curve | **web app UI** | `WTjvatCgoTEseTGQnFfWUWpZX9Bf6CeZfRaWre9s6psVF8shZdToiU7mqJWcew12HdkSu5nu1Z9T4dDMqDvYdFa` |
| Final buy that fills the curve (0.1 SOL) | SDK script | `crxSpzMMc2pYFUk7Vturo5LyJQUdzKzCvtKDpK2wf2bBogagyDTw6698JYjtCAZY4YAy3JiaXKr9Z22jYjZYaeo` |
| **Graduation: migrate to DAMM v2** | SDK script (`migrateToDammV2`) | `Tp3NAGW2ECZLsnvsKNZTCf8jVUEoSfS2MWNQWdY9apSejkyGJP97DDM3BULSifXxEAT1pLCCLA4Xggjgw37489d` |
| Sell 20M APLN on DAMM v2 | **web app UI** | `2DvoioEjpXZnZpKrkRTJCW363YRWi7s6bYEkHhGPvsCunNXENvWVHcrzveME1oR5JvwtnSK11LoZ5LiZKU5rMgEc` |

**DAMM v2 trading on a graduated Hybrid launch** ("Armory Test", ARMT, mint `Dhv3VkqeTYJiahzcVJLmJ1snApdYtQxsAKeUnv5GTNos`,
DAMM v2 pool `Azo9hpTV6HvpPUkc253NqKXEfQrPZFooeNd4dRKbwzkZ`):

| Action | Where | Signature |
| --- | --- | --- |
| Buy 0.002 SOL | SDK script | `64k3RBEL4n66fQrd9mWUVx3sRzNV7bKkEFPhgdfecjDLGJLfgMYcdCfKptfdtPGgekgNPEk2qfVJyZ97C5tv9ZRa` |
| Sell 100k ARMT | SDK script | `5KxphQnCH2g4iDxh4C3a74offLVXe1rFX5TiBktzGcpDnBUcPzTZ6shbG4ZHEZxFh8UCNyqh3PRCsLjCM127P4KK` |
| Buy 0.002 SOL | **web app UI** | `NuqYYrosqskYoQyWfbHLKCV376wD1BEbPpjB3FsYbtTRwk6hszLdbQA2g9RcWLsVxFZ4zQ1BrbAYGvxoDVWzrgV` |

## Where the integration lives

Web app (`app/`, Next.js 16 + TypeScript):

- `app/src/lib/meteora/dbc.ts`: DBC client, curve state and progress, quotes (`swapQuote2`),
  buy/sell transactions (`swap2`), plain launch (`creator.createPool`).
- `app/src/lib/meteora/damm.ts`: DAMM v2 pool lookup by mint, quotes (`getQuote2`), buy/sell (`swap2`).
- `app/src/lib/meteora/plain.ts`: lists plain launches from chain (DBC pools on the platform config).
- `app/src/components/meteora/SwapPanel.tsx`: the trade panel (curve or DAMM v2).
- `app/src/components/meteora/CurveProgress.tsx`: progress bar and graduation indicator.
- `app/src/components/armory/LaunchWizard.tsx`: launch flow (the Launch type goes live through the DBC SDK).
- `app/src/app/meteora/page.tsx`: the "How it works" page (Meteora integration overview).
- `app/src/config/integrations.ts`, `app/src/config/programs.ts`: pinned config and program IDs.
- `app/tests/meteora.test.ts`: unit tests for curve progress, fee and metadata decoding.

On-chain (Anchor, devnet; not changed for this submission):

- `programs/hybrid_launch/src/dbc.rs`: checks DBC config and pool accounts (owner, length,
  discriminator, fixed offsets) and only accepts allowlisted configs.
- `programs/hybrid_vault/src/graduation.rs`: opens the NFT vault only after it verifies the DBC pool
  migrated to DAMM.

SDKs (exact versions): `@meteora-ag/dynamic-bonding-curve-sdk` 1.5.13, `@meteora-ag/cp-amm-sdk` 1.5.1.

## Safety built into the app

Every transaction goes through one pipeline: the program allowlist (DBC and DAMM v2 are on it),
then a simulation, then a readable preview, then an explicit confirm, and only then the wallet
signs. Quotes always carry a minimum-received amount. Mainnet is refused at startup. The app never
holds private keys. The only keypair it makes is the new token's mint key, generated in the browser
for a launch.

## Run it

```bash
cd app
npm ci
NEXT_PUBLIC_SOLANA_CLUSTER=devnet npm run dev     # http://localhost:3000
npm test                                          # unit tests (offline)
NEXT_PUBLIC_SOLANA_CLUSTER=devnet npm run build   # production build
```

Use any Solana wallet (Phantom, Solflare, Backpack) set to **devnet** with some devnet SOL from
faucet.solana.com. Try this:

1. Launch, pick "Launch" (just the coin), fill in name and symbol, launch.
2. On the token page, buy and sell on the curve and watch the progress bar.
3. Open the APLN or ARMT page to trade on a DAMM v2 pool after graduation.
4. Open `/meteora` ("How it works" in the footer) for the integration overview.

**Hosting (Vercel/Netlify):** root directory `app/`, build `npm run build`, and set the
environment variable `NEXT_PUBLIC_SOLANA_CLUSTER=devnet`. There are no secrets to configure.
Details are in `app/README.md`.

## Limits and what is devnet-only

- **Unaudited.** Devnet only, with test SOL. The programs have not been audited.
- The 0.1 SOL graduation threshold is a devnet test value.
- **Migration on devnet is not automatic.** On mainnet Meteora's migration keepers handle it. On
  devnet we ran the permissionless `migrateToDammV2` call ourselves (the signature is above). The
  app shows "Curve full · migrating" until then.
- A Hybrid launch with a curve is simulate-only in the wizard (you can review the transaction but
  not send it yet). Live Hybrid curve launches were created by script (ARMT).
- Burn, Tax and Raffle are "Coming soon".
- The price chart is a placeholder, and the mascot art is placeholder art.
- The public devnet RPC is rate limited. Under load some live numbers may not load, and a reload
  fixes it.
