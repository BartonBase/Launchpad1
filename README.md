# Armory (ARMS)

Armory is a Solana launchpad for hybrid token/NFT collections. A creator can launch a plain meme coin, or a coin
with an NFT collection built in, where a fixed number of tokens converts into one NFT and back.

> **Status: devnet only. Unaudited. Nothing is live.** There is no mainnet deployment and no real funds are
> involved. A professional third-party audit is required before mainnet, and nothing goes live without Barton's
> approval.

## Launch types

| Type | What it is | State |
|---|---|---|
| Plain | Classic SPL token, fixed supply of 1,000,000,000, mint and freeze authority revoked. No NFTs. | Active in development (built on `fix/modes-1-5`, not deployed to devnet yet) |
| Hybrid | SPL token plus a Metaplex Core NFT collection. Capture turns the collection's ratio of tokens into a random NFT; release turns the NFT back into exactly that many tokens. | Active in development (on devnet) |
| Burn | Like Hybrid, but wrapping burns the tokens and mints an NFT. One way, no release. | Active in development (built on `fix/modes-1-5`, not deployed to devnet yet) |
| Tax split | Token-2022 transfer tax shared with NFT holders. | Shelved (ADR-020) |
| Raffle | Token-2022 tax pot won by one NFT. | Shelved (ADR-020) |

Tax split and raffle stay shelved until there is a public way to buy the tokens, the stuck-funds fixes are in,
and a raffle legal check is done. The site shows them as "Coming soon".

## Security model

Security is the top priority. The design is shaped by the Stonk.fun incident (see `docs/stonkfun-lessons.md`).

- **Frozen config.** Each launch's settings (ratio, fees, mint, fee wallet) are fixed when it is created. No
  instruction can change them afterwards.
- **No pause.** There is no pause or guardian switch. Nobody can halt capture, re-roll or release.
- **Per-collection escrow.** Each Hybrid collection has its own program-owned vault holding the tokens behind its
  NFTs. There is no admin withdraw.
- **Approved Switchboard randomness.** NFT assignment and re-rolls use Switchboard On-Demand randomness. The
  Switchboard program is pinned and only queues on a compile-time approved list (`APPROVED_SB_QUEUES`) are
  accepted. The program picks the oracle, so neither the user nor the creator can choose which NFT they get.
- **Release always returns exactly N tokens.** Releasing an NFT returns exactly the collection's ratio of tokens,
  with no fee taken from them.
- **Fixed supply.** Every token launches with exactly 1,000,000,000 supply and the mint authority revoked.
- **Program upgrades are not locked yet.** On devnet the upgrade authority is a throwaway deployer key. The
  planned mainnet setup (multisig and timelock) is still pending Barton's decision.
- QA's 2026-10-01 review of the Modes 1-5 code found no real bugs and confirmed that nobody can drain program-held
  funds (`qa/reports/2026-10-01-modes-1-5.md`). Audit reports are in `security/`.

## Mascot

The mascot is a comic-book knight. The art in the repo is **placeholder art** (`design/placeholder/`) until the
final mascot is made. It is not used as a logo, favicon or app icon.

## Design

Direction A, "Obsidian": a near-black, premium look with one bright accent.

- Accent: **Ember `#FF6A2B`**, the only saturated colour.
- Type: **Bricolage Grotesque** for headlines and the text wordmark, Geist for UI and body, Geist Mono for labels.
- No animation.
- Mockups: `design/directions/a-obsidian/`. Tokens and components: `design/system/`. Site map: `design/sitemap.md`.

## Run the app locally

The web app is in `app/` (Next.js, TypeScript). It runs against localnet or devnet only; mainnet is refused at
startup. It needs Node 20.19 or newer.

```bash
cd app
npm ci
NEXT_PUBLIC_SOLANA_CLUSTER=devnet npm run dev   # http://localhost:3000
npm test                                        # unit tests
```

See `app/README.md` for all scripts, environment variables and the transaction safety pipeline. The `.env.example`
file lists the only (public) settings; no secrets are needed.

The on-chain programs (`programs/`) are built and tested as described in `docs/STATUS.md` ("Build and test") and
`docs/DEV_SETUP.md`.

## Where the docs are

| Path | What |
|---|---|
| `docs/BRIEF.md` | Project brief and Barton's decisions |
| `docs/STATUS.md` | Current engineering status |
| `docs/DECISIONS.md` | Architecture decision records (ADRs) |
| `docs/ITINERARY.md` | Website plan |
| `docs/ARCHITECTURE.md`, `docs/THREAT_MODEL.md` | Program design and threat model |
| `design/` | Visual direction, design system, placeholder mascot, site map |
| `app/` | Armory web app |
| `qa/` | Test plan, findings tracker and QA reports |
| `security/` | Auditor reports and the merged findings list |
| `review/` | Review notes |

## Keys

No keys, keypairs or `.env` files are committed. `.gitignore` excludes them.
