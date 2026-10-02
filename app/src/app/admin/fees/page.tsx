import type { Metadata } from "next";
import { AdminFeesView } from "@/components/meteora/FeeClaims";

// Not linked anywhere and not indexed. Anyone can read it (all numbers are public chain data); only the platform
// config's fee wallet can sign claims.
export const metadata: Metadata = { title: "Platform fees", robots: { index: false, follow: false } };

export default function AdminFeesPage() {
  return (
    <div className="mx-auto max-w-(--container-site) space-y-8 px-4 py-10">
      <div className="space-y-2">
        <h1 className="hd text-4xl">Platform fees</h1>
        <p className="text-muted max-w-2xl">Armory&apos;s share of the trading fees on every Launch-type coin, per pool. Claims go to the fee wallet.</p>
      </div>
      <AdminFeesView />
    </div>
  );
}
