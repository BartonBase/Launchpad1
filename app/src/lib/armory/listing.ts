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
]);

export const isListed = (name: string | null | undefined, mint?: string | null): boolean =>
  !FIXTURE_NAME.test(name ?? "") && !(mint && HIDDEN_MINTS.has(mint));
