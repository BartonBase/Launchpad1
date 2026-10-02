import Image from "next/image";

/**
 * Official Armory knight mascot art (Barton, 2026-10-02). Transparent cut-outs in
 * public/brand/knights/ (source PNGs + WebP; originals in design/mascot/knights/). Served through
 * next/image so each slot gets a right-sized WebP/AVIF. Never animated (motion is zero).
 *
 * Where each one is used:
 *   B (steel + orange, round medallion): home hero, header/footer bust logo, favicon/app icons, OG image
 *   C (grey steel, orange accents):      "How it works" page, launch success
 *   A (black armor, copper shield):      Portfolio and Explore empty states
 *   D (black armor, orange crest):       404 page
 */
export const KNIGHTS = {
  a: { src: "/brand/knights/knight-a.webp", w: 768, h: 1106, alt: "Armory knight in black armor with a copper shield badge" },
  b: { src: "/brand/knights/knight-b.webp", w: 768, h: 1089, alt: "Armory knight in steel armor with orange trim and a copper medallion" },
  c: { src: "/brand/knights/knight-c.webp", w: 768, h: 1122, alt: "Armory knight in grey steel armor with orange accents" },
  d: { src: "/brand/knights/knight-d.webp", w: 768, h: 1120, alt: "Armory knight in black armor with an orange shoulder crest" },
} as const;
export type KnightId = keyof typeof KNIGHTS;

/**
 * A knight bust. `fade` softens the cropped bottom/side edges of the art into the page; `glow`
 * adds a soft ember radial gradient behind it. Decorative by default (empty alt).
 */
export function Knight({
  k,
  sizes,
  className = "",
  priority = false,
  fade = true,
  glow = false,
  decorative = true,
}: {
  k: KnightId;
  sizes: string;
  className?: string;
  priority?: boolean;
  fade?: boolean;
  glow?: boolean;
  decorative?: boolean;
}) {
  const n = KNIGHTS[k];
  return (
    <div className={`knight ${glow ? "knight-glow" : ""} ${className}`} data-testid={`knight-${k}`}>
      <Image
        src={n.src}
        width={n.w}
        height={n.h}
        alt={decorative ? "" : n.alt}
        sizes={sizes}
        priority={priority}
        className={fade ? "knight-fade" : undefined}
      />
    </div>
  );
}

/** Head-and-shoulders bust of knight B (plume, helmet, shoulders) next to the wordmark (header, footer). */
export function HelmetMark({ size = 38, className = "" }: { size?: number; className?: string }) {
  return (
    <Image
      src="/brand/knights/knight-b-bust-128.webp"
      width={size}
      height={size}
      alt=""
      unoptimized
      priority
      className={`helmet-mark shrink-0 ${className}`}
      data-testid="helmet-mark"
    />
  );
}
