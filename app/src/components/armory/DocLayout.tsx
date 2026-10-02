import type { ReactNode } from "react";

/** Shared layout for Trust / FAQ / Bug bounty (design a-obsidian trust.html, faq.html, bug-bounty.html):
 * title + intro, sticky in-page nav on the left, section cards on the right. */
export function DocLayout({ title, intro, chips, toc, children }: { title: string; intro: ReactNode; chips?: ReactNode; toc: readonly (readonly [string, string])[]; children: ReactNode }) {
  return (
    <div className="mx-auto max-w-(--container-site) space-y-8 px-4 py-10">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div className="max-w-2xl space-y-2">
          <h1 className="hd text-4xl md:text-5xl">{title}</h1>
          <p className="text-muted">{intro}</p>
        </div>
        {chips && <div className="flex gap-2">{chips}</div>}
      </div>
      <div className="grid grid-cols-[minmax(0,1fr)] gap-6 md:grid-cols-[200px_minmax(0,1fr)]">
        <nav aria-label="On this page" className="md:sticky md:top-32 md:self-start">
          <ul className="flex gap-1 overflow-x-auto text-sm md:flex-col">
            {toc.map(([id, label]) => (
              <li key={id}>
                <a href={`#${id}`} className="text-muted hover:text-fg hover:bg-surface-2 rounded-chip block px-3 py-1.5 whitespace-nowrap">{label}</a>
              </li>
            ))}
          </ul>
        </nav>
        <div className="min-w-0 space-y-6">{children}</div>
      </div>
    </div>
  );
}

export function DocSection({ id, title, aside, children }: { id?: string; title: ReactNode; aside?: ReactNode; children: ReactNode }) {
  return (
    <section id={id} className="card scroll-mt-32 space-y-4 p-6" aria-labelledby={id ? `${id}-h` : undefined}>
      <h2 id={id ? `${id}-h` : undefined} className="flex flex-wrap items-center gap-2 text-xl font-semibold">{title}{aside}</h2>
      {children}
    </section>
  );
}

export function Dots({ items, tone = "ok" }: { items: readonly ReactNode[]; tone?: "ok" | "plan" }) {
  return (
    <ul className="space-y-2 text-sm">
      {items.map((it, i) => (
        <li key={i} className="flex gap-3">
          <span aria-hidden="true" className={`mt-1.5 size-2 shrink-0 rounded-full ${tone === "ok" ? "bg-[var(--arm-color-status-success-default)]" : "border border-[var(--arm-color-accent-text)]"}`} />
          <span className="text-muted">{it}</span>
        </li>
      ))}
    </ul>
  );
}
