/**
 * /meteora ("How it works"): the technology under Armory (Meteora DBC + DAMM v2), with the platform
 * config read live from chain. Low-key page, linked from the footer only.
 */
import type { Metadata } from "next";
import Link from "next/link";
import { PublicKey } from "@solana/web3.js";
import { CLUSTER, explorerAddressUrl } from "@/config/cluster";
import { DAMM_V2_PROGRAM_ID, DBC_PROGRAM_ID, HYBRID_LAUNCH_PROGRAM_ID, HYBRID_VAULT_PROGRAM_ID } from "@/config/programs";
import { DBC_PLATFORM_CONFIG } from "@/config/integrations";
import { cachedRead } from "@/lib/armory/server";
import { dbcClient } from "@/lib/meteora/dbc";
import { fetchPlainLaunches } from "@/lib/meteora/plain";
import { fetchLaunches } from "@/lib/armory/reads";
import { formatSol, shortAddr } from "@/lib/armory/format";

export const metadata: Metadata = { title: "How it works", description: "The technology behind Armory launches: bonding curve, graduation into a locked pool, and the NFT layer on top." };

interface ConfigFacts {
  thresholdLamports: string;
  feeBps: number;
  migrationOption: number;
  supplyWhole: string;
  curvePct: number;
  dexPct: number;
  bufferPct: number;
  leftoverReceiver: string;
  feeClaimer: string;
  lockedLpPct: number;
  fixedSupply: boolean;
  immutableMetadata: boolean;
  pools: number;
}

async function loadConfig(): Promise<ConfigFacts | null> {
  const key = DBC_PLATFORM_CONFIG[CLUSTER.name];
  if (!key) return null;
  return cachedRead("meteora:config", 60_000, async (c) => {
    const cfg = await dbcClient(c).state.getPoolConfig(key);
    if (!cfg) throw new Error("DBC config not found");
    const total = BigInt(cfg.preMigrationTokenSupply.toString());
    const swap = BigInt(cfg.swapBaseAmount.toString());
    const mig = BigInt(cfg.migrationBaseThreshold.toString());
    const pct = (x: bigint) => (total > 0n ? Math.round(Number((x * 1000n) / total)) / 10 : 0);
    const pools = (await dbcClient(c).state.getPoolsByConfig(key)).length;
    return {
      thresholdLamports: cfg.migrationQuoteThreshold.toString(),
      feeBps: Math.round(Number(cfg.poolFees.baseFee.cliffFeeNumerator.toString()) / 100_000),
      migrationOption: cfg.migrationOption,
      supplyWhole: (total / 10n ** BigInt(cfg.tokenDecimal)).toString(),
      curvePct: pct(swap),
      dexPct: pct(mig),
      bufferPct: pct(total - swap - mig),
      leftoverReceiver: cfg.leftoverReceiver.toBase58(),
      feeClaimer: cfg.feeClaimer.toBase58(),
      lockedLpPct: cfg.partnerPermanentLockedLiquidityPercentage + cfg.creatorPermanentLockedLiquidityPercentage,
      fixedSupply: cfg.fixedTokenSupplyFlag === 1,
      immutableMetadata: cfg.tokenUpdateAuthority === 1,
      pools,
    } satisfies ConfigFacts;
  }).then((r) => (r.ok ? r.value : null));
}

const BUFFER_PDA = PublicKey.findProgramAddressSync([new TextEncoder().encode("dbc_buffer")], HYBRID_LAUNCH_PROGRAM_ID)[0].toBase58();

function Addr({ k, a }: { k: string; a: string }) {
  return (
    <div className="fact">
      <dt>{k}</dt>
      <dd><a className="text-accent-text font-mono text-xs" href={explorerAddressUrl(a)} target="_blank" rel="noopener noreferrer">{shortAddr(a, 6)} ↗</a></dd>
    </div>
  );
}

const FLOW = [
  ["01", "Launch on DBC", "A Launch is one Meteora DBC instruction (initialize_virtual_pool_with_spl_token) on Armory's platform config. DBC creates the 1B SPL token, revokes mint authority, writes immutable metadata and opens the curve."],
  ["02", "Trade the curve", "Buys and sells go straight to the DBC pool (swap2, exact-in). The app quotes with the SDK, shows fees and a minimum received, and simulates before your wallet signs."],
  ["03", "Graduate", "When the curve raises its SOL target, DBC migrates the liquidity into a Meteora DAMM v2 pool. The LP is permanently locked by the config; unsold curve tokens go to a program-owned buffer that can never withdraw."],
  ["04", "Trade on DAMM v2", "After migration the same Trade panel routes to the DAMM v2 pool through the cp-amm SDK, with price impact and slippage protection."],
  ["05", "NFT layer (Hybrid)", "For Hybrid launches, Armory's hybrid_launch registers the DBC pool (allowlisted config only), and hybrid_vault opens converting only after it has verified the pool migrated. Then tokens ⇄ Metaplex Core NFTs at a fixed ratio, with Switchboard randomness."],
] as const;

export default async function MeteoraPage() {
  const [cfg, plains, hybrids] = await Promise.all([
    loadConfig(),
    cachedRead("plain-launches", 20_000, fetchPlainLaunches),
    cachedRead("launches", 30_000, fetchLaunches),
  ]);
  const dbcConfig = DBC_PLATFORM_CONFIG[CLUSTER.name];
  const curveHybrids = hybrids.ok ? hybrids.value.filter((l) => l.dbcPool) : [];
  return (
    <div className="mx-auto max-w-(--container-site) space-y-12 px-4 py-10 md:py-14">
      <section className="max-w-3xl space-y-4">
        <p className="eyebrow">Technology</p>
        <h1 className="hd text-3xl md:text-4xl">How it works</h1>
        <p className="text-muted text-base">
          Armory launches trade on Meteora&apos;s <strong className="text-fg">Dynamic Bonding Curve</strong> and graduate into a locked{" "}
          <strong className="text-fg">DAMM v2</strong> pool. Armory adds one thing on top: a Hybrid launch&apos;s token can be converted into a random NFT
          from its collection, and back, at a fixed rate.
        </p>
      </section>

      <section className="space-y-4" aria-labelledby="flow-h">
        <h2 id="flow-h" className="hd text-2xl md:text-3xl">How a launch moves from curve to pool</h2>
        <ol className="grid gap-3 md:grid-cols-5">
          {FLOW.map(([n, t, d]) => (
            <li key={n} className="card space-y-2 p-4">
              <p className="text-accent-text font-mono text-xs">{n}</p>
              <h3 className="font-semibold">{t}</h3>
              <p className="text-muted text-sm">{d}</p>
            </li>
          ))}
        </ol>
      </section>

      <section className="grid gap-6 lg:grid-cols-2">
        <div className="card space-y-3 p-5" data-testid="meteora-config">
          <div className="flex items-center justify-between gap-2">
            <h2 className="font-semibold">Armory&apos;s custom DBC config</h2>
            <span className="text-dim text-xs">read from chain</span>
          </div>
          {cfg ? (
            <dl className="text-sm">
              <div className="fact"><dt>Graduation target</dt><dd>{formatSol(cfg.thresholdLamports, 2)} raised</dd></div>
              <div className="fact"><dt>Migrates to</dt><dd>{cfg.migrationOption === 1 ? "Meteora DAMM v2" : `Option ${cfg.migrationOption}`}</dd></div>
              <div className="fact"><dt>Supply</dt><dd>{Number(cfg.supplyWhole).toLocaleString("en-US")}{cfg.fixedSupply ? " · fixed" : ""}</dd></div>
              <div className="fact"><dt>Supply split</dt><dd>{cfg.curvePct}% curve · {cfg.dexPct}% DAMM v2 pool · {cfg.bufferPct}% locked buffer</dd></div>
              <div className="fact"><dt>Curve trade fee</dt><dd>{cfg.feeBps / 100}% (part goes to Meteora as protocol fee)</dd></div>
              <div className="fact"><dt>LP after migration</dt><dd>{cfg.lockedLpPct}% permanently locked</dd></div>
              <div className="fact"><dt>Token metadata</dt><dd>{cfg.immutableMetadata ? "Immutable" : "Updatable"}</dd></div>
              <div className="fact"><dt>Unsold tokens go to</dt><dd>{cfg.leftoverReceiver === BUFFER_PDA ? "Armory's locked buffer PDA (no withdraw instruction)" : shortAddr(cfg.leftoverReceiver)}</dd></div>
              <div className="fact"><dt>Pools on this config</dt><dd>{cfg.pools}</dd></div>
            </dl>
          ) : (
            <p className="text-muted text-sm">The config couldn&apos;t be read right now. Refresh to try again.</p>
          )}
        </div>
        <div className="card space-y-3 p-5">
          <h2 className="font-semibold">Addresses</h2>
          <dl className="text-sm">
            <Addr k="Meteora DBC program" a={DBC_PROGRAM_ID.toBase58()} />
            <Addr k="Meteora DAMM v2 program" a={DAMM_V2_PROGRAM_ID.toBase58()} />
            {dbcConfig && <Addr k="Armory DBC config" a={dbcConfig.toBase58()} />}
            <Addr k="Locked buffer PDA" a={BUFFER_PDA} />
            {cfg && <Addr k="Platform fee claimer" a={cfg.feeClaimer} />}
            <Addr k="Armory hybrid_launch" a={HYBRID_LAUNCH_PROGRAM_ID.toBase58()} />
            <Addr k="Armory hybrid_vault" a={HYBRID_VAULT_PROGRAM_ID.toBase58()} />
          </dl>
        </div>
      </section>

      <section className="grid gap-6 lg:grid-cols-2">
        <div className="card space-y-3 p-5">
          <h2 className="font-semibold">Why the NFT layer needs the curve</h2>
          <ul className="text-muted list-inside list-disc space-y-1 text-sm">
            <li>hybrid_launch only registers a DBC pool whose config is on its compiled allowlist, and checks the pool, config and mint on-chain.</li>
            <li>hybrid_vault&apos;s graduation verifier reads the DBC pool and refuses to open converting until it has migrated to DAMM v2.</li>
            <li>So the NFT side can only open on a token with a real, locked DAMM v2 market, never on a pre-mint the creator controls.</li>
            <li>Converting is exact both ways: lock the ratio for a random Metaplex Core NFT (Switchboard randomness), release it for exactly the ratio back.</li>
          </ul>
        </div>
        <div className="card space-y-3 p-5" data-testid="meteora-live">
          <h2 className="font-semibold">On chain now</h2>
          <dl className="text-sm">
            <div className="fact"><dt>Launches (DBC)</dt><dd>{plains.ok ? plains.value.length : "—"}</dd></div>
            <div className="fact"><dt>… graduated to DAMM v2</dt><dd>{plains.ok ? plains.value.filter((p) => p.curve.migrated).length : "—"}</dd></div>
            <div className="fact"><dt>Hybrid launches on a DBC curve</dt><dd>{hybrids.ok ? curveHybrids.length : "—"}</dd></div>
          </dl>
          <p className="text-muted text-sm">
            Try it: the <Link className="text-accent-text" href="/t/Dhv3VkqeTYJiahzcVJLmJ1snApdYtQxsAKeUnv5GTNos">ARMT</Link> Hybrid launch graduated from DBC to DAMM v2; its page trades on DAMM v2 and converts tokens to NFTs.
          </p>
        </div>
      </section>

      <section className="card space-y-2 p-5">
        <h2 className="font-semibold">Integration code</h2>
        <ul className="text-muted space-y-1 font-mono text-xs">
          <li>app/src/lib/meteora/dbc.ts: DBC SDK client: launch, curve quote + swap, curve state</li>
          <li>app/src/lib/meteora/damm.ts: cp-amm SDK client: find the migrated DAMM v2 pool, quote + swap</li>
          <li>app/src/lib/meteora/plain.ts: lists Launch tokens (pools on the platform config)</li>
          <li>app/src/components/meteora/SwapPanel.tsx · CurveProgress.tsx: trade UI and graduation indicator</li>
          <li>programs/hybrid_launch/src/dbc.rs · programs/hybrid_vault/src/graduation.rs: on-chain DBC config + migration checks</li>
        </ul>
      </section>
    </div>
  );
}
