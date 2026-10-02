# Mascot: pending

**Status (2026-10-01, Barton 5:14 PM MT): Barton is making the mascot himself in Higgsfield.** There's no mascot art in the product mockups for now.

- **Bomb mascot (Fuze era): archived** in `archive-bomb/`, with the sheet, seven pose PNGs, SVG sources and build scripts. Kept for reference; not in use.
- **Knight mascot: cancelled** before any art was made. No knight files, logo, favicon or app icon exist.
- **Placeholders:** wherever a pose sat on the pages, a dashed muted-grey box reading "Mascot, coming from Higgsfield" holds the same footprint:
  - home hero, beside the headline (168×168)
  - token.html curve progress (104×104)
  - "What happens at graduation" (132×132)
  - Convert panel (96×96)
- **Brand mark:** a plain-text **Armory** wordmark in Bricolage Grotesque, with no symbol. The old circle-and-square mark is gone.
- Still images only when the art arrives: no animation or CSS motion.

## Files here
| File | What |
|---|---|
| `palette.md`, `palette.png` | Armory palette (Ember accent) and type card |
| `_build/palette.py`, `_build/render.js` | Build `palette.html` and render `palette.png` (Chrome via playwright-core): `cd _build && python3 palette.py && node render.js` |
| `archive-bomb/` | Archived bomb mascot |

## Usage rules for whatever mascot lands
1. Ember #FF6A2B stays the only saturated colour in the UI.
2. The mascot never implies returns. No price-up, profit, "jackpot", "upgrade" or "valuable" framing.
3. The graduation pose is a celebration of a product milestone, never about gains.
4. It never covers fees, risk copy or tags such as Demo / Example / Beta.
5. Still images only.

## Open questions
- The Higgsfield art's format and size. The placeholder footprints are listed above, and they're also in `../system/tokens.json` (`size.mascotPlaceholder`).
- Logo symbol, favicon and app icon are on hold until the mascot and brand direction land. The text wordmark is the stopgap.
- Launch styles "plain" and "burn" are still undefined in BRIEF.md. The home page shows them as Planned / to be confirmed.
- Armory still needs a domain, trademark and X-handle check.
