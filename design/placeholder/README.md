# Placeholder mascot: Armory knight (TEMPORARY)

**This is placeholder art.** It's a temporary stand-in until Barton's Higgsfield mascot art arrives, and then it gets replaced everywhere. It isn't the brand mascot and isn't final.

- One comic-book knight in a front/hero pose (waving, holding a shield). The style follows the archived bomb sheet (`../mascot/archive-bomb/`): thick ink line (#050507), flat cel shading, cream highlights, friendly. Ember #FF6A2B is the only accent (the plume and the shield emblem), on the dark palette.
- Code-drawn SVG: `_build/knight.py` writes `knight-placeholder.svg`. The PNGs are rendered with headless Chrome (`/workspace/shottool/knight-render.js`).
- Not for use as a logo, favicon or app icon. The pages keep no favicon, and the nav keeps the text wordmark.

## Files
| File | What it is |
|---|---|
| `knight-placeholder.svg` | The character only, 400×400 viewBox, transparent background. Has `<title>`/`<desc>` that say it's placeholder art. |
| `knight-placeholder-1024.png` | The character only, 1024×1024, transparent background. |
| `knight-placeholder-wordmark-1024.png` | 1024×1024 on #08090A: the knight above the "Armory" wordmark (Bricolage ExtraBold) and a muted "Placeholder art · mascot coming soon" line. |
| `_build/knight.py` | Source. Re-run it, then the render script, after any edit. |

## Where it's used
Every mascot box in the Obsidian mockups (`design/directions/a-obsidian/`) and in `design/system/components.html`. It sits at the box's footprint (168 / 132 / 104 / 96 px), with alt text "Placeholder art, mascot coming soon" and a tiny muted "Placeholder art" caption on boxes 120px or taller. To swap in the real art, change `MPH` in `design/directions/_build/a_obsidian.py` and re-run it.
