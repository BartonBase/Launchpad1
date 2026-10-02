/**
 * Always-visible beta strip (sticky with the header): unaudited-beta label, deposit cap and a
 * disclosure with the locked-authorities panel. No client JS: <details> is native and keyboard
 * accessible.
 */
import Link from "next/link";
import { BETA_DEPOSIT_CAP } from "@/config/armory";
import { CLUSTER } from "@/config/cluster";
import { AuthoritiesPanel } from "./AuthoritiesPanel";

export function TrustStrip() {
  return (
    <div className="bbar" data-testid="trust-strip">
      <div className="mx-auto flex max-w-(--container-site) items-center gap-x-2 gap-y-1.5 px-4 py-1.5 whitespace-nowrap sm:flex-wrap sm:gap-x-4">
        <span className="tag tag-ua" data-testid="beta-label">
          <i />Unaudited beta
        </span>
        <span className="hidden sm:inline">
          Programs are <b>not audited</b> yet. {CLUSTER.label} only, test funds.
        </span>
        <span data-testid="deposit-cap">
          Deposit cap <b className="num">{BETA_DEPOSIT_CAP.sol} SOL per wallet</b>{" "}
          <span className="tag tag-demo hidden sm:inline-flex" title={BETA_DEPOSIT_CAP.note}>
            Example
          </span>
        </span>
        <Link href="/trust" className="text-accent-text ml-auto font-medium sm:hidden">
          → Trust
        </Link>
        <details className="group relative ml-auto hidden sm:block" data-testid="authorities-toggle">
          <summary className="text-accent-text cursor-pointer list-none font-medium select-none [&::-webkit-details-marker]:hidden">
            Authorities <span aria-hidden="true">▾</span>
          </summary>
          <div className="card bg-surface shadow-float absolute right-0 z-50 mt-2 w-[min(92vw,30rem)] p-4">
            <AuthoritiesPanel compact />
            <Link href="/trust" className="text-accent-text mt-2 inline-block font-medium">
              How Armory is secured →
            </Link>
          </div>
        </details>
      </div>
    </div>
  );
}
