import Image from "next/image";

/** Alt text per the CD (2026-10-01). Placeholder art only: never a logo, favicon or icon. */
export const MASCOT_ALT = "Placeholder art, mascot coming soon";

/**
 * TEMPORARY knight placeholder (design/placeholder/knight-placeholder.svg, copied to public/mascot/)
 * in the mascot slots: hero, curve progress, graduation, convert panel. Swap the file when the
 * Higgsfield art arrives. SVG is served as an <img> (no script execution) and unoptimized, so
 * next/image needs no dangerouslyAllowSVG.
 */
export function MascotPlaceholder({ className = "", size = 168 }: { className?: string; size?: number }) {
  return (
    <figure className={`mascot shrink-0 ${className}`} style={{ width: size }} data-testid="mascot-placeholder">
      <Image src="/mascot/knight-placeholder.svg" alt={MASCOT_ALT} width={size} height={size} unoptimized />
      {size >= 120 && <figcaption className="text-dim mt-1 text-center text-[11px]">Placeholder art</figcaption>}
    </figure>
  );
}
