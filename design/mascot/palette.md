# Armory palette and type (2026-10-01)

Brand: **Armory** (final, Barton 10/1 5:13 PM MT; formerly Fuze/Mintmark). The palette is Obsidian's near-black neutrals with one saturated accent. The machine-readable source of truth is `../system/tokens.json`, extracted from the mockup CSS. The swatch card is `palette.png`.

## Accent: Ember #FF6A2B (kept)

Ember stays. No mascot is involved; the case stands on its own:
- A forge-fire orange suits the name **Armory** (a place where arms are forged and kept). It's warm and ownable, and it moves the site off the purple-on-black look of most Solana dashboards.
- It stays clearly separate from the amber (#E8B04B) on every Demo data / Example / Pending tag, which a yellow accent wouldn't.
- Contrast: about 6.9:1 on #08090A. White on Ember is only about 2.9:1, so **filled Ember buttons use dark text #120805**.
- It's already applied and checked across all four Obsidian pages.

| Token | Value | Use |
|---|---|---|
| Accent | `#FF6A2B` | Primary buttons, progress fill, still bar tip, active chips |
| Accent text | `#FF9A62` | Accent text and icons on dark; also the hover/active text |
| On-accent text | `#120805` | Text on filled Ember |
| Accent tint | `rgba(255,106,43,.12)` | Chip and callout backgrounds |
| Accent glow | `rgba(255,106,43,.45)` | Bar-tip glow |
| Button shadow | `rgba(255,106,43,.6)` | Ember button drop shadow |
| Focus | border `rgba(255,106,43,.6)`, ring `rgba(255,106,43,.15)` | Inputs |

Not in the CSS yet: a separate pressed shade. Earlier drafts proposed #E85A1E; add it to the CSS before using it.

## Neutrals

| Token | Value |
|---|---|
| Background | `#08090A` |
| Surface 1 / 2 / 3 | `#0E0F11` / `#131417` / `#1A1B1F` |
| Border / strong | `rgba(255,255,255,.07)` / `rgba(255,255,255,.12)` |
| Text primary / muted / dim | `#ECEEF1` / `#8B909A` / `#5D626C` |

## Status

| Token | Value | Note |
|---|---|---|
| Success | `#3ECF8E` | Revoked, graduated, price up |
| Warn / demo tags | `#E8B04B` | Demo data, Example, Pending. Never decorative. |
| Error | `#F25C7A` | Errors, sells, price down |

The cosmetic rarity tints (slate #9AA3AE, sage #86C5A6, iris #A697FF, champagne #D6B67C) stay muted.

## Type

- **Headline and wordmark: Bricolage Grotesque ExtraBold (800), width 82, opsz 96.** Google Fonts, SIL OFL. Set big and condensed, it has a punchy, poster-like voice that suits "Armory" while still reading premium and modern. It's used only for big headlines and the plain-text **Armory** wordmark in the nav and footer, with no symbol.
- **Alternate: Archivo ExtraBold, condensed (width ~72).** Google Fonts, OFL. More neutral and engineered.
- **UI and body: Geist. Labels and numbers: Geist Mono.**

## Name and ticker

ARMS is the platform ticker, but **no ARMS token appears on any page**. The pages use only the name Armory, so they don't imply a platform token exists. The collection tickers (NOCT and the rest) are per-collection and unchanged.
