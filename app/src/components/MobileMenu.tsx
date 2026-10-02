"use client";
import Link from "next/link";
import { useId, useState } from "react";

/** Mobile nav disclosure (a11y.md known gap: real button with aria-expanded). Hidden from md up. */
export function MobileMenu({ nav }: { nav: readonly { href: string; label: string }[] }) {
  const [open, setOpen] = useState(false);
  const id = useId();
  return (
    <div className="md:hidden">
      <button type="button" className="btn btn-sm h-10 w-10 p-0" aria-expanded={open} aria-controls={id} aria-label={open ? "Close menu" : "Open menu"} onClick={() => setOpen((o) => !o)} data-testid="mobile-menu-button">
        <svg aria-hidden="true" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
          {open ? <path d="M6 6l12 12M18 6L6 18" /> : <path d="M4 7h16M4 12h16M4 17h16" />}
        </svg>
      </button>
      {open && (
        <div id={id} className="border-border bg-surface absolute inset-x-0 top-full z-50 space-y-3 border-b p-4" data-testid="mobile-menu">
          <nav aria-label="Mobile">
            <ul className="space-y-1">
              {nav.map((n) => (
                <li key={n.href}>
                  <Link href={n.href} onClick={() => setOpen(false)} className="hover:bg-surface-2 rounded-chip block px-3 py-2.5">{n.label}</Link>
                </li>
              ))}
            </ul>
          </nav>
          <form action="/explore" role="search" onSubmit={() => setOpen(false)}>
            <input name="q" maxLength={64} className="input" placeholder="Search tokens" aria-label="Search tokens" />
          </form>
        </div>
      )}
    </div>
  );
}
