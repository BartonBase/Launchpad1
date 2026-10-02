/**
 * Official knight mascot art (Barton, 2026-10-02): the placeholder knight is gone, the header shows
 * the helmet mark next to the wordmark, every knight slot points at a real, size-budgeted WebP, and
 * the favicon/app icons and OG image exist as Next file-convention assets.
 */
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { KNIGHTS } from "@/components/armory/Knight";

const APP = join(__dirname, "..");
const SRC = join(APP, "src");
function files(dir: string): string[] {
  return readdirSync(dir).flatMap((f) => {
    const p = join(dir, f);
    return statSync(p).isDirectory() ? files(p) : /\.(tsx?|css)$/.test(p) ? [p] : [];
  });
}

describe("brand art", () => {
  it("has no placeholder mascot left anywhere in the app", () => {
    const hits = files(SRC).filter((p) => /MascotPlaceholder|knight-placeholder|Placeholder art|mascot coming soon/i.test(readFileSync(p, "utf8")));
    expect(hits).toEqual([]);
    expect(existsSync(join(APP, "public", "mascot"))).toBe(false);
  });

  it("shows the helmet mark beside the wordmark in the header", () => {
    const header = readFileSync(join(SRC, "components", "SiteHeader.tsx"), "utf8");
    expect(header).toMatch(/<HelmetMark size=\{(2[8-9]|3[0-2])\} \/>\s*\{BRAND\.name\}/);
  });

  it("serves each knight as a size-budgeted WebP with its PNG cut-out alongside", () => {
    for (const k of Object.values(KNIGHTS)) {
      const webp = join(APP, "public", k.src);
      expect(existsSync(webp), k.src).toBe(true);
      expect(statSync(webp).size).toBeLessThan(200_000);
      expect(existsSync(webp.replace(/\.webp$/, ".png"))).toBe(true);
      expect(k.alt.length).toBeGreaterThan(10);
    }
    expect(statSync(join(APP, "public", "brand", "knights", "helmet-96.webp")).size).toBeLessThan(20_000);
  });

  it("ships favicon, app icons and the social share image", () => {
    for (const f of ["favicon.ico", "icon1.png", "icon2.png", "apple-icon.png", "opengraph-image.jpg", "twitter-image.jpg", "opengraph-image.alt.txt"]) {
      expect(existsSync(join(SRC, "app", f)), f).toBe(true);
    }
    expect(existsSync(join(SRC, "app", "icon.svg"))).toBe(false);
  });
});
