/** Tiny SPL helpers (avoids adding @solana/spl-token for two functions). */
import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@/config/programs";

export function ataAddress(owner: PublicKey, mint: PublicKey, tokenProgram = TOKEN_PROGRAM_ID): PublicKey {
  return PublicKey.findProgramAddressSync([owner.toBuffer(), tokenProgram.toBuffer(), mint.toBuffer()], ASSOCIATED_TOKEN_PROGRAM_ID)[0];
}

/** ATA program `CreateIdempotent` (instruction 1). */
export function createAtaIdempotentIx(payer: PublicKey, owner: PublicKey, mint: PublicKey): TransactionInstruction {
  return new TransactionInstruction({
    programId: ASSOCIATED_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: ataAddress(owner, mint), isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Uint8Array.from([1]) as unknown as Buffer,
  });
}

export interface MintInfo {
  readonly mintAuthority: PublicKey | null;
  readonly supply: bigint;
  readonly decimals: number;
  readonly freezeAuthority: PublicKey | null;
}

/** SPL mint layout (82 bytes; Token-2022 base is identical). */
export function decodeMint(data: Uint8Array): MintInfo {
  if (data.length < 82) throw new RangeError("Not a mint account");
  const v = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const opt = (off: number) => (v.getUint32(off, true) === 1 ? new PublicKey(data.slice(off + 4, off + 36)) : null);
  return {
    mintAuthority: opt(0),
    supply: v.getBigUint64(36, true),
    decimals: data[44]!,
    freezeAuthority: opt(46),
  };
}

/** SPL token account amount (offset 64), owner (32). */
export function decodeTokenAccount(data: Uint8Array): { mint: PublicKey; owner: PublicKey; amount: bigint } {
  if (data.length < 165) throw new RangeError("Not a token account");
  const v = new DataView(data.buffer, data.byteOffset, data.byteLength);
  return { mint: new PublicKey(data.slice(0, 32)), owner: new PublicKey(data.slice(32, 64)), amount: v.getBigUint64(64, true) };
}
