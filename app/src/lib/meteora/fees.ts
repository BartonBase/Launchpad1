/**
 * Fee claims for Launch-type tokens (Meteora DBC curve + DAMM v2 pool after graduation), built with the official
 * Meteora SDKs. Two audiences:
 *  - Creators: their share of the curve trading fee (creatorQuoteFee on each DBC pool they created on Armory's
 *    platform config), shown in Portfolio; after graduation, fees on any DAMM v2 LP position they own.
 *  - Armory (the config's fee claimer, Barton's fee wallet on mainnet): partnerQuoteFee on every pool of the platform
 *    config, the partner surplus after migration, and the DAMM v2 positions the config locked to the partner.
 *    Shown on the unlinked /admin/fees page; anyone can read the numbers, only the fee wallet can sign.
 *
 * Builders return unsigned transactions; every one goes through useSafeSend (program allowlist -> simulate ->
 * preview -> explicit confirm), like the rest of the app. Amounts are lamports / base units as bigint.
 */
import type { Connection, PublicKey, Transaction } from "@solana/web3.js";
import BN from "bn.js";
import { getUnClaimLpFee } from "@meteora-ag/cp-amm-sdk";
import { WRAPPED_SOL_MINT } from "@/config/programs";
import { dbcClient, platformDbcConfig } from "./dbc";
import { dammClient } from "./damm";

/** "Claim everything": the DBC claim takes caps; u64::MAX means no cap. */
const U64_MAX = new BN("18446744073709551615");

const big = (x: BN | { toString(): string }) => BigInt(x.toString());

export interface CurveFeeRow {
  readonly pool: string;
  readonly mint: string;
  /** Unclaimed fee in SOL lamports (the platform config collects fees in SOL only). */
  readonly unclaimedLamports: bigint;
  /** Unclaimed fee in the launch token, base units (0 on SOL-only configs). */
  readonly unclaimedBase: bigint;
  readonly migrated: boolean;
  /** Partner surplus still withdrawable (partner rows only; migrated pools). */
  readonly surplusPending?: boolean;
}

export interface LpFeeRow {
  readonly pool: string;
  readonly position: string;
  readonly positionNftAccount: string;
  readonly mint: string;
  readonly unclaimedLamports: bigint;
  readonly unclaimedBase: bigint;
}

/** The fields of a DBC virtual pool used here (SDK getPoolsBy* returns { publicKey, account: { poolState } }). */
export interface PoolFeeFields {
  readonly config: PublicKey;
  readonly baseMint: PublicKey;
  readonly isMigrated: number;
  readonly isPartnerWithdrawSurplus: number;
  readonly creatorQuoteFee: BN;
  readonly creatorBaseFee: BN;
  readonly partnerQuoteFee: BN;
  readonly partnerBaseFee: BN;
}
type PoolAcc = { publicKey: PublicKey; account: { poolState: PoolFeeFields } };

/** Pure: unclaimed fee rows for a set of pools (exported for tests). */
export function curveFeeRows(pools: readonly PoolAcc[], who: "creator" | "partner", config: PublicKey | null): CurveFeeRow[] {
  return pools
    .filter((p) => config === null || p.account.poolState.config.equals(config))
    .map((p) => {
      const a = p.account.poolState;
      const migrated = a.isMigrated !== 0;
      return {
        pool: p.publicKey.toBase58(),
        mint: a.baseMint.toBase58(),
        unclaimedLamports: big(who === "creator" ? a.creatorQuoteFee : a.partnerQuoteFee),
        unclaimedBase: big(who === "creator" ? a.creatorBaseFee : a.partnerBaseFee),
        migrated,
        ...(who === "partner" ? { surplusPending: migrated && a.isPartnerWithdrawSurplus === 0 } : {}),
      };
    })
    .sort((x, y) => (y.unclaimedLamports > x.unclaimedLamports ? 1 : y.unclaimedLamports < x.unclaimedLamports ? -1 : 0));
}

export const totalLamports = (rows: readonly { unclaimedLamports: bigint }[]) => rows.reduce((s, r) => s + r.unclaimedLamports, 0n);

/** Creator: curve fees on every pool this wallet created on Armory's platform config. */
export async function fetchCreatorCurveFees(conn: Connection, creator: PublicKey): Promise<CurveFeeRow[]> {
  const config = platformDbcConfig();
  if (!config) return [];
  const pools = await dbcClient(conn).state.getPoolsByCreator(creator);
  return curveFeeRows(pools, "creator", config);
}

/** Armory: partner fees on every pool of the platform config. */
export async function fetchPartnerCurveFees(conn: Connection): Promise<CurveFeeRow[]> {
  const config = platformDbcConfig();
  if (!config) return [];
  const pools = await dbcClient(conn).state.getPoolsByConfig(config);
  return curveFeeRows(pools, "partner", null);
}

/** DAMM v2 LP fees on positions `owner` holds, limited to SOL pools of the given launch mints (Armory graduations). */
export async function fetchLpFees(conn: Connection, owner: PublicKey, mints: ReadonlySet<string>): Promise<LpFeeRow[]> {
  if (mints.size === 0) return [];
  const damm = dammClient(conn);
  const positions = await damm.getPositionsByUser(owner);
  const out: LpFeeRow[] = [];
  for (const p of positions) {
    const pool = await damm.fetchPoolState(p.positionState.pool);
    if (!pool.tokenBMint.equals(WRAPPED_SOL_MINT) || !mints.has(pool.tokenAMint.toBase58())) continue;
    const f = getUnClaimLpFee(pool, p.positionState);
    out.push({
      pool: p.positionState.pool.toBase58(),
      position: p.position.toBase58(),
      positionNftAccount: p.positionNftAccount.toBase58(),
      mint: pool.tokenAMint.toBase58(),
      unclaimedLamports: big(f.feeTokenB),
      unclaimedBase: big(f.feeTokenA),
    });
  }
  return out;
}

export async function buildClaimCreatorFeeTx(conn: Connection, creator: PublicKey, pool: PublicKey): Promise<Transaction> {
  return dbcClient(conn).creator.claimCreatorTradingFee({ creator, payer: creator, pool, maxBaseAmount: U64_MAX, maxQuoteAmount: U64_MAX });
}

export async function buildClaimPartnerFeeTx(conn: Connection, feeClaimer: PublicKey, pool: PublicKey): Promise<Transaction> {
  return dbcClient(conn).partner.claimPartnerTradingFee({ feeClaimer, payer: feeClaimer, pool, maxBaseAmount: U64_MAX, maxQuoteAmount: U64_MAX });
}

export async function buildPartnerSurplusTx(conn: Connection, feeClaimer: PublicKey, pool: PublicKey): Promise<Transaction> {
  return dbcClient(conn).partner.partnerWithdrawSurplus({ feeClaimer, pool });
}

export async function buildClaimLpFeeTx(conn: Connection, owner: PublicKey, row: Pick<LpFeeRow, "pool" | "position" | "positionNftAccount">): Promise<Transaction> {
  const { PublicKey: PK } = await import("@solana/web3.js");
  const damm = dammClient(conn);
  const pool = new PK(row.pool);
  const st = await damm.fetchPoolState(pool);
  const { getTokenProgram } = await import("@meteora-ag/cp-amm-sdk");
  return damm.claimPositionFee({
    owner,
    position: new PK(row.position),
    pool,
    positionNftAccount: new PK(row.positionNftAccount),
    tokenAMint: st.tokenAMint,
    tokenBMint: st.tokenBMint,
    tokenAVault: st.tokenAVault,
    tokenBVault: st.tokenBVault,
    tokenAProgram: getTokenProgram(st.tokenAFlag),
    tokenBProgram: getTokenProgram(st.tokenBFlag),
  });
}
