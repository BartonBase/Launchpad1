#!/usr/bin/env python3
"""
PoC 02 (EXECUTED, off-chain model): MPL-Hybrid V2 escrow accounting hazards.
Faithful model of the *checks that exist* in capture_v2.rs / release_v2.rs / init_escrow_v2.rs /
init_recipe.rs / migrate_tokens_v1.rs at commit aacf1a53. Models only what the on-chain handlers
enforce; line refs inline. Pure arithmetic, no cluster/keys/funds.

Two hazards demonstrated:
  A) SHARED ESCROW ACROSS RECIPES. EscrowV2 is PDA ["escrow", authority] (init_escrow_v2.rs:11-14);
     capture/release seed the escrow by recipe.authority (capture_v2.rs:44-48). One authority running
     two recipes over the SAME token shares ONE escrow token ATA. Backing deposited by collection X is
     spendable by releases of collection Y -> cross-collection insolvency / drain.
  B) BurnOnCapture / BurnOnRelease reduce supply AND break the exact-unwrap backing invariant.
     capture_v2.rs:261-272 burns the user's `amount` on capture instead of escrowing it; a later release
     pays `amount` out of the shared escrow (release_v2.rs:287) that was never funded -> escrow goes
     insolvent / underflows.
"""
import sys

class Escrow:
    """Models the single escrow token ATA keyed by authority."""
    def __init__(self): self.balance = 0
    def deposit(self, n): self.balance += n
    def withdraw(self, n):
        # on-chain this is a spl_token::transfer signed by the escrow PDA; it fails if balance < n
        if self.balance < n: raise RuntimeError("SPL transfer would fail: escrow insolvent")
        self.balance -= n

results = {}
def check(name, cond, detail=""):
    results[name] = bool(cond); print(("PASS " if cond else "FAIL ")+name+(" :: "+detail if detail else ""))

# ---------- Hazard A: shared escrow across two recipes/collections, same authority + token ----------
esc = Escrow()                      # ONE escrow ATA (seeds ["escrow", authority]) shared by both recipes
R_A = 1_000_000                     # recipe A: amount per swap (collection A backing)
R_B = 1_000_000                     # recipe B: amount per swap (collection B backing)
# Users of collection A capture 3 NFTs -> deposit 3*R_A backing into the shared escrow.
for _ in range(3): esc.deposit(R_A)
backing_from_A = 3*R_A
# Collection B has NFTs in the shared escrow but ZERO backing deposited for them.
# A holder of a B NFT calls release_v2 on recipe B: escrow pays R_B out of the shared pool.
# release_v2.rs only checks escrow ATA owner==escrow & mint==recipe.token (utils.rs:34-46); it does NOT
# check the backing came from collection B.
before = esc.balance
esc.withdraw(R_B)                   # succeeds because A's backing is present
after = esc.balance
check("A_cross_collection_release_drains_shared_pool", after == before - R_B,
      f"B-release removed {R_B} from a pool funded entirely by A (shared escrow ['escrow',authority])")
# Drain the rest with B releases until A's backing is gone; A NFTs are now unbacked.
esc.withdraw(R_B); esc.withdraw(R_B)
check("A_pool_fully_drained_by_other_collection", esc.balance == 0,
      "3xR_A backing removed by 3 collection-B releases; collection-A NFTs are now unbacked")

# ---------- Hazard B: BurnOnCapture underfunds escrow, release later underflows ----------
class Mint:
    def __init__(self, supply): self.supply = supply
    def burn(self, n):
        if n > self.supply: raise RuntimeError("burn exceeds supply")
        self.supply -= n
SUPPLY0 = 1_000_000_000
mint2 = Mint(SUPPLY0)
esc2 = Escrow()
state2 = {"outside": 0}
# Path::BurnOnCapture set -> capture BURNS the user's `amount` (capture_v2.rs:261-272); escrow NOT funded,
# yet one NFT leaves the escrow (nfts_outside += 1).
def capture_burn(n):
    mint2.burn(n); state2["outside"] += 1
# release_v2 always pays `amount` out of the escrow (release_v2.rs:287); BurnOnRelease burns the NFT-side
def release_pays(n):
    esc2.withdraw(n); state2["outside"] -= 1
capture_burn(R_A); capture_burn(R_A)    # 2 captures
insolvent = False
try:
    release_pays(R_A)             # holder tries to unwrap -> escrow has 0
except RuntimeError:
    insolvent = True
check("B_burn_on_capture_makes_release_insolvent", insolvent and esc2.balance < R_A * state2["outside"],
      f"escrow={esc2.balance} < R x outside={R_A*state2['outside']}; first release fails")
# Supply is computed, not asserted: after the burns the mint supply is below the fixed 1B.
check("B_burn_paths_violate_fixed_supply", mint2.supply < SUPPLY0 and mint2.supply == SUPPLY0 - 2 * R_A,
      f"supply {SUPPLY0} -> {mint2.supply} after 2 BurnOnCapture captures (fixed-1B / exact-unwrap invariant broken)")

# ---------- Control: honest classic-SPL config keeps the invariant ----------
esc3 = Escrow(); COLLECTION = 10
# capture = tokens -> NFT: user deposits R into escrow, one NFT leaves escrow (nfts_outside += 1)
# release = NFT -> tokens: NFT returns to escrow, escrow pays R out            (nfts_outside -= 1)
state = {"outside": 5}
for _ in range(5): esc3.deposit(R_A)      # 5 NFTs already in circulation, escrow holds 5R
def honest_capture(): esc3.deposit(R_A); state["outside"] += 1
def honest_release(): esc3.withdraw(R_A); state["outside"] -= 1
ok = True
import random; rng = random.Random(0)
for _ in range(10000):
    if state["outside"] < COLLECTION and (state["outside"] == 0 or rng.random() < 0.5): honest_capture()
    elif state["outside"] > 0: honest_release()
    if esc3.balance != R_A*state["outside"]: ok = False; break
check("C_classic_spl_no_burn_keeps_backing_invariant", ok,
      "escrow_balance == R x nfts_outside holds across 10k random honest capture/release ops")

print("\nSUMMARY:", sum(results.values()), "/", len(results), "checks passed")
sys.exit(0 if all(results.values()) else 1)
