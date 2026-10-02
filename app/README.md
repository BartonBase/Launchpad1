# Armory web app

Next.js (App Router) + TypeScript (strict) + Tailwind v4 front-end for **Armory**: launch a Solana
meme coin on its own, or with an NFT collection built in (Hybrid: a fixed number of tokens
converts into one random Metaplex Core NFT and back). Wired to the DEVNET `hybrid_launch` and
`hybrid_vault` programs.

> **Unaudited beta, localnet / devnet only.** Mainnet is refused at startup, no private keys
> exist anywhere in this app (the only keypair is a throwaway mint key generated in the browser
> for a launch), and all funds are test funds.

## Quick start

```bash
npm ci
NEXT_PUBLIC_SOLANA_CLUSTER=devnet npm run dev   # http://localhost:3000
```

| Script                | What it does |
| --------------------- | ------------ |
| `npm run dev`         | Next dev server on port 3000 |
| `npm run build`       | Production build (fails on mainnet config) |
| `npm run start`       | Serve the production build |
| `npm run lint`        | ESLint, 0 warnings allowed |
| `npm run typecheck`   | `tsc --noEmit` (strict, noUncheckedIndexedAccess) |
| `npm test`            | Vitest unit tests (offline) |
| `npm run test:devnet` | DEVNET checks, simulation only: reads + `simulateTransaction` of launch / capture (incl. oracle retry) / re-roll / settle size / DBC launch / unwrap, gateway allowlist, lookup tables. Uses the public key of `DEVNET_TEST_WALLET` (default `/workspace/scratch/frontend-devnet/wallet.json`, must be `7TyR…`) as payer; the secret key is never used, nothing is signed or sent. |
| `npm run check:headings` | Fetches every route from the running dev server and checks exactly one `h1` and no skipped heading levels |
| `npm run icons`       | Regenerate the launch-type icon elements from the design SVGs (`scripts/build-icons.mjs`) |
| `npm run tokens`      | Regenerate `src/styles/tokens.css` from `../design/system/tokens.json` |
| `npm run gen:idl`     | Regenerate `src/lib/generated/idlMeta.ts` from `src/lib/generated/idl/*.json` |

Next.js collects anonymous telemetry by default; opt out with
`npx next telemetry disable` or `NEXT_TELEMETRY_DISABLED=1`.

### Localnet (default)

Requires the Solana CLI (not installed on this box):

```bash
solana-test-validator --reset
# SPL Token, Token-2022, Associated Token and Memo are built into the validator.
```

Metaplex Core, Switchboard and the Armory programs are not built in. Clone them from **devnet**
(never mainnet) when you need them (`--clone-upgradeable-program` for
`CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d`, `9Loc4hQZ…`, `BEfL9dcc…`). Devnet is the
supported target for this pass.

Fund a test wallet with `solana airdrop 10 <address> --url localhost`. In your
wallet extension, switch to a localnet/custom RPC (`http://127.0.0.1:8899`).

### Devnet

```bash
NEXT_PUBLIC_SOLANA_CLUSTER=devnet npm run dev
```

Uses `https://api.devnet.solana.com` (public, rate limited). Get devnet SOL
from the faucet (`solana airdrop` / faucet.solana.com).

### Environment variables

Only `NEXT_PUBLIC_*` variables exist and all of them are public (inlined into
the browser bundle). There are no secrets.

| Variable                      | Values                          | Notes |
| ----------------------------- | ------------------------------- | ----- |
| `NEXT_PUBLIC_SOLANA_CLUSTER`  | `localnet` (default), `devnet`  | Anything containing `mainnet` throws `MainnetForbiddenError`; unknown values throw (no silent fallback). |
| `NEXT_PUBLIC_SOLANA_RPC_URL`  | empty, or one of the cluster's allowed endpoints | localnet: `http://127.0.0.1:8899` / `http://localhost:8899`; devnet: `https://api.devnet.solana.com`. Credentials, paths, query strings and other hosts are rejected, since the CSP is derived from the same list. |

Changing either variable requires restarting `dev` / rebuilding (values are
inlined at build time).

## Layout

```
src/
  app/
    layout.tsx              shell: always-visible trust strip (unaudited beta, deposit cap,
                            authorities) + header + footer; ISR 30 s
    page.tsx                /                    landing (hero, how it works, launch styles, live launches)
    explore/                /explore             all launches read from chain; ?q= search, type/phase filters
    t/[mint]/               /t/<mint>            token page: phase, Convert panel (capture / release /
                                                 re-roll / reveal / settle / expire), facts read from chain.
                                                 ?panel=capture|release|reroll preselects; /t/example-plain and
                                                 /t/example-burn are static design previews. /token/<mint> and
                                                 /collections/<mint> redirect here (307).
    launch/                 /launch              wizard, ?type=plain|hybrid|burn (native hybrid launch on devnet,
                                                 plus a simulate-only Meteora DBC preview)
    portfolio/              /portfolio           wallet tokens, NFTs, pending requests
    trust/ faq/ bug-bounty/ static trust pages (+ live authorities, pipeline self-test)
    fonts.ts                next/font/google: Bricolage Grotesque, Geist, Geist Mono (self-hosted)
  config/
    cluster.ts              cluster config + mainnet rejection
    programs.ts             pinned program IDs + top-level allowlist (CPI-only programs labelled)
    armory.ts               product constants, tier fees, deposit, launch types + feature flags
  lib/
    generated/              typed clients (see "Devnet wiring")
    armory/                 reads.ts (chain reads -> serializable DTOs), builders.ts (tx builders),
                            fees.ts (capture cost math), launchForm.ts, format.ts, server.ts (TTL cache)
    tx/                     transaction safety pipeline (see below)
    validate.ts             untrusted-input validators
  components/armory/        TrustStrip, AuthoritiesPanel, ConvertPanel, CostBreakdown, LaunchWizard,
                            PortfolioView, LaunchCard, LaunchTypes, MascotPlaceholder
  styles/tokens.css         GENERATED from ../design/system/tokens.json (scripts/build-tokens.mjs)
tests/                      unit tests; tests/devnet/*.devnet.ts = read-only devnet checks
```

## Devnet wiring

| Program | ID | IDL |
| --- | --- | --- |
| `hybrid_launch` | `9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8` | `src/lib/generated/idl/hybrid_launch.json` |
| `hybrid_vault`  | `BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG` | `src/lib/generated/idl/hybrid_vault.json` |

- **IDL source: only `/workspace/idl/devnet/`** (copied verbatim with its README and `_errors.ts`).
  They are byte-for-byte matches for the deployed programs (commit `5cd7a7e`). The repo's
  `target/idl/` is the undeployed `fix/modes-1-5` branch and must not be used. There are no
  on-chain IDL accounts.
- Clients are hand-written against those IDLs (`hybridLaunch.ts`, `hybridVault.ts`) with
  discriminators/errors generated into `idlMeta.ts`; `switchboard.ts` and `core.ts` are minimal
  read-only decoders. All instructions go through the safe-send pipeline.
- Deployed and wired: `launch` (native hybrid launch), `request_capture`, `request_reroll`
  (v0; bundles Switchboard `init_randomness` with a fresh randomness keypair when the vault has no
  idle account), `unwrap` (release), `expire_request`, `reveal_randomness` (pending request →
  Reveal), `settle_capture` / `settle_reroll` (Settle; mints the picked NFT with its leaf + proof).
- **Oracle retry** (`lib/armory/oracle.ts`): the request is simulated first. 6050 WrongOracle →
  rebuild with the oracle from the `Right:` log; 6053 stale → add the stale oracle as a remaining
  account and try the next queue oracle (max 6 attempts).
- **Reveal**: the gateway URL is read from the oracle account and must match
  `SWITCHBOARD_GATEWAY_RE` (devnet: `https://<ipv4>.xip.switchboard-oracles.xyz/devnet`); the
  request is a plain `fetch` POST to `<gateway>/gateway/api/v1/randomness_reveal` (no redirects, no
  credentials, 15 s timeout, 4 KB cap) and the response is parsed strictly (`lib/armory/reveal.ts`,
  no SDK). `tests/reveal.test.ts` checks the request body, parsed response and the
  `reveal_randomness` instruction byte-for-byte against `tests/fixtures/switchboard-reveal.json`,
  captured from `@switchboard-xyz/on-demand` 3.10.6 and the e2e.cjs Anchor flow before the SDK was
  removed.
- **Settle**: the asset is learned from a probe simulation (`WrongAsset` → `Right:`). Minting needs
  the launch's traits manifest (`lib/armory/leaves.ts`); only the E2E launch has one. Over 1232
  bytes it requires a pinned lookup table (`PINNED_LOOKUP_TABLES`, none pinned yet); N=100 is 978
  bytes, N=10,000 is estimated at ~1242.
- **Meteora DBC** (`register_dbc_launch`): `buildDbcLaunchTx` creates the DBC pool on the pinned
  platform config (`DBC_PLATFORM_CONFIG`, `config/integrations.ts`) and registers it in one v0 tx.
  The graduation target is the config's `migration_quote_threshold` (devnet 0.1 SOL). The wizard
  offers it as preview only (`useSafeSend().start(build, { previewOnly })`: Confirm stays disabled).
- **Plain and burn are NOT deployed** (branch `fix/modes-1-5` @ `c43be58`, no date): shown as
  "Pending deploy", no builders. **Tax and raffle**: always shown as "Coming soon" cards (display
  only, no builders).
- **Devnet differences:** the launch program is the `devnet-e2e` test build with a 0.1 SOL
  graduation minimum (`MIN_GRADUATION_LAMPORTS` per cluster in `config/armory.ts`; real build
  10 SOL); each launch's threshold is read from its LaunchConfig. The devnet vault predates the
  exact-tier fee check and never raises 6061, so the app checks the exact tier client-side
  (`isExactTierFee`) and refuses to build capture / re-roll otherwise; tests never rely on 6061.
- Lazy-mint deposit: capture and re-roll require tier fee + refundable 0.0063381 SOL deposit +
  refundable temp rent + network fee; `CostBreakdown` itemizes it and warns on insufficient
  balance (`capture-insufficient-balance` / `reroll-insufficient-balance`).

### Stable E2E test ids

`trust-strip`, `beta-label`, `deposit-cap`, `authorities-toggle`, `authorities-panel`,
`authority-hybrid_launch|hybrid_vault`, `wordmark`, `cluster-badge`, `launch-card`,
`launch-type-<id>` (`data-status`), `token-page`, `token-not-found`, `facts-card`,
`fact-mint-authority`, `fact-freeze-authority`, `fact-fee`, `fee-not-tier`, `convert-panel`,
`tab-capture|release|reroll`, `wallet-sol-balance`, `wallet-token-balance`,
`capture-cost-tier-fee|deposit|rent|network|total`, `capture-deposit-explainer`,
`capture-insufficient-balance`, `capture-blocked-reason`, `capture-submit` (same with `reroll-`),
`release-nft-option-<i>`, `reroll-nft-option-<i>`, `release-cost`, `release-submit`,
`pending-requests`, `pending-request-<seq>`, `expire-<seq>`, `convert-success`, `convert-error`,
`launch-step-<n>`, `launch-type-option-<id>`, `launch-name`, `launch-symbol`, `ratio-<n>`,
`launch-collection-size`, `launch-graduation`, `launch-graduation-min`, `launch-error-<field>`,
`launch-math`, `launch-review`, `launch-native-warning`, `launch-next`, `launch-back`,
`launch-submit`, `launch-success`, `tx-preview`, `tx-confirm`, `tx-cancel`, `mascot-placeholder`,
`skip-link`, `mobile-menu-button`, `nav-search`, `explore-search`, `type-filter-<id>`,
`soon-card-<id>`, `positions`, `portfolio-summary`, `portfolio-capslot`, `portfolio-nfts`,
`portfolio-history`, `tx-lookup-tables`, `tx-preview-only`, `upgrades-copy`, `reveal-<seq>`,
`settle-<seq>`, `launch-dbc`, `launch-dbc-preview`, `launch-dbc-graduation`.

## Transaction safety pipeline (`src/lib/tx`)

UI code never calls `wallet.sendTransaction` directly. It calls
`useSafeSend().start(buildFn)` and renders `<TxPreviewModal safeSend={...} />`.

```
build ─► validate ─► simulate ─► preview ─► [user clicks Confirm] ─► sign ─► tamper check ─► send ─► confirm
          (allowlist)  (RPC)     (modal)                               (wallet)  (bytes equal)  (our RPC)
```

1. **Build**: `buildFn({ connection, payer, recentBlockhash, lastValidBlockHeight })`
   returns a legacy `Transaction` or a `VersionedTransaction`, or `{ tx, signers }` when a
   local keypair must co-sign (the launch's fresh mint key; it signs after Confirm, before the
   wallet, and signatures don't change the previewed message bytes). The fee payer must be the
   connected wallet.
2. **Validate** (`validate.ts`, `validateInstructions`): every top-level
   instruction's program must be a **top-level** entry of the pinned allowlist for the active
   cluster (`config/programs.ts`); CPI-only entries are rejected (fixed: they used to pass).
   Otherwise the tx is rejected before simulation. v0 lookup tables must be pinned
   (`PINNED_LOOKUP_TABLES`), frozen and active, and every index must resolve (`lookup.ts`);
   loaded accounts are listed in the preview and included in the writable diff. Empty or oversized (>32 ix) transactions are rejected too.
3. **Simulate** (`simulate.ts`): `simulateTransaction` with `sigVerify: false`,
   `replaceRecentBlockhash: true` and `accounts` = every writable static
   account. Pre-state comes from `getMultipleAccountsInfo` and is diffed
   against the simulated post-state. The result has the logs, error, compute
   units, SOL deltas, SPL Token / Token-2022 token deltas (with mint decimals),
   and every program invoked, including CPIs parsed from logs.
4. **Preview** (`preview.ts`, `TxPreview`): a human-readable model built only
   from the message and the simulation: programs by name and full ID (unknown
   CPI programs are flagged), accounts written, SOL/token deltas, fee
   (`getFeeForMessage`), compute units, a SHA-256 message fingerprint,
   warnings, and **blocking errors**. `canSend` is false if validation or
   simulation failed or the fee payer isn't the wallet.
5. **Confirm**: `TxPreviewModal` is portaled to `<body>` with max z-index,
   makes the rest of the page `inert`, shows full program IDs with red/green
   deltas, and arms Confirm only after a 1.2 s delay. Confirm is disabled
   whenever `canSend` is false.
6. **Sign → tamper check → send**: if the wallet supports `signTransaction`,
   the app signs and then checks that the signed message bytes exactly equal
   the previewed bytes. Any modification by the wallet aborts the send and
   asks the user to review again. The signed tx is broadcast through the
   app's pinned localnet/devnet RPC, not the wallet's default RPC, then
   confirmed with blockhash + lastValidBlockHeight. Wallets without sign-only
   support fall back to `sendTransaction`, and the preview warns about that.
   `confirm()` is one-shot, so a double click can't send twice.

Known limitation (shown to the user as a warning): the pre-state read and
the simulation are not atomic. CPIs into unknown programs produce a warning but don't block,
because the top-level program is allowlisted and may legitimately CPI.

`PipelineSelfTest` on the Trust page runs the whole flow with a harmless memo
instruction to check wallet and RPC wiring.

## Program IDs (`src/config/programs.ts`)

Top-level allowlist: System, Compute Budget, SPL Token, Associated Token Account, SPL Memo,
`hybrid_launch`, `hybrid_vault`, Meteora DBC (`dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN`,
devnet; verified against docs.meteora.ag). CPI-only (labelled in previews, never a top-level
target): Token-2022, Metaplex Core, Metaplex Token Metadata, Switchboard On-Demand
(`Aio4gaXj…`; reveal goes through `hybrid_vault.reveal_randomness` because the randomness
authority is the vault PDA). The address lookup table program and wrapped SOL mint are pinned
constants. MPL-Hybrid was removed.

## Design

- Tokens: `../design/system/tokens.json` -> `npm run tokens` -> `src/styles/tokens.css`, mapped
  to Tailwind utilities in `globals.css`. Accent Ember `#FF6A2B`, `#120805` text on Ember.
- Layouts follow `../design/directions/a-obsidian/` (NOTE.md). Plain-text **Armory** wordmark
  (Bricolage Grotesque 800). Mascot spots (hero, curve progress, graduation, convert panel) use
  `public/mascot/knight-placeholder.svg`, alt "Placeholder art, mascot coming soon"; never a
  favicon/logo/icon.
- Accessibility (`../design/system/a11y.md`): static 2px focus outline from the focus tokens, skip
  link, one `h1` per page (`npm run check:headings`), card titles as `h2`.
- Motion is zero: a global reset disables all animations and transitions (incl. the wallet modal).
- Breakpoints: `src/styles/breakpoints.css` (mobile-first).

## Placeholders

- **Deposit cap**: `BETA_DEPOSIT_CAP` (10 SOL per wallet) shown site-wide, tagged "Example", not enforced.
- **Upgrade governance**: `PROGRAM_UPGRADES_COPY` in `config/armory.ts` (Trust, FAQ, authorities
  panel) is **pending Barton's approval**; the live upgrade authority is read from chain (devnet:
  throwaway deployer `An3ZmiB4…`).
- Curve progress and buying on the curve (Meteora DBC), token metadata, marketplace links,
  bug-bounty terms.

## Security decisions

- **Mainnet refused** in `src/config/cluster.ts` at module load. Any cluster
  name or RPC URL containing `mainnet` throws. `next.config.ts` imports it,
  so `next dev` and `next build` fail before anything runs, and the browser
  bundle throws too. Unknown clusters throw instead of falling back.
- **No keys or secrets**: the only keypair is the launch's fresh mint key, generated in the
  browser per launch and never stored; all other signing happens in the user's wallet. `.gitignore` excludes `.env*` (except
  `.env.example`) and keypair-looking files.
- **Wallets**: `wallets={[]}` with Wallet Standard auto-detection instead of
  the `wallet-adapter-wallets` mega-package, which keeps the dependency
  surface small. `autoConnect={false}`, so connecting is always an explicit
  user action.
- **Program allowlist + simulation + explicit confirmation** before any
  signature (see pipeline above). The signed bytes must match the previewed
  bytes, and broadcasting goes through the app's pinned RPC.
- **Input validation** (`src/lib/validate.ts`):
  - Public keys: strict base58, exactly 32 bytes, canonical round-trip,
    optional on-/off-curve check.
  - Amounts: `bigint` base units only. No floats, signs, exponents or extra
    decimals, and bounds are enforced (default 1..u64::MAX).
  - Metadata URIs: only `https://` (no credentials, IP literals, localhost or
    non-443 ports; optional host allowlist), `ipfs://<CID>` and
    `ar://<txid>`. Max 200 bytes (the Metaplex limit).
  - Route params are validated server-side. Invalid input renders a clean
    error message rather than throwing.
- **Security headers** (`next.config.ts`, all routes):
  - `Content-Security-Policy`:
    - `default-src 'self'`.
    - `connect-src` allows only the active cluster's RPC/WS origins (plus
      Next's HMR websocket in dev) and, on devnet only,
      `https://*.xip.switchboard-oracles.xyz` for reveals. Wildcard rather than the 9 current
      hosts because oracles rotate; `SWITCHBOARD_GATEWAY_RE` narrows it to IPv4 hosts on `/devnet`.
    - `img-src 'self' data: blob:`, `object-src 'none'`, `frame-src 'none'`,
      `base-uri 'self'`, `form-action 'self'`, `frame-ancestors 'none'`.
    - No other third-party origins. Fonts are self-hosted via `next/font`; the
      wallet-adapter CSS is vendored without its Google Fonts import.
  - `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`,
    `Referrer-Policy: strict-origin-when-cross-origin`.
  - `Permissions-Policy` denies camera, mic, geolocation, payment, USB, HID,
    serial, MIDI, sensors, display capture and topics.
  - `COOP same-origin-allow-popups`, `CORP same-origin`, HSTS in production,
    `poweredByHeader: false`.
- **CSP trade-off:** `script-src` includes `'unsafe-inline'` because App
  Router emits inline bootstrap/RSC scripts and a static header can't carry
  per-request nonces. `'unsafe-eval'` is dev-only. To go stricter, move the
  CSP to a `proxy.ts` (Next 16's middleware) that issues a per-request nonce
  (`script-src 'self' 'nonce-…' 'strict-dynamic'`). The cost is dynamic
  rendering of every page. Next's experimental SRI is the other option.
- **Dependencies:**
  - Pinned exactly, with `save-exact=true` in `.npmrc` and `package-lock.json`
    committed.
  - `.npmrc` also sets `legacy-peer-deps=true`, so npm does not auto-install
    `react-native` (~600 MB). It is a native-only peer of
    `@solana-mobile/wallet-adapter-mobile`, which `wallet-adapter-react`
    pulls in. Every peer the app actually needs is a direct dependency.
  - The Switchboard On-Demand SDK was tried and removed (6 high audit findings, a second Anchor
    copy) in favour of the small typed gateway client above.
