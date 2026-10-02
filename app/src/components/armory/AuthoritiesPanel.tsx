/**
 * Locked-authorities panel (server component). Program upgrade authorities are read LIVE from the
 * BPF upgradeable loader on the configured cluster; anything not yet enforced on-chain is shown as
 * a clearly tagged placeholder. Never claims "immutable" for an upgradeable program.
 */
import { explorerAddressUrl } from "@/config/cluster";
import { PROGRAM_UPGRADES_COPY } from "@/config/armory";
import { cachedRead } from "@/lib/armory/server";
import { fetchProgramStatus } from "@/lib/armory/reads";
import { shortAddr } from "@/lib/armory/format";

export async function AuthoritiesPanel({ compact = false }: { compact?: boolean }) {
  const st = await cachedRead("program-status", 60_000, fetchProgramStatus);
  return (
    <div data-testid="authorities-panel" className="space-y-3 text-sm">
      {!compact && (
        <p className="text-muted">
          Read live from the chain. A program with an upgrade authority can still be changed by that key; it is shown
          here so you can verify it yourself.
        </p>
      )}
      <ul className="divide-border divide-y">
        {st.ok ? (
          st.value.map((p) => (
            <li key={p.programId} className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 py-2" data-testid={`authority-${p.name}`}>
              <span className="font-mono text-xs">{p.name}</span>
              <span className="text-muted text-xs">
                Upgrade authority:{" "}
                {p.upgradeAuthority ? (
                  <a className="text-fg font-mono underline-offset-2 hover:underline" href={explorerAddressUrl(p.upgradeAuthority)} target="_blank" rel="noopener noreferrer">
                    {shortAddr(p.upgradeAuthority)}
                  </a>
                ) : (
                  <span className="text-positive-text">none (frozen)</span>
                )}{" "}
                {p.upgradeAuthority && <span className="tag ml-1">Deployer key</span>}
              </span>
            </li>
          ))
        ) : (
          <li className="text-warning py-2" role="status">
            Couldn&apos;t read program authorities from the RPC right now ({st.error.slice(0, 80)}).
          </li>
        )}
        <li className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 py-2">
          <span className="font-mono text-xs">Token mint / freeze</span>
          <span className="text-muted text-xs">Revoked at launch for every token, checked on each token page</span>
        </li>
        <li className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 py-2">
          <span className="font-mono text-xs">Fee + ratio</span>
          <span className="text-muted text-xs">Written once per launch; no instruction can change them</span>
        </li>
        <li className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1 py-2">
          <span className="font-mono text-xs">Program upgrades</span>
          <span className="text-muted text-xs" data-testid="upgrades-copy">
            {PROGRAM_UPGRADES_COPY.status}. {PROGRAM_UPGRADES_COPY.today} {PROGRAM_UPGRADES_COPY.planned} {PROGRAM_UPGRADES_COPY.after} <span className="tag ml-1">Planned, not set up</span>
          </span>
        </li>
      </ul>
    </div>
  );
}
