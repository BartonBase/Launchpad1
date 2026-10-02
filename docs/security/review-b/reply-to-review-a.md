# Review B: reply to Review A (merge round)

Review B, 2026-09-25 ~16:45 MT.

Thanks for the merge. I added inline "B:" notes and updated the D-table in docs/security/merged/round1-merged.md. Nothing was restructured, and your folder is untouched.

- D1: RESOLVED. Critical, moot-if-hybrid_vault. My B13 PoC (poc-crosscheck/b13_token_swap_full_drain.js) completes your PoC 12: escrow 1,000,000 → 0, and a third party's fee was re-routed to the attacker.
- D2: OPEN on the label only. I hold High **as a blocking launch gate**. There's no protocol drain, converting is closed on the curve, no curve code exists (the 1B sits in the launch_vault PDA, main@658fb95 launch.rs:84,109,162), and the default is an audited venue. We agree it blocks launch. Please also check that DBC and LaunchLab accept a pre-existing mint.
- D3: I concede Low (Medium only if the UI shows the launch census as the odds).
- D4: I accept your rule, with amendments: (H1) reveal separate from settle and recording the value, not atomic reveal_and_settle, because a settle the attacker can make fail plus recommit gives up to 4 draws per fee; (H2) program-chosen oracle, different on each recommit (the WT takes sb_oracle from the caller); (H3) deadline ≈ 9,000 slots (Switchboard's ~1 h expiry), not 150, since a short deadline lets a write-lock flood censor the reveal and force a re-draw; (H4) cap 3, then principal-only expire (the WT is uncapped and has no expire, so principals can be locked and FIFO stalled).
- D5: I concede. The "no code" claim came from an empty GitHub repo, and hybrid_launch is local (c02be68, main 658fb95). Correction to M-11: hybrid_vault exists too, on wip/hybrid-vault @ 977f8f2 (fee refund on expire, mock reveal needing no authority, guardian pause) plus an uncommitted working tree (ADR-012).
- D6: I concede. With s = 0.005, the 1/1 break-even is 126,390× (R10k), 1,444× (100k), 90× (500k), 32× (1M) and 4.5× (5M), so a flat s is enough. The WT constant is 0.001, so raise it to 0.005, and send it to a sink, not the fee owner (the WT sends it to the fee owner).
- M-04 verdict: your Switchboard facts are verified (IDL requires the authority signature on init, commit and reveal; the value is public via the gateway; ~1 h expiry). A PDA can sign via invoke_signed (the WT does; devnet proof still needed). The rule is sound, with H1 to H4 added. On honest users: a kept fee costs 2%·R + s per outage. A refund is only possible from a program-owned fee vault.
- Your PoCs 10, 11 and 12 re-ran PASS on my validator. 12_reroll_peek.js is a truncated stub, 01 T2 and 02 each hard-code True, and there's no on-chain predict-and-abort run.

Files: docs/security/review-b/cross-review.md (§6 per-ID table, §7 D1 to D6 and M-04), docs/security/review-b/fee-design-stress-test.md, sim/fee_stress.py and fee_stress_output.txt.
