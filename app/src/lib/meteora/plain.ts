/**
 * Plain launches = Meteora DBC pools on Armory's platform config that are NOT registered with
 * hybrid_launch (no LaunchConfig PDA for the mint). Read from chain via the DBC SDK; names and
 * symbols come from the Metaplex token metadata DBC writes at pool creation.
 */
import { PublicKey, type Connection } from "@solana/web3.js";
import { launchConfigPda } from "@/lib/generated/hybridLaunch";
import { metadataPda } from "@/lib/generated/dbc";
import { HYBRID_LAUNCH_PROGRAM_ID } from "@/config/programs";
import { curveStateFrom, dbcClient, platformDbcConfig, platformPoolForMint, type CurveStateDTO } from "./dbc";

export interface PlainLaunchDTO {
  readonly type: "plain";
  readonly mint: string;
  readonly name: string;
  readonly symbol: string;
  readonly uri: string;
  readonly curve: CurveStateDTO;
  readonly supplyBase: string;
  readonly mintAuthority: string | null;
  readonly freezeAuthority: string | null;
}

/** Minimal Metaplex Token Metadata decode: key(1) update_authority(32) mint(32) name symbol uri (borsh strings, NUL-padded). */
export function decodeMetadataStrings(data: Uint8Array): { name: string; symbol: string; uri: string } | null {
  try {
    const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
    let o = 65;
    const str = () => {
      const len = dv.getUint32(o, true);
      o += 4;
      if (len > 512 || o + len > data.length) throw new Error("bad string");
      const s = new TextDecoder().decode(data.subarray(o, o + len)).replace(/\0+$/, "").trim();
      o += len;
      return s;
    };
    return { name: str(), symbol: str(), uri: str() };
  } catch {
    return null;
  }
}

function decodeMintInfo(data: Uint8Array): { supply: bigint; mintAuthority: string | null; freezeAuthority: string | null } {
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const hasMint = dv.getUint32(0, true) === 1;
  const supply = dv.getBigUint64(36, true);
  const hasFreeze = dv.getUint32(46, true) === 1;
  return {
    supply,
    mintAuthority: hasMint ? new PublicKey(data.subarray(4, 36)).toBase58() : null,
    freezeAuthority: hasFreeze ? new PublicKey(data.subarray(50, 82)).toBase58() : null,
  };
}

async function toDTOs(conn: Connection, pools: { publicKey: PublicKey; account: Awaited<ReturnType<ReturnType<typeof dbcClient>["state"]["getPool"]>> }[]): Promise<PlainLaunchDTO[]> {
  const live = pools.filter((p): p is { publicKey: PublicKey; account: NonNullable<typeof p.account> } => p.account !== null);
  if (live.length === 0) return [];
  const cfgKey = platformDbcConfig();
  if (!cfgKey) return [];
  const cfg = await dbcClient(conn).state.getPoolConfig(cfgKey);
  if (!cfg) return [];
  const mints = live.map((p) => p.account.poolState.baseMint);
  // Exclude hybrid launches (registered with hybrid_launch).
  const lcInfos = await conn.getMultipleAccountsInfo(mints.map((m) => launchConfigPda(m)));
  const plain = live.filter((_, i) => !(lcInfos[i] && lcInfos[i]!.owner.equals(HYBRID_LAUNCH_PROGRAM_ID)));
  if (plain.length === 0) return [];
  const pm = plain.map((p) => p.account.poolState.baseMint);
  const [metaInfos, mintInfos] = await Promise.all([conn.getMultipleAccountsInfo(pm.map((m) => metadataPda(m))), conn.getMultipleAccountsInfo(pm)]);
  return plain.map((p, i) => {
    const meta = metaInfos[i] ? decodeMetadataStrings(new Uint8Array(metaInfos[i]!.data)) : null;
    const mi = mintInfos[i] ? decodeMintInfo(new Uint8Array(mintInfos[i]!.data)) : null;
    const mint = pm[i]!.toBase58();
    return {
      type: "plain",
      mint,
      name: meta?.name || `Token ${mint.slice(0, 4)}…${mint.slice(-4)}`,
      symbol: meta?.symbol ?? "",
      uri: meta?.uri ?? "",
      curve: curveStateFrom(p.publicKey, p.account, cfg),
      supplyBase: (mi?.supply ?? 0n).toString(),
      mintAuthority: mi?.mintAuthority ?? null,
      freezeAuthority: mi?.freezeAuthority ?? null,
    };
  });
}

/** Every Plain launch on the platform config (most SOL raised first). */
export async function fetchPlainLaunches(conn: Connection): Promise<PlainLaunchDTO[]> {
  const cfg = platformDbcConfig();
  if (!cfg) return [];
  const pools = await dbcClient(conn).state.getPoolsByConfig(cfg);
  const dtos = await toDTOs(conn, pools);
  return dtos.sort((a, b) => Number(BigInt(b.curve.quoteReserveLamports) - BigInt(a.curve.quoteReserveLamports)));
}

/** One Plain launch by mint, or null (no platform DBC pool, or it's a hybrid launch). */
export async function fetchPlainLaunch(conn: Connection, mint: PublicKey): Promise<PlainLaunchDTO | null> {
  const pool = platformPoolForMint(mint);
  if (!pool) return null;
  const account = await dbcClient(conn).state.getPool(pool);
  if (!account) return null;
  const [dto] = await toDTOs(conn, [{ publicKey: pool, account }]);
  return dto ?? null;
}
