import type { Metadata } from "next";
import Link from "next/link";
import { DocLayout, DocSection, Dots } from "@/components/armory/DocLayout";

export const metadata: Metadata = { title: "Bug bounty", description: "What's in scope, how to report, and where rewards stand." };

const TOC = [["status", "Status"], ["scope", "Scope"], ["report", "How to report"], ["rules", "Ground rules"], ["rewards", "Rewards"]] as const;
const b = (s: string) => <b className="text-fg">{s}</b>;

export default function BugBountyPage() {
  return (
    <DocLayout
      title="Bug bounty"
      intro="Help keep Armory safe. Here's what's in scope, how to report, and where rewards stand."
      chips={<><span className="tag tag-demo">Not live yet</span><span className="tag">Rewards to be announced</span></>}
      toc={TOC}
    >
      <DocSection id="status" title="Status">
        <p className="text-muted text-sm">The bug bounty {b("isn't live yet")}. It opens before mainnet, as part of the launch checklist alongside the third-party audit. Reports about the devnet beta are welcome now.</p>
        <div className="flex gap-2"><span className="tag tag-demo">Not live yet</span><span className="tag tag-ua"><i />Unaudited beta</span></div>
      </DocSection>
      <DocSection id="scope" title="Scope" aside={<span className="tag tag-demo">Proposed</span>}>
        <p className="text-muted text-sm">This is a proposed scope, drawn from what Armory runs. The final scope is published when the bounty opens.</p>
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2"><h3 className="text-sm font-semibold">In scope</h3>
            <Dots items={[<>{b("Armory's on-chain programs:")} the launch program and the vault program, covering Plain, Hybrid and Burn.</>, <>{b("The Armory website")} and the transactions it asks you to sign.</>]} />
          </div>
          <div className="space-y-2"><h3 className="text-sm font-semibold">Out of scope</h3>
            <Dots tone="plan" items={[<>{b("Third-party programs:")} Meteora, Switchboard, Metaplex, DEXes and marketplaces. Please report to them directly.</>, <>{b("Social engineering")}, phishing or physical attacks.</>, <>{b("Denial of service")} and spam.</>, <>{b("Issues already listed")} under <Link href="/trust#limits" className="text-accent-text">Known limitations</Link>.</>]} />
          </div>
        </div>
      </DocSection>
      <DocSection id="report" title="How to report">
        <p className="text-muted text-sm">Send reports privately. Please don&apos;t open a public issue or post details until a fix is out.</p>
        <p className="capslot text-sm" data-testid="security-contact">Placeholder: security contact address, to be published</p>
        <Dots items={[<>{b("What's affected:")} the program or page, plus the launch type.</>, <>{b("Steps to reproduce")} on devnet, with transaction signatures if you have them.</>, <>{b("The impact")} you think it has.</>]} />
      </DocSection>
      <DocSection id="rules" title="Ground rules">
        <Dots items={[<>{b("Test on devnet only")}, with test tokens.</>, <>{b("Don't touch other people's funds or data.")}</>, <>{b("Give us reasonable time to fix")} before you publish.</>]} />
        <p className="text-muted text-sm">Safe-harbor terms are published with the bounty.</p>
      </DocSection>
      <DocSection id="rewards" title="Rewards">
        <p className="text-muted text-sm">{b("Rewards to be announced.")} Reward tiers and amounts haven&apos;t been decided. They&apos;ll be published here when the bounty goes live.</p>
      </DocSection>
    </DocLayout>
  );
}
