# Audit fixes: round 1 (merged findings M-01..M-41, Auditor A v2.1)

Baseline: tag `audit-baseline-r1` on `wip/hybrid-vault` (local only). Source of findings:
`security/merged/round1-merged.md`. Tests: `tests/track-a-hybrid/vault/vault.rs` (V), `launch/launch.rs` (L),
program unit tests (U). Localnet/LiteSVM only; not audited.

Status key: **closed** = fixed + regression test · **moot** = not applicable to hybrid_vault · **accepted** =
documented trust assumption · **open** / **blocked** = not done (reason given) · **Barton** = needs a decision.

| ID | Status | Fix (file) | Regression test(s) |
|---|---|---|---|
| M-01 | moot | No admin/update ix in hybrid_vault; economics immutable | V `idl_has_no_cancel_refund_update_close_withdraw_or_burn_instruction`, `regress_poc10_escrow_drain_must_fail` |
| M-02 | closed | VRF selection at settle; the caller never names the asset (`selection.rs`, `settle.rs`) | V `attack_capturer_cannot_choose_asset_settle_with_other_asset_rejected`, `regress_poc11_cherrypick_must_fail`, `attack_user_cannot_abort_after_reveal_recommit_refused_once_revealed` |
| M-03 | closed on localnet (real DBC program; graduation simulated) | DBC creates the mint. `hybrid_launch::register_dbc_launch` checks the config is on the compile-time platform allowlist `APPROVED_DBC_CONFIGS`, then verifies DBC's pool, config and mint (owner, discriminator, length, `base_mint`, config link, pool creator = signer, SPL, not yet migrated, wSOL quote, fixed 1B with pre == post, decimals, Immutable, `leftover_receiver` = our buffer PDA, threshold in range; mint on classic Token with mint and freeze authority None and supply exactly 1B). It records `dbc_config` and `dbc_pool` (LaunchConfig v4, ADR-014) | V `vault_dbc_graduation`: `register_records_dbc_pool_and_mint`, `register_rejects_wrong_signer_wrong_config_fake_pool_and_repeat`, `register_rejects_config_without_our_buffer_or_with_burn_or_low_threshold`, `register_rejects_already_migrated_pool`, `register_rejects_config_not_on_platform_allowlist`, `research_dbc_mint_is_created_by_dbc_with_authorities_revoked` |
| M-04 | closed, with two parts open | Program-chosen oracle + heartbeat filter (skip only with proof), `OracleStale`; recommit ≤ 3; expire refunds principal + escrow, not the fee (`randomness.rs`, `randomness_ix.rs`, `expire.rs`). Batch expire `expire_requests(count ≤ 7)` added after the baseline. **Open:** real devnet reveal (init/commit/recommit are proven against the real program in LiteSVM, `vault/real_switchboard.rs`) | V `m04_stale_oracle_is_skipped_only_with_proof_and_live_oracle_cannot_be_skipped`, `attack_caller_chosen_oracle_is_rejected_program_selects_it`, `recommits_are_capped_then_expire_returns_principal_only_fee_kept`, `expire_several_stuck_requests_in_one_transaction`, `m04_batch_expire_*` (5 tests) |
| M-05 | closed | Fee only to the `PLATFORM_FEE_RECIPIENT` constant (account constraint in request.rs); launch-time recipient check | V `attack_fee_to_own_wallet_or_other_recipient_is_rejected_fail_closed`; L `attack_fee_recipient_not_system_owned_or_executable_is_rejected` |
| M-06 | closed (copy: Barton) | One tier fee for capture and re-roll; release free (ADR-013) | V `reroll_charges_same_flat_sol_fee_moves_no_tokens_and_never_returns_handed_in_asset`, `capture_then_release_returns_exactly_ratio_tokens_and_release_is_free` |
| M-07 | closed | `hybrid_launch` derives the fee from the ratio tier; no token fee (`needs_barton.rs`, `validation.rs`) | L `fee_tier_is_derived_from_ratio_for_all_7_ratios_and_stored_immutably`, `attack_fee_cap_cannot_be_exceeded_and_params_carry_no_fee_or_recipient` |
| M-08 | closed in code (disclosure: Barton) | Charge `min(stored, tier or u64::MAX, MAX_FEE)`; never reverts on mismatch; version == 4 fail-closed (`config.rs::econ`) | V `m08_request_fee_is_min_of_stored_tier_and_cap_never_an_exact_match_revert` |
| M-09 | open (Barton) | Squads multisig + timelock plan in `admin-multisig-timelock.md`; one upgrade key per program today | none |
| M-10 | closed on localnet (graduation simulated) | `graduation::verify` is the real DBC check in every build: the proof must be the recorded `dbc_pool`, owned by DBC with the right discriminator and length, naming this mint and config, with `is_migrated == 1` and progress CreatedPool. The mock is additionally accepted only under `test-mock-graduation`. The 25% buffer goes to ATA(`["dbc_buffer"]` PDA, mint) through DBC's own `withdraw_leftover`, and no instruction signs for the PDA. **Limit:** the migration itself is simulated by flipping the two pool bytes, because a real DAMM v2 migration isn't loaded; a real migrated devnet pool is parsed to confirm the fields | V `open_vault_needs_the_recorded_dbc_pool_to_have_migrated`, `buffer_receives_leftover_from_real_dbc_and_can_never_be_withdrawn`, `buffer_seed_is_never_used_for_signing`, `real_devnet_migrated_pool_parses_as_graduated`, `attack_open_without_verified_graduation_rejected` |
| M-11 | in progress | hybrid_vault committed at this baseline; not audited | full V suite (70) |
| M-12 | moot | No shared escrow, no burn path | V `regress_poc12_token_swap_must_fail` |
| M-13 | resolved (native launch and DBC) | Native: 1B minted, mint and freeze authority revoked (`launch.rs`). DBC: DBC mints 1B into its base vault (owner = DBC pool authority) and revokes the mint authority in the same ix; `register_dbc_launch` re-checks authorities None and supply == 1B, and pre == post means DBC burns nothing at migration | L `launch_mints_exactly_1b_revokes_mint_and_freeze_authority_and_records_immutable_config` |
| M-14 | closed | The fee is SOL and never backing; exact token deltas; release free | V `capture_fee_is_flat_sol_to_fixed_recipient_no_token_fee_no_burn`, `property_solvency_invariant_holds_under_random_operation_sequences` |
| M-15 | closed | Release takes no fee account | V `release_succeeds_when_fee_recipient_does_not_exist_or_was_emptied`, `…_is_program_owned`, `…_is_executable` |
| M-16 | **accepted** | Insider discount / upgrade trust assumption documented (THREAT_MODEL T-GRAD-05, ADR-017) | n/a |
| M-17 | Barton | Fee wallet custody | n/a |
| M-18 | superseded | SOL fee; disclosure kept | n/a |
| M-19 | Barton (disclosure) + partial | Strict pool floor `free > max(5, 2% N)` checked before any charge | V `pool_floor_rejects_before_any_fee_or_token_is_charged`, `pool_floor_math_is_max_5_or_2_percent_never_below_2`, `capture_down_to_pool_floor_then_next_request_rejected_no_asset_available` |
| M-20 | partial | Pool floor; FIFO + permissionless expire unblocks the head | V `recommits_are_capped_then_expire_returns_principal_only_fee_kept`, `attack_settle_out_of_fifo_order_rejected` |
| M-21 | closed on-chain; verifier open | Leaf binds index, traits, sha256(image), sha256(JSON), content-addressed URI; verified at the lazy mint (`merkle.rs`, `settle.rs`). The off-chain image verifier isn't built | V `attack_settle_never_minted_pick_without_or_with_wrong_leaf_is_rejected` |
| M-22 | closed (superseded by lazy mint) | No crank or graduation fund (ADR-016); vault-side `collection_size ≤ 10,000` (`CollectionAboveCap`) | V `m22_vault_rejects_collection_size_above_cap`, `mint_crank_instruction_no_longer_exists`; L `lazy_mint_no_affordability_rule_any_n_up_to_10k_at_min_threshold` |
| M-23 | closed | Seq-tagged incoming ring; handed-in index pushed after the pick; leaf v2 binding | V `unwrapped_nft_is_not_drawable_by_earlier_pending_request`, `every_index_is_drawable_from_the_start_minted_or_not`, `uniform_below_reaches_every_index_and_is_unbiased_enough` |
| M-24 | resolved | n/a (UI) | n/a |
| M-25 | open | Curve integration not built | none |
| M-26 | closed | No pause (ADR-015) | V `no_pause_path_exists_no_key_can_halt_any_instruction` |
| M-27 | Barton | Creator allocations outside the pool | n/a |
| M-28 | moot | upstream | n/a |
| M-29 | closed | Compile-time fee bounds (`tiers_within_bounds`) | U hybrid_launch |
| M-30 | closed | Recipient must be system-owned, data-less, non-executable at launch | L `attack_fee_recipient_not_system_owned_or_executable_is_rejected` |
| M-31 | open | MEV around converts; the VRF two-step removes pick MEV | none |
| M-32 | moot + guard | | V `no_todo_or_fixme_left_in_program_sources` |
| M-33 | closed in this baseline | DECISIONS ADR-011..017, THREAT_MODEL, graduation-design, marketplaces, qa-answers, hybrid-rarity purged of burn/2%/bps | n/a (docs) |
| M-34 | moot | | n/a |
| M-35 | partial | Pinned program ids, discriminators, owner checks | V `attack_fake_program_ids_rejected`, `attack_randomness_not_owned_by_switchboard_rejected`, `attack_randomness_on_other_queue_rejected` |
| M-36 | closed | Release is free (Barton 5:04 PM MT); FeeVault/sweep removed | V `release_has_no_fee_account_in_its_interface` |
| M-37 | decided; partly measured | The requester pays VRF at cost, separately (ADR-012). Mainnet queue reward 5 lamports; estimate ≈ 0.00002 SOL/request. One-off `init_randomness` setup measured against the real devnet Switchboard program in LiteSVM: 8,229,760 lamports, reusable. The per-reveal oracle fee still needs a devnet run, which is blocked on devnet SOL | `real_switchboard_init_cost_breakdown` |
| M-38 | partial | Mock graduation only under `test-mock-graduation` (built into `target/test-sbf`); production `.so` checked for the marker (0 hits); `deploy-devnet.sh` refuses if `mock_switchboard.so` is in `target/deploy`. The real Switchboard program (dumped from devnet `Aio4gaXj…`, devnet queue, state and oracle accounts as fixtures) runs our hand-built init/commit/recommit CPIs in LiteSVM and rejects a forged reveal with `InvalidSecpSignature`. A gateway-signed reveal still needs devnet (blocked on SOL). The deploy script stages an allowlist and checks markers (09f8b3a) | `vault/real_switchboard.rs` (4 tests), `scripts/test-deploy-guards.sh` |
| M-39 | closed | Tag `audit-baseline-r1` | n/a |
| M-40 | closed | INV-1 counts pending re-rolls | V `property_solvency_invariant_holds_under_random_operation_sequences` |
| M-41 | closed | Exits read the frozen LaunchConfig prefix (`config::exit_view`, `hybrid_launch::stable_layout`, ADR-017) | V `m41_*` (6 tests); U `offsets_are_frozen`, `appended_field_keeps_prefix` |

## Lazy mint (ADR-016, Barton 5:13 PM MT); not a numbered finding

Escrow PDA per request; uniform pick over all held indices; minted bitmap; on-chain leaf verification; settler pays
nothing; expire refunds principal + escrow. Tests: V `request_escrows_worst_case_mint_cost_in_a_per_request_escrow_pda`,
`first_pick_is_minted_to_user_from_escrow_settler_pays_only_tx_fee_no_tip`, `returned_asset_is_transferred_again_never_reminted`,
`reroll_hand_in_returns_to_vault_no_remint_no_burn`, `expire_refunds_principal_and_full_mint_escrow_never_the_fee`,
`many_captures_keep_minted_count_le_n_and_bitmap_consistent`, `attack_prefund_asset_pda_does_not_block_lazy_mint_lamports_go_to_the_user`.
Interface: [lazy-mint-interface.md](lazy-mint-interface.md).

## Known QA-suite review triggers (QA-owned files, not edited)

- `qa_regression` (QA's ported suite) now runs by default, with the `required-features` line removed: 52/1/0 at HEAD.
  - `qa_M01_A01_no_instruction_can_change_ratio_fees_or_mint` flags the new vault instruction `expire_requests`
    (batch expire, M-04). It only runs the existing expire core per request (principal + escrow back, fee kept), with
    no economics or escrow mutation. It needs QA review, then QA's allowlist gets `expire_requests`.
  - QA's 53/0/0 predates `2be21da` and `786a1c9`. `qa_M10_GRAD02b` (production build → 6037) and `qa_M10_GRAD02`
    (test build → 6036) both pass again: a launch with no DBC pool yields `GraduationCheckUnavailable` in
    production and `GraduationNotVerified` in mock builds.
- `qa_launch`: `qa_hl02_no_withdraw_path_from_launch_vault`, `qa_FEE13_M08_fee_immutable_after_launch_each_tier` and
  `qa_FEE05_M05_fee_wallet_fixed_at_launch_and_immutable` assert the IDL is exactly `["launch"]`. The new
  `register_dbc_launch` (ADR-014) only `init`s a new LaunchConfig, never takes `launch_vault` or
  `launch_destination`, and has no fee input. QA will add it to their allowlist.
