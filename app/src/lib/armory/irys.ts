/**
 * Minimal Irys (ANS-104) uploader for devnet art, client-side, no server and no secret.
 *
 * Each file becomes one ANS-104 data item signed by a throwaway ed25519 key generated in the browser
 * (signature type 2, the same format @irys/upload-solana produces), then POSTed to the Irys devnet
 * node. Irys devnet stores items under 100 KB for free, so the wizard downsizes images to fit and the
 * creator pays nothing for hosting; a larger item is rejected with a clear error. The key never
 * leaves the page and holds no funds. Content is addressed by the returned id: `ar://<id>` on-chain
 * (the vault only accepts ipfs:// or ar://), `https://devnet.irys.xyz/<id>` for display.
 */
import { ed25519 } from "@noble/curves/ed25519";
import bs58encode from "./bs58";

export const IRYS_DEVNET_NODE = "https://devnet.irys.xyz";
/** Items at or under this size are free on the Irys devnet node (measured 2026-10-02: 90 KB ok, 120 KB → 402). */
export const IRYS_FREE_BYTES = 100 * 1024;

export interface IrysTag {
  readonly name: string;
  readonly value: string;
}
export interface IrysSigner {
  readonly publicKey: Uint8Array; // 32 bytes
  sign(message: Uint8Array): Uint8Array; // 64 bytes
}

/** Throwaway ed25519 signer (random key; lives only in memory). */
export function throwawaySigner(): IrysSigner {
  const sk = ed25519.utils.randomPrivateKey();
  const pk = ed25519.getPublicKey(sk);
  return { publicKey: pk, sign: (m) => ed25519.sign(m, sk) };
}

const enc = new TextEncoder();
function concat(parts: readonly Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let o = 0;
  for (const p of parts) {
    out.set(p, o);
    o += p.length;
  }
  return out;
}
async function digest(alg: "SHA-256" | "SHA-384", data: Uint8Array): Promise<Uint8Array> {
  return new Uint8Array(await globalThis.crypto.subtle.digest(alg, data as BufferSource));
}

/** Arweave deep hash (SHA-384) over blobs and lists. */
type Chunk = Uint8Array | Chunk[];
export async function deepHash(data: Chunk): Promise<Uint8Array> {
  if (Array.isArray(data)) {
    let acc = await digest("SHA-384", concat([enc.encode("list"), enc.encode(String(data.length))]));
    for (const c of data) acc = await digest("SHA-384", concat([acc, await deepHash(c)]));
    return acc;
  }
  const tag = await digest("SHA-384", concat([enc.encode("blob"), enc.encode(String(data.byteLength))]));
  return digest("SHA-384", concat([tag, await digest("SHA-384", data)]));
}

/** Avro zig-zag varint (long). */
function avroLong(n: number, out: number[]): void {
  let m = n >= 0 ? n * 2 : -n * 2 - 1;
  do {
    let b = m % 128;
    m = Math.floor(m / 128);
    if (m > 0) b |= 0x80;
    out.push(b);
  } while (m > 0);
}
/** ANS-104 tag encoding (Avro array of {name, value} strings). Empty list → no bytes. */
export function serializeTags(tags: readonly IrysTag[]): Uint8Array {
  if (tags.length === 0) return new Uint8Array(0);
  const out: number[] = [];
  avroLong(tags.length, out);
  for (const t of tags) {
    for (const s of [t.name, t.value]) {
      const b = enc.encode(s);
      avroLong(b.length, out);
      out.push(...b);
    }
  }
  avroLong(0, out);
  return Uint8Array.from(out);
}
function le(n: number, bytes: number): Uint8Array {
  const b = new Uint8Array(bytes);
  let v = n;
  for (let i = 0; i < bytes; i++) {
    b[i] = v % 256;
    v = Math.floor(v / 256);
  }
  return b;
}

const SIG_TYPE_ED25519 = 2;

/** Builds and signs one ANS-104 data item (no target). Returns the raw bytes and the item id. */
export async function createDataItem(data: Uint8Array, tags: readonly IrysTag[], signer: IrysSigner, anchor?: Uint8Array): Promise<{ raw: Uint8Array; id: string }> {
  if (signer.publicKey.length !== 32) throw new Error("ed25519 public key must be 32 bytes");
  const a = anchor ?? globalThis.crypto.getRandomValues(new Uint8Array(32));
  if (a.length !== 32) throw new Error("anchor must be 32 bytes");
  const tagBytes = serializeTags(tags);
  const message = await deepHash([
    enc.encode("dataitem"), enc.encode("1"), enc.encode(String(SIG_TYPE_ED25519)),
    signer.publicKey, new Uint8Array(0), a, tagBytes, data,
  ]);
  const signature = signer.sign(message);
  if (signature.length !== 64) throw new Error("ed25519 signature must be 64 bytes");
  const raw = concat([
    le(SIG_TYPE_ED25519, 2), signature, signer.publicKey,
    Uint8Array.of(0), // no target
    Uint8Array.of(1), a, // anchor
    le(tags.length, 8), le(tagBytes.length, 8), tagBytes,
    data,
  ]);
  return { raw, id: bs58encode(await digest("SHA-256", signature)) };
}

export const arUri = (id: string) => `ar://${id}`;
export const irysUrl = (id: string) => `${IRYS_DEVNET_NODE}/${id}`;
/** ar://<id> → the devnet gateway URL (display only); anything else unchanged. */
export function resolveArUri(uri: string): string {
  const m = /^ar:\/\/([1-9A-HJ-NP-Za-km-z]{32,64}|[A-Za-z0-9_-]{43})(\/.*)?$/.exec(uri.trim());
  return m ? `${IRYS_DEVNET_NODE}/${m[1]}${m[2] ?? ""}` : uri;
}

export interface UploadFile {
  readonly data: Uint8Array;
  readonly contentType: string;
  readonly label: string;
}

/** Signs and uploads one file; returns the Irys id. Throws on any non-200 answer. */
export async function uploadOne(f: UploadFile, signer: IrysSigner, fetcher: typeof fetch = fetch): Promise<string> {
  if (f.data.length > IRYS_FREE_BYTES) throw new Error(`${f.label} is ${Math.ceil(f.data.length / 1024)} KB; files must be at most 100 KB on devnet hosting.`);
  const item = await createDataItem(f.data, [{ name: "Content-Type", value: f.contentType }, { name: "App-Name", value: "Armory" }], signer);
  let last = "";
  for (let attempt = 0; attempt < 3; attempt++) {
    const res = await fetcher(`${IRYS_DEVNET_NODE}/tx/solana`, { method: "POST", headers: { "Content-Type": "application/octet-stream" }, body: item.raw as BodyInit });
    if (res.ok) {
      const body = (await res.json().catch(() => ({}))) as { id?: string };
      if (body.id && body.id !== item.id) throw new Error(`Irys returned id ${body.id}, expected ${item.id}`);
      return item.id;
    }
    last = `${res.status} ${await res.text().catch(() => "")}`.slice(0, 160);
    if (res.status === 402 || res.status === 400) break;
    await new Promise((r) => setTimeout(r, 800 * (attempt + 1)));
  }
  throw new Error(`Upload of ${f.label} failed: ${last}`);
}

/** Uploads many files with bounded concurrency; ids come back in input order. */
export async function uploadMany(files: readonly UploadFile[], signer: IrysSigner, onProgress?: (done: number, total: number) => void, concurrency = 6, fetcher: typeof fetch = fetch): Promise<string[]> {
  const ids = new Array<string>(files.length);
  let next = 0;
  let done = 0;
  const worker = async () => {
    while (next < files.length) {
      const i = next++;
      ids[i] = await uploadOne(files[i]!, signer, fetcher);
      onProgress?.(++done, files.length);
    }
  };
  await Promise.all(Array.from({ length: Math.min(concurrency, files.length) }, worker));
  return ids;
}
