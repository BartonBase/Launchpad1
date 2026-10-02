# Burn mode (Mode 3): blind assignment, PROPOSED (not built; needs audit sign-off)

## Problem
`wrap_permanent` burns `ratio` tokens and mints asset index `vault.minted_count` in one instruction.
The leaf for every index (traits, URI) is public before minting, because the minter must pass it.
So the next piece is always known and a rare one can be sniped by timing the burn. Any single-instruction
design has this property: whoever supplies the leaf knows what they get before signing. A real fix
needs the program to pick the index AFTER the user is committed, i.e. a two-step flow.

## Proposed design (smallest change that reuses Hybrid's audited pieces)
1. `request_burn` (user): moves `ratio` tokens from the user's ATA into a vault-authority escrow ATA
   (NOT burned yet), pays the tier fee to `PLATFORM_FEE_RECIPIENT` and the lazy-mint deposit (as Hybrid's
   capture), and opens a `Request` (kind = BURN) bound to a Switchboard On-Demand randomness account
   committed in that slot (same init / commit / RandLock rules as Hybrid).
2. `reveal_randomness`: the existing permissionless reveal, generalised to accept the Mode 3 vault.
3. `settle_burn` (permissionless): requires the revealed value; picks uniformly from the remaining
   unminted indices with the same swap-remove pool as Hybrid (`selection.rs`); checks the picked leaf +
   proof; burns the escrowed `ratio` (vault authority signs; supply must drop by exactly `ratio`);
   lazy-mints the Core asset to `request.user`; refunds the unused deposit; closes the request.
4. `expire_burn` (permissionless, Hybrid's last-resort rules): if the value is never revealed, returns
   the escrowed tokens and the full deposit; the flat SOL fee is not refunded (as in Hybrid).

No re-roll and no release (Mode 3 stays one-way). Fee amount and recipient unchanged.

## Why this is material
- New accounts: token escrow ATA, pool of unminted indices, `Request`/`RandLock` for Mode 3, new
  `PermanentVault` fields (queue, sequence counters, pending count, pool key). No Mode 3 vault exists on
  any cluster, so the layout change breaks nothing deployed.
- Changed fund flow: user tokens sit in program custody between request and settle (new drain surface),
  plus a refund path. Today's wrap burns atomically and holds nothing.
- Shared code: the randomness instructions are typed to the Hybrid `Vault`; they must be generalised
  or duplicated for Mode 3.

## Size estimate
- Programs: ~500–700 changed lines (Mode 3 grows from 348 to ~900; randomness/selection reuse).
- LiteSVM tests: ~400–600 lines (uniform pick, no pre-knowledge of the index, escrow cannot be drained,
  supply drops exactly `ratio` per settle, expiry refund, double settle, wrong leaf, wrong queue).
- App: ~300 lines (builders, wizard Burn path, token page burn panel reusing reveal/settle UI).
- Devnet: both programs upgraded (current program data has room; no extend expected).

## Rejected cheaper options
- Slot-hash entropy in the same instruction: the user still supplies the leaf, so they would retry until
  the rare one lands. Worse than today.
- Slot-hash two-step (no oracle): same new accounts and escrow as above, and leader-biasable.
- Secret manifest held by a server that hands out the next leaf: needs a server secret, and the holder
  (creator or Armory) can still snipe.
