/**
 * Token images for launches whose metadata is hosted by this site (public/tokens/<slug>.json, with
 * the image next to it as <slug>.png). Served from our own origin so the CSP (img-src 'self') holds;
 * any other metadata URI falls back to the letter placeholder.
 */
const HOSTED = /^https:\/\/armory-ten\.vercel\.app\/tokens\/([a-z0-9-]{1,40})\.json$/;

export function tokenImage(uri: string | null | undefined): string | null {
  const m = HOSTED.exec((uri ?? "").trim());
  return m ? `/tokens/${m[1]}.png` : null;
}
