/**
 * Meteora DAMM v2 (cp-amm) integration via the official @meteora-ag/cp-amm-sdk. After a DBC curve
 * graduates, DBC migrates the liquidity into a DAMM v2 pool (token A = the launch's mint, token B =
 * wrapped SOL). Armory finds that pool by token A mint and trades through it with exact-in quotes
 * and minimumAmountOut slippage protection.
 */
import { PublicKey, type Connection, type Transaction } from "@solana/web3.js";
import BN from "bn.js";
import { CpAmm, SwapMode, getCurrentPoint, getPriceFromSqrtPrice, type PoolState } from "@meteora-ag/cp-amm-sdk";
import { TOKEN_PROGRAM_ID, WRAPPED_SOL_MINT } from "@/config/programs";
import type { Side, SwapQuoteView } from "./dbc";

const clients = new WeakMap<Connection, CpAmm>();
export function dammClient(conn: Connection): CpAmm {
  let c = clients.get(conn);
  if (!c) {
    c = new CpAmm(conn);
    clients.set(conn, c);
  }
  return c;
}

export interface DammPoolDTO {
  readonly pool: string;
  readonly priceSol: number;
  readonly liquidity: string;
}

async function findPool(conn: Connection, mint: PublicKey): Promise<{ publicKey: PublicKey; account: PoolState } | null> {
  const pools = await dammClient(conn).fetchPoolStatesByTokenAMint(mint);
  const sol = pools.filter((p) => p.account.tokenBMint.equals(WRAPPED_SOL_MINT));
  if (sol.length === 0) return null;
  // Deepest SOL pool (the migrated one, normally the only one).
  sol.sort((a, b) => b.account.liquidity.cmp(a.account.liquidity));
  return sol[0]!;
}

/** The DAMM v2 SOL pool for a graduated mint, or null. */
export async function fetchDammPool(conn: Connection, mint: PublicKey, decimals: number): Promise<DammPoolDTO | null> {
  const p = await findPool(conn, mint);
  if (!p) return null;
  const price = Number(getPriceFromSqrtPrice(p.account.sqrtPrice, decimals, 9).toString());
  return { pool: p.publicKey.toBase58(), priceSol: Number.isFinite(price) ? price : 0, liquidity: p.account.liquidity.toString() };
}

async function load(conn: Connection, pool: PublicKey): Promise<PoolState> {
  return dammClient(conn).fetchPoolState(pool);
}

export async function quoteDammSwap(conn: Connection, pool: PublicKey, mint: PublicKey, decimals: number, side: Side, amountIn: bigint, slippageBps: number): Promise<SwapQuoteView> {
  const st = await load(conn, pool);
  const currentPoint = await getCurrentPoint(conn, st.activationType);
  const q = dammClient(conn).getQuote2({
    inputTokenMint: side === "buy" ? WRAPPED_SOL_MINT : mint, slippage: slippageBps /* cp-amm takes bps */, currentPoint, poolState: st,
    tokenADecimal: decimals, tokenBDecimal: 9, hasReferral: false, swapMode: SwapMode.ExactIn, amountIn: new BN(amountIn.toString()),
  });
  const b = (x: BN | undefined) => (x ? BigInt(x.toString()) : 0n);
  return {
    amountIn,
    amountOut: b(q.outputAmount),
    minimumAmountOut: b(q.minimumAmountOut),
    tradingFee: b(q.claimingFee) + b(q.compoundingFee) + b(q.protocolFee),
    protocolFee: b(q.protocolFee),
    priceImpactPct: Number(q.priceImpact.toString()),
    partial: b(q.amountLeft) > 0n,
  };
}

export async function buildDammSwapTx(conn: Connection, owner: PublicKey, pool: PublicKey, mint: PublicKey, side: Side, amountIn: bigint, minimumAmountOut: bigint): Promise<Transaction> {
  const st = await load(conn, pool);
  return dammClient(conn).swap2({
    payer: owner, pool, inputTokenMint: side === "buy" ? WRAPPED_SOL_MINT : mint, outputTokenMint: side === "buy" ? mint : WRAPPED_SOL_MINT,
    tokenAMint: st.tokenAMint, tokenBMint: st.tokenBMint, tokenAVault: st.tokenAVault, tokenBVault: st.tokenBVault,
    tokenAProgram: TOKEN_PROGRAM_ID, tokenBProgram: TOKEN_PROGRAM_ID, referralTokenAccount: null, poolState: st,
    swapMode: SwapMode.ExactIn, amountIn: new BN(amountIn.toString()), minimumAmountOut: new BN(minimumAmountOut.toString()),
  });
}
