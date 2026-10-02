import { describe, expect, it, vi } from "vitest";
import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { BaseFeeMode, getFeeSchedulerParams } from "@meteora-ag/dynamic-bonding-curve-sdk";
import { retryDelayMs, withRetries } from "@/lib/rpc";
import { antiSnipe, configProblems, steadyFeeBps, PLANNED_MAINNET_TERMS } from "@/lib/meteora/terms";
import { CURVE_FEE_BPS, INCINERATOR, MAINNET_FEE_CLAIMER, launchDisclosure } from "@/config/launchTerms";
import { launchTypeStatus } from "@/config/armory";
import { armoryProgramsEnabled, getProgramRegistry, DBC_PROGRAM_ID, DAMM_V2_PROGRAM_ID, HYBRID_LAUNCH_PROGRAM_ID, SWITCHBOARD_PROGRAM_ID, WRAPPED_SOL_MINT } from "@/config/programs";
import { parseEnvAddress } from "@/config/integrations";
import { ClusterConfigError } from "@/config/cluster";
import { MAX_FREE_BYTES, TURBO_UPLOAD_URL, UploadError, buildMetadataJson, uploadToArweave, uploadTokenMetadata } from "@/lib/metadata/arweave";
import { curveFeeRows, totalLamports, type PoolFeeFields } from "@/lib/meteora/fees";

const res = (status: number, headers: Record<string, string> = {}) => new Response("{}", { status, headers });

describe("RPC retries (429 handling)", () => {
  it("retries 429/503 then returns the success, honouring Retry-After", async () => {
    const inner = vi.fn<typeof fetch>().mockResolvedValueOnce(res(429, { "retry-after": "2" })).mockResolvedValueOnce(res(503)).mockResolvedValueOnce(res(200));
    const sleeps: number[] = [];
    const f = withRetries(inner, { sleep: async (ms) => void sleeps.push(ms), random: () => 0.5 });
    const r = await f("https://rpc.example.com", { method: "POST" });
    expect(r.status).toBe(200);
    expect(inner).toHaveBeenCalledTimes(3);
    expect(sleeps[0]).toBe(2000); // Retry-After: 2
    expect(sleeps[1]).toBeGreaterThan(0);
  });

  it("gives up after the retry budget and returns the last 429", async () => {
    const inner = vi.fn<typeof fetch>().mockResolvedValue(res(429));
    const f = withRetries(inner, { retries: 2, sleep: async () => {} });
    expect((await f("x")).status).toBe(429);
    expect(inner).toHaveBeenCalledTimes(3);
  });

  it("does not retry other errors", async () => {
    const inner = vi.fn<typeof fetch>().mockResolvedValue(res(400));
    expect((await withRetries(inner, { sleep: async () => {} })("x")).status).toBe(400);
    expect(inner).toHaveBeenCalledTimes(1);
  });

  it("retries network errors, and caps delays at 8 s", async () => {
    const inner = vi.fn<typeof fetch>().mockRejectedValueOnce(new TypeError("fetch failed")).mockResolvedValueOnce(res(200));
    expect((await withRetries(inner, { sleep: async () => {} })("x")).status).toBe(200);
    expect(retryDelayMs(0, "600")).toBe(8000);
    expect(retryDelayMs(10, null, 500, () => 1)).toBe(8000);
  });
});

describe("mainnet launch terms", () => {
  const fee = getFeeSchedulerParams(5000, 200, BaseFeeMode.FeeSchedulerExponential, 12, 60);
  const baseFee = { ...fee, baseFeeMode: BaseFeeMode.FeeSchedulerExponential, firstFactor: fee.firstFactor, secondFactor: fee.secondFactor, thirdFactor: fee.thirdFactor } as never;

  it("reads the steady 2% fee and the 50% -> 2% anti-snipe over 60 s from the fee schedule", () => {
    expect(steadyFeeBps(baseFee)).toBeGreaterThanOrEqual(199);
    expect(steadyFeeBps(baseFee)).toBeLessThanOrEqual(201);
    expect(antiSnipe(baseFee, 1)).toEqual({ startBps: 5000, seconds: 60 });
  });

  it("discloses cost, 2% fee, the 50/50 split, anti-snipe and graduation in plain English", () => {
    const text = launchDisclosure(PLANNED_MAINNET_TERMS).join("\n");
    expect(text).toContain("0.0206 SOL");
    expect(text).toContain("2% fee");
    expect(text).toContain("Meteora keeps 20%");
    expect(text).toContain("50/50");
    expect(text).toContain("about 0.8% of curve volume");
    expect(text).toContain("first 60 seconds");
    expect(text).toContain("50%");
    expect(text).toContain("40 SOL");
    expect(CURVE_FEE_BPS["mainnet-beta"]).toBe(200);
  });

  it("refuses a config that pays someone else or doesn't quote in SOL", () => {
    const ok = { feeClaimer: MAINNET_FEE_CLAIMER, quoteMint: WRAPPED_SOL_MINT };
    expect(configProblems(ok, "mainnet-beta")).toEqual([]);
    expect(configProblems({ ...ok, feeClaimer: INCINERATOR }, "mainnet-beta")).toHaveLength(1);
    expect(configProblems({ ...ok, feeClaimer: INCINERATOR }, "devnet")).toEqual([]);
    expect(configProblems({ ...ok, quoteMint: INCINERATOR }, "devnet")).toHaveLength(1);
  });
});

describe("mainnet modes and programs", () => {
  it("Launch is live on mainnet; Hybrid is coming soon there and still live on devnet", () => {
    expect(launchTypeStatus("plain", "mainnet-beta")).toBe("live");
    expect(launchTypeStatus("hybrid", "mainnet-beta")).toBe("coming-soon");
    expect(launchTypeStatus("hybrid", "devnet")).toBe("live");
  });

  it("the mainnet program allowlist has Meteora but no Armory or Switchboard program", () => {
    const ids = getProgramRegistry("mainnet-beta").map((p) => p.id.toBase58());
    expect(ids).toContain(DBC_PROGRAM_ID.toBase58());
    expect(ids).toContain(DAMM_V2_PROGRAM_ID.toBase58());
    expect(ids).not.toContain(HYBRID_LAUNCH_PROGRAM_ID.toBase58());
    expect(ids).not.toContain(SWITCHBOARD_PROGRAM_ID.toBase58());
    expect(armoryProgramsEnabled("mainnet-beta")).toBe(false);
    expect(armoryProgramsEnabled("devnet")).toBe(true);
  });

  it("parseEnvAddress: unset is null, valid parses, garbage fails the build", () => {
    expect(parseEnvAddress(undefined, "X")).toBeNull();
    expect(parseEnvAddress("  ", "X")).toBeNull();
    expect(parseEnvAddress(MAINNET_FEE_CLAIMER.toBase58(), "X")?.equals(MAINNET_FEE_CLAIMER)).toBe(true);
    expect(() => parseEnvAddress("not-a-key", "X")).toThrow(ClusterConfigError);
  });
});

describe("Arweave metadata upload (mocked; never hits the network)", () => {
  const id = "A".repeat(43);
  it("builds Metaplex-style JSON", () => {
    expect(JSON.parse(buildMetadataJson({ name: " Dog ", symbol: "DOG", description: "", image: "https://arweave.net/x" }))).toEqual({ name: "Dog", symbol: "DOG", image: "https://arweave.net/x" });
  });

  it("posts raw bytes with tags and returns the gateway URL", async () => {
    const f = vi.fn<typeof fetch>().mockResolvedValue(new Response(JSON.stringify({ id }), { status: 200 }));
    const url = await uploadToArweave(new Uint8Array([1, 2, 3]), "image/webp", f);
    expect(url).toBe(`https://arweave.net/${id}`);
    const [u, init] = f.mock.calls[0]!;
    expect(u).toBe(TURBO_UPLOAD_URL);
    expect(init?.method).toBe("POST");
    const tags = JSON.parse((init?.headers as Record<string, string>)["x-data-item-tags"]!);
    expect(tags).toContainEqual({ name: "Content-Type", value: "image/webp" });
  });

  it("uploads the image first, then JSON pointing at it", async () => {
    const f = vi.fn<typeof fetch>()
      .mockResolvedValueOnce(new Response(JSON.stringify({ id: "I".repeat(43) })))
      .mockResolvedValueOnce(new Response(JSON.stringify({ id: "J".repeat(43) })));
    const r = await uploadTokenMetadata({ name: "Dog", symbol: "DOG", imageBytes: new Uint8Array([9]) }, f);
    expect(r.uri).toBe(`https://arweave.net/${"J".repeat(43)}`);
    const jsonBody = new TextDecoder().decode(f.mock.calls[1]![1]!.body as Uint8Array);
    expect(JSON.parse(jsonBody).image).toBe(`https://arweave.net/${"I".repeat(43)}`);
  });

  it("rejects oversize files, HTTP errors and bad responses", async () => {
    await expect(uploadToArweave(new Uint8Array(MAX_FREE_BYTES + 1), "image/webp", vi.fn())).rejects.toThrow(UploadError);
    await expect(uploadToArweave(new Uint8Array([1]), "x", vi.fn<typeof fetch>().mockResolvedValue(res(500)))).rejects.toThrow(/HTTP 500/);
    await expect(uploadToArweave(new Uint8Array([1]), "x", vi.fn<typeof fetch>().mockResolvedValue(new Response('{"id":"short"}')))).rejects.toThrow(UploadError);
  });
});

describe("fee claim rows", () => {
  const cfg = new PublicKey("DuQYHUCToW6uHkWngXFiU4uGVSjcVKKCTJwViEb87Em9");
  const pool = (k: number, over: Partial<PoolFeeFields> = {}) => ({
    publicKey: new PublicKey(new Uint8Array(32).fill(k)),
    account: { poolState: { config: cfg, baseMint: new PublicKey(new Uint8Array(32).fill(k + 100)), isMigrated: 0, isPartnerWithdrawSurplus: 0, creatorQuoteFee: new BN(5), creatorBaseFee: new BN(0), partnerQuoteFee: new BN(7), partnerBaseFee: new BN(0), ...over } },
  });
  it("picks the creator or partner share, filters by config and sorts by amount", () => {
    const rows = curveFeeRows([pool(1), pool(2, { creatorQuoteFee: new BN(50) }), pool(3, { config: INCINERATOR })], "creator", cfg);
    expect(rows.map((r) => r.unclaimedLamports)).toEqual([50n, 5n]);
    expect(totalLamports(rows)).toBe(55n);
    const p = curveFeeRows([pool(1, { isMigrated: 1 })], "partner", null);
    expect(p[0]).toMatchObject({ unclaimedLamports: 7n, migrated: true, surplusPending: true });
  });
});
