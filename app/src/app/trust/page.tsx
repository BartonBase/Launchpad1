import type { Metadata } from "next";
import Link from "next/link";
import { BETA_DEPOSIT_CAP, PROGRAM_UPGRADES_COPY } from "@/config/armory";
import { CLUSTER, explorerAddressUrl } from "@/config/cluster";
import { HYBRID_LAUNCH_PROGRAM_ID, HYBRID_VAULT_PROGRAM_ID, SWITCHBOARD_PROGRAM_ID } from "@/config/programs";
import { AuthoritiesPanel } from "@/components/armory/AuthoritiesPanel";
import { DocLayout, DocSection, Dots } from "@/components/armory/DocLayout";
import { PipelineSelfTest } from "@/components/PipelineSelfTest";

export const metadata: Metadata = { title: "Trust & security", description: "What's locked, what isn't yet, and what happens before Armory goes to mainnet." };

const TOC = [
  ["audit", "Audit status"], ["authorities", "Locked authorities"], ["upgrades", "Program upgrades"], ["keys", "Key policy"],
  ["randomness", "Randomness"], ["fees", "Fees"], ["cap", "Deposit cap"], ["limits", "Known limitations"], ["addresses", "Program addresses"],
] as const;

const OK = "text-[var(--arm-color-status-success-text)]";
const AUTH: [string, string, string, string][] = [
  ["Mint authority", "Revoked", "Revoked", "Revoked"],
  ["Freeze authority", "Never set", "Never set", "Never set"],
  ["Token metadata", "Immutable", "Immutable", "Immutable"],
  ["Launch settings", "Locked", "Locked", "Locked"],
  ["Pause switch", "None in Armory", "None", "None"],
  ["Program upgrades", "Not locked yet", "Not locked yet", "Not locked yet"],
  ["NFT collection", "Not applicable", "Locked", "Program-held"],
  ["Vault", "Not applicable", "No withdrawals", "Not applicable"],
  ["Fee recipient", "Not applicable", "Fixed in code", "Fixed in code"],
];
const neutral = (v: string) => v === "Not locked yet" || v === "Not applicable";

function Table({ head, rows, okCells = false }: { head: string[]; rows: string[][]; okCells?: boolean }) {
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[560px] text-sm">
        <thead className="text-dim text-left font-mono text-xs uppercase">
          <tr>{head.map((h) => <th key={h} scope="col" className="border-border border-b p-2.5 font-medium">{h}</th>)}</tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r[0]} className="border-border border-b last:border-0">
              <th scope="row" className="p-2.5 text-left font-medium">{r[0]}</th>
              {r.slice(1).map((c, i) => <td key={i} className={`p-2.5 ${okCells && !neutral(c) ? OK : "text-muted"}`}>{c}</td>)}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function TrustPage() {
  return (
    <DocLayout
      title="Trust & security"
      intro="What's locked, what isn't yet, and what happens before Armory goes to mainnet. Plain answers, checked against our own engineering records."
      chips={<><span className="tag tag-ua"><i />Unaudited beta</span><span className="tag">{CLUSTER.label}</span></>}
      toc={TOC}
    >
      <dl className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4" data-testid="trust-summary">
        {[
          ["Audit", "Not audited yet", "Internal reviews done. A paid third-party audit comes before mainnet.", true],
          ["Network", `Solana ${CLUSTER.label.toLowerCase()} only`, "Test tokens and demo data. No real funds.", false],
          ["Deposit cap", `${BETA_DEPOSIT_CAP.sol} SOL per wallet`, "Example. The real value isn't set yet.", false],
          ["Program upgrades", "Not locked yet", "Dev key now · multisig planned. Frozen after the audit.", true],
        ].map(([k, v, d, warn]) => (
          <div key={k as string} className="card p-4">
            <dt className="text-muted text-xs">{k}</dt>
            <dd className={`mt-1 font-semibold ${warn ? "text-warning" : ""}`}>{v}</dd>
            <dd className="text-muted mt-1 text-xs">{d}</dd>
          </div>
        ))}
      </dl>

      <DocSection id="audit" title="Audit status">
        <p className="text-muted text-sm">Armory is an <b className="text-fg">unaudited beta</b>. The programs run on Solana {CLUSTER.label.toLowerCase()} with test tokens. Treat everything here as experimental.</p>
        <div className="grid gap-3 md:grid-cols-2">
          <div className="bg-surface-2 rounded-panel border-border space-y-3 border p-4">
            <h3 className="text-sm font-semibold">Done so far</h3>
            <Dots items={[<><b className="text-fg">Two internal security reviews</b> of the programs, merged into one list of 41 findings.</>, <><b className="text-fg">Each closed finding has a regression test</b> that fails if the bug comes back.</>, <><b className="text-fg">Still open:</b> moving upgrade keys to a multisig, among others.</>]} />
          </div>
          <div className="bg-surface-2 rounded-panel border-border space-y-3 border p-4">
            <h3 className="text-sm font-semibold">Planned before mainnet</h3>
            <Dots tone="plan" items={[<><b className="text-fg">A paid, professional third-party audit.</b></>, <><b className="text-fg">A live bug bounty</b> (<Link href="/bug-bounty" className="text-accent-text">details</Link>).</>, <><b className="text-fg">The deposit cap switched on.</b></>, <><b className="text-fg">Upgrade keys held only by humans</b>, in a 3-of-5 multisig.</>, <><b className="text-fg">A verifiable build</b>, with the audited commit and build hash published.</>]} />
          </div>
        </div>
        <p className="text-muted text-sm">The &quot;Unaudited beta&quot; labels come off only after the audit report is published.</p>
      </DocSection>

      <DocSection id="authorities" title="Locked authorities, explained">
        <p className="text-muted text-sm">An &quot;authority&quot; is a key that&apos;s allowed to change something on-chain. Here&apos;s what each one means and where it stands for each launch type.</p>
        <Dots items={[<><b className="text-fg">Mint authority.</b> The power to create new tokens. Revoked means the supply can never grow.</>, <><b className="text-fg">Freeze authority.</b> The power to lock tokens inside someone&apos;s wallet. It&apos;s never set on Armory tokens.</>, <><b className="text-fg">Metadata update authority.</b> The power to change a token&apos;s name, ticker or image. Launches must be created without one.</>, <><b className="text-fg">Upgrade authority.</b> The power to replace a program&apos;s code. This is the one power that still exists during beta. Here&apos;s who holds it and how it ends.</>]} />
        <Table head={["Authority", "Plain", "Hybrid", "Burn"]} rows={AUTH} okCells />
        <p className="text-dim text-xs">Plain and Burn are built but pending deploy on {CLUSTER.label}; their column describes the program code.</p>
        <div className="border-border border-t pt-4">
          <h3 className="mb-2 text-sm font-semibold">Read live from {CLUSTER.label}</h3>
          <AuthoritiesPanel />
        </div>
      </DocSection>

      <DocSection id="upgrades" title="Program upgrades">
        <p className="text-muted text-sm">Upgrades exist so bugs found before and during the audit can be fixed. They end once the code is stable.</p>
        <ol className="grid gap-3 md:grid-cols-3" data-testid="upgrades-copy-trust">
          <li className="rounded-panel border border-[var(--arm-color-accent-border)] p-4">
            <p className="text-dim font-mono text-xs uppercase">Now · {CLUSTER.label}</p>
            <h3 className="mt-1 font-semibold">{PROGRAM_UPGRADES_COPY.today}</h3>
            <p className="text-muted mt-1 text-xs">Test keys on a test network, not mainnet keys. This is a known open item.</p>
          </li>
          <li className="bg-surface-2 rounded-panel border-border border p-4">
            <p className="text-dim font-mono text-xs uppercase">Mainnet</p>
            <h3 className="mt-1 font-semibold">{PROGRAM_UPGRADES_COPY.planned}</h3>
            <p className="text-muted mt-1 text-xs">Any upgrade needs 3 of 5 signers and then waits 7 days in public before it can run. Signers are named before mainnet.</p>
          </li>
          <li className="bg-surface-2 rounded-panel border-border border p-4">
            <p className="text-dim font-mono text-xs uppercase">After audit + stabilization</p>
            <h3 className="mt-1 font-semibold">Upgrades switched off</h3>
            <p className="text-muted mt-1 text-xs">The upgrade authority is removed, so the code can never change again. How long stabilization lasts isn&apos;t decided yet.</p>
          </li>
        </ol>
      </DocSection>

      <DocSection id="keys" title="Key policy">
        <div className="rounded-panel border border-[var(--arm-color-accent-border)] bg-[var(--arm-color-accent-tint)] p-4">
          <p className="hd text-lg">No AI agent holds mainnet keys. Mainnet keys are held by humans only.</p>
          <p className="text-muted mt-1 text-sm">This applies to every mainnet key: program upgrades, the multisig, and the platform fee address.</p>
        </div>
        <Dots items={[<><b className="text-fg">Today:</b> devnet uses throwaway test keys. No mainnet keys exist on our development machines.</>, <><b className="text-fg">At mainnet (planned):</b> the upgrade authority is a 3-of-5 multisig held by people. A single person&apos;s key is never enough.</>]} />
      </DocSection>

      <DocSection id="randomness" title="Randomness and fairness">
        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-2"><h3 className="text-sm font-semibold">Hybrid</h3>
            <Dots items={[<><b className="text-fg">Art and traits are committed before launch</b> as a fingerprint (Merkle root), so nobody can swap a piece later.</>, <><b className="text-fg">Switchboard VRF</b> (a verifiable random function: a random number that comes with a proof) picks which NFT you get at every capture and re-roll.</>, <>The randomness provider is <b className="text-fg">pinned to an approved list</b> written into the program.</>]} />
          </div>
          <div className="space-y-2"><h3 className="text-sm font-semibold">Burn <span className="tag tag-demo ml-1">Pending deploy</span></h3>
            <Dots items={[<><b className="text-fg">Art is committed before launch</b>, and each minted NFT is checked against it.</>, <><b className="text-fg">NFTs go out in collection order</b>, with no randomness.</>, <><b className="text-fg">Known limitation:</b> the order is public, so anyone can see which piece is next, rare ones included, and time a burn for it.</>]} />
          </div>
        </div>
      </DocSection>

      <DocSection id="fees" title="Fees">
        <p className="text-muted text-sm">All Armory fees are flat and paid in SOL. Tokens are never taken as a fee. Each collection&apos;s fee is set from its ratio at launch, and the program caps it at 0.01 SOL.</p>
        <Table head={["Fee", "Plain", "Hybrid", "Burn"]} rows={[
          ["Platform fee per NFT action", "None (no NFTs)", "0.002 / 0.005 / 0.01 SOL by ratio, per capture or re-roll", "Same tiers, per burn"],
          ["Mint cost", "—", "Refundable 0.0063 SOL deposit; about 0.003–0.004 SOL is spent only the first time an NFT is captured", "≈ 0.005 SOL, every burn (estimate)"],
          ["Randomness", "—", "No separate fee; temporary account rent is refunded", "None"],
          ["Converting back", "—", "Free (network fee only)", "Not possible"],
          ["Trading on the curve", "1% · Example", "1% · Example", "1% · Example"],
          ["Trading after graduation", "Pool fee set by the DEX, e.g. 0.25% · Example", "Same", "Same"],
          ["Launch fee", "0.02 SOL, one-time · Example", "Same", "Same"],
        ]} />
        <p className="text-muted text-xs">Ratio tiers: 50K = 0.002 SOL; 100K and 200K = 0.005 SOL; 500K to 5M = 0.01 SOL. The mint cost is Solana rent plus the Metaplex Core fee, not an Armory fee. Fees can only go to the platform address written into the program.</p>
      </DocSection>

      <DocSection id="cap" title="Deposit cap">
        <p className="text-muted text-sm">While the programs are unaudited, how much one wallet can put in is capped. That limits how much anyone can lose to a bug we haven&apos;t found yet.</p>
        <div className="capslot"><span className="tag tag-ua">Unaudited beta</span> Deposit cap <b>{BETA_DEPOSIT_CAP.sol} SOL per wallet</b> <span className="tag tag-demo">Example, not set</span> <span className="text-xs">The real value will be published here before it applies.</span></div>
      </DocSection>

      <DocSection id="limits" title="Known limitations">
        <Dots tone="plan" items={[<><b className="text-fg">Upgrades are a trust assumption</b> until they&apos;re switched off. During beta, whoever holds the upgrade key could change the code.</>, <><b className="text-fg">Third-party programs</b> (Meteora&apos;s curve and pools, Switchboard randomness, Metaplex Core NFTs, DEXes and marketplaces) are outside Armory&apos;s control.</>, <><b className="text-fg">Burn order is public</b>, so rare pieces can be targeted.</>, <><b className="text-fg">Burn mint cost is an estimate.</b> It&apos;s Solana rent plus the Metaplex fee, about 0.005 SOL.</>, <><b className="text-fg">Tax split and Raffle aren&apos;t live.</b> They stay off until there&apos;s a public way to buy, the stuck-funds fixes are done, and Raffle has a legal check.</>]} />
      </DocSection>

      <DocSection id="addresses" title="Program addresses">
        <p className="text-muted text-sm">The mainnet program addresses and the verifiable build hash will be published here at launch. {CLUSTER.label} programs this app talks to:</p>
        <ul className="space-y-1 text-sm">
          {[["hybrid_launch", HYBRID_LAUNCH_PROGRAM_ID], ["hybrid_vault", HYBRID_VAULT_PROGRAM_ID], ["Switchboard On-Demand", SWITCHBOARD_PROGRAM_ID]].map(([n, id]) => (
            <li key={String(n)} className="fact"><span>{String(n)}</span><a className="font-mono text-xs break-all underline-offset-2 hover:underline" href={explorerAddressUrl(id!.toString())} target="_blank" rel="noopener noreferrer">{id!.toString()}</a></li>
          ))}
        </ul>
        <p className="capslot text-xs">Placeholder: mainnet program addresses and build hash, published at mainnet launch</p>
      </DocSection>

      <PipelineSelfTest />

      <DocSection title="Found a problem?">
        <p className="text-muted text-sm">Please report security issues privately rather than posting them publicly.</p>
        <Link href="/bug-bounty" className="btn btn-primary w-fit">Report a vulnerability</Link>
      </DocSection>
    </DocLayout>
  );
}
