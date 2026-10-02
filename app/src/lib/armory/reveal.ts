/**
 * Randomness reveal (no public crank on devnet: the app runs it). Reads the request's Switchboard
 * randomness account, finds the assigned oracle's gateway, checks the gateway URL against the
 * per-cluster allowlist (same pattern as the CSP connect-src), and POSTs the reveal request itself.
 * No SDK: request body and response handling mirror @switchboard-xyz/on-demand 3.10.6
 * `Gateway.fetchRandomnessReveal` byte for byte (tests/fixtures/switchboard-reveal.json). The
 * signature is verified on-chain by Switchboard inside reveal_randomness, so a bad gateway response
 * can only fail, never pick an NFT.
 */
import { PublicKey, type Connection } from "@solana/web3.js";
import { CLUSTER, type ClusterName } from "@/config/cluster";
import { SWITCHBOARD_GATEWAY_RE } from "@/config/integrations";
import { decodeOracleGatewayUri, decodeRandomness, type RandomnessView } from "@/lib/generated/switchboard";
import type { RevealArgs } from "@/lib/generated/hybridVault";

export const GATEWAY_REVEAL_PATH = "/gateway/api/v1/randomness_reveal";
const MAX_RESPONSE_CHARS = 4096;
const TIMEOUT_MS = 15_000;

export interface RevealRequest {
  readonly randomness: PublicKey;
  readonly seedSlothash: Uint8Array; // 32
  readonly seedSlot: bigint;
  readonly rpc: string;
}

/** JSON body, same key order and encoding as the SDK (slothash as a byte array, key as hex). */
export function revealRequestBody(r: RevealRequest): string {
  if (r.seedSlothash.length !== 32) throw new Error("Malformed seed slothash");
  const hex = Array.from(r.randomness.toBytes(), (b) => b.toString(16).padStart(2, "0")).join("");
  return JSON.stringify({ slothash: Array.from(r.seedSlothash), randomness_key: hex, slot: Number(r.seedSlot), rpc: r.rpc });
}

/** POSTs `body` to an allowlisted gateway and returns the raw response text. */
export type RevealFetcher = (gatewayUrl: string, body: string) => Promise<string>;

export const gatewayRevealFetcher: RevealFetcher = async (gatewayUrl, body) => {
  const res = await fetch(checkGatewayUrl(gatewayUrl) + GATEWAY_REVEAL_PATH, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body,
    credentials: "omit",
    redirect: "error",
    cache: "no-store",
    signal: AbortSignal.timeout(TIMEOUT_MS),
  });
  if (!res.ok) throw new Error(`Switchboard gateway returned HTTP ${res.status}.`);
  const text = await res.text();
  if (text.length > MAX_RESPONSE_CHARS) throw new Error("Switchboard gateway response is too large.");
  return text;
};

export function checkGatewayUrl(url: string, cluster: ClusterName = CLUSTER.name): string {
  const re = SWITCHBOARD_GATEWAY_RE[cluster];
  if (!re || !re.test(url)) throw new Error(`Switchboard gateway ${JSON.stringify(url.slice(0, 80))} is not on the allowlist for ${cluster}.`);
  return url;
}

const BYTE = (x: unknown) => Number.isInteger(x) && (x as number) >= 0 && (x as number) <= 255;
const B64_64 = /^[A-Za-z0-9+/]{86}==$/; // exactly 64 bytes, canonical padding

/**
 * Strict parse of `{ signature: base64(64 bytes), recovery_id: 0..3, value: 32 byte array }`.
 * `value` is a JSON byte array (what the SDK forwards into the [u8; 32] arg and e2e.cjs reads with
 * Buffer.from); anything else, including extra shapes, is rejected.
 */
export function parseRevealResponse(text: string): RevealArgs {
  const bad = () => new Error("Malformed reveal from the Switchboard gateway.");
  let j: unknown;
  try {
    j = JSON.parse(text);
  } catch {
    throw bad();
  }
  if (!j || typeof j !== "object" || Array.isArray(j)) throw bad();
  const { signature, recovery_id: recoveryId, value } = j as Record<string, unknown>;
  if (typeof signature !== "string" || !B64_64.test(signature)) throw bad();
  if (!Number.isInteger(recoveryId) || (recoveryId as number) < 0 || (recoveryId as number) > 3) throw bad();
  if (!Array.isArray(value) || value.length !== 32 || !value.every(BYTE)) throw bad();
  const sig = Uint8Array.from(globalThis.atob(signature), (c) => c.charCodeAt(0));
  if (sig.length !== 64) throw bad();
  return { signature: sig, recoveryId: recoveryId as number, value: Uint8Array.from(value as number[]) };
}

export interface PreparedReveal {
  readonly randomness: RandomnessView;
  readonly gatewayUrl: string;
  readonly args: RevealArgs;
}

export async function fetchReveal(conn: Connection, randomness: PublicKey, fetcher: RevealFetcher = gatewayRevealFetcher): Promise<PreparedReveal> {
  const ri = await conn.getAccountInfo(randomness);
  if (!ri) throw new Error("Randomness account not found.");
  const rd = decodeRandomness(new Uint8Array(ri.data));
  if (rd.oracle.equals(PublicKey.default)) throw new Error("Randomness has no assigned oracle yet (not committed).");
  if (rd.revealSlot > rd.seedSlot) throw new Error("Randomness was already revealed; settle next.");
  const oi = await conn.getAccountInfo(rd.oracle);
  if (!oi) throw new Error("Assigned Switchboard oracle not found.");
  const gatewayUrl = checkGatewayUrl(decodeOracleGatewayUri(new Uint8Array(oi.data)));
  const body = revealRequestBody({ randomness, seedSlothash: rd.seedSlothash, seedSlot: rd.seedSlot, rpc: CLUSTER.rpcUrl });
  return { randomness: rd, gatewayUrl, args: parseRevealResponse(await fetcher(gatewayUrl, body)) };
}
