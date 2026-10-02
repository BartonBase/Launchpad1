import type { Metadata } from "next";
import { LaunchWizard } from "@/components/armory/LaunchWizard";
import type { WizardType } from "@/lib/armory/wizardMath";
import { CLUSTER } from "@/config/cluster";

export const metadata: Metadata = { title: "Launch", description: "Launch a Solana meme coin on a bonding curve, on its own or with an NFT collection built in." };

export default async function LaunchPage({ searchParams }: { searchParams: Promise<{ type?: string }> }) {
  const q = (await searchParams).type;
  // Burn is "Coming soon": /launch?type=burn opens the Hybrid wizard with a notice.
  const type: WizardType = q === "plain" || CLUSTER.isMainnet ? "plain" : "hybrid";
  return (
    <div className="mx-auto max-w-(--container-site) space-y-8 px-4 py-10">
      <div className="space-y-2">
        <h1 className="hd text-4xl md:text-5xl">Launch a token</h1>
        <p className="text-muted max-w-2xl">
          {CLUSTER.isMainnet
            ? "Launch your coin on a bonding curve. Hybrid, which adds an NFT collection, is coming soon. The token settings can't change once you launch."
            : "Pick a launch type, then set it up. Launch is just the coin, on a bonding curve; Hybrid adds an NFT collection. The type and the token settings can't change once you launch."}
        </p>
      </div>
      {q === "burn" && (
        <p className="card-soon rounded-panel border p-3 text-sm" data-testid="launch-burn-soon">
          <span className="tag tag-soon mr-1">Coming soon</span> Burn launches aren&apos;t available yet. {CLUSTER.isMainnet ? "Use Launch below." : "Pick Launch or Hybrid below."}
        </p>
      )}
      {/* key: re-mount when ?type changes */}
      <LaunchWizard key={type} initialType={type} />
    </div>
  );
}
