# Tax mode spike: results (devnet, Oct 2 2026)

A one-day, script-only experiment run before the full Tax build (planned after Oct 12; see
[tax-mode-design.md](./tax-mode-design.md)). **No program changes were made.** Everything below
happened on Solana **devnet** with throwaway wallets. Total cost was about **0.05 SOL** net.

## The question

Can a "Tax" launch be a plain Token-2022 transfer-fee token that sits in a one-sided Meteora
DAMM v2 pool whose liquidity is locked forever, with the fees collected later by a vault? And
does the whole launch (mint + pool + lock) fit in one transaction?

## Setup

| Item | Value |
| --- | --- |
| Mint (Token-2022) | [`DUnFEqx4BZDYnk9qADfzZgdfq2Di7Y2jNTqMHPqSyvMS`](https://explorer.solana.com/address/DUnFEqx4BZDYnk9qADfzZgdfq2Di7Y2jNTqMHPqSyvMS?cluster=devnet) |
| Supply | 1,000,000,000 tokens (6 decimals), all deposited into the pool |
| Transfer fee | 1% (100 bps); `maximum_fee` = whole supply, so the fee is never capped |
| Fee rate authority | **None** (the fee can never change) |
| Withdraw-withheld authority | `3m5h7DhyvfSbrjR2j8t4onQd69UUwbUBRhe7YGRzavJQ` (throwaway key standing in for a program PDA) |
| Mint / freeze authority | None / None |
| Metadata | Token-2022 metadata pointer + on-mint metadata; update authority removed after setup |
| Pool (DAMM v2, customizable) | [`8dqJL89xMidh9XWhcvc8JyiYwWXfqGJhgyR2ra9BosG8`](https://explorer.solana.com/address/8dqJL89xMidh9XWhcvc8JyiYwWXfqGJhgyR2ra9BosG8?cluster=devnet) |
| Pool settings | one-sided (token only), 1% pool fee, fees collected in SOL (`collectFeeMode` 1), min price 1e-8 SOL |
| Position | `8VzR3c9cWg5S9aL9BKxrXSZCwDaoeT7ysqZUrPhqa3nu` |
| Lock | `isLockLiquidity: true` at pool creation. On-chain check after the spike: unlocked liquidity **0**, permanently locked = all pool liquidity |

## Launch transactions

| Tx | Signature | Size | Compute |
| --- | --- | --- | --- |
| Mint + fee config + metadata + mint supply + drop authorities | [`3Z4p2cj2…5Axe7cSnV`](https://explorer.solana.com/tx/3Z4p2cj2z8y2VbuBXKnJia11RWnouUhrvgotPzHbyjNBaeX2r4ir9NSfK1VzPGKRP7sozyvVxho47my5Axe7cSnV?cluster=devnet) | 809 bytes | 42,623 CU |
| Create pool + position + permanent lock | [`3sZ92Ld5…cxRe9jFiyy2aD`](https://explorer.solana.com/tx/3sZ92Ld5N7yq4v3btXU1FRBxsMQWPgbTuNNi4cnrAzmngkzGL716ZyisphoS6ueeF5mh5rdxmgUcxRe9jFiyy2aD?cluster=devnet) | 1,054 bytes | 136,241 CU |

Together that is about **179k CU**, far below the 1.4M per-transaction limit. **Compute is not the
problem; transaction size is.**

## Does it fit in one transaction?

Solana's limit is 1,232 bytes per transaction.

| Composition (built client-side) | Instructions | Accounts | Signers | Size | Fits? |
| --- | --- | --- | --- | --- | --- |
| Mint (no metadata) + pool + lock | 12 | 21 | 3 | 1,287 bytes | No (55 over) |
| Same, with a static address lookup table (programs, WSOL, pool authority, event authority) | 12 | 21 | 3 | 1,228 bytes | Yes, only 4 bytes spare |
| Same + token metadata | — | — | — | over the limit even with the lookup table | No |
| Pool + lock only | 7 | 21 | 2 | 1,054 bytes (964 with lookup table) | Yes |

**Takeaway:** a fully client-composed atomic launch only fits with a lookup table and no metadata,
and with almost no headroom. The robust way to make it atomic is **one instruction in our own
program** that does the mint setup, pool creation and lock through CPIs: the client then sends a
few accounts and a few bytes of data. A two-transaction launch also works, but leaves a short
window where the mint exists without a pool. A program could guard that window (for example by
holding the supply in a PDA until the pool exists).

## Transfer fees in practice

- **The initial pool deposit pays the fee too.** Depositing 1B tokens into the pool vault withheld
  1% (10,000,000 tokens) inside the vault at creation. The pool only got 990M of working liquidity.
  The full build must plan for that (or exempt the deposit by funding the vault differently).
- Trades (two buys, one sell), all with the 1% fee applied by Token-2022:

| Trade | Signature |
| --- | --- |
| Buy, 0.02 SOL (trader 1) | [`4Df3VDPG…eEykaoqE7zFhH3Hf1RmwiXhZxXfz2S3afD`](https://explorer.solana.com/tx/4Df3VDPGRFaQFw2112PQEsbu6Bv1gdxuuv39fjP9PysZd8LvjexmTEeykaoqE7zFhH3Hf1RmwiXhZxXfz2S3afD?cluster=devnet) |
| Buy, 0.01 SOL (trader 2) | [`2CSWH98A…xV51uJbA8kTofZMpTk`](https://explorer.solana.com/tx/2CSWH98ARTGN29bhyUUbALTpo9H6e7ZgBLrRczL8GPXaSaMNPRjcNF2v3uPNHntCJUfZCgxV51uJbA8kTofZMpTk?cluster=devnet) |
| Sell, ~978k tokens (trader 1) | [`4uM4n2UJ…NQNEVHR9KH4zD`](https://explorer.solana.com/tx/4uM4n2UJY2enBH4BVARLxRbpAu9ApuojHSYFZBXXj9NPxLhZCRCndVqKu5z2oHdWfJm9tNFjrTHNQNEVHR9KH4zD?cluster=devnet) |

- Fees are withheld **in the receiving account**: on buys that is the trader's wallet account, on
  sells it is the pool vault. Withheld before harvest (base units):

| Account | Withheld |
| --- | --- |
| Pool token vault | 10,009,781,437,126 |
| Trader 1 | 19,760,479,042 |
| Trader 2 | 9,850,687,459 |
| **Total** | **10,039,392,603,627** |

## Harvesting

1. `HarvestWithheldTokensToMint` is **permissionless**, and it worked on the live pool vault (owned
   by the pool authority) as well as on trader accounts:
   [`5ccFKtnk…CSSJWf3qQQMav`](https://explorer.solana.com/tx/5ccFKtnk6UdCMphSBMGzz8RKeB9yXziuKTFy7ehtKJwVVsXbNv9bcnTJeAv7uKZKLPBMPx2fJqCCSSJWf3qQQMav?cluster=devnet)
2. `WithdrawWithheldTokensFromMint`, signed by the withdraw authority, sent everything to the
   treasury token account `4g1ENtGYCP6t8Yg6ae7A9sGkDxyafEHFMYcqLz93QyTA`:
   [`5x8JgQGT…4W64WeCePgC6NT5B9kmBZdq`](https://explorer.solana.com/tx/5x8JgQGTnjbkaDVoZ3iTgTdye8kM42wV998MCqPv6rmL787FFUKW1tr1WoDUQ5boe4W64WeCePgC6NT5B9kmBZdq?cluster=devnet)
3. Treasury balance afterwards: **10,039,392,603,627**, which matches the total above exactly. All
   withheld amounts read 0 after the harvest.

In the full build, step 2 is signed by a program PDA instead of a key, so collected fees can only go
where the program says (the vault). Step 1 can be run by anyone (a crank or the UI), so no trust is needed.

## Jupiter and Phantom

- **Not checkable on devnet.** Jupiter has no devnet routing, and this pool has no mainnet twin.
- From public docs: Phantom supports Token-2022 transfer-fee tokens and shows the fee on the send
  confirmation screen. Jupiter routes Token-2022 tokens when the pool program supports the mint's
  extensions (DAMM v2 does). Some Jupiter order products (limit and recurring orders) may exclude
  transfer-fee tokens. Both need a mainnet check before the Tax launch.
- Wallets and explorers label these tokens as "transfer fee 1%", which is honest and expected, but
  the UI must say clearly that every transfer (including wallet-to-wallet sends) pays the fee.

## Recommendation for the full build (after Oct 12)

1. Launch atomically through **one program instruction** (CPI into Token-2022 and DAMM v2) rather
   than a client-built transaction. If that is not ready, use a lookup table and two transactions,
   with the supply held by a PDA until the pool exists.
2. Fee rate authority **None**, withdraw authority = **program PDA**, `maximum_fee` = supply.
3. Account for the 1% withheld on the initial pool deposit in the pricing and the copy.
4. A permissionless harvest crank (harvest-to-mint, then a PDA-signed withdraw into the vault).
5. Keep the legal and security notes in [tax-mode-design.md](./tax-mode-design.md): a fee on every
   transfer can look like a tax or securities feature in some places, and it needs review and an
   audit before mainnet.

## Cost

About 0.05 SOL net on devnet: rent for the mint, pool and position, plus fees and the test trades.
Leftover SOL from the throwaway wallets was sent back to the deployer.
