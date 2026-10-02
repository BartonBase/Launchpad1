/**
 * RPC transport with bounded retries for rate limits (HTTP 429) and transient gateway errors (502/503/504).
 *
 * Why: the free public endpoints (api.mainnet-beta.solana.com, api.devnet.solana.com) rate-limit hard (~100
 * requests / 10 s per IP and lower for heavy calls such as getProgramAccounts). Without a private RPC key the app
 * still works, just slower: each request is retried up to MAX_RETRIES times with exponential backoff + jitter,
 * honouring Retry-After (capped). web3.js's own 429 loop is disabled (disableRetryOnRateLimit) so retries don't
 * stack. A private RPC (Helius free tier, NEXT_PUBLIC_SOLANA_RPC_URL / SOLANA_RPC_URL) removes most 429s.
 */
import { Connection, type Commitment } from "@solana/web3.js";

export const MAX_RETRIES = 4;
const RETRY_STATUS = new Set([429, 502, 503, 504]);
const MAX_DELAY_MS = 8_000;

export interface RetryOptions {
  readonly retries?: number;
  readonly baseDelayMs?: number;
  readonly sleep?: (ms: number) => Promise<void>;
  readonly random?: () => number;
}

/** Delay before retry `attempt` (0-based): Retry-After if the server sent one, else 2^attempt backoff with jitter. */
export function retryDelayMs(attempt: number, retryAfter: string | null, baseDelayMs = 500, random: () => number = Math.random): number {
  if (retryAfter) {
    const secs = Number(retryAfter);
    if (Number.isFinite(secs) && secs >= 0) return Math.min(MAX_DELAY_MS, Math.round(secs * 1000));
    const at = Date.parse(retryAfter);
    if (Number.isFinite(at)) return Math.min(MAX_DELAY_MS, Math.max(0, at - Date.now()));
  }
  const exp = baseDelayMs * 2 ** attempt;
  return Math.min(MAX_DELAY_MS, Math.round(exp / 2 + random() * (exp / 2)));
}

/** fetch() wrapper used as the Connection transport. */
export function withRetries(inner: typeof fetch = fetch, o: RetryOptions = {}): typeof fetch {
  const retries = o.retries ?? MAX_RETRIES;
  const sleep = o.sleep ?? ((ms: number) => new Promise<void>((r) => setTimeout(r, ms)));
  return (async (input: Parameters<typeof fetch>[0], init?: Parameters<typeof fetch>[1]) => {
    for (let attempt = 0; ; attempt++) {
      let res: Response;
      try {
        res = await inner(input, init);
      } catch (e) {
        if (attempt >= retries) throw e; // network error: retry too
        await sleep(retryDelayMs(attempt, null, o.baseDelayMs, o.random));
        continue;
      }
      if (!RETRY_STATUS.has(res.status) || attempt >= retries) return res;
      await sleep(retryDelayMs(attempt, res.headers.get("retry-after"), o.baseDelayMs, o.random));
    }
  }) as typeof fetch;
}

/** Connection config shared by the browser provider and server reads. */
export function connectionConfig(wsEndpoint?: string, commitment: Commitment = "confirmed") {
  return { commitment, wsEndpoint, disableRetryOnRateLimit: true, fetch: withRetries() } as const;
}

export function makeConnection(url: string, wsEndpoint?: string): Connection {
  return new Connection(url, connectionConfig(wsEndpoint));
}
