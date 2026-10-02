/**
 * Hybrid collection builder: turns the creator's images into a committed NFT collection, hosted on
 * Irys devnet (lib/armory/irys), all in the browser.
 *
 *   images ──► one metadata JSON per NFT (#0 … #N-1; NFT i shows image i mod k, trait "Art")
 *          ──► leaf_i = merkle.rs leaf v2 (traits, derived salt, sha256(image), sha256(json), ar://json)
 *          ──► trait root + schema hash for init_vault
 *          ──► leaf manifest (split into ≤ 90 KB parts) + collection JSON pointing at it
 *
 * The collection JSON (its ar:// URI is committed on-chain by init_vault) carries the manifest
 * header and part ids under `armory`, so anyone can rebuild every leaf and proof later
 * (manifestLeafSource) and check them against the vault's trait root. Rarity is cosmetic.
 */
import type { PublicKey } from "@solana/web3.js";
import { u32le } from "@/lib/generated/borsh";
import { arUri, irysUrl, uploadMany, uploadOne, type IrysSigner, type UploadFile } from "./irys";
import { leafHash, merkleLevels, merkleRoot, sha, staticLeafSource, type LeafPre, type LeafSource } from "./leaves";

export const MANIFEST_VERSION = 1;
/** Trait schema (8 slots, merkle.rs). Slot 0 = which artwork; the rest are unused (0). */
export const TRAIT_SCHEMA = { name: "armory-traits-v1", traits: ["Art", "", "", "", "", "", "", ""] } as const;
const PART_MAX_BYTES = 90 * 1024;

export const hex = (b: Uint8Array) => Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
export function unhex(s: string): Uint8Array {
  if (!/^([0-9a-f]{2})*$/i.test(s)) throw new Error("bad hex");
  return Uint8Array.from(s.match(/../g) ?? [], (x) => parseInt(x, 16));
}
export const schemaHash = () => sha(JSON.stringify(TRAIT_SCHEMA));
export const deriveSalt = (master: Uint8Array, i: number) => sha("armory-salt-v1", master, u32le(i));

export interface ArtImage {
  readonly data: Uint8Array;
  readonly contentType: string;
  readonly name: string;
}
export interface ManifestHeader {
  readonly v: number;
  readonly launchConfig: string;
  readonly size: number;
  readonly schemaHash: string;
  readonly root: string;
  readonly masterSalt: string;
  readonly schema: typeof TRAIT_SCHEMA;
  readonly images: readonly { readonly id: string; readonly sha256: string; readonly type: string }[];
  readonly parts: readonly string[];
}
/** One manifest leaf: [metadata JSON id, sha256(JSON) hex, image index]. */
export type ManifestLeaf = readonly [string, string, number];

export function nftMetadata(p: { collectionName: string; symbol: string; description: string; index: number; imageUrl: string; imageType: string; artNo: number }): Uint8Array {
  return new TextEncoder().encode(
    JSON.stringify({
      name: `${p.collectionName} #${p.index}`,
      symbol: p.symbol,
      description: p.description,
      image: p.imageUrl,
      attributes: [{ trait_type: "Art", value: `#${p.artNo}` }],
      properties: { files: [{ uri: p.imageUrl, type: p.imageType }], category: "image" },
    }),
  );
}

/** Leaf preimages from a manifest (shared by the builder and the reader, so they can't drift). */
export async function leafPres(h: Pick<ManifestHeader, "masterSalt" | "images">, leaves: readonly ManifestLeaf[]): Promise<LeafPre[]> {
  const master = unhex(h.masterSalt);
  return Promise.all(
    leaves.map(async ([jsonId, jsonSha, img], i) => {
      const im = h.images[img];
      if (!im) throw new Error(`Manifest leaf ${i} points at a missing image`);
      return { traitValues: [img, 0, 0, 0, 0, 0, 0, 0], salt: await deriveSalt(master, i), imageSha256: unhex(im.sha256), jsonSha256: unhex(jsonSha), uri: arUri(jsonId) };
    }),
  );
}

export function splitParts(leaves: readonly ManifestLeaf[]): ManifestLeaf[][] {
  const parts: ManifestLeaf[][] = [];
  let cur: ManifestLeaf[] = [];
  let bytes = 64;
  for (const l of leaves) {
    const b = JSON.stringify(l).length + 1;
    if (cur.length > 0 && bytes + b > PART_MAX_BYTES) {
      parts.push(cur);
      cur = [];
      bytes = 64;
    }
    cur.push(l);
    bytes += b;
  }
  if (cur.length) parts.push(cur);
  return parts;
}

export interface BuiltCollection {
  readonly collectionUri: string;
  readonly collectionUrl: string;
  readonly traitRoot: Uint8Array;
  readonly schemaHash: Uint8Array;
  readonly size: number;
  readonly imageUrls: readonly string[];
}

const json = (o: unknown, label: string): UploadFile => ({ data: new TextEncoder().encode(JSON.stringify(o)), contentType: "application/json", label });

/** Uploads images, per-NFT metadata, the leaf manifest and the collection JSON; returns the commitment. */
export async function buildCollection(p: {
  collectionName: string;
  symbol: string;
  description: string;
  images: readonly ArtImage[];
  size: number;
  launchConfig: PublicKey;
  signer: IrysSigner;
  onProgress?: (stage: string, done: number, total: number) => void;
  fetcher?: typeof fetch;
}): Promise<BuiltCollection> {
  const { images, size } = p;
  if (images.length < 1) throw new Error("Add at least one image.");
  if (images.length > size) throw new Error("More images than NFTs.");
  const prog = (s: string) => (d: number, t: number) => p.onProgress?.(s, d, t);
  const imageIds = await uploadMany(images.map((im) => ({ data: im.data, contentType: im.contentType, label: im.name })), p.signer, prog("Images"), 6, p.fetcher);
  const imageShas = await Promise.all(images.map((im) => sha(im.data)));
  const metas = Array.from({ length: size }, (_, i) => {
    const k = i % images.length;
    return nftMetadata({ collectionName: p.collectionName, symbol: p.symbol, description: p.description, index: i, imageUrl: irysUrl(imageIds[k]!), imageType: images[k]!.contentType, artNo: k + 1 });
  });
  const jsonIds = await uploadMany(metas.map((m, i) => ({ data: m, contentType: "application/json", label: `NFT #${i} metadata` })), p.signer, prog("NFT metadata"), 8, p.fetcher);
  const leaves: ManifestLeaf[] = await Promise.all(metas.map(async (m, i) => [jsonIds[i]!, hex(await sha(m)), i % images.length] as const));
  const masterSalt = globalThis.crypto.getRandomValues(new Uint8Array(32));
  const imgTable = images.map((im, k) => ({ id: imageIds[k]!, sha256: hex(imageShas[k]!), type: im.contentType }));
  const schema = await schemaHash();
  const pres = await leafPres({ masterSalt: hex(masterSalt), images: imgTable }, leaves);
  const root = merkleRoot(await merkleLevels(await Promise.all(pres.map((l, i) => leafHash(schema, p.launchConfig, i, l)))));
  const parts = splitParts(leaves);
  const partIds = await uploadMany(parts.map((leavesPart, i) => json({ v: MANIFEST_VERSION, part: i, leaves: leavesPart }, `manifest part ${i}`)), p.signer, prog("Manifest"), 4, p.fetcher);
  const header: ManifestHeader = { v: MANIFEST_VERSION, launchConfig: p.launchConfig.toBase58(), size, schemaHash: hex(schema), root: hex(root), masterSalt: hex(masterSalt), schema: TRAIT_SCHEMA, images: imgTable, parts: partIds };
  const collection = { name: p.collectionName, symbol: p.symbol, description: p.description, image: irysUrl(imageIds[0]!), properties: { files: [{ uri: irysUrl(imageIds[0]!), type: images[0]!.contentType }], category: "image" }, armory: header };
  const colId = await uploadOne(json(collection, "collection metadata"), p.signer, p.fetcher);
  p.onProgress?.("Done", 1, 1);
  return { collectionUri: arUri(colId), collectionUrl: irysUrl(colId), traitRoot: root, schemaHash: schema, size, imageUrls: imageIds.map(irysUrl) };
}

/** Token metadata (Metaplex JSON) for the DBC pool: image + JSON on Irys; returns the https URI. */
export async function uploadTokenMetadata(p: { name: string; symbol: string; description: string; image: ArtImage | null; signer: IrysSigner; fetcher?: typeof fetch }): Promise<string> {
  let image: string | undefined;
  if (p.image) image = irysUrl(await uploadOne({ data: p.image.data, contentType: p.image.contentType, label: "token image" }, p.signer, p.fetcher));
  const id = await uploadOne(json({ name: p.name, symbol: p.symbol, description: p.description, ...(image ? { image } : {}) }, "token metadata"), p.signer, p.fetcher);
  return irysUrl(id);
}

/** Rebuilds the leaf source from a collection JSON (as published by buildCollection). */
export async function manifestLeafSource(collectionJson: unknown, expectedRoot: Uint8Array, launchConfig: PublicKey, fetcher: typeof fetch = fetch): Promise<LeafSource> {
  const h = (collectionJson as { armory?: ManifestHeader }).armory;
  if (!h || h.v !== MANIFEST_VERSION || !Array.isArray(h.parts) || !Array.isArray(h.images)) throw new Error("This collection has no Armory leaf manifest.");
  if (h.launchConfig !== launchConfig.toBase58()) throw new Error("Leaf manifest belongs to a different launch.");
  const leaves: ManifestLeaf[] = [];
  for (const id of h.parts) {
    const r = await fetcher(irysUrl(id));
    if (!r.ok) throw new Error(`Couldn't load manifest part ${id} (${r.status}).`);
    const part = (await r.json()) as { leaves?: ManifestLeaf[] };
    leaves.push(...(part.leaves ?? []));
  }
  if (leaves.length !== h.size) throw new Error(`Leaf manifest has ${leaves.length} leaves, expected ${h.size}.`);
  const schema = unhex(h.schemaHash);
  const src = staticLeafSource("Published Armory leaf manifest", schema, await leafPres(h, leaves));
  const root = await src.root(launchConfig);
  if (hex(root) !== hex(expectedRoot)) throw new Error("Leaf manifest doesn't match the trait root committed on-chain.");
  return src;
}
