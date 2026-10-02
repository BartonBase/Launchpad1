import Link from "next/link";
import { BRAND } from "@/config/armory";
import { ClusterBadge } from "./ClusterBadge";
import { WalletButton } from "./WalletButton";
import { MobileMenu } from "./MobileMenu";

const NAV = [
  { href: "/explore", label: "Explore" },
  { href: "/launch", label: "Launch" },
  { href: "/portfolio", label: "Portfolio" },
  { href: "/trust", label: "Trust" },
  { href: "/faq", label: "FAQ" },
] as const;

export function SiteHeader() {
  return (
    <header className="sticky top-0 z-40">
      <div className="border-border bg-nav relative border-b backdrop-blur">
        <div className="mx-auto flex max-w-(--container-site) items-center gap-3 px-4 py-2.5 md:gap-6">
          <Link href="/" className="wordmark text-fg flex items-center gap-2" aria-label={`${BRAND.name} home`} data-testid="wordmark">
            {BRAND.name}
          </Link>
          <nav aria-label="Main" className="hidden min-w-0 flex-1 gap-1 text-sm md:flex">
            {NAV.map((n) => (
              <Link key={n.href} href={n.href} className="text-muted hover:text-fg hover:bg-surface-2 rounded-chip px-2.5 py-1.5 whitespace-nowrap">
                {n.label}
              </Link>
            ))}
          </nav>
          <form action="/explore" role="search" className="hidden lg:block">
            <input name="q" maxLength={64} className="input h-9 w-52 text-sm" placeholder="Search tokens" aria-label="Search tokens" data-testid="nav-search" />
          </form>
          <div className="ml-auto flex items-center gap-3 md:ml-0">
            <span className="hidden sm:inline">
              <ClusterBadge />
            </span>
            <WalletButton />
            <MobileMenu nav={NAV} />
          </div>
        </div>
      </div>
    </header>
  );
}
