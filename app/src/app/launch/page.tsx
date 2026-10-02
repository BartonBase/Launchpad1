import type { Metadata } from "next";
import { CLUSTER } from "@/config/cluster";
import { LaunchWizard } from "@/components/armory/LaunchWizard";
import type { WizardType } from "@/lib/armory/wizardMath";

export const metadata: Metadata = { title: "Launch", description: "Launch a Plain, Hybrid or Burn token on Armory (devnet beta)." };

export default async function LaunchPage({ searchParams }: { searchParams: Promise<{ type?: string }> }) {
  const q = (await searchParams).type;
  const type: WizardType = q === "plain" || q === "burn" ? q : "hybrid";
  return (
    <div className="mx-auto max-w-(--container-site) space-y-8 px-4 py-10">
      <div className="space-y-2">
        <span className="tag tag-ua">Unaudited beta · {CLUSTER.label} · test tokens only</span>
        <h1 className="hd text-4xl md:text-5xl">Launch a token</h1>
        <p className="text-muted max-w-2xl">
          Pick a launch type, then set it up. Plain is just the coin; Hybrid and Burn add an NFT collection. The type and anything marked Permanent
          can&apos;t change once you launch.
        </p>
      </div>
      {/* key: re-mount when ?type changes */}
      <LaunchWizard key={type} initialType={type} />
    </div>
  );
}
