/**
 * Token metadata hosting for the Launch form: the creator's image + a Metaplex-style metadata JSON are uploaded to
 * Arweave (permanent, public) through ArDrive Turbo's free unsigned-upload endpoint. No account, key or payment:
 * Turbo signs and pays for small items (≤ 105 KiB each). Nothing here runs without an explicit click, and the form
 * tells the creator the files are public and permanent before uploading.
 *
 * The resulting `https://arweave.net/<id>` URL of the JSON is what goes into the token's on-chain metadata URI.
 */

export const TURBO_UPLOAD_URL = "https://upload.ardrive.io/v1/x402/data-item/unsigned";
export const ARWEAVE_GATEWAY = "https://arweave.net";
/** Turbo's free tier: items up to 105 KiB. */
export const MAX_FREE_BYTES = 107_520;
/** Target for the downscaled token image (leaves room under MAX_FREE_BYTES). */
export const IMAGE_TARGET_BYTES = 100_000;
export const IMAGE_MAX_PX = 512;

export class UploadError extends Error {}

export interface TokenMetadataInput {
  readonly name: string;
  readonly symbol: string;
  readonly description?: string;
  readonly image?: string;
}

/** Metaplex fungible-token metadata JSON (what wallets and explorers read from the URI). */
export function buildMetadataJson(m: TokenMetadataInput): string {
  const out: Record<string, string> = { name: m.name.trim(), symbol: m.symbol.trim() };
  const d = m.description?.trim();
  if (d) out.description = d.slice(0, 500);
  if (m.image) out.image = m.image;
  return JSON.stringify(out);
}

/** Uploads one item; returns its permanent gateway URL. */
export async function uploadToArweave(bytes: Uint8Array, contentType: string, fetchImpl: typeof fetch = fetch): Promise<string> {
  if (bytes.byteLength === 0) throw new UploadError("Nothing to upload.");
  if (bytes.byteLength > MAX_FREE_BYTES) throw new UploadError(`File is ${Math.ceil(bytes.byteLength / 1024)} KB; the free upload limit is ${MAX_FREE_BYTES / 1024} KB.`);
  let res: Response;
  try {
    res = await fetchImpl(TURBO_UPLOAD_URL, {
      method: "POST",
      headers: {
        "Content-Type": "application/octet-stream",
        "x-data-item-tags": JSON.stringify([
          { name: "Content-Type", value: contentType },
          { name: "App-Name", value: "Armory" },
        ]),
      },
      body: bytes as unknown as BodyInit,
    });
  } catch {
    throw new UploadError("Couldn't reach the Arweave upload service. Check your connection and try again.");
  }
  if (!res.ok) throw new UploadError(res.status === 429 || res.status === 402 ? "The free Arweave upload limit was reached. Paste your own metadata URI instead." : `Arweave upload failed (HTTP ${res.status}).`);
  const body = (await res.json().catch(() => null)) as { id?: unknown } | null;
  const id = typeof body?.id === "string" ? body.id : "";
  if (!/^[A-Za-z0-9_-]{43}$/.test(id)) throw new UploadError("Arweave upload returned an unexpected response.");
  return `${ARWEAVE_GATEWAY}/${id}`;
}

/** Browser only: downscale an image to ≤ 512 px WebP under IMAGE_TARGET_BYTES. */
export async function downscaleImage(file: Blob): Promise<Uint8Array> {
  const bmp = await createImageBitmap(file);
  const scale = Math.min(1, IMAGE_MAX_PX / Math.max(bmp.width, bmp.height));
  let w = Math.max(1, Math.round(bmp.width * scale));
  let h = Math.max(1, Math.round(bmp.height * scale));
  for (let attempt = 0; attempt < 8; attempt++) {
    const canvas = document.createElement("canvas");
    canvas.width = w;
    canvas.height = h;
    canvas.getContext("2d")!.drawImage(bmp, 0, 0, w, h);
    const quality = Math.max(0.4, 0.85 - attempt * 0.1);
    const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, "image/webp", quality));
    if (!blob) break;
    if (blob.size <= IMAGE_TARGET_BYTES) return new Uint8Array(await blob.arrayBuffer());
    if (attempt >= 3) {
      w = Math.round(w * 0.8);
      h = Math.round(h * 0.8);
    }
  }
  throw new UploadError("Couldn't shrink this image under 100 KB. Try a simpler PNG or JPG.");
}

/** Full flow: optional image, then the JSON pointing at it. Returns the JSON URL (the token's metadata URI). */
export async function uploadTokenMetadata(m: Omit<TokenMetadataInput, "image"> & { imageBytes?: Uint8Array }, fetchImpl: typeof fetch = fetch): Promise<{ uri: string; image?: string }> {
  const image = m.imageBytes ? await uploadToArweave(m.imageBytes, "image/webp", fetchImpl) : undefined;
  const json = new TextEncoder().encode(buildMetadataJson({ ...m, image }));
  const uri = await uploadToArweave(json, "application/json", fetchImpl);
  return { uri, image };
}
