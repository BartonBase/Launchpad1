/**
 * Public listings (Home, Explore) hide Armory's own test launches. Their on-chain names can't be
 * changed, so they are filtered by mint (HIDDEN_MINTS) and, as a backstop, by name. Hidden
 * launches still load by address (/t/<mint>) and still show in the holder's Portfolio.
 */
export const FIXTURE_NAME = /\b(devnet|e2e|test|plain|demo)\b/i;

/** Devnet test launches made while building Armory (Armory Plain Demo, Armory Hackathon Plain, Armory Test, …). */
export const HIDDEN_MINTS: ReadonlySet<string> = new Set([
  "AQ9p3TKktkTGPs1BjJkrvy6MAcRXZvgmMkJb7rqjGd63", // Armory Plain Demo (APLN)
  "FN1d6Z15FcxywjKRNaw26x3oYDc5eRjACzZDgVXLRoP4", // Armory Hackathon Plain (AHACK)
  "Dhv3VkqeTYJiahzcVJLmJ1snApdYtQxsAKeUnv5GTNos", // Armory Test (ARMT, Hybrid)
  "3GC9zFWzE2fVTFM7Q9Zo3BqCUArPYQEK57UVv4zvpJAu", // end-to-end test pool
  "Ewj5G9r1eTgX42JPWvpCxRmS52yCwUwXMhsuGpbnfuU6", // end-to-end test pool
  "9mhVWxbmAQNReUdkTcin4WW1Mc3ykkBhxfcR1ZBhzn3z", // unfinished first attempt at "Armory Hybrid Live" (curve only, never registered as Hybrid)
  "CzDzhYGoCP5BDc2gYmNnwd3s7gW5MWnK8VD7TrbCwVDQ", // Armory Hybrid Live (AHLIVE): web-app end-to-end test launch
]);

/** Demo collections shown first on Home and Explore, in this order. */
export const FEATURED_MINTS: readonly string[] = [
  "PyngwuDKX8ZDfsY91wVXgX78fFmz4w9F6Zc7tjBdMMU", // Forge Gems (GEMS), Hybrid demo collection
  "5VVsjp6oKi3mcSKb5YnvQ1MWZBPejqC57RLTqSy33ryE", // Shieldwall (SHLD), Hybrid demo collection
];

/** Sort key: featured mints first (in FEATURED_MINTS order), then everything else in its original order. */
export const featuredRank = (mint: string): number => {
  const i = FEATURED_MINTS.indexOf(mint);
  return i < 0 ? FEATURED_MINTS.length : i;
};

export const isListed = (name: string | null | undefined, mint?: string | null): boolean =>
  !FIXTURE_NAME.test(name ?? "") && !(mint && HIDDEN_MINTS.has(mint));

/** Hybrid launches carry two names (token and collection); both must pass. */
export const isHybridListed = (l: { tokenName?: string | null; collectionName?: string | null; mint: string }): boolean =>
  isListed(l.tokenName, l.mint) && isListed(l.collectionName, l.mint);
