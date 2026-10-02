/**
 * Pinned program IDs + the transaction allowlist.
 *
 * TOP-LEVEL allowlist: every instruction in a transaction built by this app must target one of
 * the `topLevel: true` programs below, or src/lib/tx/validate.ts rejects it before simulation.
 * `topLevel: false` entries are known programs our programs CPI into; they are only used to
 * LABEL programs seen in simulation logs (unknown CPIs still produce a warning).
 *
 * Verification:
 *  - System / ComputeBudget / SPL Token / Token-2022 / ATA / Memo / Metaplex Core: upstream
 *    declare_id! / READMEs (2026-09-24).
 *  - hybrid_launch / hybrid_vault: devnet getAccountInfo 2026-10-01: both executable, owned by
 *    BPFLoaderUpgradeable, upgrade authority An3ZmiB4… (throwaway devnet deployer). IDLs: ONLY
 *    /workspace/idl/devnet (byte-identical to the deployed programs), copied verbatim into
 *    src/lib/generated/idl. Never ../target/idl (undeployed fix/modes-1-5 branch).
 *  - Switchboard On-Demand (devnet) Aio4gaX… and the devnet queue EYiAm…: pinned in
 *    hybrid_vault's constants (ADR-012/ADR-020) and checked on devnet. Re-verified 2026-10-01
 *    against docs.switchboard.xyz/docs-by-chain/solana-svm ("Devnet Program ID", "Default Devnet
 *    Queue") and switchboard-xyz/solana-sdk src/program_id.rs (ON_DEMAND_DEVNET_PID). Still
 *    CPI-only: the app reveals through hybrid_vault.reveal_randomness (the randomness authority
 *    is the vault PDA, so a direct Switchboard randomnessReveal can't be signed by anyone else).
 *  - Meteora DBC dbcij3L…: docs.meteora.ag/developer-guides/dbc ("same on mainnet and devnet")
 *    and github.com/MeteoraAg/dynamic-bonding-curve, 2026-10-01. Also pinned in
 *    hybrid_launch/src/dbc.rs. Top-level for Plain launches (initialize_virtual_pool_with_spl_token), curve buys/sells (swap2)
 *    and the hybrid DBC launch path.
 *  - Meteora DAMM v2 cpamdpZ…: @meteora-ag/cp-amm-sdk CP_AMM_PROGRAM_ID and docs.meteora.ag (same on
 *    devnet and mainnet); the ARMT devnet pool Azo9hp… is owned by it (checked 2026-10-02).
 *  - Metaplex Token Metadata metaqbx… (DBC CPI) and the Address Lookup Table program (Switchboard
 *    CPI at init_randomness): well-known IDs, listed in the Meteora SDK reference / web3.js.
 */
import { PublicKey } from "@solana/web3.js";
import { CLUSTER, type ClusterName } from "./cluster";

export type ProgramKind = "native" | "spl" | "metaplex" | "armory" | "oracle" | "dex";

export interface ProgramEntry {
  readonly id: PublicKey;
  readonly name: string;
  readonly kind: ProgramKind;
  /** May be the target of a top-level instruction we build. */
  readonly topLevel: boolean;
}

const pk = (s: string) => new PublicKey(s);

export const SYSTEM_PROGRAM_ID = pk("11111111111111111111111111111111");
export const COMPUTE_BUDGET_PROGRAM_ID = pk("ComputeBudget111111111111111111111111111111");
export const TOKEN_PROGRAM_ID = pk("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
export const TOKEN_2022_PROGRAM_ID = pk("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
export const ASSOCIATED_TOKEN_PROGRAM_ID = pk("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
export const MEMO_PROGRAM_ID = pk("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
export const MPL_CORE_PROGRAM_ID = pk("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
export const SWITCHBOARD_PROGRAM_ID = pk("Aio4gaXjXzJNVLtzwtNVmSqGKpANtXhybbkhtAC94ji2");
export const SWITCHBOARD_DEVNET_QUEUE = pk("EYiAmGSdsQTuCw413V5BzaruWuCCSDgTPtBGvLkXHbe7");
export const DBC_PROGRAM_ID = pk("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN");
/** Meteora DAMM v2 (cp-amm): where a graduated DBC curve migrates. Same ID on devnet and mainnet (@meteora-ag/cp-amm-sdk CP_AMM_PROGRAM_ID). */
export const DAMM_V2_PROGRAM_ID = pk("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
export const TOKEN_METADATA_PROGRAM_ID = pk("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
export const ADDRESS_LOOKUP_TABLE_PROGRAM_ID = pk("AddressLookupTab1e1111111111111111111111111");
export const WRAPPED_SOL_MINT = pk("So11111111111111111111111111111111111111112");
export const SLOT_HASHES_SYSVAR = pk("SysvarS1otHashes111111111111111111111111111");

/** Armory programs. Same IDs on localnet (deploy from ../target/deploy) and devnet. */
export const HYBRID_LAUNCH_PROGRAM_ID = pk("9Loc4hQZJh4SuBCGPiPs1wAfwywUAM7av5upyGHfc6Q8");
export const HYBRID_VAULT_PROGRAM_ID = pk("BEfL9dccCUtgBVfLmJieeSr3ju29fpVqLM3NgttxqXqG");

/** Platform fee wallet compiled into both programs (LaunchConfig.fee_recipient). */
export const PLATFORM_FEE_RECIPIENT = pk("7J3AajxfajAMgGmmwzfZeYGNieRtHRCuEgNTfd4gTjDN");

const COMMON: readonly ProgramEntry[] = [
  { id: SYSTEM_PROGRAM_ID, name: "System Program", kind: "native", topLevel: true },
  { id: COMPUTE_BUDGET_PROGRAM_ID, name: "Compute Budget", kind: "native", topLevel: true },
  { id: TOKEN_PROGRAM_ID, name: "SPL Token", kind: "spl", topLevel: true },
  { id: ASSOCIATED_TOKEN_PROGRAM_ID, name: "Associated Token Account", kind: "spl", topLevel: true },
  { id: MEMO_PROGRAM_ID, name: "SPL Memo", kind: "spl", topLevel: true },
  // CPI-only (labels for the preview; never a top-level target from this app):
  { id: TOKEN_2022_PROGRAM_ID, name: "Token-2022", kind: "spl", topLevel: false },
  { id: MPL_CORE_PROGRAM_ID, name: "Metaplex Core", kind: "metaplex", topLevel: false },
  { id: SWITCHBOARD_PROGRAM_ID, name: "Switchboard On-Demand", kind: "oracle", topLevel: false },
  { id: TOKEN_METADATA_PROGRAM_ID, name: "Metaplex Token Metadata", kind: "metaplex", topLevel: false },
  { id: ADDRESS_LOOKUP_TABLE_PROGRAM_ID, name: "Address Lookup Table", kind: "native", topLevel: false },
];

/** Meteora DBC + DAMM v2: top-level on devnet and mainnet (same program IDs on both). */
const METEORA: readonly ProgramEntry[] = [
  { id: DBC_PROGRAM_ID, name: "Meteora Dynamic Bonding Curve", kind: "dex", topLevel: true },
  // Swaps on a graduated launch's DAMM v2 pool (buy/sell after migration).
  { id: DAMM_V2_PROGRAM_ID, name: "Meteora DAMM v2", kind: "dex", topLevel: true },
];

const ARMORY: readonly ProgramEntry[] = [
  { id: HYBRID_LAUNCH_PROGRAM_ID, name: "Armory hybrid_launch", kind: "armory", topLevel: true },
  { id: HYBRID_VAULT_PROGRAM_ID, name: "Armory hybrid_vault", kind: "armory", topLevel: true },
];

/** Mainnet: no Armory programs, no Switchboard / lookup-table / Metaplex Core labels (Launch type only, ADR-022). */
const MAINNET_EXCLUDED = new Set([SWITCHBOARD_PROGRAM_ID, ADDRESS_LOOKUP_TABLE_PROGRAM_ID, MPL_CORE_PROGRAM_ID].map((k) => k.toBase58()));

export const PROGRAMS_BY_CLUSTER: Record<ClusterName, readonly ProgramEntry[]> = {
  localnet: [...COMMON, ...ARMORY],
  devnet: [...COMMON, ...ARMORY, ...METEORA],
  "mainnet-beta": [...COMMON.filter((p) => !MAINNET_EXCLUDED.has(p.id.toBase58())), ...METEORA],
};

/**
 * Armory's own programs (hybrid_launch / hybrid_vault) exist on this cluster. FALSE on mainnet: nothing reads,
 * derives against or sends to them there (reads return empty, Hybrid is "Coming soon").
 */
export function armoryProgramsEnabled(cluster: ClusterName = CLUSTER.name): boolean {
  return getProgramRegistry(cluster).some((p) => p.kind === "armory");
}

export function getProgramRegistry(cluster: ClusterName = CLUSTER.name): readonly ProgramEntry[] {
  return PROGRAMS_BY_CLUSTER[cluster];
}

const cache = new Map<ClusterName, Map<string, ProgramEntry>>();
function registry(cluster: ClusterName): Map<string, ProgramEntry> {
  let m = cache.get(cluster);
  if (!m) {
    m = new Map(getProgramRegistry(cluster).map((p) => [p.id.toBase58(), p]));
    cache.set(cluster, m);
  }
  return m;
}

/** Any known program (top-level or CPI-only), for labels. */
export function lookupProgram(programId: PublicKey | string, cluster: ClusterName = CLUSTER.name): ProgramEntry | undefined {
  return registry(cluster).get(typeof programId === "string" ? programId : programId.toBase58());
}

/** TRUE iff a top-level instruction may target this program on the cluster. */
export function isAllowedProgram(programId: PublicKey | string, cluster: ClusterName = CLUSTER.name): boolean {
  return lookupProgram(programId, cluster)?.topLevel === true;
}
