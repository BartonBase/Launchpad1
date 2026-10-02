/** Plain-language messages for program errors users can actually hit (shown in the tx preview). */
import { HYBRID_VAULT_PROGRAM_ID } from "@/config/programs";

export const NO_CURVE_MESSAGE =
  "This launch has no bonding curve (native devnet test launch), so it can never graduate and converting can never open.";

const FRIENDLY: Record<string, Record<number, string>> = {
  [HYBRID_VAULT_PROGRAM_ID.toBase58()]: {
    6037: NO_CURVE_MESSAGE, // GraduationCheckUnavailable: no DBC pool recorded (native `launch`)
  },
};

export function friendlyProgramError(programId: string, code: number): string | null {
  return FRIENDLY[programId]?.[code] ?? null;
}

/** Parses `{"InstructionError":[i,{"Custom":n}]}` (as JSON text or object). */
export function instructionCustomError(err: unknown): { index: number; code: number } | null {
  let e = err;
  if (typeof e === "string") {
    try {
      e = JSON.parse(e);
    } catch {
      return null;
    }
  }
  const ie = (e as { InstructionError?: [unknown, unknown] } | null)?.InstructionError;
  const inner = ie?.[1] as { Custom?: unknown } | undefined;
  return ie && typeof ie[0] === "number" && typeof inner?.Custom === "number" ? { index: ie[0], code: inner.Custom } : null;
}
