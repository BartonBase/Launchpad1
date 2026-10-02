/**
 * Step 4: the TxPreview model — a human-readable description of what the
 * transaction will do, derived ONLY from the static message + simulation.
 * It is what TxPreviewModal renders; nothing in it comes from page content.
 */
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import type { ClusterConfig } from "@/config/cluster";
import { lookupProgram } from "@/config/programs";
import { formatAmount } from "@/lib/validate";
import type { MessageView } from "./message";
import type { SimulationResult, SolChange, TokenChange } from "./simulate";
import type { ValidationResult } from "./validate";

export interface PreviewProgram {
  readonly programId: string;
  readonly name: string | null;
  readonly allowlisted: boolean;
  /** Called directly by a top-level instruction. */
  readonly topLevel: boolean;
  /** Only seen as a cross-program invocation in simulation logs. */
  readonly cpiOnly: boolean;
}

export interface PreviewSolChange extends SolChange {
  /** e.g. "-0.000005 SOL" */
  readonly display: string;
}

export interface PreviewTokenChange extends TokenChange {
  /** e.g. "+12.5" (or raw base units when decimals are unknown) */
  readonly display: string;
}

export interface PreviewAccount {
  readonly address: string;
  readonly isSigner: boolean;
  readonly isFeePayer: boolean;
}

export interface PreviewLookupTable {
  readonly address: string;
  readonly pinned: boolean;
  readonly frozen: boolean;
  readonly writable: readonly string[];
  readonly readonly: readonly string[];
}

export interface TxPreview {
  readonly clusterLabel: string;
  readonly feePayer: string;
  /** SHA-256 of the exact message bytes the wallet will be asked to sign. */
  readonly fingerprint: string;
  readonly summary: readonly string[];
  readonly programs: readonly PreviewProgram[];
  readonly writableAccounts: readonly PreviewAccount[];
  /** Accounts loaded through address lookup tables, resolved and shown (v0 only). */
  readonly lookupTables: readonly PreviewLookupTable[];
  readonly solChanges: readonly PreviewSolChange[];
  readonly tokenChanges: readonly PreviewTokenChange[];
  readonly feeLamports: bigint | null;
  readonly feeDisplay: string;
  readonly unitsConsumed: number | null;
  readonly logs: readonly string[];
  readonly warnings: readonly string[];
  /** Blocking problems. Non-empty => sending is disabled. */
  readonly errors: readonly string[];
  readonly canSend: boolean;
}

export function formatSol(lamports: bigint, signed = false): string {
  const s = formatAmount(lamports, 9);
  const withSign = signed && lamports > 0n ? "+" + s : s;
  return `${withSign} SOL`;
}

export const LARGE_SOL_OUTFLOW = BigInt(LAMPORTS_PER_SOL); // warn above 1 SOL out

export interface BuildPreviewInput {
  readonly cluster: ClusterConfig;
  readonly view: MessageView;
  readonly fingerprint: string;
  readonly validation: ValidationResult;
  /** null when simulation was skipped because validation failed. */
  readonly simulation: SimulationResult | null;
  readonly feeLamports: bigint | null;
  readonly signer: string;
  /** Extra warnings from the caller (e.g. wallet can't sign-only). */
  readonly extraWarnings?: readonly string[];
}

export function buildTxPreview(input: BuildPreviewInput): TxPreview {
  const { cluster, view, validation, simulation, feeLamports, signer } = input;
  const warnings: string[] = [...(input.extraWarnings ?? [])];
  const summary0: string[] = [];
  const errors: string[] = [...validation.errors];

  // Programs: top-level from the message, CPIs from simulation logs.
  const programs = new Map<string, PreviewProgram>();
  for (const ix of view.instructions) {
    const id = ix.programId.toBase58();
    const entry = lookupProgram(id, cluster.name);
    programs.set(id, { programId: id, name: entry?.name ?? null, allowlisted: !!entry, topLevel: true, cpiOnly: false });
  }
  for (const inv of simulation?.invokedPrograms ?? []) {
    if (programs.has(inv.programId)) continue;
    const entry = lookupProgram(inv.programId, cluster.name);
    programs.set(inv.programId, {
      programId: inv.programId,
      name: entry?.name ?? null,
      allowlisted: !!entry,
      topLevel: false,
      cpiOnly: true,
    });
    if (!entry) warnings.push(`An allowlisted program calls an UNKNOWN program via CPI: ${inv.programId}`);
  }

  const signerSet = new Set(view.signers.map((k) => k.toBase58()));
  const feePayer = view.feePayer.toBase58();
  if (feePayer !== signer) errors.push(`Fee payer ${feePayer} is not the connected wallet ${signer}.`);

  const writableAccounts: PreviewAccount[] = view.writable.map((k) => {
    const a = k.toBase58();
    return { address: a, isSigner: signerSet.has(a), isFeePayer: a === feePayer };
  });

  if (simulation === null) {
    errors.push("Simulation was not run because the transaction failed validation.");
  } else if (!simulation.ok) {
    errors.push(`Simulation failed: ${simulation.error ?? "unknown error"}`);
  }
  errors.push(...view.lookups.errors);
  if (view.usesLookupTables && view.lookups.errors.length === 0) {
    summary0.push(`Loads ${view.loadedWritable.length + view.loadedReadonly.length} account(s) from ${view.lookups.tables.length} pinned lookup table(s); they are listed and diffed below.`);
  }
  if (feeLamports === null) warnings.push("Network fee could not be estimated.");
  if (simulation) {
    warnings.push("Balance changes come from a simulation and may differ if on-chain state changes before sending.");
  }

  const solChanges: PreviewSolChange[] = (simulation?.solChanges ?? []).map((c) => ({
    ...c,
    display: formatSol(c.deltaLamports, true),
  }));
  const tokenChanges: PreviewTokenChange[] = (simulation?.tokenChanges ?? []).map((c) => {
    const abs = c.decimals === null ? c.deltaAmount.toString() + " (base units)" : formatAmount(c.deltaAmount, c.decimals);
    return { ...c, display: c.deltaAmount > 0n && !abs.startsWith("+") ? "+" + abs : abs };
  });

  const signerSol = solChanges.find((c) => c.address === signer);
  if (signerSol && -signerSol.deltaLamports >= LARGE_SOL_OUTFLOW) {
    warnings.push(`Large SOL outflow from your wallet: ${signerSol.display}`);
  }

  const summary: string[] = [...summary0];
  summary.push(`Runs on ${cluster.label}. Invokes ${programs.size} program${programs.size === 1 ? "" : "s"}.`);
  summary.push(`Writes to ${writableAccounts.length} account${writableAccounts.length === 1 ? "" : "s"}.`);
  if (signerSol) summary.push(`Your SOL balance changes by ${signerSol.display} (includes network fee if charged in simulation).`);
  for (const t of tokenChanges.filter((t) => t.ownedBySigner)) {
    summary.push(`Your token balance for mint ${t.mint} changes by ${t.display}.`);
  }
  if (feeLamports !== null) summary.push(`Estimated network fee: ${formatSol(feeLamports)}.`);

  return {
    clusterLabel: cluster.label,
    feePayer,
    fingerprint: input.fingerprint,
    summary,
    programs: [...programs.values()],
    writableAccounts,
    lookupTables: view.lookups.tables.map((t, i) => {
      const lk = view.message.addressTableLookups[i];
      const acct = view.lookups.accounts.find((a) => a.key.toBase58() === t.address);
      const at = (idx: readonly number[]) => idx.map((j) => acct?.state.addresses[j]?.toBase58() ?? `#${j} (unresolved)`);
      return { address: t.address, pinned: t.pinned, frozen: t.frozen, writable: lk ? at(lk.writableIndexes) : [], readonly: lk ? at(lk.readonlyIndexes) : [] };
    }),
    solChanges,
    tokenChanges,
    feeLamports,
    feeDisplay: feeLamports === null ? "unknown" : formatSol(feeLamports),
    unitsConsumed: simulation?.unitsConsumed ?? null,
    logs: simulation?.logs ?? [],
    warnings,
    errors,
    canSend: errors.length === 0,
  };
}
