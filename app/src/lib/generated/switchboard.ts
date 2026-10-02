/**
 * Read-only Switchboard On-Demand helpers (no SDK). Offsets mirror switchboard-on-demand 0.13.0
 * `QueueAccountData` / `RandomnessAccountData` (repr(C), after the 8-byte discriminator), the same
 * structs hybrid_vault's `randomness::select_oracle_in` reads on-chain.
 */
import { PublicKey } from "@solana/web3.js";
import { u64le, utf8 } from "./borsh";

export const QUEUE_DISCRIMINATOR = [217, 194, 55, 127, 184, 83, 138, 1] as const;
export const RANDOMNESS_DISCRIMINATOR = [10, 66, 229, 135, 220, 239, 217, 114] as const;
const Q_ORACLE_KEYS = 1056; // after authority(32) + mr_enclaves(32*32)
const Q_ORACLE_KEYS_LEN = 5196;
const Q_CURR_IDX = 5204;
const Q_BODY_MIN = 5212;

export interface QueueView {
  readonly oracleKeys: readonly PublicKey[];
  readonly currIdx: number;
}

export function decodeQueue(data: Uint8Array): QueueView {
  for (let i = 0; i < 8; i++) if (data[i] !== QUEUE_DISCRIMINATOR[i]) throw new Error("Not a Switchboard queue");
  const body = data.subarray(8);
  if (body.length < Q_BODY_MIN) throw new RangeError("Queue account too short");
  const v = new DataView(body.buffer, body.byteOffset, body.byteLength);
  const len = v.getUint32(Q_ORACLE_KEYS_LEN, true);
  if (len === 0 || len > 78) throw new Error("Queue has no oracles");
  const oracleKeys = Array.from({ length: len }, (_, i) => new PublicKey(body.slice(Q_ORACLE_KEYS + 32 * i, Q_ORACLE_KEYS + 32 * (i + 1))));
  return { oracleKeys, currIdx: v.getUint32(Q_CURR_IDX, true) };
}

/**
 * The program's deterministic oracle order (M-04), mirrored off-chain so we pass the right
 * account: start = (curr % len + H("hybrid_vault/oracle", vault, seq)[..8] % len + used) % len.
 * The program still decides; a wrong guess just fails simulation (WrongOracle).
 */
export async function oracleCandidates(queue: QueueView, vault: PublicKey, seq: bigint, used = 0): Promise<PublicKey[]> {
  const input = new Uint8Array([...utf8("hybrid_vault/oracle"), ...vault.toBytes(), ...u64le(seq)]);
  const h = new Uint8Array(await globalThis.crypto.subtle.digest("SHA-256", input));
  const spread = new DataView(h.buffer).getBigUint64(0, true);
  const len = BigInt(queue.oracleKeys.length);
  const start = (BigInt(queue.currIdx) % len + (spread % len) + BigInt(used)) % len;
  const out: PublicKey[] = [];
  for (let s = 0n; s < len; s++) {
    const k = queue.oracleKeys[Number((start + s) % len)]!;
    if (!k.equals(PublicKey.default)) out.push(k);
  }
  return out;
}

export interface RandomnessView {
  readonly authority: PublicKey;
  readonly queue: PublicKey;
  readonly seedSlothash: Uint8Array;
  readonly seedSlot: bigint;
  readonly oracle: PublicKey;
  readonly revealSlot: bigint;
}
export function decodeRandomness(data: Uint8Array): RandomnessView {
  for (let i = 0; i < 8; i++) if (data[i] !== RANDOMNESS_DISCRIMINATOR[i]) throw new Error("Not a Switchboard randomness account");
  const v = new DataView(data.buffer, data.byteOffset, data.byteLength);
  return {
    authority: new PublicKey(data.slice(8, 40)),
    queue: new PublicKey(data.slice(40, 72)),
    seedSlothash: data.slice(72, 104),
    seedSlot: v.getBigUint64(104, true),
    oracle: new PublicKey(data.slice(112, 144)),
    revealSlot: v.getBigUint64(144, true),
  };
}

/** OracleAccountData.gateway_uri: [u8; 64] at byte 3584 (incl. discriminator), NUL-padded. Checked
 * against devnet oracle Hdu1niJg… (4,816-byte account) on 2026-10-01. */
const ORACLE_GATEWAY_OFF = 3584;
export function decodeOracleGatewayUri(data: Uint8Array): string {
  if (data.length < ORACLE_GATEWAY_OFF + 64) throw new RangeError("Oracle account too short");
  const raw = data.subarray(ORACLE_GATEWAY_OFF, ORACLE_GATEWAY_OFF + 64);
  const end = raw.indexOf(0);
  return new TextDecoder().decode(end < 0 ? raw : raw.subarray(0, end));
}

/** PDA ["OracleRandomnessStats", oracle] under the Switchboard program. */
export function oracleStatsPda(oracle: PublicKey, sbProgram: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync([utf8("OracleRandomnessStats"), oracle.toBuffer()], sbProgram)[0];
}
/** PDA ["LutSigner", randomness] under the Switchboard program, and its lookup table for `recentSlot`. */
export function sbLutAccounts(randomness: PublicKey, recentSlot: bigint, sbProgram: PublicKey, altProgram: PublicKey) {
  const lutSigner = PublicKey.findProgramAddressSync([utf8("LutSigner"), randomness.toBuffer()], sbProgram)[0];
  const lut = PublicKey.findProgramAddressSync([lutSigner.toBuffer(), u64le(recentSlot)], altProgram)[0];
  return { lutSigner, lut };
}
