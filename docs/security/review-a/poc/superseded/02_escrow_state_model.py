#!/usr/bin/env python3
"""
PoC 02 - executable state-machine model of MPL-Hybrid V2 (recipe + shared escrow)
at commit aacf1a53c8a43bf395db7b562c02cf016ce604c5. Each instruction below mirrors
the checks (and ONLY the checks) of the on-chain handler; line refs are to
programs/mpl-hybrid/src/instructions/*.rs. This is a model, NOT an on-chain run:
the upstream crate does not build with the local SBF toolchain (see README.md).
Pure local computation: no cluster access, no keys, no funds.
"""
from dataclasses import dataclass, field

class TxError(Exception): pass

NO_REROLL, BLOCK_CAPTURE, BLOCK_RELEASE, BURN_ON_CAPTURE, BURN_ON_RELEASE = 0, 1, 2, 3, 4
bit = lambda path, b: bool(path & (1 << b))

@dataclass
class Collection: key: str; update_authority: str
@dataclass
class Asset: key: str; collection: str; owner: str; rarity: int = 0
@dataclass
class Recipe:
    collection: str; authority: str; token: str; fee_location: str
    amount: int; fee_amount_capture: int; fee_amount_release: int; path: int; count: int = 1
    max: int = 2; min: int = 0

@dataclass
class World:
    bal: dict = field(default_factory=dict)          # (owner, mint) -> amount
    supply: dict = field(default_factory=dict)
    collections: dict = field(default_factory=dict)
    assets: dict = field(default_factory=dict)
    escrows: dict = field(default_factory=dict)      # "escrow:"+authority -> authority
    recipes: dict = field(default_factory=dict)      # collection -> Recipe

    # --- SPL token (legacy transfer / burn semantics) ---
    def mint_to(self, who, mint, amt):
        self.bal[(who, mint)] = self.bal.get((who, mint), 0) + amt
        self.supply[mint] = self.supply.get(mint, 0) + amt
    def transfer(self, fr
