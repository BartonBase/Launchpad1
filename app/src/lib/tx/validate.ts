/**
 * Step 2 of the pipeline: static allowlist check. Runs BEFORE simulation and
 * signing. Any top-level instruction targeting a program that is not pinned
 * in src/config/programs.ts rejects the whole transaction.
 */
import type { ClusterName } from "@/config/cluster";
import { CLUSTER } from "@/config/cluster";
import { lookupProgram } from "@/config/programs";
import { viewMessage, type AnyTransaction } from "./message";

export interface InstructionCheck {
  readonly index: number;
  readonly programId: string;
  readonly programName: string | null;
  readonly allowed: boolean;
}

export interface ValidationResult {
  readonly ok: boolean;
  readonly instructions: readonly InstructionCheck[];
  /** Human-readable blocking reasons (empty when ok). */
  readonly errors: readonly string[];
}

export const MAX_INSTRUCTIONS = 32;

export function validateInstructions(
  tx: AnyTransaction,
  cluster: ClusterName = CLUSTER.name,
): ValidationResult {
  let view;
  try {
    view = viewMessage(tx);
  } catch (e) {
    return {
      ok: false,
      instructions: [],
      errors: [`Malformed transaction: ${e instanceof Error ? e.message : String(e)}`],
    };
  }

  const errors: string[] = [];
  if (view.instructions.length === 0) errors.push("Transaction has no instructions.");
  if (view.instructions.length > MAX_INSTRUCTIONS) {
    errors.push(`Transaction has too many instructions (${view.instructions.length} > ${MAX_INSTRUCTIONS}).`);
  }

  const instructions = view.instructions.map((ix): InstructionCheck => {
    const entry = lookupProgram(ix.programId, cluster);
    const check = {
      index: ix.index,
      programId: ix.programId.toBase58(),
      programName: entry?.name ?? null,
      // CPI-only entries are known (for labels) but never valid top-level targets.
      allowed: entry?.topLevel === true,
    };
    if (!check.allowed) {
      errors.push(`Instruction #${ix.index} calls a program that is not allowlisted as a top-level target: ${check.programId}${entry ? ` (${entry.name}, CPI-only)` : ""}`);
    }
    return check;
  });

  return { ok: errors.length === 0, instructions, errors };
}
