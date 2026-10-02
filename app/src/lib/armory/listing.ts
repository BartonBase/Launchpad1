/**
 * Public listings (Home, Explore) hide Armory's own end-to-end test fixtures. Their on-chain names
 * ("Devnet E2E", "Mintmark Devnet E2E") can't be changed, so they are filtered by name. Hidden
 * launches still load by address (/t/<mint>) and still show in the holder's Portfolio.
 */
export const FIXTURE_NAME = /\b(devnet|e2e)\b/i;
export const isListed = (name: string | null | undefined): boolean => !FIXTURE_NAME.test(name ?? "");
