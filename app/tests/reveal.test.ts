/**
 * Our reveal code vs. a fixture captured from @switchboard-xyz/on-demand 3.10.6 (request body,
 * path, headers, parsed response) and the e2e.cjs Anchor-built reveal_randomness instruction.
 */
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PublicKey } from "@solana/web3.js";
import { SWITCHBOARD_PROGRAM_ID } from "@/config/programs";
import { parseRevealResponse, revealRequestBody } from "@/lib/armory/reveal";
import { revealRandomnessIx } from "@/lib/generated/hybridVault";
import { oracleStatsPda } from "@/lib/generated/switchboard";

const F = JSON.parse(readFileSync(new URL("./fixtures/switchboard-reveal.json", import.meta.url), "utf8"));
const I = F.input;
const key = (k: string) => new PublicKey(I[k]);
const hex = (h: string) => Uint8Array.from(Buffer.from(h, "hex"));
const req = { randomness: key("randomness"), seedSlothash: hex(I.seedSlothash), seedSlot: BigInt(I.seedSlot), rpc: I.rpc };
const GW = "https://141.95.35.110.xip.switchboard-oracles.xyz/devnet";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.unstubAllEnvs();
  vi.resetModules();
});

describe("Switchboard reveal without the SDK", () => {
  it("request body is byte-identical to the SDK's", () => {
    expect(revealRequestBody(req)).toBe(F.gatewayRequest.body);
  });

  it("fetcher POSTs the same path, method and content type, only to allowlisted gateways", async () => {
    vi.stubEnv("NEXT_PUBLIC_SOLANA_CLUSTER", "devnet");
    const calls: [string, RequestInit][] = [];
    vi.stubGlobal("fetch", async (u: string, init: RequestInit) => {
      calls.push([u, init]);
      return new Response(F.gatewayResponseText, { status: 200 });
    });
    const { gatewayRevealFetcher } = await import("@/lib/armory/reveal");
    expect(await gatewayRevealFetcher(GW, F.gatewayRequest.body)).toBe(F.gatewayResponseText);
    const [u, init] = calls[0]!;
    expect(u).toBe(GW + F.gatewayRequest.path);
    expect(init.method).toBe(F.gatewayRequest.method);
    expect((init.headers as Record<string, string>)["Content-Type"]).toBe(F.gatewayRequest.contentType);
    expect(init.body).toBe(F.gatewayRequest.body);
    expect(init.redirect).toBe("error");
    await expect(gatewayRevealFetcher("https://evil.example/devnet", "{}")).rejects.toThrow(/allowlist/);
    expect(calls).toHaveLength(1);
  });

  it("parses the response exactly like the SDK and rejects anything else", () => {
    const a = parseRevealResponse(F.gatewayResponseText);
    expect(Buffer.from(a.signature).toString("base64")).toBe(F.sdkParsed.signature);
    expect(a.recoveryId).toBe(F.sdkParsed.recovery_id);
    expect([...a.value]).toEqual(F.sdkParsed.value);
    const ok = JSON.parse(F.gatewayResponseText);
    for (const bad of [
      "not json", "null", "[]",
      { ...ok, signature: ok.signature.slice(4) },
      { ...ok, signature: 123 },
      { ...ok, recovery_id: 4 },
      { ...ok, recovery_id: "1" },
      { ...ok, value: ok.value.slice(1) },
      { ...ok, value: [...ok.value.slice(1), 256] },
      { ...ok, value: Buffer.from(ok.value).toString("hex") },
    ]) expect(() => parseRevealResponse(typeof bad === "string" ? bad : JSON.stringify(bad))).toThrow(/Malformed/);
  });

  it("reveal_randomness instruction is byte-identical to the e2e.cjs/Anchor build", () => {
    const ix = revealRandomnessIx({
      payer: key("payer"), vault: key("vault"), request: key("request"), randomness: key("randomness"),
      sbOracle: key("oracle"), sbQueue: key("queue"), sbStats: oracleStatsPda(key("oracle"), SWITCHBOARD_PROGRAM_ID),
      sbProgramState: PublicKey.findProgramAddressSync([Buffer.from("STATE")], SWITCHBOARD_PROGRAM_ID)[0],
      args: parseRevealResponse(F.gatewayResponseText),
    });
    expect(ix.programId.toBase58()).toBe(F.ix.programId);
    expect(ix.keys.map((k) => ({ pubkey: k.pubkey.toBase58(), isSigner: k.isSigner, isWritable: k.isWritable }))).toEqual(F.ix.keys);
    expect(Buffer.from(ix.data).toString("hex")).toBe(F.ix.dataHex);
  });
});
