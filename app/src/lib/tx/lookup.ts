/**
 * Address lookup table resolution for v0 transactions. The preview must show (and simulate/diff)
 * every account a transaction touches, including ones loaded through a lookup table, so each table
 * the message references is fetched, validated and expanded here:
 *  - the table must be PINNED for the cluster (src/config/integrations.ts PINNED_LOOKUP_TABLES),
 *  - it must be frozen (no authority), or its on-chain contents must equal the pinned list
 *    (PINNED_LOOKUP_TABLE_CONTENTS), so the indexes the message uses can't point anywhere else,
 *  - it must not be deactivated, and every index must be in range.
 * Anything else is a blocking error; the resolved keys are still returned so the user sees them.
 */
import { PINNED_LOOKUP_TABLE_CONTENTS } from "@/config/integrations";
import { PublicKey, type AddressLookupTableAccount, type Connection, type MessageAccountKeys, type VersionedMessage } from "@solana/web3.js";

export interface LookupTableCheck {
  readonly address: string;
  readonly pinned: boolean;
  readonly frozen: boolean;
  /** On-chain contents equal the pinned list for this table. */
  readonly verified: boolean;
  readonly active: boolean;
  readonly found: boolean;
}

export interface ResolvedLookups {
  readonly tables: readonly LookupTableCheck[];
  readonly accounts: readonly AddressLookupTableAccount[];
  readonly writable: readonly PublicKey[];
  readonly readonly: readonly PublicKey[];
  readonly errors: readonly string[];
}

export const NO_LOOKUPS: ResolvedLookups = { tables: [], accounts: [], writable: [], readonly: [], errors: [] };
const U64_MAX = 18446744073709551615n;

/** True when the table holds exactly the pinned addresses, in order. */
export function contentsMatch(t: AddressLookupTableAccount, list: readonly string[] | undefined): boolean {
  return !!list && t.state.addresses.length === list.length && t.state.addresses.every((k, i) => k.toBase58() === list[i]);
}

/** Pure validation of fetched tables against a message (exported for tests). */
export function checkLookups(
  message: VersionedMessage,
  fetched: ReadonlyMap<string, AddressLookupTableAccount | null>,
  pinned: readonly PublicKey[],
  expected: Readonly<Record<string, readonly string[]>> = PINNED_LOOKUP_TABLE_CONTENTS,
): ResolvedLookups {
  const lookups = message.addressTableLookups;
  if (lookups.length === 0) return NO_LOOKUPS;
  const pinnedSet = new Set(pinned.map((p) => p.toBase58()));
  const errors: string[] = [];
  const tables: LookupTableCheck[] = [];
  const accounts: AddressLookupTableAccount[] = [];
  const writable: PublicKey[] = [];
  const ro: PublicKey[] = [];
  for (const l of lookups) {
    const address = l.accountKey.toBase58();
    const t = fetched.get(address) ?? null;
    const check: LookupTableCheck = {
      address,
      pinned: pinnedSet.has(address),
      found: t !== null,
      frozen: t !== null && t.state.authority === undefined,
      verified: t !== null && contentsMatch(t, expected[address]),
      active: t !== null && BigInt(t.state.deactivationSlot) === U64_MAX,
    };
    tables.push(check);
    if (!check.pinned) errors.push(`Lookup table ${address} is not pinned for this cluster.`);
    if (!t) {
      errors.push(`Lookup table ${address} was not found.`);
      continue;
    }
    if (expected[address] && !check.verified) errors.push(`Lookup table ${address} doesn't match its pinned contents.`);
    if (!check.frozen && !expected[address]) errors.push(`Lookup table ${address} still has an authority and no pinned contents, so its contents could change before sending.`);
    if (!check.active) errors.push(`Lookup table ${address} is deactivated.`);
    const pick = (idx: readonly number[], into: PublicKey[]) => {
      for (const i of idx) {
        const k = t.state.addresses[i];
        if (!k) errors.push(`Lookup table ${address} has no entry #${i}.`);
        else into.push(k);
      }
    };
    pick(l.writableIndexes, writable);
    pick(l.readonlyIndexes, ro);
    accounts.push(t);
  }
  return { tables, accounts, writable, readonly: ro, errors };
}

export async function resolveLookups(connection: Connection, message: VersionedMessage, pinned: readonly PublicKey[]): Promise<ResolvedLookups> {
  if (message.addressTableLookups.length === 0) return NO_LOOKUPS;
  const fetched = new Map<string, AddressLookupTableAccount | null>();
  await Promise.all(
    message.addressTableLookups.map(async (l) => {
      try {
        fetched.set(l.accountKey.toBase58(), (await connection.getAddressLookupTable(l.accountKey)).value);
      } catch {
        fetched.set(l.accountKey.toBase58(), null);
      }
    }),
  );
  return checkLookups(message, fetched, pinned);
}

/** Full account keys (static + loaded) when every table resolved, else null. */
export function fullAccountKeys(message: VersionedMessage, r: ResolvedLookups): MessageAccountKeys | null {
  if (message.addressTableLookups.length === 0) return message.getAccountKeys();
  if (r.accounts.length !== message.addressTableLookups.length) return null;
  try {
    return message.getAccountKeys({ addressLookupTableAccounts: [...r.accounts] });
  } catch {
    return null;
  }
}
