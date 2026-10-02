import { describe, expect, it } from "vitest";
import BN from "bn.js";
import { Keypair, PublicKey } from "@solana/web3.js";
import { getSqrtPriceFromPrice, type PoolConfig, type VirtualPool } from "@meteora-ag/dynamic-bonding-curve-sdk";
import { curveStateFrom } from "@/lib/meteora/dbc";
import { decodeMetadataStrings } from "@/lib/meteora/plain";

const pk = () => Keypair.generate().publicKey;

function mockCurve(reserveLamports: number, migrated: 0 | 1) {
  const config = pk();
  const vp = {
    poolState: {
      config,
      baseMint: pk(),
      creator: pk(),
      quoteReserve: new BN(reserveLamports),
      sqrtPrice: getSqrtPriceFromPrice("0.0000001", 6, 9),
      isMigrated: migrated,
    },
  } as unknown as VirtualPool;
  const cfg = {
    migrationQuoteThreshold: new BN(100_000_000),
    tokenDecimal: 6,
    migrationOption: 1,
    poolFees: { baseFee: { cliffFeeNumerator: new BN(10_000_000) } },
  } as unknown as PoolConfig;
  return { pool: pk(), vp, cfg, config };
}

describe("Meteora DBC curve state", () => {
  it("computes progress toward the migration threshold", () => {
    const { pool, vp, cfg, config } = mockCurve(25_000_000, 0);
    const s = curveStateFrom(pool, vp, cfg);
    expect(s.progressPct).toBe(25);
    expect(s.migrated).toBe(false);
    expect(s.curveComplete).toBe(false);
    expect(s.config).toBe(config.toBase58());
    expect(s.thresholdLamports).toBe("100000000");
    expect(s.migrationOption).toBe(1);
  });

  it("reads the 1% fee from the config (cliffFeeNumerator out of 1e9)", () => {
    const { pool, vp, cfg } = mockCurve(0, 0);
    expect(curveStateFrom(pool, vp, cfg).feeBps).toBe(100);
  });

  it("derives a spot price in SOL per token", () => {
    const { pool, vp, cfg } = mockCurve(0, 0);
    expect(curveStateFrom(pool, vp, cfg).priceSol).toBeCloseTo(0.0000001, 9);
  });

  it("caps progress at 100% and flags a complete curve", () => {
    const { pool, vp, cfg } = mockCurve(150_000_000, 0);
    const s = curveStateFrom(pool, vp, cfg);
    expect(s.progressPct).toBe(100);
    expect(s.curveComplete).toBe(true);
  });

  it("shows 100% once migrated to DAMM v2", () => {
    const { pool, vp, cfg } = mockCurve(10, 1);
    const s = curveStateFrom(pool, vp, cfg);
    expect(s.migrated).toBe(true);
    expect(s.progressPct).toBe(100);
  });
});

describe("Metaplex metadata decode (plain launches)", () => {
  const borshStr = (s: string, pad: number) => {
    const body = new Uint8Array(pad);
    body.set(new TextEncoder().encode(s));
    const len = new Uint8Array(4);
    new DataView(len.buffer).setUint32(0, pad, true);
    return [...len, ...body];
  };

  it("decodes NUL-padded name, symbol and uri", () => {
    const head = new Uint8Array(65);
    head[0] = 4;
    head.set(new PublicKey(new Uint8Array(32).fill(7)).toBytes(), 1);
    const data = Uint8Array.from([...head, ...borshStr("Armory Plain Demo", 32), ...borshStr("APLN", 10), ...borshStr("", 200)]);
    expect(decodeMetadataStrings(data)).toEqual({ name: "Armory Plain Demo", symbol: "APLN", uri: "" });
  });

  it("returns null on truncated data", () => {
    expect(decodeMetadataStrings(new Uint8Array(70))).toBeNull();
  });
});

describe("resolveTokenImage", () => {
  it("keeps self-hosted images and ignores Irys off devnet (unit tests run on localnet)", async () => {
    const { resolveTokenImage } = await import("@/lib/meteora/tokenImage");
    let calls = 0;
    const f = (async () => { calls++; return new Response(JSON.stringify({ image: "https://devnet.irys.xyz/2PZCqmVeYqBUZ4qSmVD3a3ZY88JQbsUvsc22rLA5uEFm" })); }) as typeof fetch;
    expect(await resolveTokenImage("https://armory-ten.vercel.app/tokens/moon-mace.json", f)).toBe("/tokens/moon-mace.png");
    expect(await resolveTokenImage("https://devnet.irys.xyz/2mbJishA5cxP5ALMMZZcGkwfRqWAznNNk1XTWSnUTKv8", f)).toBeNull();
    expect(await resolveTokenImage("https://evil.example/x.json", f)).toBeNull();
    expect(calls).toBe(0);
  });
});

describe("Core asset uri + public listing", () => {
  const str = (s: string) => { const b = new TextEncoder().encode(s); const l = new Uint8Array(4); new DataView(l.buffer).setUint32(0, b.length, true); return [...l, ...b]; };
  it("reads the uri of an AssetV1 with a collection update authority, null for burned assets", async () => {
    const { decodeCoreAssetUri } = await import("@/lib/generated/core");
    const owner = new Uint8Array(32).fill(3);
    const coll = new Uint8Array(32).fill(9);
    const data = Uint8Array.from([1, ...owner, 2, ...coll, ...str("Forge #3"), ...str("https://devnet.irys.xyz/abc"), 0]);
    expect(decodeCoreAssetUri(data)).toBe("https://devnet.irys.xyz/abc");
    const noUa = Uint8Array.from([1, ...owner, 0, ...str("X"), ...str("u")]);
    expect(decodeCoreAssetUri(noUa)).toBe("u");
    expect(decodeCoreAssetUri(Uint8Array.from([0]))).toBeNull();
  });
  it("hides test launches by either Hybrid name or by mint", async () => {
    const { isHybridListed } = await import("@/lib/armory/listing");
    expect(isHybridListed({ mint: pk().toBase58(), tokenName: "Forge Gems", collectionName: "Forge Gems" })).toBe(true);
    expect(isHybridListed({ mint: pk().toBase58(), tokenName: "Armory Test", collectionName: "Knights" })).toBe(false);
    expect(isHybridListed({ mint: "CzDzhYGoCP5BDc2gYmNnwd3s7gW5MWnK8VD7TrbCwVDQ", tokenName: "Armory Hybrid Live", collectionName: "Armory Live Knights" })).toBe(false);
  });
});
