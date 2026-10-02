import { createElement } from "react";
import type { LaunchTypeId } from "@/config/armory";
import { TYPE_ICON_ELEMENTS } from "./typeIcons";

/**
 * Launch-type line icon (design/system/icons, 24px grid, 1.75 stroke, currentColor). Shapes are
 * generated from the design repo's static SVGs (tag + geometry attributes only, allow-listed).
 */
export function TypeIcon({ type, size = 24, className = "" }: { type: LaunchTypeId; size?: number; className?: string }) {
  return (
    <svg aria-hidden="true" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.75} strokeLinecap="round" strokeLinejoin="round" className={`shrink-0 ${className}`}>
      {TYPE_ICON_ELEMENTS[type].map(([tag, attrs], i) => createElement(tag, { key: i, ...attrs }))}
    </svg>
  );
}
