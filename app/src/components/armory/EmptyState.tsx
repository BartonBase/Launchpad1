import type { ReactNode } from "react";
import { Knight, type KnightId } from "./Knight";

/** Empty / not-found state with a small knight from the official mascot art. */
export function EmptyState({
  k,
  title,
  children,
  testId,
  className = "",
  as: Heading = "h2",
}: {
  k: KnightId;
  title: string;
  children?: ReactNode;
  testId?: string;
  className?: string;
  as?: "h1" | "h2" | "p";
}) {
  return (
    <div className={`card empty-state flex flex-col items-center gap-5 overflow-hidden px-6 pt-6 pb-8 text-center sm:flex-row sm:items-end sm:gap-7 sm:pt-5 sm:pb-0 sm:text-left ${className}`} data-testid={testId}>
      <Knight k={k} sizes="150px" className="w-[118px] shrink-0 sm:w-[150px]" />
      <div className="space-y-2 sm:pb-9">
        <Heading className={Heading === "h1" ? "hd text-3xl md:text-4xl" : "text-lg font-semibold"}>{title}</Heading>
        <div className="text-muted space-y-2 text-sm">{children}</div>
      </div>
    </div>
  );
}
