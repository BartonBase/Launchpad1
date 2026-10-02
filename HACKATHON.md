# Armory: Meteora DBC hackathon submission

**Track:** Best use of Meteora DBC (Superteam Earn). **Status:** working demo on Solana **devnet**,
**unaudited**. Test SOL only, nothing here has real value. **Live site:** https://armory-ten.vercel.app

**Both launch types are live on devnet: Launch (just the coin) and Hybrid (coin + NFT collection).**

## What to try (5 minutes)

1. Open the home page. The strip under the hero shows tokens launched, curve trades and NFTs captured,
   read from chain.
2. Open a demo collection: **[Forge Gems](https://armory-ten.vercel.app/t/PyngwuDKX8ZDfsY91wVXgX78fFmz4w9F6Zc7tjBdMMU)** (GEMS)
   or **[Shieldwall](https://armory-ten.vercel.app/t/5VVsjp6oKi3mcSKb5YnvQ1MWZBPejqC57RLTqSy33ryE)** (SHLD). Both
   graduated from their Meteora curve to a DAMM v2 pool, and have NFTs minted.
3. Connect a wallet set to devnet (Phantom, Solflare or Backpack, with SOL from faucet.solana.com) and
   **buy** some tokens in the trade panel. After graduation the panel trades on the DAMM v2 pool.
4. **Capture**: convert 1,000,000 tokens into one random NFT. Switchboard picks it; press Reveal and
   then Settle when the buttons appear (a few seconds each on devnet).
5. **Re-roll** your NFT for another random one, or **release** it back for exactly 1,000,000 tokens.
6. **Launch your own**: Launch → pick Launch or Hybrid, add a dev buy if you like. Hybrid needs 1–50
   images; they're resized in your browser and uploaded to Irys devnet.
7. Read **How it works** (`/meteora`, in the footer) for the Meteora flow and every address.

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
  be selected (see the roadmap below).

## How it uses Meteora

| Step | What happens | Meteora piece |
| --- | --- | --- |
| 1. Launch | Both types: `creator.createPool` (or `createPoolWithFirstBuy` when the creator adds a dev buy, same transaction) on Armory's platform DBC config, with token metadata uploaded from the browser. Hybrid then calls `hybrid_launch::register_dbc_launch` to record the pool and `hybrid_vault::init_vault` to commit the NFT art, in one second transaction. | DBC |
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

**Demo collections (Hybrid, made in the web app UI on armory-ten.vercel.app).** The art is
comic-book style made for these demos: 12 gems and 12 shields, each under 100 KB on Irys devnet.
Each collection has 100 NFTs (the program minimum) that share the 12 images in turn.

| | Forge Gems (GEMS) | Shieldwall (SHLD) |
| --- | --- | --- |
| Mint | `PyngwuDKX8ZDfsY91wVXgX78fFmz4w9F6Zc7tjBdMMU` | `5VVsjp6oKi3mcSKb5YnvQ1MWZBPejqC57RLTqSy33ryE` |
| Launch on DBC + 0.02 SOL dev buy (**UI**) | `59jrh7M8Kgs5t9rFe8CcwBxjpnBHjoXtruZZo9mPcFVyWsW4LAiKRxNmNxwTFCAAyXgva5KwiXAKfz5eMgJmDSaH` | `4PAnGr8XXjvALaySVJnQEnPGyu2FWEip9LL2ChdieyxuJn2xy1j62HA4vjzu4VGjVHsUaV47uBaAJMj8hohudEfq` |
| `register_dbc_launch` + `init_vault` (**UI**) | `4k52LLBe3xDrhvnFE5GzV9JwDaTwstZVtG9tgKGRC3d838pv9UfwEETL9NdgHeU7JNdezRLZStxVMyFmidA2h3rt` | `48xqnHYDvT9G82vyepuCQkJ4UG8wXfUjbDozAqkmSXfGF9gTRnqoMA59fRe3JKU6SYc5A7FiuzMBcaArNh95ak1N` |
| Curve buys / sells from two other wallets (SDK script) | `5cLft8zYh7C63qVJRZVUfGzccuJjgpTtLMj1govZB43gDZ1ULMM62h2VMqoDMz8JrzKqApe9Pe1GXrcnNZWr47Jr`, `3moashhnTctG6nHGpdavZeNGRKy6jL3Vv4cdMshV8v1vLpCxhWGBAFY75Pyb6n25H7b5dQZaBRxniNtB4ivdsgaZ`, sell `4iaHKBD8kxWtc6YCteB4P7LzLyFE8rASksNjcBgRBiW41V9BSnF1FexgmS3MjmVGpudWt3PKzDpPd2CB8pCig5jd` | `aEboGUGVo3npDxugMb7LXwxhmtPNW9R3fEkRRUP8GzpR7PMgzo2a5Z3FQg9eSuMfN6jLr9dVbcYePqDNvnYUN5i`, `3pg3jCpCpUntT2qzwzaFZqS43Mw3ZRMoA4867sif1MegDVo387pQFfheh3wkhVL9L8edgXqxqqZcdaT7cuSc82eR`, sell `2Piobwm7sZ45J37nvjmxJebAipj8EfH9frFTnDRHtNgYH724j5cbp37W8HF4vgME6XPxxpgSntxmna1LCCW3yuZ` |
| Buy that fills the curve (SDK script) | `5SiBEDDNN1wCTCMUyEcbmDTzvRTXVqN9Bn7EYgmMz2MjTs54jUAch5widVBWHxX6wLTAZg6toRu3yiZX6nVdPaGN` | `52guzTJhurrpvk2YyfGzo1EmbMSr8aab7fqaVykQxwRFBZstVSoT7119rtKe29FoS2iZTvyzQpFktJnXSw3J6DYu` |
| Graduate to DAMM v2 (**UI**) | `2ZakBwshjHEGhRhH99TPPDnhxNxdLRhkY6bA4L1V8e6gnurxJbnaXkRsNrLLn1K4F6BFH5wLsJmTTQ6Fa2EsWc82` | `UQ62aAcg28gvVHDvPipSfp1NRn8YFHC9bmW7QxSVRB7Sr13i1CVYfs16AfmaQdrDMQpG3PkWNFpCXUv6UwtZ664` |
| Unsold tokens to the locked buffer (**UI**) | `3ZwsM8kw9PrPuhmq1xcBXCAkYUn6cSaSePh6LPhbbhA2C2xTbWJds5suFPyBbB9Ge7uvgKLHfGW8YMo8XhsHFU4Y` | `2PToUPNTFP4gZuFC1LXdZEHJkuCvhxS2DoKsPEdUcyTEcYXujVnfjUEezrjpc9EkCN1ozt9jLAguzdaAQkF53WG1` |
| Open the NFT vault (**UI**) | `2q4HN774huck7iY58VRVUuWafihWZ2ZHsG4mT9FpV4GfPUKcoL3rbfjYZM8JLTY3PdYB5cN4vzFbhUQWF4fMHMut` | `4VboTrRV7MKqv1hBh69VgEx93vQvaGxmycQncPCZyuK6EkDTQi8nMEc5ukgntCbXT6Pph1yCMuDHgokbJnF38Vwd` |
| Capture 1: request / reveal / settle (**UI**) | `CuhHxnDfQVKi8kVuFBBtMZkhvZDoE5y5BVS3VGVAS5tihdmwsiSsuo3yN7z2C4CiZdWFMLQkk9mqMKeTVwEZrt8`, `3D7LoemBB6bmQoiqjns9HrRKtySZgMBegX49Q7NJBAnW7cYjeuJFd5d6nDxN7uABwfwDAT9NHtWxUCJzceDNLbaz`, `61HHJuzqcKdztrjNGmWnHgHQQwARmaP6kHrXok1tEoMK1zjy1ir5f6Pn9LmaqweXT8uNvQ6K4STpmpTuPyC5iH2A` | `JGmUfgbCSTK94fnRgncvrzqc8ZG3ZQES243N8hyQVQxhjTJPGYr4XqUzLZy7QDrn9zjR4DnpguDdtoqvUgxYCM5`, `2FZWDGUshdw2EXpP1ocQB2maM6RwbYyJZ7tar4dCPUUtTuBq1bdWUzcoS6cnyyTA9xx9WYoFw5Hx1vJZHXQhRXxs`, `25ARqBUmpp8Qfe6VW5K9UTEYTPYWosKx2P8NKEPX6ZHGnGw1kKBTx4L5Pvi3XDTWF13LeaR1LNih4Yh2obWaiBCH` |
| Capture 2 (**UI**) | `4Uf3cFTR31iNfLginA2SCgzqaYYy28ttumWhYM8AAhpAL2aCMUtrrB5NTd5wGEoibtsBxnQi6aRj4AwVv4mDutnE`, `5892uPGogsXnsUwNmUvdF9pnKefvjjESfvB2fqLmXM2kWWFgP9bDL74n69bLWbY2ahaCHbyESkCPFUCZ92rvnUJm`, `32J6VdKu6CZ9RKoG3KmqNikNAX8qTiVCZr16PVmZsHCj3cvktHqRdBZz5toP2M7NXm45PQFH6FeMQLXsirztUNQS` | `2pwBjZTN9Xx3RUS9rgyNEZ2xtcZn9msHnEnkyPcyYb7rJzYDwS55fzq4rYdjQNa14bGYErbNsJi7L82KAS5CANMw`, `4kujDtdJx3eh6nUKWaNoYA2WSax1S16KPwVoPd8tJBKGy7zCfM2FtGLupmBcB2rShY47ZdHmBeunVoJz5M2ArrFV`, `3WoNMCvLSQ13oPhzn6A41eSiJEkXnebQNk8SjbxVjoofuMsrVV2XXry3nQ5gwi5BUHchSXERmWJP3kEPeUmrgN6G` |
| Capture 3 (**UI**) | `5kJgrt966V893uXUJfVi8v2ahBT3njvC8ojfegWnb1AwBkVqWEbmuBusRM7c8YHdUYtrVQXd7KLJBvDhy4YXos93`, `3bLSUKDQrSKphLoY2Zoon2w4UQZN8199kXYRcWMWpnkAubb3PgMXNibC4Y5LYsPTRWtuK7m4YqU3fDaxrawtHThS`, `3azQmEtjk9bxbcaF6inxWo7NJ1i1TL2BBQ4K1Yw13NSQTHCXSDt6URXyaVe5rQPTJHF4nXHakQpUGkqDAPR4QSpR` | `5cDB1GXAZhA4vC8D1sxD8d17RuYgtD37capAPW8MC6vZrk1ywtmFWMUMep3AJHStteqwTrZmeFwgLh64qPfdTakF`, `iZp53m67QD5hypcC3hs5qunahRWPJyikLb3eyyk4RHQj6L3kJb4P4ckPjWp3UGNZWf35aJ6VV5Vcy18PvqZzqr1`, `3sBdJzrNHSMqqUAfSFJ9AsjBkhiq7QH3TpPHcNrr6UhjiMvVxjRPWqqoRp2eyV5LfHaAi8EeLfuwqAQQRoBr4DPR` |
| Re-roll NFT #61: request / reveal / settle (**UI**) | | `3psAQYVVCwMEhTmTdmRTmsq7mEw2JoMubfo6u4Tz2QsoyHivhqCrmfvTvqijmbT34v2rnEikGBFDxBEThxP7GtMR`, `3vzHNT3vVRA6gfdcf9MxZ8bQ6rGzxhCiJe1f6bFqhsqWPWAtvdfPziZyMwmqWjnJYFyk21hDrkvDrETdm6Qqw2fx`, `3PPuBxQGxRD3dXj9QzT5hKU7iEoux2QjDjRpx1nHrowUfBehfhRycPgwCxyoiKhBeuzkBjQok2FnBdVxrN9ZhyEz` |
| Trades on DAMM v2 after graduation (**UI**, other wallets) | buy `2JFA2w9VC9dfgU18pnJMrRyfxT5wQbzd8g56Zk56MqnvpHtHaVotNx1e3mFkR3t9veQ4Gj2VijVP4WDZDThvfNwV` | buy `eq7LBbqee3epvUjsonS2H9bAJavjfbapYrJ6HDJBvVG1V4doNjTEG9XxbEFp9KvX9tQjrzHYULdJn6531gFry2B`, sell `4hAuKs1VdUgb5Wvcu6uU1SVJ5nLzVPimfKrG3h794njFVFHjDWYr1PwzeESE8PryVQfg1hwF7eEyC1thgsZRGEiS` |

Each capture and the re-roll paid the 0.01 SOL platform fee to the fee wallet
`BVKxZMjuXryATqCeifPgh6Ee9H93BH5UGv8eML66yT3j`, which went from 0.04 to **0.11 SOL** (7 × 0.01).

Older test launches (APLN, AHACK, ARMT, AHYB, AHLIVE) are hidden from Home and Explore but still open by
address; their proof is below.

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

**Full Hybrid lifecycle on devnet, built with the same app code the wizard and token page use**
("Armory Hybrid E2E", AHYB, mint `HJUw5ETgeTUxgzbJ6EuNozUDhHBGjzyaJe4a8GHs76q5`, DBC pool
`7oyAYJUGxPzrqVEGnUVB4bvckArQGzjaAX43GtuhuPbn`, vault `H1qEpAUzVH7kueMN297upcvLSH4cRRUuYBvLJk3yCjft`,
collection metadata `ar://7AGUktX7u4u8iSfqpnWnFFe3VzF3LyRtjAkjJQy6Kru6` on Irys devnet):

| Action | Signature |
| --- | --- |
| Launch on DBC + 0.02 SOL dev buy (one tx) | `4WLAkvmMBYkNYpCKbqF1B72oZcNih5ZSvKciJBw6hfnxBR83RStUE4k1bvm163youdyL2vDy9PQpHLik5EBhsf4v` |
| `register_dbc_launch` + `init_vault` (art commitment) | `3ohMMW2mcwjk83YrQfJSfbmQF2woxSQNDbWkjAiR8We1vFFp8jnfA9pjF7sMm94vYXCW4oZob1CjxJhTBdgmG8Gt` |
| Buy that fills the curve | `2TMdVBw9RKQRbm4uLt8suqmpUNfpbn7xPuB3nZd6Hj11HKiCjisSZWV6dibyaQZ6bQhR8qzdn8t5VcuQ4TzY3aRN` |
| Graduation: migrate to DAMM v2 | `5jdcQ24vUUTvE4Rw21PS7hT3JHK8kupzMXocjgmHMP51oLpj6zbDExPHeR9BDUkcYQE6eLGU1hFoR7ttwA7pjrtN` |
| Unsold tokens to the locked buffer | `35Dfbhq8J1JNY4o3qoRutxpwesFYZRJzzRYSf1MLSYRwFYpYksTtfxZgER72ypGnmGKfoMmhLqD7AHP4FJArTojG` |
| `open_vault` | `2paj2FPEzcwFxrxDktVxbtgKoF6dibeTR3YggChWDFJtsmFYW6QegqSSqiZN2gvShAqCbhg3Y6ZXSNsiqZGPdMfv` |
| Capture request (0.01 SOL fee) | `fMW4Hg67c2hFGESefvspj8w195F6XyybSszF5hk12TrVNFTAchSprA3vNGWy7QjBfkhairraeerd8j6foECwnz8` |
| Reveal (Switchboard) | `4Qe6Lr5xVLs2oK3EoUuTLtyFEJesjTVKjfpFpi5wQ5DsGm9jBYenVykSLa9eSFioRFFmaVUDxYxKUwanazWq5CTU` |
| Settle, lazy-mints NFT #73 | `36wDUJnb2wdYrk9yQuugJYhMYNevHjrBx71DGCmwHemDVb2vjHynSbK191ReAHkbCmqzGZMyD5iVbV1zkEPVcQEY` |
| Re-roll request (0.01 SOL fee) | `2xqxEEvDDBES5W9NRRWwQkvAZpt5hCmsjwq8riGFvA3AqAJ36axGDxzMsdm3AATmL3GPeJJNayUxjJ9Z1eoUqeCb` |
| Re-roll reveal | `56cxcVNU87RWj7XmLq5NQ7U6bAgZnoAURjaBVdFYCp2Cfv1zX8ztJrTCXWCcgVoCiG1BNzjB1cB2mMakhf5vqwtM` |
| Re-roll settle, lazy-mints NFT #62 | `4NsBhnr2WmJjPeWBXyfZyyrqPWdUkH6sikjkvLsWFDtjF5Qaor4HhdvLa2SPVXPMazVVRAqE52i9JcRftAVmUNbm` |
| Release NFT #62 back to tokens | `3XV3mm2A8sWRwB6HyVNBJGMuv9GNebdqx8nZJT5ignp4UiQ6FiRWt9XD1heUPDv8inFuRWYpYS4hFLq67xkf87qz` |

The fee wallet `BVKxZMjuXryATqCeifPgh6Ee9H93BH5UGv8eML66yT3j` went from 0 to 0.02 SOL (one capture + one re-roll).

**Hybrid made and graduated in the web app UI** on armory-ten.vercel.app ("Armory Hybrid Live", AHLIVE,
collection "Armory Live Knights", mint `CzDzhYGoCP5BDc2gYmNnwd3s7gW5MWnK8VD7TrbCwVDQ`):

| Action | Where | Signature |
| --- | --- | --- |
| Launch on DBC + 0.01 SOL dev buy | **web app UI** (wizard step 1 of 2) | `4pYibVHkcZvcwAsQ2tk6Edkw8gQGdL7p2HSv7ppAvpXZnoziJYVLcApkDThkt7uByhgYesXPdbJFuMSL2ASBuveM` |
| `register_dbc_launch` + `init_vault` | **web app UI** (wizard step 2 of 2) | `5u7p3qkn817brkpgEiNL39WetfsR275xBKi95wwq3PAhrG9KMayfafNSaev7hEZKWGXYEpNeK51RP9TKHGk3P2HF` |
| Buy that fills the curve | SDK script | `2V8CTC8pjAjD29atGZSDBXWuaoz3duRhPAaLavxGM7MbE1uGHLo2By5iRexubUB1a68zUQ7WiQPePMDcHp4Qkeqg` |
| Graduate to DAMM v2 | **web app UI** (token page) | `8BgtwwbHkXfgJER3xGerfg6zGLHW96NHikwvztiFSt7SM9jRySqcU49BiowUnoCAu2yjEcm3Dbik9JSkMN3hr1e` |
| Unsold tokens to the locked buffer | **web app UI** | `4YheQfNcTkTQz4qc9X39J7TUusfhHdgjw8xQzA3L9nX7agY2SxmnS6xJnoYfAQN87992VA7wPaXERCyJUHL8RUxt` |
| Open the NFT vault | **web app UI** | `3QxGwMX1Vti8jzhx4a4T45uD521dtC5cw7jDHjiwYGgrwFq2UEgKAqewJTVXCureszs3Go97BZkZ5X78ipWfPyED` |
| Capture (0.01 SOL fee to the fee wallet) | **web app UI** | `622QDh83RTGVC3j3gv95cbL5xVPSGC2HGHufyVdd8UMS7gZa6EryDRMUCc89zJb3bCjGa4CTBbPebww1DzqqRhQj` |
| Reveal | **web app UI** | `4akcWBhu6XuzNWSPHZSeBbSLR1nA9Y8WdpjohSGb4dpqtRWGs8b79kkdb3uq6uSY8v5tnfU9ecTeacF7RdaSRUhz` |
| Settle, lazy-mints the NFT from the published manifest | **web app UI** | `5PFgBmvUkYxNpsQDoujUZqWanGksmg9U5AXmfbGjyhti8X3mfwktQLC1TBqTX84bAmZ3fWqibfH6NyVZPKWVnkMD` |
| Release NFT #53 back to 1,000,000 AHLIVE | **web app UI** | `4JCLNRK25Kxa24PsKogicF136MK69QN9GmabE786rP8ja97xoezUawSAfUYPFEc7wywqEFZS3mBnWa6r2Bt6j4bc` |
| Second capture / reveal / settle (mints #59) | **web app UI** | `4C6AfPRCUhBxnDUFw2mpq1Rg4U4G8RHTusRKEcPfEqH8Peu1HoQFZ55Xiex3kTnuWhQcj6nPW2kTofVyANwTsCei`, `5g8Zw8AgUUBJo3hEMxYDXtf36W4s5mryck8YUkv3cTfqoGw1BC9yN7uWUnEx5u1Amfgxc4QMGpxNURMzXE8NnPTU`, `3o4m24eAKiZCTFGzNK9JReNpNyiMYwYmSd3JXKdo3mnSiQPtF4JE1YbiVU6yN6idQ8DvxxXjTrVSbeVNFNfpCBNx` |
| Release NFT #59 (holdings refresh on their own) | **web app UI** | `VEB6eX4a46Sw3ivUWGwhDYMC8ZcT61VY4YpCPiMREk3TpAcEqo21rkMJMt83W1uMiq5RhJtJTfoUGBvi3MhTF9x` |

## Roadmap

- **Burn** (burn tokens to mint an NFT, one-way): coming soon. It was built, then held back because
  its NFTs came out in a public order, so rare pieces could be sniped. The fix (the same Switchboard
  blind pick as Hybrid) is a vault change that needs review first:
  [docs/burn-mode-blind-assignment.md](docs/burn-mode-blind-assignment.md).
- **Tax split** (a fixed Token-2022 transfer tax shared with NFT holders, in a locked DAMM v2 pool):
  full build planned after Oct 12. Design, security and legal notes:
  [docs/tax-mode-design.md](docs/tax-mode-design.md).
- **Raffle**: later.
- **Mainnet**: after an independent audit of the Hybrid programs, with upgrades behind a multisig
  and timelock.

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
3. Open the Forge Gems or Shieldwall page to trade on a DAMM v2 pool after graduation and to capture,
   re-roll and release NFTs.
4. Open `/meteora` ("How it works" in the footer) for the integration overview.

**Hosting (Vercel/Netlify):** root directory `app/`, build `npm run build`, and set the
environment variable `NEXT_PUBLIC_SOLANA_CLUSTER=devnet`. There are no secrets to configure.
Details are in `app/README.md`.

## Limits and what is devnet-only

- **Unaudited.** Devnet only, with test SOL. The programs have not been audited.
- The 0.1 SOL graduation threshold is a devnet test value.
- **Migration on devnet is not automatic.** On mainnet Meteora's migration keepers handle it. On
  devnet the token page shows a "Graduate to the trading pool" button (the permissionless
  `migrateToDammV2`), then "Send unsold tokens to the locked buffer" and, for Hybrid, "Open the NFT vault".
- Hybrid art and metadata are stored on the Irys **devnet** node: free for files up to 100 KB (the
  wizard resizes images in the browser) and not permanent. Mainnet needs a funded upload.
- The devnet curve config has no anti-snipe schedule, so a dev buy pays the flat 1% trade fee.
- Burn, Tax and Raffle are "Coming soon".
- Each Hybrid collection has at least 100 NFTs (a program rule); the wizard takes up to 50 images and
  the NFTs share them in turn.
- NFT metadata is stored as `ar://<id>` on chain. On devnet the app reads those items from the Irys
  devnet node; wallets and marketplaces that resolve `ar://` through arweave.net won't find devnet items.
- The home stats strip counts curve trades from each pool's transaction list minus its launch and
  graduation steps; trades on the DAMM v2 pool after graduation aren't included.
- The price chart is a placeholder.
- The public devnet RPC is rate limited. Under load some live numbers may not load, and a reload
  fixes it.
