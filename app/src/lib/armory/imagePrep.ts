/**
 * Browser-side image preparation for devnet hosting: decode, scale down and re-encode (WebP, then
 * JPEG as a fallback) until the file fits the Irys devnet free-item limit. Square-ish art keeps its
 * aspect ratio. Browser only (uses createImageBitmap + canvas).
 */
import { IRYS_FREE_BYTES } from "./irys";
import type { ArtImage } from "./art";

const TARGET = IRYS_FREE_BYTES - 4 * 1024; // headroom for the data-item envelope
const ACCEPT = /^image\/(png|jpe?g|webp|gif)$/;

export async function prepareImage(file: File): Promise<ArtImage> {
  if (!ACCEPT.test(file.type)) throw new Error(`${file.name}: use a PNG, JPEG, WebP or GIF image.`);
  if (file.size <= TARGET && file.type !== "image/gif") return { data: new Uint8Array(await file.arrayBuffer()), contentType: file.type, name: file.name };
  const bmp = await createImageBitmap(file);
  try {
    for (const side of [1024, 768, 512, 384, 256]) {
      const scale = Math.min(1, side / Math.max(bmp.width, bmp.height));
      const w = Math.max(1, Math.round(bmp.width * scale));
      const h = Math.max(1, Math.round(bmp.height * scale));
      const canvas = document.createElement("canvas");
      canvas.width = w;
      canvas.height = h;
      const ctx = canvas.getContext("2d");
      if (!ctx) throw new Error("Canvas unavailable");
      ctx.drawImage(bmp, 0, 0, w, h);
      for (const [type, q] of [["image/webp", 0.85], ["image/webp", 0.7], ["image/jpeg", 0.8], ["image/jpeg", 0.65]] as const) {
        const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, type, q));
        if (blob && blob.size <= TARGET && blob.type === type) return { data: new Uint8Array(await blob.arrayBuffer()), contentType: type, name: file.name };
      }
    }
  } finally {
    bmp.close();
  }
  throw new Error(`${file.name}: couldn't shrink this image under 96 KB.`);
}
