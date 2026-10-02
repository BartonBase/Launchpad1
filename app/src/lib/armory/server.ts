/** Server-only module (imported only from server components). */
/**
 * Server-side reads with a small in-memory TTL cache so pages don't hammer the free public devnet
 * RPC (429s). Failures never crash a page: callers get { ok: false } and render a notice.
 */
import { Connection } from "@solana/web3.js";
import { CLUSTER } from "@/config/cluster";

let conn: Connection | null = null;
export function serverConnection(): Connection {
  conn ??= new Connection(CLUSTER.rpcUrl, { commitment: "confirmed", disableRetryOnRateLimit: true });
  return conn;
}

const cache = new Map<string, { at: number; value: unknown }>();
export type Read<T> = { ok: true; value: T } | { ok: false; error: string };

export async function cachedRead<T>(key: string, ttlMs: number, fn: (c: Connection) => Promise<T>): Promise<Read<T>> {
  const hit = cache.get(key);
  if (hit && Date.now() - hit.at < ttlMs) return { ok: true, value: hit.value as T };
  try {
    const value = await fn(serverConnection());
    cache.set(key, { at: Date.now(), value });
    return { ok: true, value };
  } catch (e) {
    if (hit) return { ok: true, value: hit.value as T }; // serve stale on RPC errors
    return { ok: false, error: e instanceof Error ? e.message : String(e) };
  }
}
