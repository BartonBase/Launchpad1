/** Read-only Metaplex Core decoders (AssetV1 owner, CollectionV1 name/uri/counts). */
import { PublicKey } from "@solana/web3.js";
import { Reader } from "./borsh";

const KEY_ASSET_V1 = 1;
const KEY_COLLECTION_V1 = 5;

export function decodeCoreAssetOwner(data: Uint8Array): PublicKey | null {
  if (data.length < 33 || data[0] !== KEY_ASSET_V1) return null; // burned assets keep 1 byte
  return new PublicKey(data.slice(1, 33));
}

/** AssetV1 uri (key, owner, update authority enum, name, uri). Null for burned / unknown layouts. */
export function decodeCoreAssetUri(data: Uint8Array): string | null {
  try {
    if (data.length < 34 || data[0] !== KEY_ASSET_V1) return null;
    const tag = data[33];
    const r = new Reader(data, 34 + (tag === 1 || tag === 2 ? 32 : 0));
    r.string(64);
    return r.string(256);
  } catch {
    return null;
  }
}

export interface CoreCollection {
  readonly updateAuthority: PublicKey;
  readonly name: string;
  readonly uri: string;
  readonly numMinted: number;
  readonly currentSize: number;
}
export function decodeCoreCollection(data: Uint8Array): CoreCollection {
  if (data[0] !== KEY_COLLECTION_V1) throw new Error("Not a Core collection");
  const r = new Reader(data, 1);
  const updateAuthority = r.pubkey();
  const name = r.string(64);
  const uri = r.string(256);
  return { updateAuthority, name, uri, numMinted: r.u32(), currentSize: r.u32() };
}
