/**
 * Strict validators for UNTRUSTED input (URL params, form fields, metadata).
 *
 * All validators return a Result instead of throwing so UI code is forced to
 * handle the error branch. Token amounts are bigint base units — never floats.
 */
import { PublicKey } from "@solana/web3.js";

export type Result<T> = { ok: true; value: T } | { ok: false; error: string };
const ok = <T,>(value: T): Result<T> => ({ ok: true, value });
const err = <T,>(error: string): Result<T> => ({ ok: false, error });

// ---------------------------------------------------------------------------
// Public keys
// ---------------------------------------------------------------------------

const BASE58_RE = /^[1-9A-HJ-NP-Za-km-z]+$/;

export interface PublicKeyOptions {
  /**
   * "on"  => must be on the ed25519 curve (a wallet / keypair address)
   * "off" => must be OFF curve (a PDA)
   * "any" => either (default; e.g. mints, which are usually keypairs)
   */
  readonly curve?: "on" | "off" | "any";
}

export function parsePublicKey(input: unknown, opts: PublicKeyOptions = {}): Result<PublicKey> {
  if (typeof input !== "string") return err("Address must be a string.");
  // No trimming: surrounding whitespace in a URL/param is treated as invalid
  // input rather than silently "fixed".
  if (input.length < 32 || input.length > 44) {
    return err("Address must be 32–44 base58 characters.");
  }
  if (!BASE58_RE.test(input)) return err("Address contains non-base58 characters.");
  let key: PublicKey;
  try {
    key = new PublicKey(input);
  } catch {
    return err("Address is not a valid 32-byte public key.");
  }
  // Canonical round-trip: rejects non-canonical encodings of the same bytes.
  if (key.toBase58() !== input) return err("Address is not canonically encoded.");

  const curve = opts.curve ?? "any";
  if (curve !== "any") {
    const onCurve = PublicKey.isOnCurve(key.toBytes());
    if (curve === "on" && !onCurve) return err("Address must be a wallet (on-curve) address.");
    if (curve === "off" && onCurve) return err("Address must be a program-derived (off-curve) address.");
  }
  return ok(key);
}

// ---------------------------------------------------------------------------
// Token amounts (bigint base units, no floats)
// ---------------------------------------------------------------------------

export const U64_MAX = (1n << 64n) - 1n;
export const MAX_DECIMALS = 18; // SPL mints use u8 decimals; >18 is nonsense for UI input

export interface AmountOptions {
  /** Mint decimals (0..18). */
  readonly decimals: number;
  /** Inclusive minimum in BASE units. Default 1n (zero rejected). */
  readonly min?: bigint;
  /** Inclusive maximum in BASE units. Default u64::MAX. */
  readonly max?: bigint;
}

const AMOUNT_RE = /^(0|[1-9][0-9]*)(?:\.([0-9]+))?$/;

/**
 * Parse a human decimal string ("1.5") into base units (1_500_000n for 6 dp).
 * Rejects: signs, exponents, commas, whitespace, leading zeros, more fractional
 * digits than `decimals`, values outside [min, max].
 */
export function parseAmount(input: unknown, opts: AmountOptions): Result<bigint> {
  const { decimals } = opts;
  if (!Number.isInteger(decimals) || decimals < 0 || decimals > MAX_DECIMALS) {
    return err(`Invalid decimals: ${String(decimals)}.`);
  }
  if (typeof input !== "string") return err("Amount must be a string.");
  if (input.length === 0) return err("Amount is required.");
  if (input.length > 40) return err("Amount is too long.");
  const m = AMOUNT_RE.exec(input);
  if (!m) return err("Amount must be a plain decimal number, e.g. 12.5");
  const whole = m[1] ?? "0";
  const frac = m[2] ?? "";
  if (frac.length > decimals) {
    return err(decimals === 0 ? "Amount must be a whole number." : `At most ${decimals} decimal places.`);
  }
  const base = BigInt(whole + frac.padEnd(decimals, "0"));
  const min = opts.min ?? 1n;
  const max = opts.max ?? U64_MAX;
  if (min > max) return err("Invalid bounds.");
  if (base < min) return err(`Amount must be at least ${formatAmount(min, decimals)}.`);
  if (base > max) return err(`Amount must be at most ${formatAmount(max, decimals)}.`);
  return ok(base);
}

/** Format base units as a decimal string without floats. Handles negatives. */
export function formatAmount(base: bigint, decimals: number): string {
  if (!Number.isInteger(decimals) || decimals < 0 || decimals > 255) {
    throw new RangeError("decimals out of range");
  }
  const neg = base < 0n;
  const abs = neg ? -base : base;
  if (decimals === 0) return (neg ? "-" : "") + abs.toString();
  const s = abs.toString().padStart(decimals + 1, "0");
  const whole = s.slice(0, -decimals);
  const frac = s.slice(-decimals).replace(/0+$/, "");
  return (neg ? "-" : "") + whole + (frac ? "." + frac : "");
}

// ---------------------------------------------------------------------------
// Metadata / asset URIs
// ---------------------------------------------------------------------------

/** Metaplex on-chain URI field limit (bytes). */
export const MAX_URI_LENGTH = 200;

export type UriKind = "https" | "ipfs" | "arweave";
export interface ParsedUri {
  readonly kind: UriKind;
  /** The original, validated URI (normalized by URL parser for https). */
  readonly uri: string;
}

export interface UriOptions {
  /**
   * Optional https host allowlist. Entries match exactly, or as a suffix when
   * written as ".example.com" (subdomains only). Undefined => any public host.
   */
  readonly allowedHttpsHosts?: readonly string[];
}

const IPFS_CID_RE = /^(Qm[1-9A-HJ-NP-Za-km-z]{44}|b[a-z2-7]{58,})$/;
const ARWEAVE_TX_RE = /^[A-Za-z0-9_-]{43}$/;
const SAFE_PATH_RE = /^(\/[A-Za-z0-9._~!$&'()*+,;=:@%-]*)*$/;

function isPrivateOrLocalHost(host: string): boolean {
  const h = host.toLowerCase().replace(/^\[|\]$/g, "");
  if (h === "localhost" || h.endsWith(".localhost") || h.endsWith(".local") || h.endsWith(".internal")) {
    return true;
  }
  // Any IP literal (v4 or v6) is rejected: metadata should use DNS names.
  if (/^\d{1,3}(\.\d{1,3}){3}$/.test(h) || h.includes(":")) return true;
  return false;
}

export function parseMetadataUri(input: unknown, opts: UriOptions = {}): Result<ParsedUri> {
  if (typeof input !== "string") return err("URI must be a string.");
  if (input.length === 0) return err("URI is required.");
  if (new TextEncoder().encode(input).length > MAX_URI_LENGTH) {
    return err(`URI exceeds ${MAX_URI_LENGTH} bytes.`);
  }
  if (/[\s\u0000-\u001f\u007f]/.test(input)) return err("URI contains whitespace or control characters.");

  if (input.startsWith("ipfs://")) {
    const rest = input.slice("ipfs://".length);
    const slash = rest.indexOf("/");
    const cid = slash === -1 ? rest : rest.slice(0, slash);
    const path = slash === -1 ? "" : rest.slice(slash);
    if (!IPFS_CID_RE.test(cid)) return err("Invalid IPFS CID.");
    if (!SAFE_PATH_RE.test(path) || path.includes("..")) return err("Invalid IPFS path.");
    return ok({ kind: "ipfs", uri: input });
  }

  if (input.startsWith("ar://")) {
    const rest = input.slice("ar://".length);
    const slash = rest.indexOf("/");
    const tx = slash === -1 ? rest : rest.slice(0, slash);
    const path = slash === -1 ? "" : rest.slice(slash);
    if (!ARWEAVE_TX_RE.test(tx)) return err("Invalid Arweave transaction id.");
    if (!SAFE_PATH_RE.test(path) || path.includes("..")) return err("Invalid Arweave path.");
    return ok({ kind: "arweave", uri: input });
  }

  let url: URL;
  try {
    url = new URL(input);
  } catch {
    return err("URI is not a valid URL.");
  }
  if (url.protocol !== "https:") return err("Only https://, ipfs:// and ar:// URIs are allowed.");
  if (url.username || url.password) return err("URI must not contain credentials.");
  if (url.port && url.port !== "443") return err("URI must not specify a non-standard port.");
  const host = url.hostname.toLowerCase();
  if (isPrivateOrLocalHost(host)) return err("URI host must be a public DNS name.");
  if (opts.allowedHttpsHosts) {
    const allowed = opts.allowedHttpsHosts.some((a) =>
      a.startsWith(".") ? host.endsWith(a.toLowerCase()) : host === a.toLowerCase(),
    );
    if (!allowed) return err(`Host "${host}" is not in the allowlist.`);
  }
  const kind: UriKind = host === "arweave.net" || host.endsWith(".arweave.net") ? "arweave" : "https";
  return ok({ kind, uri: url.toString() });
}

/** Resolve ipfs:// / ar:// to an https gateway URL for display/fetching. */
export function toGatewayUrl(
  parsed: ParsedUri,
  gateways: { ipfs: string; arweave: string } = {
    ipfs: "https://ipfs.io/ipfs/",
    arweave: "https://arweave.net/",
  },
): string {
  if (parsed.uri.startsWith("ipfs://")) return gateways.ipfs + parsed.uri.slice("ipfs://".length);
  if (parsed.uri.startsWith("ar://")) return gateways.arweave + parsed.uri.slice("ar://".length);
  return parsed.uri;
}
