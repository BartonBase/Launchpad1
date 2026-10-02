/**
 * Minimal Meteora DBC client (no SDK): only what the DBC launch path needs. Hand-written from
 * MeteoraAg/dynamic-bonding-curve source (research checkout f552f20, 2026-09-09):
 * ix_initialize_virtual_pool_with_spl_token.rs (accounts + #[event_cpi]) and state/config.rs.
 * Config offsets are the ones hybrid_launch/src/dbc.rs pins and checks on-chain (DBC 0.2.1).
 */
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { DBC_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_METADATA_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@/config/programs";
import { Writer, utf8 } from "./borsh";

const PID = DBC_PROGRAM_ID;
/** sha256("global:initialize_virtual_pool_with_spl_token")[..8] (checked in tests). */
export const DBC_INIT_POOL_SPL_DISC = [140, 85, 215, 176, 102, 54, 104, 79] as const;
export const DBC_CONFIG_DISCRIMINATOR = [26, 108, 14, 123, 116, 230, 129, 43] as const;
export const DBC_CONFIG_LEN = 8 + 1040;

export const dbcPoolAuthority = () => PublicKey.findProgramAddressSync([utf8("pool_authority")], PID)[0];
export const dbcEventAuthority = () => PublicKey.findProgramAddressSync([utf8("__event_authority")], PID)[0];
export function dbcPoolPda(config: PublicKey, baseMint: PublicKey, quoteMint: PublicKey): PublicKey {
  const [a, b] = [baseMint.toBytes(), quoteMint.toBytes()];
  const cmp = Buffer.compare(Buffer.from(a), Buffer.from(b));
  const [max, min] = cmp >= 0 ? [a, b] : [b, a];
  return PublicKey.findProgramAddressSync([utf8("pool"), config.toBuffer(), max, min], PID)[0];
}
export const dbcTokenVault = (mint: PublicKey, pool: PublicKey) => PublicKey.findProgramAddressSync([utf8("token_vault"), mint.toBuffer(), pool.toBuffer()], PID)[0];
export const metadataPda = (mint: PublicKey) =>
  PublicKey.findProgramAddressSync([utf8("metadata"), TOKEN_METADATA_PROGRAM_ID.toBuffer(), mint.toBuffer()], TOKEN_METADATA_PROGRAM_ID)[0];

export interface DbcConfigView {
  readonly quoteMint: PublicKey;
  readonly tokenDecimal: number;
  readonly tokenType: number;
  readonly migrationQuoteThreshold: bigint;
}
export function decodeDbcConfig(data: Uint8Array): DbcConfigView {
  if (data.length !== DBC_CONFIG_LEN) throw new Error("Not a DBC PoolConfig (length)");
  for (let i = 0; i < 8; i++) if (data[i] !== DBC_CONFIG_DISCRIMINATOR[i]) throw new Error("Not a DBC PoolConfig");
  const v = new DataView(data.buffer, data.byteOffset, data.byteLength);
  return { quoteMint: new PublicKey(data.slice(8, 40)), tokenDecimal: data[235]!, tokenType: data[237]!, migrationQuoteThreshold: v.getBigUint64(264, true) };
}

/** `initialize_virtual_pool_with_spl_token(name, symbol, uri)`: DBC creates the mint (fresh keypair,
 * co-signs), revokes its authorities per config and opens the curve. */
export function initializeVirtualPoolWithSplTokenIx(a: {
  config: PublicKey; quoteMint: PublicKey; creator: PublicKey; payer: PublicKey; baseMint: PublicKey; name: string; symbol: string; uri: string;
}): TransactionInstruction {
  const pool = dbcPoolPda(a.config, a.baseMint, a.quoteMint);
  const ro = (pubkey: PublicKey) => ({ pubkey, isSigner: false, isWritable: false });
  const rw = (pubkey: PublicKey) => ({ pubkey, isSigner: false, isWritable: true });
  return new TransactionInstruction({
    programId: PID,
    keys: [
      ro(a.config),
      ro(dbcPoolAuthority()),
      { pubkey: a.creator, isSigner: true, isWritable: false },
      { pubkey: a.baseMint, isSigner: true, isWritable: true },
      ro(a.quoteMint),
      rw(pool),
      rw(dbcTokenVault(a.baseMint, pool)),
      rw(dbcTokenVault(a.quoteMint, pool)),
      rw(metadataPda(a.baseMint)),
      ro(TOKEN_METADATA_PROGRAM_ID),
      { pubkey: a.payer, isSigner: true, isWritable: true },
      ro(TOKEN_PROGRAM_ID), // token_quote_program (wSOL is SPL Token)
      ro(TOKEN_PROGRAM_ID),
      ro(SYSTEM_PROGRAM_ID),
      ro(dbcEventAuthority()),
      ro(PID),
    ],
    data: new Writer().raw(DBC_INIT_POOL_SPL_DISC).string(a.name).string(a.symbol).string(a.uri).toBuffer(),
  });
}
