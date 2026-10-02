/**
 * Read paths against the configured cluster (devnet in practice). Isomorphic: used by server
 * components (via cachedRead) and client components (with the wallet-adapter Connection).
 * Everything returned is a plain, serializable DTO (strings for keys and bigints).
 */
import { PublicKey, type Connection, type GetProgramAccountsFilter } from "@solana/web3.js";
import bs58Encode from "./bs58";
import { HYBRID_LAUNCH_PROGRAM_ID, HYBRID_VAULT_PROGRAM_ID, SWITCHBOARD_PROGRAM_ID } from "@/config/programs";
import { isExactTierFee } from "@/config/armory";
import { decodeLaunchConfig, launchConfigPda } from "@/lib/generated/hybridLaunch";
import {
  REQUEST_USER_OFFSET,
  assetPda,
  decodePool,
  decodeRequest,
  decodeVault,
  randLockPda,
  randomnessAuthorityPda,
  vaultPda,
} from "@/lib/generated/hybridVault";
import { hybridLaunchAccounts, hybridVaultAccounts } from "@/lib/generated/idlMeta";
import { decodeCoreAssetOwner, decodeCoreAssetUri, decodeCoreCollection } from "@/lib/generated/core";
import { decodeMint, decodeTokenAccount, ataAddress } from "@/lib/generated/spl";
import { metadataPda } from "@/lib/generated/dbc";
import { decodeMetadataStrings } from "@/lib/meteora/plain";

export type LaunchState = "native" | "curve" | "graduated";

export interface LaunchDTO {
  readonly type: "hybrid";
  readonly mint: string;
  readonly launchConfig: string;
  readonly creator: string;
  readonly decimals: number;
  readonly ratioWholeTokens: string;
  readonly ratioBase: string;
  readonly collectionSize: number;
  readonly feeLamports: string;
  readonly feeIsExactTier: boolean;
  readonly feeRecipient: string;
  readonly graduationThresholdLamports: string;
  readonly launchedAt: number;
  readonly dbcPool: string | null;
  readonly vault: string | null;
  readonly vaultOpen: boolean;
  readonly collection: string | null;
  readonly collectionName: string | null;
  /** Token metadata (Metaplex) written when the DBC pool was created; null when absent. */
  readonly tokenName?: string | null;
  readonly tokenSymbol?: string | null;
  readonly tokenUri?: string | null;
  readonly mintedCount: number;
  readonly assetsOutside: string;
  readonly totalCaptures: string;
  readonly totalRerolls: string;
  readonly totalUnwraps: string;
  readonly state: LaunchState;
  /** Mint authorities read from the mint account (null = revoked). */
  readonly mintAuthority: string | null;
  readonly freezeAuthority: string | null;
  readonly supplyBase: string;
}

const s = (k: PublicKey) => k.toBase58();

async function buildDTOs(conn: Connection, configs: { address: PublicKey; data: Uint8Array }[]): Promise<LaunchDTO[]> {
  const lcs = configs.map((c) => ({ address: c.address, lc: decodeLaunchConfig(c.data) }));
  if (lcs.length === 0) return [];
  const vaultKeys = lcs.map((x) => vaultPda(x.address));
  const [vaultInfos, mintInfos, metaInfos] = await Promise.all([
    conn.getMultipleAccountsInfo(vaultKeys),
    conn.getMultipleAccountsInfo(lcs.map((x) => x.lc.mint)),
    conn.getMultipleAccountsInfo(lcs.map((x) => metadataPda(x.lc.mint))),
  ]);
  const vaults = vaultInfos.map((v) => {
    try {
      return v ? decodeVault(new Uint8Array(v.data)) : null;
    } catch {
      return null;
    }
  });
  const collKeys = vaults.map((v) => v?.collection ?? null);
  const collInfos = await conn.getMultipleAccountsInfo(collKeys.filter((k): k is PublicKey => k !== null));
  const collNames = new Map<string, string>();
  collKeys.filter((k): k is PublicKey => k !== null).forEach((k, i) => {
    const info = collInfos[i];
    if (!info) return;
    try {
      collNames.set(s(k), decodeCoreCollection(new Uint8Array(info.data)).name);
    } catch {
      /* not a Core collection */
    }
  });
  return lcs.map(({ address, lc }, i) => {
    const v = vaults[i] ?? null;
    const mi = mintInfos[i] ? decodeMint(new Uint8Array(mintInfos[i]!.data)) : null;
    const meta = metaInfos[i] ? decodeMetadataStrings(new Uint8Array(metaInfos[i]!.data)) : null;
    const state: LaunchState = !lc.onCurve ? "native" : v?.open ? "graduated" : "curve";
    return {
      type: "hybrid",
      mint: s(lc.mint),
      launchConfig: s(address),
      creator: s(lc.creator),
      decimals: lc.decimals,
      ratioWholeTokens: lc.ratioWholeTokens.toString(),
      ratioBase: lc.ratioBase.toString(),
      collectionSize: Number(lc.collectionSize),
      feeLamports: lc.feeLamports.toString(),
      feeIsExactTier: isExactTierFee(lc.ratioWholeTokens, lc.feeLamports),
      feeRecipient: s(lc.feeRecipient),
      graduationThresholdLamports: lc.graduationThresholdLamports.toString(),
      launchedAt: Number(lc.launchedAt),
      dbcPool: lc.onCurve ? s(lc.dbcPool) : null,
      vault: v ? s(vaultKeys[i]!) : null,
      vaultOpen: v?.open ?? false,
      collection: v ? s(v.collection) : null,
      collectionName: v ? (collNames.get(s(v.collection)) ?? null) : null,
      tokenName: meta?.name || null,
      tokenSymbol: meta?.symbol || null,
      tokenUri: meta?.uri || null,
      mintedCount: v?.mintedCount ?? 0,
      assetsOutside: (v?.assetsOutside ?? 0n).toString(),
      totalCaptures: (v?.totalCaptures ?? 0n).toString(),
      totalRerolls: (v?.totalRerolls ?? 0n).toString(),
      totalUnwraps: (v?.totalUnwraps ?? 0n).toString(),
      state,
      mintAuthority: mi?.mintAuthority ? s(mi.mintAuthority) : null,
      freezeAuthority: mi?.freezeAuthority ? s(mi.freezeAuthority) : null,
      supplyBase: (mi?.supply ?? 0n).toString(),
    };
  });
}

const discFilter = (disc: readonly number[]): GetProgramAccountsFilter => ({
  memcmp: { offset: 0, bytes: bs58Encode(Uint8Array.from(disc)) },
});

/** All hybrid launches (LaunchConfig accounts) on the cluster, newest first. */
export async function fetchLaunches(conn: Connection): Promise<LaunchDTO[]> {
  const res = await conn.getProgramAccounts(HYBRID_LAUNCH_PROGRAM_ID, { filters: [discFilter(hybridLaunchAccounts.LaunchConfig)] });
  const dtos = await buildDTOs(conn, res.map((r) => ({ address: r.pubkey, data: new Uint8Array(r.account.data) })));
  return dtos.sort((a, b) => b.launchedAt - a.launchedAt);
}

/** One launch by mint, or null if this mint was not launched by Armory. */
export async function fetchLaunch(conn: Connection, mint: PublicKey): Promise<LaunchDTO | null> {
  const lc = launchConfigPda(mint);
  const info = await conn.getAccountInfo(lc);
  if (!info || !info.owner.equals(HYBRID_LAUNCH_PROGRAM_ID)) return null;
  const [dto] = await buildDTOs(conn, [{ address: lc, data: new Uint8Array(info.data) }]);
  return dto ?? null;
}

export interface ProgramStatus {
  readonly name: string;
  readonly programId: string;
  readonly executable: boolean;
  readonly upgradeAuthority: string | null;
  readonly lastDeploySlot: number | null;
}

/** Program accounts + upgrade authority (BPF upgradeable loader ProgramData). */
export async function fetchProgramStatus(conn: Connection): Promise<ProgramStatus[]> {
  const progs = [
    { name: "hybrid_launch", id: HYBRID_LAUNCH_PROGRAM_ID },
    { name: "hybrid_vault", id: HYBRID_VAULT_PROGRAM_ID },
  ];
  const infos = await conn.getMultipleAccountsInfo(progs.map((p) => p.id));
  const pdKeys = infos.map((i) => (i && i.data.length >= 36 ? new PublicKey(i.data.subarray(4, 36)) : null));
  const pdInfos = await conn.getMultipleAccountsInfo(pdKeys.filter((k): k is PublicKey => k !== null), { dataSlice: { offset: 0, length: 45 } });
  let j = 0;
  return progs.map((p, i) => {
    const info = infos[i];
    const pd = pdKeys[i] ? pdInfos[j++] : null;
    const d = pd ? new Uint8Array(pd.data) : null;
    const dv = d ? new DataView(d.buffer, d.byteOffset, d.byteLength) : null;
    return {
      name: p.name,
      programId: s(p.id),
      executable: info?.executable ?? false,
      upgradeAuthority: d && d[12] === 1 ? s(new PublicKey(d.slice(13, 45))) : null,
      lastDeploySlot: dv ? Number(dv.getBigUint64(4, true)) : null,
    };
  });
}

export interface Holdings {
  readonly tokenBase: bigint;
  readonly nftIndexes: number[];
  /** Metadata URI of each owned NFT (from the Core asset), by index. */
  readonly nftUris?: Readonly<Record<number, string>>;
}

/** User's token balance and the indexes of this collection's NFTs they own. */
export async function fetchHoldings(conn: Connection, user: PublicKey, launch: LaunchDTO): Promise<Holdings> {
  const mint = new PublicKey(launch.mint);
  const ata = await conn.getAccountInfo(ataAddress(user, mint));
  const tokenBase = ata ? decodeTokenAccount(new Uint8Array(ata.data)).amount : 0n;
  if (!launch.vault || launch.mintedCount === 0) return { tokenBase, nftIndexes: [] };
  const vault = new PublicKey(launch.vault);
  const vInfo = await conn.getAccountInfo(vault);
  if (!vInfo) return { tokenBase, nftIndexes: [] };
  const v = decodeVault(new Uint8Array(vInfo.data));
  const poolInfo = await conn.getAccountInfo(v.pool);
  if (!poolInfo) return { tokenBase, nftIndexes: [] };
  const minted = decodePool(new Uint8Array(poolInfo.data)).mintedIndexes();
  const nftIndexes: number[] = [];
  const nftUris: Record<number, string> = {};
  for (let k = 0; k < minted.length; k += 100) {
    const chunk = minted.slice(k, k + 100);
    const infos = await conn.getMultipleAccountsInfo(chunk.map((i) => assetPda(vault, i)));
    infos.forEach((a, n) => {
      const owner = a ? decodeCoreAssetOwner(new Uint8Array(a.data)) : null;
      if (owner?.equals(user)) {
        nftIndexes.push(chunk[n]!);
        const uri = decodeCoreAssetUri(new Uint8Array(a!.data));
        if (uri) nftUris[chunk[n]!] = uri;
      }
    });
  }
  return { tokenBase, nftIndexes, nftUris };
}

export interface RequestDTO {
  readonly address: string;
  readonly vault: string;
  readonly seq: string;
  readonly kind: "capture" | "reroll";
  readonly revealed: boolean;
  readonly deadlineSlot: string;
  readonly commits: number;
  readonly mintEscrowLamports: string;
}

/** Pending capture/re-roll requests of a user (optionally for one vault). */
export async function fetchUserRequests(conn: Connection, user: PublicKey, vault?: string): Promise<RequestDTO[]> {
  const res = await conn.getProgramAccounts(HYBRID_VAULT_PROGRAM_ID, {
    filters: [discFilter(hybridVaultAccounts.Request), { memcmp: { offset: REQUEST_USER_OFFSET, bytes: s(user) } }],
  });
  return res
    .map((r) => ({ address: s(r.pubkey), rq: decodeRequest(new Uint8Array(r.account.data)) }))
    .filter((x) => !vault || s(x.rq.vault) === vault)
    .map(({ address, rq }) => ({
      address,
      vault: s(rq.vault),
      seq: rq.seq.toString(),
      kind: rq.handedInIndex === 0xffff_ffff ? "capture" : "reroll",
      revealed: rq.revealed,
      deadlineSlot: rq.deadlineSlot.toString(),
      commits: rq.commits,
      mintEscrowLamports: rq.mintEscrowLamports.toString(),
    }));
}

/**
 * A Switchboard randomness account created for this vault (authority = its randomness_authority
 * PDA) that is not currently locked by another request.
 */
export async function findIdleRandomness(conn: Connection, vault: PublicKey): Promise<PublicKey | null> {
  const ra = randomnessAuthorityPda(vault);
  const res = await conn.getProgramAccounts(SWITCHBOARD_PROGRAM_ID, {
    filters: [{ memcmp: { offset: 8, bytes: s(ra) } }],
    dataSlice: { offset: 0, length: 0 },
  });
  if (res.length === 0) return null;
  const locks = await conn.getMultipleAccountsInfo(res.map((r) => randLockPda(r.pubkey)));
  const idx = locks.findIndex((l) => l === null);
  return idx === -1 ? null : res[idx]!.pubkey;
}
