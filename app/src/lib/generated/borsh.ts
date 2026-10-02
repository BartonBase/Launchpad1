/** Minimal little-endian Borsh reader/writer for the hand-written Anchor clients (no deps). */
import { PublicKey } from "@solana/web3.js";

export class Reader {
  private o: number;
  private readonly v: DataView;
  constructor(private readonly b: Uint8Array, offset = 0) {
    this.o = offset;
    this.v = new DataView(b.buffer, b.byteOffset, b.byteLength);
  }
  get offset(): number {
    return this.o;
  }
  private need(n: number): void {
    if (this.o + n > this.b.length) throw new RangeError("Account data too short");
  }
  u8(): number {
    this.need(1);
    return this.b[this.o++]!;
  }
  bool(): boolean {
    return this.u8() !== 0;
  }
  u16(): number {
    this.need(2);
    const x = this.v.getUint16(this.o, true);
    this.o += 2;
    return x;
  }
  u32(): number {
    this.need(4);
    const x = this.v.getUint32(this.o, true);
    this.o += 4;
    return x;
  }
  u64(): bigint {
    this.need(8);
    const x = this.v.getBigUint64(this.o, true);
    this.o += 8;
    return x;
  }
  i64(): bigint {
    this.need(8);
    const x = this.v.getBigInt64(this.o, true);
    this.o += 8;
    return x;
  }
  bytes(n: number): Uint8Array {
    this.need(n);
    const x = this.b.slice(this.o, this.o + n);
    this.o += n;
    return x;
  }
  pubkey(): PublicKey {
    return new PublicKey(this.bytes(32));
  }
  string(max = 1024): string {
    const n = this.u32();
    if (n > max) throw new RangeError("String too long");
    return new TextDecoder().decode(this.bytes(n));
  }
}

export class Writer {
  private parts: number[] = [];
  u8(x: number): this {
    this.parts.push(x & 0xff);
    return this;
  }
  u16(x: number): this {
    this.parts.push(x & 0xff, (x >>> 8) & 0xff);
    return this;
  }
  /** Borsh string: u32 length + UTF-8 bytes. */
  string(s: string): this {
    const b = new TextEncoder().encode(s);
    return this.u32(b.length).raw(b);
  }
  u32(x: number): this {
    for (let i = 0; i < 4; i++) this.parts.push((x >>> (8 * i)) & 0xff);
    return this;
  }
  u64(x: bigint): this {
    if (x < 0n || x > 0xffff_ffff_ffff_ffffn) throw new RangeError("u64 out of range");
    for (let i = 0n; i < 8n; i++) this.parts.push(Number((x >> (8n * i)) & 0xffn));
    return this;
  }
  raw(b: ArrayLike<number>): this {
    for (let i = 0; i < b.length; i++) this.parts.push(b[i]!);
    return this;
  }
  toBuffer(): Buffer {
    // web3.js v1 types TransactionInstruction.data as Buffer; a Uint8Array works at runtime.
    return Uint8Array.from(this.parts) as unknown as Buffer;
  }
}

export function u64le(x: bigint): Uint8Array {
  const w = new Writer().u64(x).toBuffer();
  return w as unknown as Uint8Array;
}
export function u32le(x: number): Uint8Array {
  return new Writer().u32(x).toBuffer() as unknown as Uint8Array;
}
export const utf8 = (s: string) => new TextEncoder().encode(s);

export function hasDiscriminator(data: Uint8Array, disc: readonly number[]): boolean {
  if (data.length < 8) return false;
  for (let i = 0; i < 8; i++) if (data[i] !== disc[i]) return false;
  return true;
}
