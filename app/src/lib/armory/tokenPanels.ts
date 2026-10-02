/**
 * Token page panels by launch type and phase: design/sitemap.md "Token page: panels by launch
 * type". Pure so the matrix is unit-tested against the sitemap.
 */
export type TokenType = "plain" | "hybrid" | "burn";
export type Phase = "curve" | "graduated" | "native";

export type Panel =
  | "market"
  | "curveCard"
  | "graduatedCard"
  | "tradePanel"
  | "plainFacts"
  | "convertCapture"
  | "convertRelease"
  | "burnToMint"
  | "holdings"
  | "yourNfts"
  | "reroll"
  | "collectionPreview"
  | "collectionOrder"
  | "floor"
  | "supplyBurned"
  | "tradeNfts"
  | "feesControl"
  | "nobodyCanChange"
  | "facts";

const BASE: readonly Panel[] = ["market", "tradePanel", "holdings", "feesControl", "nobodyCanChange", "facts"];

const BY_TYPE: Record<TokenType, readonly Panel[]> = {
  plain: ["plainFacts"],
  hybrid: ["convertCapture", "convertRelease", "yourNfts", "reroll", "floor", "tradeNfts"],
  burn: ["burnToMint", "yourNfts", "collectionOrder", "supplyBurned", "tradeNfts"],
};

export function tokenPanels(type: TokenType, phase: Phase): ReadonlySet<Panel> {
  const s = new Set<Panel>([...BASE, ...BY_TYPE[type]]);
  // A native devnet test launch has no curve; treat it like "curve" for card selection
  // (converting can never open, which the card explains).
  s.add(phase === "graduated" ? "graduatedCard" : "curveCard");
  if (type !== "plain" && phase !== "graduated") s.add("collectionPreview");
  return s;
}

/** Holdings mode per type (sitemap "Holdings" row). */
export const HOLDINGS_MODE: Record<TokenType, "tokens" | "tokens+nfts-valued" | "tokens+nfts-not-valued"> = {
  plain: "tokens",
  hybrid: "tokens+nfts-valued",
  burn: "tokens+nfts-not-valued",
};
