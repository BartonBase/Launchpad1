"use client";
/** Small client read hooks around the wallet-adapter Connection. Refetch by bumping `nonce`. */
import { useEffect, useState } from "react";
import { useConnection } from "@solana/wallet-adapter-react";
import type { Connection } from "@solana/web3.js";

export interface Async<T> {
  readonly data: T | undefined;
  readonly error: string | null;
  readonly loading: boolean;
}

interface Settled<T> {
  readonly id: string;
  readonly data: T | undefined;
  readonly error: string | null;
}

const RATE_LIMITED = /\b429\b|rate limit|Too Many Requests|fetch failed|Failed to fetch/i;

/** Read errors as plain words (the public devnet RPC rate-limits bursts; raw JSON-RPC bodies are not shown). */
export function friendlyReadError(e: unknown): string {
  const m = e instanceof Error ? e.message : String(e);
  if (RATE_LIMITED.test(m)) return "The public devnet RPC is busy right now. Wait a few seconds and try again.";
  return m.length > 160 ? `${m.slice(0, 157)}…` : m;
}

async function withRetry<T>(fn: () => Promise<T>): Promise<T> {
  for (let n = 0; ; n++) {
    try {
      return await fn();
    } catch (e) {
      if (n >= 2 || !RATE_LIMITED.test(e instanceof Error ? e.message : String(e))) throw e;
      await new Promise((r) => setTimeout(r, 1500 * 2 ** n));
    }
  }
}

/**
 * `key` encodes every input of `fn` (null = disabled). Results are tagged with the request id, so
 * stale responses are ignored and `loading` is derived instead of set synchronously in the effect.
 */
export function useChainRead<T>(key: string | null, fn: (c: Connection) => Promise<T>, nonce = 0): Async<T> {
  const { connection } = useConnection();
  const id = key === null ? null : `${key}#${nonce}`;
  const [settled, setSettled] = useState<Settled<T> | null>(null);
  useEffect(() => {
    if (id === null) return;
    let live = true;
    withRetry(() => fn(connection)).then(
      (data) => live && setSettled({ id, data, error: null }),
      (e: unknown) => live && setSettled({ id, data: undefined, error: friendlyReadError(e) }),
    );
    return () => {
      live = false;
    };
    // fn is keyed by `key` by contract.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id, connection]);
  if (id === null) return { data: undefined, error: null, loading: false };
  const fresh = settled?.id === id;
  // Keep showing the previous value for the same key while a refetch is in flight.
  const sameKey = settled !== null && key !== null && settled.id.startsWith(`${key}#`);
  return { data: fresh || sameKey ? settled!.data : undefined, error: fresh ? settled!.error : null, loading: !fresh };
}
