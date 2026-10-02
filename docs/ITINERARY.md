# Armory website itinerary (2026-10-01)

Based on Claude's "Frontend design brief & itinerary" PDF, updated for Barton's decisions. Name: Armory (ticker ARMS). Mascot: a comic-looking knight. No animation.

1. **Name check.** Domain, trademark and X handle for Armory. The name is final; this step only picks the best domain and handle.
2. **Brand kit** (Creative Director). A dark palette with exactly one bright accent and a headline font plus the current data font. The knight mascot, logo, favicon and app icon are on hold: Barton will make the mascot in Higgsfield and the team builds from that.
3. **Design system.** Reusable buttons, cards, badges, chart styling, and an icon for each of the five launch types. Tax and raffle get "coming soon" versions.
4. **Site map.** Landing, explore, token page, launch wizard, portfolio, trust and security page, FAQ. The token page and wizard switch by launch type, so tax and raffle slot in later without a redesign.
5. **Marketing landing page.** A hero with the knight, a "coin that's also an NFT" explainer, a launch-type comparison, the trust section, and a visible beta banner.
6. **App screens.** Explore, token pages for plain, hybrid and burn, the launch wizard with live math, capture, wrap/unwrap and re-roll flows, and the portfolio.
7. **Safety and trust features.** An "unaudited beta" label, the deposit cap shown in the wizard and trades, the locked-authorities panel, a bug bounty page, and a published rule that no AI holds mainnet keys.
8. **Mobile, accessibility and wording.** A real mobile layout, contrast fixes, keyboard navigation through the wizard, and consistent plain-English copy in the brand voice.
9. **Build** (Frontend Engineer). A working site wired to the devnet programs, with tax and raffle behind a toggle.
10. **Testing.** Cross-browser and phone testing, performance on busy pages, and fuzz testing on the programs.
11. **Launch gate.** Paid audit, live bug bounty, deposit cap on, upgrade keys held only by humans. Then update the "audit pending" badges.
12. **Later: tax and raffle.** Only after the stuck-funds fixes, a public buy path, and a raffle legal check.

## Changed or dropped from Claude's plan
- Mascot: the fused creature is out. The comic knight replaces it, and the bomb is superseded.
- Animation: Claude's motion phase is dropped entirely. Still poses only.
- Colors: the gold and burgundy wax-seal palette is dropped. One bright accent goes with the knight.
- Name: Fuze is replaced by Armory. Fuze/Fuse had clashes (Fuze Finance, the Fuse wallet on Solana, Fuse Network).
- Randomness toggle: the wizard's "VRF *or* pre-committed" choice is fixed to "both". Artist-defined rarity is fine, but assignment stays random.
- Launch list: all five types, with plain, hybrid and burn live.
