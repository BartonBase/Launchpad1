/** Display formatting. bigint-in, string-out; never floats for on-chain amounts. */
import { LAMPORTS_PER_SOL } from "@/config/armory";

function groupInt(s: string): string {
  return s.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

/** Fixed-point bigint -> decimal string, trimming trailing zeros, max `dp` decimals (truncates). */
export function formatUnits(value: bigint, decimals: number, dp = decimals): string {
  const neg = value < 0n;
  const v = neg ? -value : value;
  const base = 10n ** BigInt(decimals);
  const whole = v / base;
  let frac = (v % base).toString().padStart(decimals, "0").slice(0, dp).replace(/0+$/, "");
  if (decimals === 0) frac = "";
  return `${neg ? "-" : ""}${groupInt(whole.toString())}${frac ? "." + frac : ""}`;
}

export function formatSol(lamports: bigint | string, dp = 9): string {
  return `${formatUnits(BigInt(lamports), 9, dp)} SOL`;
}

export function formatTokens(base: bigint | string, decimals: number, dp = 2): string {
  return formatUnits(BigInt(base), decimals, dp);
}

/** 1000000 -> "1M", 50000 -> "50K", 2500000 -> "2.5M". */
export function compact(n: number | bigint | string): string {
  const x = Number(n);
  if (x >= 1_000_000_000) return `${+(x / 1_000_000_000).toFixed(2)}B`;
  if (x >= 1_000_000) return `${+(x / 1_000_000).toFixed(2)}M`;
  if (x >= 1_000) return `${+(x / 1_000).toFixed(1)}K`;
  return String(x);
}

export function shortAddr(a: string, n = 4): string {
  return a.length <= 2 * n + 1 ? a : `${a.slice(0, n)}…${a.slice(-n)}`;
}

export function solToLamports(sol: string): bigint | null {
  if (!/^\d+(\.\d{1,9})?$/.test(sol.trim())) return null;
  const [w, f = ""] = sol.trim().split(".");
  return BigInt(w!) * LAMPORTS_PER_SOL + BigInt(f.padEnd(9, "0"));
}
