/**
 * Token images for launches whose metadata is hosted by this site (public/tokens/<slug>.json, with
 * the image next to it as <slug>.png). Served from our own origin so the CSP (img-src 'self') holds;
 * Metadata uploaded by the launch wizard to the Irys devnet node (https://devnet.irys.xyz/<id>) is
 * resolved server-side by resolveTokenImage: its `image` must be another Irys devnet item, which the
 * devnet CSP allows. Any other metadata URI falls back to the letter placeholder.
 */
import { CLUSTER } from "@/config/cluster";

const IRYS_ITEM = /^https:\/\/devnet\.irys\.xyz\/[1-9A-HJ-NP-Za-km-z]{32,64}$/;
const HOSTED = /^https:\/\/armory-ten\.vercel\.app\/tokens\/([a-z0-9-]{1,40})\.json$/;

export function tokenImage(uri: string | null | undefined): string | null {
  const m = HOSTED.exec((uri ?? "").trim());
  return m ? `/tokens/${m[1]}.png` : null;
}

/** Hosted image, or (devnet) the Irys image named by Irys-hosted metadata. Never throws. */
export async function resolveTokenImage(uri: string | null | undefined, fetcher: typeof fetch = fetch): Promise<string | null> {
  const hosted = tokenImage(uri);
  if (hosted) return hosted;
  const u = (uri ?? "").trim();
  if (CLUSTER.name !== "devnet" || !IRYS_ITEM.test(u)) return null;
  try {
    const res = await fetcher(u, { signal: AbortSignal.timeout(3000), next: { revalidate: 3600 } } as RequestInit);
    if (!res.ok) return null;
    const j = (await res.json()) as { image?: unknown };
    return typeof j.image === "string" && IRYS_ITEM.test(j.image) ? j.image : null;
  } catch {
    return null;
  }
}
