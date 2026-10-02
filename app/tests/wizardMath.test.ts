/** Wizard live math vs design/directions/a-obsidian/NOTE.md "Wizard math" and launch.html render(). */
import { describe, expect, it } from "vitest";
import { MIN_GRADUATION_LAMPORTS } from "@/config/armory";
import { designSol, wizardMath, EXAMPLE_SOL_USD } from "@/lib/armory/wizardMath";
import { validateLaunchForm } from "@/lib/armory/launchForm";

const SOL = 1_000_000_000n;
const base = { type: "hybrid" as const, ratio: 1_000_000, size: 500, targetLamports: 85n * SOL, chainMinLamports: MIN_GRADUATION_LAMPORTS.devnet };

describe("wizard math (NOTE.md)", () => {
  it("default: 500 NFTs at 1M = 500M (50%) in NFT form, 500M always tokens", () => {
    const m = wizardMath(base);
    expect(m.valid).toBe(true);
    expect(m.nftTokens).toBe(500_000_000n);
    expect(m.nftPct).toBe(50);
    expect(m.restTokens).toBe(500_000_000n);
    expect(m.maxSize).toBe(1000);
    expect(m.feeLamports).toBe(10_000_000n);
  });
  it("max = min(10,000, 1B / ratio) for every ratio", () => {
    const want: Record<number, number> = { 50_000: 10_000, 100_000: 10_000, 200_000: 5_000, 500_000: 2_000, 1_000_000: 1_000, 2_500_000: 400, 5_000_000: 200 };
    for (const [r, max] of Object.entries(want)) expect(wizardMath({ ...base, ratio: Number(r) }).maxSize).toBe(max);
  });
  it("size bounds: 100 <= size <= max (scripted cases from NOTE: 5M@99/100/200, 2.5M@400/401)", () => {
    expect(wizardMath({ ...base, ratio: 5_000_000, size: 99 }).sizeError).toBe("low");
    expect(wizardMath({ ...base, ratio: 5_000_000, size: 100 }).sizeOk).toBe(true);
    expect(wizardMath({ ...base, ratio: 5_000_000, size: 200 }).sizeOk).toBe(true);
    expect(wizardMath({ ...base, ratio: 5_000_000, size: 200 }).nftPct).toBe(100);
    expect(wizardMath({ ...base, ratio: 2_500_000, size: 400 }).sizeOk).toBe(true);
    expect(wizardMath({ ...base, ratio: 2_500_000, size: 401 }).sizeError).toBe("high");
    expect(wizardMath({ ...base, size: 500.5 }).sizeOk).toBe(false);
    expect(wizardMath({ ...base, size: Number.NaN }).sizeError).toBe("low");
  });
  it("graduation target: the cluster's chain minimum is the floor (devnet 0.1 SOL, production 10 SOL)", () => {
    expect(wizardMath({ ...base, targetLamports: 85n * SOL }).targetOk).toBe(true);
    expect(wizardMath({ ...base, targetLamports: SOL / 10n }).targetOk).toBe(true);
    expect(wizardMath({ ...base, targetLamports: SOL / 10n - 1n }).targetOk).toBe(false);
    expect(wizardMath({ ...base, chainMinLamports: 10n * SOL, targetLamports: 9n * SOL }).minTargetLamports).toBe(10n * SOL);
    expect(wizardMath({ ...base, targetLamports: null }).valid).toBe(false);
  });
  it("plain: only T matters; no NFT side, no per-NFT fee", () => {
    const m = wizardMath({ ...base, type: "plain", size: 0 });
    expect(m.valid).toBe(true);
    expect(m.nftTokens).toBe(0n);
    expect(m.restTokens).toBe(1_000_000_000n);
    expect(m.feeLamports).toBeNull();
    expect(wizardMath({ ...base, type: "plain", targetLamports: MIN_GRADUATION_LAMPORTS.devnet - 1n }).valid).toBe(false);
    expect(wizardMath({ ...base, type: "plain", targetLamports: 10n * SOL }).valid).toBe(true);
  });
  it("burn: same validity as hybrid; most that can burn = size x ratio, supply floor = 1B - that", () => {
    const m = wizardMath({ ...base, type: "burn", ratio: 1_000_000, size: 500 });
    expect(m.valid).toBe(true);
    expect(m.nftTokens).toBe(500_000_000n);
    expect(m.restTokens).toBe(500_000_000n);
    expect(wizardMath({ ...base, type: "burn", size: 1001 }).valid).toBe(false);
  });
  it("fee tiers per ratio (0.002 / 0.005 / 0.01)", () => {
    expect(wizardMath({ ...base, ratio: 50_000 }).feeLamports).toBe(2_000_000n);
    expect(wizardMath({ ...base, ratio: 100_000 }).feeLamports).toBe(5_000_000n);
    expect(wizardMath({ ...base, ratio: 200_000 }).feeLamports).toBe(5_000_000n);
    expect(wizardMath({ ...base, ratio: 500_000 }).feeLamports).toBe(10_000_000n);
  });
  it("NFT price estimate = ratio x example price (design: 1M -> 0.028 / 0.570 SOL, $4.20 / $85.50)", () => {
    const m = wizardMath(base);
    expect(designSol(m.nftPriceLaunchSol)).toBe("0.028");
    expect(designSol(m.nftPriceGraduationSol)).toBe("0.570");
    expect((m.nftPriceLaunchSol * EXAMPLE_SOL_USD).toFixed(2)).toBe("4.20");
    expect((m.nftPriceGraduationSol * EXAMPLE_SOL_USD).toFixed(2)).toBe("85.50");
    expect(designSol(wizardMath({ ...base, ratio: 50_000 }).nftPriceLaunchSol)).toBe("0.0014");
    expect(designSol(wizardMath({ ...base, ratio: 5_000_000 }).nftPriceGraduationSol)).toBe("2.85");
  });
});

describe("launch form (type-aware)", () => {
  const f = { type: "hybrid" as const, name: "Low Orbit", symbol: "ORBIT", ratio: 1_000_000, collectionSize: "500", graduationSol: "85" };
  it("hybrid builds launch params", () => {
    const r = validateLaunchForm(f, MIN_GRADUATION_LAMPORTS.devnet);
    expect(r.errors).toEqual({});
    expect(r.params).toMatchObject({ ratioWholeTokens: 1_000_000n, collectionSize: 500n, graduationThresholdLamports: 85n * SOL });
  });
  it("plain and burn validate but never produce params (pending deploy: no builders)", () => {
    expect(validateLaunchForm({ ...f, type: "plain" }, 0n)).toMatchObject({ errors: {}, params: null });
    expect(validateLaunchForm({ ...f, type: "burn" }, 0n)).toMatchObject({ errors: {}, params: null });
  });
  it("ticker 3-6 letters, size with thousands separators", () => {
    expect(validateLaunchForm({ ...f, symbol: "AB" }, 0n).errors.symbol).toBeDefined();
    expect(validateLaunchForm({ ...f, symbol: "ABCDEFG" }, 0n).errors.symbol).toBeDefined();
    expect(validateLaunchForm({ ...f, ratio: 50_000, collectionSize: "10,000" }, 0n).errors).toEqual({});
  });
});

import { tokenPanels } from "@/lib/armory/tokenPanels";
describe("token panels by type (sitemap.md matrix)", () => {
  it("plain: no NFT panels, plain facts", () => {
    const p = tokenPanels("plain", "curve");
    for (const x of ["market", "curveCard", "tradePanel", "plainFacts", "holdings", "feesControl", "nobodyCanChange", "facts"] as const) expect(p.has(x)).toBe(true);
    for (const x of ["convertCapture", "convertRelease", "burnToMint", "yourNfts", "reroll", "tradeNfts", "floor", "collectionPreview"] as const) expect(p.has(x)).toBe(false);
  });
  it("hybrid: capture + release + your NFTs + re-roll + floor + trade NFTs; preview only pre-graduation", () => {
    const g = tokenPanels("hybrid", "graduated");
    for (const x of ["convertCapture", "convertRelease", "yourNfts", "reroll", "floor", "tradeNfts", "graduatedCard"] as const) expect(g.has(x)).toBe(true);
    expect(g.has("collectionPreview")).toBe(false);
    expect(g.has("burnToMint")).toBe(false);
    expect(tokenPanels("hybrid", "curve").has("collectionPreview")).toBe(true);
  });
  it("burn: burn-to-mint, order, supply/burned, your NFTs without re-roll, no floor", () => {
    const b = tokenPanels("burn", "graduated");
    for (const x of ["burnToMint", "collectionOrder", "supplyBurned", "yourNfts", "tradeNfts"] as const) expect(b.has(x)).toBe(true);
    for (const x of ["reroll", "floor", "convertCapture", "convertRelease"] as const) expect(b.has(x)).toBe(false);
  });
});
