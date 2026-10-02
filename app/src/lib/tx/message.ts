/**
 * Normalizes legacy `Transaction` and `VersionedTransaction` into one
 * read-only view so validation / simulation / preview share a single code path.
 */
import {
  PublicKey,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
  type VersionedMessage,
} from "@solana/web3.js";
import { NO_LOOKUPS, fullAccountKeys, type ResolvedLookups } from "./lookup";

export type AnyTransaction = Transaction | VersionedTransaction;

export interface InstructionView {
  readonly index: number;
  readonly programId: PublicKey;
  /** Account keys referenced; `null` only for lookup-table keys that could not be resolved. */
  readonly accounts: readonly (PublicKey | null)[];
  readonly data: Uint8Array;
}

export interface MessageView {
  readonly feePayer: PublicKey;
  /** Static account keys (lookup-table keys are NOT included). */
  readonly staticKeys: readonly PublicKey[];
  readonly instructions: readonly InstructionView[];
  readonly writable: readonly PublicKey[];
  readonly signers: readonly PublicKey[];
  readonly usesLookupTables: boolean;
  /** Accounts loaded through lookup tables (resolved), by access. */
  readonly loadedWritable: readonly PublicKey[];
  readonly loadedReadonly: readonly PublicKey[];
  readonly lookups: ResolvedLookups;
  readonly message: VersionedMessage;
}

/** Placeholder blockhash used only when simulating with replaceRecentBlockhash. */
const DUMMY_BLOCKHASH = PublicKey.default.toBase58();

export function isVersioned(tx: AnyTransaction): tx is VersionedTransaction {
  return !(tx instanceof Transaction);
}

/**
 * Compile to a VersionedMessage WITHOUT mutating the caller's transaction.
 * Legacy transactions must have a fee payer.
 */
export function toVersionedMessage(tx: AnyTransaction): VersionedMessage {
  if (isVersioned(tx)) return tx.message;
  if (!tx.feePayer) throw new Error("Transaction has no fee payer.");
  if (tx.recentBlockhash) return tx.compileMessage();
  return new TransactionMessage({
    payerKey: tx.feePayer,
    recentBlockhash: DUMMY_BLOCKHASH,
    instructions: tx.instructions,
  }).compileToLegacyMessage();
}

export function viewMessage(tx: AnyTransaction, lookups: ResolvedLookups = NO_LOOKUPS): MessageView {
  const message = toVersionedMessage(tx);
  const staticKeys = message.staticAccountKeys;
  const all = fullAccountKeys(message, lookups);
  const keyAt = (i: number): PublicKey | null => (all ? (all.get(i) ?? null) : (staticKeys[i] ?? null));
  const instructions: InstructionView[] = message.compiledInstructions.map((ix, index) => {
    const programId = staticKeys[ix.programIdIndex];
    if (!programId) {
      // Program IDs can never come from lookup tables; treat as malformed.
      throw new Error(`Instruction ${index} has an invalid program id index.`);
    }
    return {
      index,
      programId,
      accounts: ix.accountKeyIndexes.map(keyAt),
      data: ix.data,
    };
  });
  const writable: PublicKey[] = [];
  const signers: PublicKey[] = [];
  staticKeys.forEach((k, i) => {
    if (message.isAccountWritable(i)) writable.push(k);
    if (message.isAccountSigner(i)) signers.push(k);
  });
  const loadedWritable = all ? [...lookups.writable] : [];
  const loadedReadonly = all ? [...lookups.readonly] : [];
  writable.push(...loadedWritable);
  const feePayer = staticKeys[0];
  if (!feePayer) throw new Error("Transaction has no accounts.");
  return {
    feePayer,
    staticKeys,
    instructions,
    writable,
    signers,
    usesLookupTables: message.addressTableLookups.length > 0,
    loadedWritable,
    loadedReadonly,
    lookups,
    message,
  };
}

/** Serialized message bytes: the exact thing a wallet signs. */
export function messageBytes(tx: AnyTransaction): Uint8Array {
  return isVersioned(tx) ? tx.message.serialize() : tx.serializeMessage();
}

export function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
  return true;
}

/** Short hex SHA-256 fingerprint of the message, shown in the preview. */
export async function messageFingerprint(bytes: Uint8Array): Promise<string> {
  const copy = new Uint8Array(bytes); // ensure a plain ArrayBuffer-backed view
  const digest = await globalThis.crypto.subtle.digest("SHA-256", copy);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, "0")).join("");
}
