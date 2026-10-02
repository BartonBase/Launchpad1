/** Launch wizard validation (pure; unit-tested). Size/target rules come from wizardMath. */
import { RATIOS } from "@/config/armory";
import type { LaunchParams } from "@/lib/generated/hybridLaunch";
import { parseMetadataUri } from "@/lib/validate";
import { solToLamports } from "./format";
import { wizardMath, type WizardMath, type WizardType } from "./wizardMath";

export const MAX_GRADUATION_LAMPORTS = 100_000_000_000_000n; // IDL constant (100,000 SOL)
export const LAUNCH_DECIMALS = 6;

export interface LaunchForm {
  readonly type: WizardType;
  readonly name: string;
  readonly symbol: string;
  readonly ratio: number;
  readonly collectionSize: string;
  readonly graduationSol: string;
  /** Token metadata URI (https://, ipfs:// or ar://). Used by the curve (DBC) path; optional. */
  readonly metadataUri?: string;
  /** Art commitment (init_vault): Hybrid and Burn only. */
  readonly collectionName?: string;
  readonly collectionUri?: string;
  readonly traitRoot?: string;
  readonly schemaHash?: string;
}

export type ArtFields = "collectionName" | "collectionUri" | "traitRoot" | "schemaHash";
export type FieldErrors = Partial<Record<"name" | "symbol" | "ratio" | "collectionSize" | "graduationSol" | "metadataUri" | ArtFields, string>>;

export interface ArtCommitmentInput {
  readonly collectionName: string;
  readonly collectionUri: string;
  readonly traitRoot: Uint8Array;
  readonly traitSchemaHash: Uint8Array;
}

/** 32-byte hex, optional 0x prefix. */
export function parseHash32(s: string): Uint8Array | null {
  const h = s.trim().replace(/^0x/i, "");
  if (!/^[0-9a-fA-F]{64}$/.test(h)) return null;
  return Uint8Array.from(h.match(/../g)!, (b) => parseInt(b, 16));
}

/** Art commitment rules mirror init_vault: name <= 32 bytes, content-addressed URI <= 200 bytes. */
export function validateArt(f: LaunchForm): { errors: FieldErrors; art: ArtCommitmentInput | null } {
  const errors: FieldErrors = {};
  const name = (f.collectionName ?? "").trim();
  if (name.length < 1 || new TextEncoder().encode(name).length > 32) errors.collectionName = "Collection name: 1 to 32 characters.";
  const uri = (f.collectionUri ?? "").trim();
  const pu = parseMetadataUri(uri);
  if (!pu.ok || !/^(ipfs|ar):\/\//.test(uri)) errors.collectionUri = "Collection URI must be ipfs:// or ar:// (content-addressed).";
  const root = parseHash32(f.traitRoot ?? "");
  if (!root) errors.traitRoot = "Trait root: 32 bytes as 64 hex characters.";
  const schema = parseHash32(f.schemaHash ?? "");
  if (!schema) errors.schemaHash = "Schema hash: 32 bytes as 64 hex characters.";
  return { errors, art: Object.keys(errors).length === 0 ? { collectionName: name, collectionUri: uri, traitRoot: root!, traitSchemaHash: schema! } : null };
}

export function validateLaunchForm(
  f: LaunchForm,
  chainMinLamports: bigint,
): { errors: FieldErrors; math: WizardMath; params: LaunchParams | null } {
  const errors: FieldErrors = {};
  const name = f.name.trim();
  if (name.length < 1 || name.length > 32) errors.name = "Name must be 1–32 characters.";
  if (!/^[A-Z]{3,6}$/.test(f.symbol.trim())) errors.symbol = "Ticker: 3 to 6 letters.";
  const nft = f.type !== "plain";
  if (nft && !(RATIOS as readonly number[]).includes(f.ratio)) errors.ratio = "Pick a ratio.";
  const sizeStr = f.collectionSize.trim().replace(/,/g, "");
  const size = /^\d{1,6}$/.test(sizeStr) ? Number(sizeStr) : NaN;
  let grad = solToLamports(f.graduationSol);
  if (grad !== null && grad > MAX_GRADUATION_LAMPORTS) grad = null;
  const math = wizardMath({ type: f.type, ratio: (RATIOS as readonly number[]).includes(f.ratio) ? f.ratio : 1_000_000, size, targetLamports: grad, chainMinLamports });
  if (nft && !math.sizeOk) {
    errors.collectionSize =
      math.sizeError === "high"
        ? `The maximum for this ratio is ${math.maxSize.toLocaleString("en-US")} NFTs${math.maxSize === 10_000 ? " (collections are capped at 10,000)" : ""}.`
        : `Collections need at least 100 NFTs. Choose any size from 100 to ${math.maxSize.toLocaleString("en-US")}.`;
  }
  const mu = (f.metadataUri ?? "").trim();
  if (mu !== "" && !parseMetadataUri(mu).ok) errors.metadataUri = "Use an https://, ipfs:// or ar:// URI (max 200 bytes).";
  if (!math.targetOk) errors.graduationSol = `The graduation target must be at least ${Number(math.minTargetLamports) / 1e9} SOL${grad === null && f.graduationSol.trim() !== "" ? " (and at most 100,000)" : ""}.`;
  const ok = Object.keys(errors).length === 0;
  return {
    errors,
    math,
    params:
      ok && f.type === "hybrid"
        ? { decimals: LAUNCH_DECIMALS, ratioWholeTokens: BigInt(f.ratio), collectionSize: BigInt(size), graduationThresholdLamports: grad! }
        : null,
  };
}
