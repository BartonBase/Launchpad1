import { describe, expect, it } from "vitest";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  MINT_ESCROW_LAMPORTS,
  RATIOS,
  isExactTierFee,
  launchTypeStatus,
  tierFeeLamports,
} from "@/config/armory";
import { checkBalance, requestCost } from "@/lib/armory/fees";
import { compact, formatSol, formatUnits, solToLamports } from "@/lib/armory/format";
import bs58Encode from "@/lib/armory/bs58";
import { hybridLaunchIx, hybridVaultErrors, hybridVaultIx } from "@/lib/generated/idlMeta";
import { decodeLaunchConfig, launchConfigPda, launchIx } from "@/lib/generated/hybridLaunch";
import { requestPda, vaultPda } from "@/lib/generated/hybridVault";
import launchIdl from "@/lib/generated/idl/hybrid_launch.json";
import vaultIdl from "@/lib/generated/idl/hybrid_vault.json";

describe("devnet IDLs", () => {
  it("are the devnet IDLs (no undeployed modes) and match the pinned program ids", () => {
    expect(launchIdl.address).toBe("9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8");
    expect(vaultIdl.address).toBe("BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG");
    expect(launchIdl.instructions.map((i) => i.name).sort()).toEqual(["launch", "register_dbc_launch"]);
    expect(Object.keys(hybridLaunchIx).sort()).toEqual(["launch", "registerDbcLaunch"]);
    expect(vaultIdl.instructions).toHaveLength(13);
  });
  it("devnet vault has no FeeNotTier (6061) error", () => {
    expect(hybridVaultErrors[6061]).toBeUndefined();
    expect(Object.values(hybridVaultErrors).some((e) => e.name === "FeeNotTier")).toBe(false);
  });
  it("generated discriminators match the IDL", () => {
    const d = (n: string) => vaultIdl.instructions.find((i) => i.name === n)!.discriminator;
    expect([...hybridVaultIx.requestCapture]).toEqual(d("request_capture"));
    expect([...hybridVaultIx.unwrap]).toEqual(d("unwrap"));
  });
});

describe("tier fees and exact-tier check (client-side; devnet has no 6061)", () => {
  it("tiers", () => {
    expect(tierFeeLamports(50_000)).toBe(2_000_000n);
    expect(tierFeeLamports(200_000)).toBe(5_000_000n);
    expect(tierFeeLamports(5_000_000)).toBe(10_000_000n);
    expect(() => tierFeeLamports(10_000)).toThrow();
  });
  it("exact tier only", () => {
    for (const r of RATIOS) expect(isExactTierFee(r, tierFeeLamports(r))).toBe(true);
    expect(isExactTierFee(1_000_000, 5_000_000n)).toBe(false); // under-tier
    expect(isExactTierFee(50_000, 5_000_000n)).toBe(false); // over-tier
    expect(isExactTierFee(10_000, 2_000_000n)).toBe(false); // unsupported ratio
    expect(isExactTierFee(1_000_000n, 10_000_000n)).toBe(true);
  });
});

describe("request cost (lazy-mint deposit)", () => {
  const c = requestCost({ tierFeeLamports: 10_000_000n, tempRentLamports: 3_000_000n });
  it("adds fee + refundable deposit + rent + network fee", () => {
    expect(c.deposit).toBe(MINT_ESCROW_LAMPORTS);
    expect(c.deposit).toBe(6_338_100n);
    expect(c.requiredBalance).toBe(10_000_000n + 6_338_100n + 3_000_000n + 5_000n);
    expect(c.minNetCost).toBe(10_005_000n);
  });
  it("flags insufficient balance with the shortfall", () => {
    expect(checkBalance(c.requiredBalance, c)).toEqual({ ok: true });
    expect(checkBalance(c.requiredBalance - 1n, c)).toEqual({ ok: false, shortfall: 1n });
    // Enough for the fee but not the deposit:
    expect(checkBalance(11_000_000n, c).ok).toBe(false);
  });
});

describe("launch types", () => {
  it("plain + burn pending deploy, tax + raffle coming soon, hybrid live", () => {
    expect(launchTypeStatus("hybrid", "devnet")).toBe("live");
    expect(launchTypeStatus("plain", "devnet")).toBe("pending-deploy");
    expect(launchTypeStatus("burn", "devnet")).toBe("pending-deploy");
    expect(launchTypeStatus("tax", "devnet")).toBe("coming-soon");
    expect(launchTypeStatus("raffle", "devnet")).toBe("coming-soon");
  });
});

describe("clients", () => {
  it("launch ix: discriminator + params + accounts match the devnet IDL", () => {
    const creator = Keypair.generate().publicKey;
    const mint = Keypair.generate().publicKey;
    const ix = launchIx(creator, mint, { decimals: 6, ratioWholeTokens: 1_000_000n, collectionSize: 100n, graduationThresholdLamports: 100_000_000n });
    expect([...ix.data.subarray(0, 8)]).toEqual([...hybridLaunchIx.launch]);
    expect(ix.data.length).toBe(8 + 1 + 8 * 3);
    expect(ix.keys).toHaveLength(launchIdl.instructions.find((i) => i.name === "launch")!.accounts.length);
    expect(ix.keys[2]!.pubkey.equals(launchConfigPda(mint))).toBe(true);
  });
  it("PDAs are deterministic", () => {
    const lc = new PublicKey("11111111111111111111111111111112");
    expect(requestPda(vaultPda(lc), 0n).equals(requestPda(vaultPda(lc), 0n))).toBe(true);
    expect(requestPda(vaultPda(lc), 0n).equals(requestPda(vaultPda(lc), 1n))).toBe(false);
  });
  it("rejects non-LaunchConfig data", () => {
    expect(() => decodeLaunchConfig(new Uint8Array(300))).toThrow();
  });
});

describe("format", () => {
  it("formats", () => {
    expect(formatSol(6_338_100n)).toBe("0.0063381 SOL");
    expect(formatUnits(1_000_000_000_000n, 6)).toBe("1,000,000");
    expect(compact(2_500_000)).toBe("2.5M");
    expect(solToLamports("0.1")).toBe(100_000_000n);
    expect(solToLamports("1.0000000001")).toBeNull();
    expect(bs58Encode(new Uint8Array([0, 0, 1]))).toBe("112");
  });
});

describe("rent estimate", () => {
  it("matches the rent-exempt formula (Request + RandLock)", async () => {
    const { estimateRentExempt } = await import("@/lib/armory/fees");
    expect(estimateRentExempt(0)).toBe(890_880n); // known 0-data minimum
  });
});
