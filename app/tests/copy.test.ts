/**
 * Live-product copy guard (Barton, 2026-10-02): the UI reads like a live product. No devnet /
 * unaudited-demo / deposit-cap / test-funds disclaimers, "Plain" is shown as "Launch", and Meteora
 * lives on one low-key footer-linked page. The only network indicator is the small chip by the
 * wallet button (ClusterBadge). Internal ids ("plain") are unchanged.
 */
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { LAUNCH_TYPES, STATUS_LABEL } from "@/config/armory";

const ROOT = join(__dirname, "..", "src");
function files(dir: string): string[] {
  return readdirSync(dir).flatMap((f) => {
    const p = join(dir, f);
    return statSync(p).isDirectory() ? files(p) : /\.tsx?$/.test(p) ? [p] : [];
  });
}
const UI = [...files(join(ROOT, "app")), ...files(join(ROOT, "components"))];
const src = (p: string) => readFileSync(join(ROOT, p), "utf8");
/** Source minus code comments (comments may describe the devnet setup; only rendered copy matters). */
const code = (p: string) => readFileSync(p, "utf8").replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "");

const BANNED = [
  /unaudited (demo|beta)/i,
  /test SOL/i,
  /test funds/i,
  /test wallet/i,
  /not enforced/i,
  /deposit cap/i,
  /Example ·/,
  /Native test/,
  /Live on devnet/i,
  /Built on Meteora/i,
  /Plain launch/,
  /"Plain"/,
  />Plain</,
  /devnet setting/i,
  /CLUSTER\.label/,
  /not wired|indexer is wired/i,
];

describe("live-product copy", () => {
  it("names the plain launch type \"Launch\" and the live status \"Live\"", () => {
    expect(LAUNCH_TYPES.find((t) => t.id === "plain")!.name).toBe("Launch");
    expect(STATUS_LABEL.live).toBe("Live");
    for (const t of LAUNCH_TYPES) expect(`${t.name} ${t.short} ${t.description}`).not.toMatch(/devnet|Meteora|Plain/i);
  });

  it("has no devnet/demo/deposit-cap disclaimers in pages or components", () => {
    const hits = UI.flatMap((p) => {
      const text = code(p);
      return BANNED.filter((re) => re.test(text)).map((re) => `${p.slice(ROOT.length)}: ${re}`);
    });
    expect(hits).toEqual([]);
  });

  it("keeps Meteora out of the main nav and links it from the footer only", () => {
    expect(src("components/SiteHeader.tsx")).not.toMatch(/meteora/i);
    expect(src("components/SiteFooter.tsx")).toMatch(/href="\/meteora"/);
    expect(src("components/SiteFooter.tsx")).toMatch(/Powered by/);
  });

  it("never claims to be audited", () => {
    const hits = UI.filter((p) => /\b(is|are|fully|been) audited\b/i.test(code(p).replace(/not (yet )?(been )?audited|call itself audited/gi, "")));
    expect(hits).toEqual([]);
  });
});

describe("public listings", () => {
  it("hide Armory's own E2E fixtures by on-chain name, keep real launches", async () => {
    const { isListed } = await import("@/lib/armory/listing");
    expect(isListed("Devnet E2E")).toBe(false);
    expect(isListed("Mintmark Devnet E2E")).toBe(false);
    expect(isListed("Armory Test")).toBe(true);
    expect(isListed("Salt Flats")).toBe(true);
    expect(isListed(null)).toBe(true);
  });
});
