---
id: TM-002
title: "DistANN Requirements Test Matrix"
type: TestMatrix
name: distann
status: PARTIAL
---
# DistANN Test Matrix (FR-075..FR-090, NFR-017..NFR-022)

Scope: every acceptance criterion (AC) and constraint (CON) in
`spec/functional/distann/` (FR-075..FR-090 across build/read/storage/lifecycle)
and `spec/non-functional/NFR-017..NFR-022`. This matrix supplements the
repo-wide `spec/tests.md` (TM-001) with verified criterion-level coverage:
every cited test was located by grep (file plus test/drill function name)
before being written into a row. Rows follow the `/spec-matrix` skill shape.

Test-case group ids (TC-037..TC-044, TC-050, TC-051) are the planning
groups defined in `spec/tests.md`; this matrix cites the concrete tests that
exist today underneath them.

## Test Matrix Rules

1. Coverage: every AC/CON has exactly one row; no row cites a test that was not
   verified to exist.
2. Option permutation: codec, control-mode, roster, and replay permutations are
   tracked in `spec/tests.md` §Option Permutation Matrix and are not repeated
   here.
3. Constraint boundary: every FR CON has its own row.
4. Error path: EC_* error categories are enumerated in `spec/tests.md`
   §Error Path Tests; rows below cite the concrete guard tests.
5. State transition: FR-082 lifecycle transitions are covered by the
   `src/tests/ec_distann_physical_lifecycle.rs` suite and the multinode drills.
6. Edge case: EC-019..EC-037 in `spec/tests.md` remain the edge-case register.

## Status Legend

- **✅ Covered** — a verified test (or, for Inspection-verified ACs, the
  completed Task 214 inspection) exists for the criterion.
- **✅ Bench** — verification is Analysis (bench) and the owning review packet
  exists with the cited measurement.
- **⚠️ Partial** — a test covers part of the criterion, or evidence exists but
  a spec-flagged gap or unverified clause remains.
- **❌ Planned** — no test exists yet; the owner column names the owning task
  (211/212/213) or `code-fix backlog` for spec-flagged implementation gaps and
  missing-test debt.

## Evidence Index (verified files)

| Shorthand | Path |
| --- | --- |
| basic | `src/tests/ec_distann_basic.rs` |
| phys | `src/tests/ec_distann_physical_lifecycle.rs` |
| scanreg-pg | `src/tests/ec_distann_scan_registry.rs` |
| regsec | `src/tests/ec_distann_registry_security.rs` |
| regconc | `src/tests/ec_distann_registry_concurrency.rs` (TC-040 tag) |
| d8 | `src/tests/ec_distann_d8.rs` |
| mc | `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs` (multinode fixture drills: TC-042 fault matrix, FR-082 lifecycle drills, `run_task199_replica_lifecycle_drills`, `run_materialization_correctness`, `run_coverage_memory_regression`, `run_physical_benchmarks`) |
| fixtures | `tests/on_disk_fixtures.rs` (TC-050 golden/endian/independent-decode fixtures) |
| sizes | `tests/size_of_assertions.rs` (TC-050 static layout assertions) |
| unit:`<mod>` | `src/am/ec_distann/<mod>.rs` in-module unit tests |

## Functional Requirement Coverage

### FR-075 — Access-Method Surface (`spec/functional/distann/FR-075-ec-distann-access-method-surface.md`)

| AC | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-075-AC-1 | Test | basic: `test_ec_distann_access_method_is_registered`, `test_ec_distann_build_persists_graph_structures` | ✅ Covered |
| FR-075-AC-2 | Test | basic: `test_ec_distann_rejects_out_of_range_graph_degree` (tagged FR-075-AC-2) | ✅ Covered |
| FR-075-AC-3 | Test | basic: `test_ec_distann_ordered_scan_scores_are_monotone` (tagged FR-075-AC-3) | ✅ Covered |
| FR-075-AC-4 | Test (bench A/B) | `reviews/task-162/002-m0-bench-cells/`, `reviews/task-162/004-parity-remeasure/` | ✅ Bench |
| FR-075-AC-5 | Test (TC-040, TC-042) | basic: `test_distann_control_metadata_and_fail_closed`, `test_distann_control_rejects_temporary_relation`, `test_distann_control_requires_permanent_wal_logged_relation`; phys: `test_distann_multi_epoch_publish` | ✅ Covered |
| FR-075-AC-6 | Test (TC-040, TC-042) | phys: `test_distann_generation_drop_and_reindex_clean_dependencies`; basic: `test_distann_control_mode_change_reindex` | ✅ Covered |
| FR-075-AC-7 | Test | basic: `test_ec_distann_guc_defaults` asserts only `beam_width`/`hop_rounds`/`top_k`; unit:options `distann_default_options_match_spec_defaults`. The other ten listed GUC defaults are unasserted | ⚠️ Partial |
| FR-075-AC-8 | Test | basic: `test_ec_distann_rejects_invalid_neighbor_code_format`; unit:options `distann_source_identity_parses_include_only` | ✅ Covered |
| FR-075-AC-9 | Test | none found (`ec_distann.roster`/`.epoch` appear in tests only as legacy-lane inputs, never as a physical-lane inertness assertion) | ❌ Planned (code-fix backlog) |

### FR-076 — Graph Node / Record Format (`spec/functional/distann/storage/FR-076-distann-graph-node-record-format.md`)

| AC / CON | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-076-AC-1 | Test | unit:tuple `distann_node_round_trip_is_byte_exact` (tagged FR-076-AC-1) | ✅ Covered |
| FR-076-AC-2 | Test | basic: `test_ec_distann_rebuild_assigns_identical_vec_ids`; unit:identity `distann_vec_id_is_deterministic_for_identical_identity` | ✅ Covered |
| FR-076-AC-3 | Test | basic: `test_ec_distann_search_codes_match_direct_codec_encoding` (tagged FR-076-AC-3) | ✅ Covered |
| FR-076-AC-4 | Test | basic: `test_ec_distann_tombstone_excludes_and_preserves_live_vectors`; unit:scan `distann_orchestration_excludes_tombstones_but_traverses_their_edges` | ✅ Covered |
| FR-076-AC-5 | Test | unit:tuple lean-record structural assertions (tagged FR-076-AC-5) | ✅ Covered |
| FR-076-AC-6 | Test | unit:tuple `distann_node_encoded_len_is_dimension_independent` (tagged FR-076-AC-6) | ✅ Covered |
| FR-076-AC-7 | Test (TC-040) | unit:handoff_wire `handoff_entry_round_trip_preserves_graph_and_row_bytes`; fixtures: `distann_handoff_entry_and_batch_v1_fixtures_decode_independently` | ✅ Covered |
| FR-076-AC-8 | Test (TC-040) | unit:handoff_wire `handoff_preflight_rejects_shape_nulls_order_and_oversize`; a dedicated forbidden-field structural inspection of entry contents was not found | ⚠️ Partial |
| FR-076-AC-9 | Test (TC-040) | unit:row_schema `row_schema_rejects_version_order_and_unsupported_binary_identity`; phys: `test_distann_stage_seal_zero_mutation_matrix` | ✅ Covered |
| FR-076-AC-10 | Test (TC-040) | unit:handoff_wire `handoff_batch_round_trip_verifies_digest_and_order`, `owner_stream_digest_is_stable_and_order_sensitive` | ✅ Covered |
| FR-076-AC-11 | Test | unit:identity `distann_vec_id_local_mode_is_deterministic_and_tid_sensitive`, `distann_vec_id_domains_do_not_alias`, `distann_vec_id_values_are_pinned_against_accidental_hash_changes` | ✅ Covered |
| FR-076-AC-12 | Test | basic: `test_distann_control_requires_include_identity` | ✅ Covered |
| FR-076-AC-13 | Test (TC-040) | basic: `test_ec_distann_include_mode_rejects_short_bytea_identity` | ✅ Covered |
| FR-076-AC-14 | Test | unit:tuple `distann_physical_node_v1_rejects_legacy_and_unknown_versions`; fixtures: `distann_metadata_v4_and_control_v5_fixtures_decode_independently`, `distann_physical_graph_record_v1_fixture_decodes_and_rejects_swap` | ✅ Covered |
| FR-076-CON-1 | Benchmark storage step | Storage ratio rows are emitted and mechanically paired (`assert_distann_storage_ratio_rows`), but the 4.0× budget itself is manually reviewed (NFR-018 audit 2026-08-01) | ⚠️ Partial |
| FR-076-CON-2 | Unit test + build assertion | unit:tuple `distann_node_rejects_neighbor_count_above_graph_degree` (tagged FR-076-CON-2) | ✅ Covered |

### FR-077 — Sharded Build and Stitch (`spec/functional/distann/build/FR-077-distann-sharded-build-and-stitch.md`)

| AC / CON | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-077-AC-1 | Test (bench A/B) | `reviews/task-163/001-m1-stitch-ab/` | ✅ Bench |
| FR-077-AC-2 | Test (property) | unit:shard_build `tc038_stitch_idempotence` | ✅ Covered |
| FR-077-AC-3 | Inspection | spec-flagged unsatisfiable: stitch statistics reach the build log, not the epoch manifest (Task 214 audit F2) | ❌ Planned (code-fix backlog) |
| FR-077-AC-4 | Test (proptest) | unit:shard_build `tc038_*` property suite (degree, uniqueness, reachability, determinism, alpha-prune, corrupt-spool rejections) | ✅ Covered |
| FR-077-AC-5 | Test (TC-038, TC-040) | phys: `test_distann_source_capture_spools_complete_frozen_rows`; unit:handoff_wire entry round-trip; unit:shard_build `tc038_uniqueness_and_valid_edges` | ✅ Covered |
| FR-077-AC-6 | Test (property) | unit:shard_build `repair_reachability_bounds_degree_on_disconnected_graph` | ✅ Covered |
| FR-077-CON-1 | Property test | unit:shard_build `tc038_degree_bounded`, `tc038_alpha_prune_invariant` (tagged FR-077-CON-1) | ✅ Covered |
| FR-077-CON-2 | Property test | unit:shard_build `tc038_uniqueness_and_valid_edges` | ✅ Covered |
| FR-077-CON-3 | Property test (BFS) | unit:shard_build `tc038_medoid_reachability` (tagged FR-077-CON-3) | ✅ Covered |
| FR-077-CON-4 | Analysis (manifest peak-memory row) | unit:shard_build `tc038_d8_spill_and_cursor_bound`; d8: `test_ec_distann_d8_multiblock_buffile_spool`; scale evidence `reviews/task-163/005-d8-scale-memory/`; the manifest row itself is spec-flagged unsatisfiable | ⚠️ Partial |

### FR-078 — Hash Placement and Streamed Handoff (`spec/functional/distann/build/FR-078-distann-hash-placement.md`)

| AC / CON | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-078-AC-1 | Test (TC-040) | unit:placement `placement_is_deterministic`, `placement_hash_v1_golden_vectors` (tagged FR-078-AC-1) | ✅ Covered |
| FR-078-AC-2 | Analysis (TC-044) | unit:placement `placement_is_balanced_across_three_nodes` (tagged FR-078-AC-2); the 100k three-owner ±10% analysis row awaits the TC-044 gate packet | ⚠️ Partial |
| FR-078-AC-3 | Test (TC-042) | unit:epoch `fingerprint_is_roster_order_sensitive`; unit:placement `placement_changes_only_with_node_count` (tagged FR-078-AC-3) | ✅ Covered |
| FR-078-AC-4 | Test (TC-040) | phys: `test_distann_three_owner_physical_handoff`; mc: `run_materialization_correctness` | ✅ Covered |
| FR-078-AC-5 | Test (TC-040, TC-044) | phys: `test_distann_three_owner_physical_handoff` (disjoint unions); phys: `test_distann_generation_topology_reports_ready_and_building` | ✅ Covered |
| FR-078-AC-6 | Test (TC-040) | phys: `test_distann_stage_batch_atomic_replay_and_directory`, `test_distann_seal_ready_replay_and_receipt` | ✅ Covered |
| FR-078-AC-7 | Test (TC-040) | phys: `test_distann_stage_seal_zero_mutation_matrix` | ✅ Covered |
| FR-078-AC-8 | Test (TC-042) | unit:handoff_router `router_retains_exact_unacknowledged_batch_after_failure` (unit resume); a live participant-PostgreSQL-restart resume drill was not found | ⚠️ Partial |
| FR-078-AC-9 | Test (TC-040) | mc: `run_materialization_correctness`; basic: `test_ec_distann_materialize_row_payloads_ships_binary_columns` | ✅ Covered |
| FR-078-AC-10 | Test (TC-040, TC-044) | phys: `test_distann_generation_topology_reports_ready_and_building`; mc: topology preflight before the fault matrix | ✅ Covered |
| FR-078-AC-11 | Test (TC-040) | mc: `run_coverage_memory_regression`; unit:handoff_router `router_bounds_batches_routes_owners_and_sends_empty_sequence_zero`; phys: `test_distann_physical_seed_detoast_memory_is_bounded` | ✅ Covered |
| FR-078-AC-12 | Test (TC-040) | phys: `test_distann_three_owner_physical_handoff`; an explicit coordinator-outside-roster zero-record assertion was not individually verified | ⚠️ Partial |
| FR-078-AC-13 | Test (TC-040, TC-050) | unit:generation_descriptor `generation_descriptor_round_trip_binds_schema_codec_and_roster`; unit:quantizer `codec_artifact_restores_codes_and_prepared_scores_without_retraining`; fixtures: `distann_generation_descriptor_v2_fixture_decodes_independently_and_rejects_swap` | ✅ Covered |
| FR-078-AC-14 | Test (TC-040) | phys: `test_distann_node_registration_provenance_and_guards`, `test_distann_node_registration_binds_indexed_key_attnum`; regsec: `test_distann_registry_security_definer_path_and_acl` | ✅ Covered |
| FR-078-AC-15 | Test (TC-040, TC-050) | unit:handoff_wire `owner_stream_hash_initial_state_golden_is_frozen`, `owner_stream_hash_resume_matches_one_shot_at_every_byte_boundary`, `owner_stream_hash_restore_rejects_malformed_state_and_digest`, `empty_owner_stream_round_trip_does_not_advance_state`; fixtures: `distann_owner_stream_hash_state_v1_fixture_is_independent_and_fixed` | ✅ Covered |
| FR-078-AC-16 | Test (TC-042, TC-050) | phys: `test_distann_begin_build_lock_lifecycle`, `test_distann_begin_build_competing_backend_busy`, `test_distann_build_lock_recovery_guards`, `test_distann_abort_epoch_build_clears_gate_and_is_idempotent`; fixtures: `distann_build_registration_v1_fixture_decodes_and_digests_independently` | ✅ Covered |
| FR-078-AC-17 | Test (TC-040) | phys: `test_distann_trained_head_build_replay_publish_and_inspection` | ✅ Covered |
| FR-078-CON-1 | Boundary test (TC-040) | unit:handoff_wire `handoff_preflight_rejects_shape_nulls_order_and_oversize`; unit:source_spool `source_spool_rejects_shape_corruption_and_oversize_handoff` | ✅ Covered |
| FR-078-CON-2 | Instrumented integration test (TC-040) | unit:handoff_router `router_enforces_real_multi_owner_capacity_minus_exact_and_plus_one` | ✅ Covered |
| FR-078-CON-3 | Unit and integration test (TC-040) | unit:handoff_wire preflight order checks; unit:shard_build `tc038_corrupt_spool_rejects_unsorted_stream` | ✅ Covered |
| FR-078-CON-4 | Topology audit (TC-040, TC-044) | phys: `test_distann_stage_seal_zero_mutation_matrix` (wrong-owner), `test_distann_generation_topology_reports_ready_and_building` | ✅ Covered |
| FR-078-CON-5 | Peak-memory result row (TC-040) | mc: `run_coverage_memory_regression` | ✅ Covered |

### FR-079 — Remote Expansion Protocol (`spec/functional/distann/read/FR-079-distann-remote-expansion-protocol.md`)

| AC | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-079-AC-1 | Test | unit:remote_transport `reassembles_interleaved_request_across_owners`, `missing_coverage_is_an_error`, `wrong_response_count_is_an_error` (tagged FR-079-AC-1); basic: `test_ec_distann_expand_nodes_single_node_matches_local` | ✅ Covered |
| FR-079-AC-2 | Test | basic: `test_ec_distann_expand_nodes_rejects_epoch_mismatch` (tagged FR-079-AC-2) | ✅ Covered |
| FR-079-AC-3 | Test | basic: `test_ec_distann_expand_nodes_rejects_nonowned_placement` (tagged FR-079-AC-3), `test_ec_distann_tombstone_excludes_and_preserves_live_vectors` | ✅ Covered |
| FR-079-AC-4 | Test | unit:quantizer `simd_diff_distann_codec_batches_match_direct_scalar_scores_across_widths`, `seeded_codec_v1_golden_score_vectors` | ✅ Covered |
| FR-079-AC-5 | Test | unit:insert `simd_diff_exact_distance_is_shared_diskann_inner_product_negation`; mc: `run_materialization_correctness` | ✅ Covered |
| FR-079-AC-6 | Test (TC-040) | basic: `test_ec_distann_materialize_row_payloads_ships_binary_columns`, `test_ec_distann_materialize_rows_ships_heap_identity` | ✅ Covered |
| FR-079-AC-7 | Test (TC-040) | mc: `run_materialization_correctness`; frozen-row qual-outcome equivalence beyond the drill's assertions unverified | ⚠️ Partial |
| FR-079-AC-8 | Test (TC-040, TC-042) | basic: `test_ec_distann_fault_drills_distinct_classes` (TC-042/NFR-020 tagged); phys: `test_distann_stage_seal_zero_mutation_matrix`; unit:expand_error `classes_have_distinct_sqlstates_and_categories` | ✅ Covered |
| FR-079-AC-9 | Test (TC-040) | unit:row_schema `row_schema_rejects_version_order_and_unsupported_binary_identity`; a dedicated structural inspection of the request wire shape was not found | ⚠️ Partial |
| FR-079-AC-10 | Test (TC-042) | phys: `test_distann_multi_epoch_publish` (retained old/new); explicit cross-generation-read-isolation assertion not individually verified | ⚠️ Partial |
| FR-079-AC-11 | Test (TC-040) | basic: `test_distann_remote_endpoint_acl_class` (enumerates protected overloads from `pg_proc`); regsec: `test_distann_registry_security_definer_path_and_acl` | ✅ Covered |
| FR-079-AC-12 | Test (TC-040, TC-041) | unit:custom_scan `ranked_windows_are_deterministic_and_proven_bounded`, `materialized_payload_survives_rank_shift_without_refetch` | ✅ Covered |
| FR-079-AC-13 | Test (TC-042) | mc: TC-042 fault matrix (`hop_round_failure_mid_beam` injection via `ec_distann` fault GUCs); the specific later-window owner-failure case was not individually verified | ⚠️ Partial |
| FR-079-AC-14 | Test | unit:generation_read `physical_query_cache_requires_matching_digest_and_reuses_arc` | ✅ Covered |
| FR-079-AC-15 | Test | unit:gateway_copy `split_request_separates_locally_answerable_ids_in_request_order`, `gateway_fill_and_rebatch_matches_the_owner_only_batch_semantics` | ✅ Covered |
| FR-079-AC-16 | Test + inspection | basic: `test_distann_remote_endpoint_acl_class` covers the privilege class; the off-loopback non-invocation half is inspection-only, not mechanized | ⚠️ Partial |

### FR-080 — Coordinator Head Index (`spec/functional/distann/read/FR-080-distann-coordinator-head-index.md`)

| AC / CON | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-080-AC-1 | Test + storage audit | `reviews/task-210/006-zero-byte-head/` (zero coordinator derived bytes); unit:head_sample `merged_head_seeds_are_bounded_deduplicated_and_deterministic` | ✅ Bench |
| FR-080-AC-2 | Test | unit:head_sample `owner_built_shards_match_coordinator_side_partitioning`, `head_shard_server_spreads_across_replicas_and_stays_deterministic`; basic: `test_ec_distann_head_sample_is_deterministic_across_reindex` | ✅ Covered |
| FR-080-AC-3 | Test (property/BFS) | none found — no connected-component-representation test exists in `head_sample.rs` or `shard_build.rs` | ❌ Planned (code-fix backlog) |
| FR-080-AC-4 | Analysis (bench) | `reviews/task-179/038-head-cap-sensitivity/` (cited by the spec's §Measured head-cap outcome) | ✅ Bench |
| FR-080-AC-5 | Test + benchmark | unit:generation_read `physical_epoch_cache_is_bounded_and_lru`; basic: `test_ec_distann_head_cache_invalidates_across_reindex` | ✅ Covered |
| FR-080-AC-6 | Test | unit:head_sample `training_query_set_digest_and_exact_policy_are_deterministic`; phys: `test_distann_trained_head_build_replay_publish_and_inspection` | ✅ Covered |
| FR-080-AC-7 | Test | unit:head_sample `head_shard_server_spreads_across_replicas_and_stays_deterministic`; the attestation-incomplete clamp path was not individually verified | ⚠️ Partial |
| FR-080-AC-8 | Test | membership blob decode partially exercised by head persistence tests; the CHECK itself is a planned FR-087-AC-3 test and the coordinator-local-path exclusion was not individually verified | ⚠️ Partial |
| FR-080-CON-1 | Analysis + storage audit | `reviews/task-210/006-zero-byte-head/`; unit:head_sample seed-bound tests | ✅ Bench |
| FR-080-CON-2 | Analysis + unit test | unit:head_sample `persisted_head_graph_is_deterministic_and_loadable`, `head_shards_partition_every_landmark_exactly_once`; FR-088 law resolution is Task 211 scope | ⚠️ Partial |

### FR-081 — Query Orchestration (`spec/functional/distann/read/FR-081-distann-query-orchestration.md`)

| AC | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-081-AC-1 | Test | mc: multi-node distinct-recall gates; basic: `test_ec_distann_expand_nodes_single_node_matches_local`; an exact 2-node-vs-single-node top-k identity assertion was not individually verified | ⚠️ Partial |
| FR-081-AC-2 | Test (counter assertion) | unit:scan `distann_orchestration_hop_rounds_cap_bounds_expansions` (tagged FR-081-AC-2); the per-benchmarked-cell counter assertion is unmechanized (NFR-019 audit: release builds compile the cap check out) | ⚠️ Partial |
| FR-081-AC-3 | Test | unit:scan `distann_orchestration_expands_no_vec_id_twice_and_respects_cap` | ✅ Covered |
| FR-081-AC-4 | Test (A/B on fixed corpus) | unit:scan `distann_orchestration_early_exits_when_beam_cannot_improve_kth`, `distann_orchestration_does_not_early_exit_before_equal_score_tie_break` | ✅ Covered |
| FR-081-AC-5 | Inspection | spec-flagged gap (Task 214 audit F8): `ExplainCustomScan` is unimplemented; counters exist only behind the `scan_profile_notice` debug GUC | ❌ Planned (code-fix backlog) |
| FR-081-AC-6 | Test | basic: `test_ec_distann_remote_transport_statement_timeout`, `test_ec_distann_remote_transport_cancel_then_reuse`, `test_ec_distann_remote_transport_cancel_mid_connect_then_reuse`; unit:remote_transport `remote_await_enforces_client_deadline`, `interrupt_poll_accepts_cancel_and_backend_termination`. Spec-flagged gap (F9): four head-path RPCs bypass the deadline/interrupt wrapper | ⚠️ Partial |
| FR-081-AC-7 | Inspection + Test (TC-040, TC-049) | unit:custom_scan `ranked_windows_are_deterministic_and_proven_bounded`; the production-build no-GUC inspection is not mechanized | ⚠️ Partial |
| FR-081-AC-8 | Test (counter assertion) | unit:custom_scan window-bound tests; NFR-019 counter machinery does not exist yet | ⚠️ Partial |

### FR-082 — Epoch Lifecycle (`spec/functional/distann/lifecycle/FR-082-distann-epoch-lifecycle.md`)

| AC / CON | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-082-AC-1 | Test (TC-042) | mc: epoch-swap-under-load drill (tagged FR-082-AC-1); unit:scan restart suite | ✅ Covered |
| FR-082-AC-2 | Test (TC-042) | unit:scan `restart_once_after_epoch_mismatch_then_succeeds`, `restart_second_epoch_mismatch_errors` (tagged FR-082-AC-2); mc swap-race drill | ✅ Covered |
| FR-082-AC-3 | Test (TC-042) | mc: live retention gate drill (tagged FR-082-AC-3); phys: `test_distann_participant_retire_reclaim_and_rollback`; unit:scan_registry fence suite | ✅ Covered |
| FR-082-AC-4 | Test (TC-042, TC-043) | mc: concurrency drill (tagged FR-082-AC-4); basic: `test_ec_distann_fold_multi_row_clustered_delta`; complete per-record-state exposure under Tier-2 mutation remains future FR-083 scope | ⚠️ Partial |
| FR-082-AC-5 | Test (TC-042) | mc: frozen-vector drift drill (tagged FR-082-AC-5); packet `reviews/task-165/018-ac5-frozen-vector/` | ✅ Covered |
| FR-082-AC-6 | Test (TC-042) | basic: `test_ec_distann_epoch_lifecycle_publish_retire_override`; phys: `test_distann_participant_retire_reclaim_and_rollback` | ✅ Covered |
| FR-082-AC-7 | Test (TC-042) | phys: `test_distann_generation_topology_reports_ready_and_building`, `test_distann_multi_epoch_publish` | ✅ Covered |
| FR-082-AC-8 | Test (TC-042) | phys: `test_distann_seal_ready_replay_and_receipt`, `test_distann_participant_publish_negative_guards` | ✅ Covered |
| FR-082-AC-9 | Test (TC-042) | phys recovery tests + `reviews/task-179/018-publish-crash-window/`; exhaustive every-boundary crash drill not verified per boundary | ⚠️ Partial |
| FR-082-AC-10 | Test (TC-042) | phys: `test_distann_participant_publish_status_replay_and_conflict`, `test_distann_multi_epoch_publish` | ✅ Covered |
| FR-082-AC-11 | Test (TC-042) | phys: `test_distann_begin_build_lock_lifecycle`, `test_distann_begin_build_rejects_inherited_source_topology`; breadth of the source DML/schema-change block matrix not individually verified | ⚠️ Partial |
| FR-082-AC-12 | Test (TC-042) | unit:epoch `fingerprint_is_roster_order_sensitive`; unit:placement `placement_changes_only_with_node_count` | ✅ Covered |
| FR-082-AC-13 | Test (TC-042) | unit:scan_registry 24-test suite (capacity, reaping, namespaces, fence recycling); scanreg-pg: `test_ec_distann_scan_registry_contract_and_gucs`, `test_ec_distann_scan_registry_two_backend_retirement_contention`; regconc | ✅ Covered |
| FR-082-AC-14 | Test (TC-042, TC-050) | phys: `test_distann_seal_ready_replay_and_receipt`; unit:manifest_v2 `ready_receipt_round_trip_verifies_digest_state_and_counts`; fixtures: `distann_build_candidate_v1_fixture_decodes_independently_and_rejects_version_swap` | ✅ Covered |
| FR-082-AC-15 | Test (TC-042, TC-050) | unit:lifecycle_wire abandon/audit suite; fixtures: `distann_successor_activation_v1_...`, `distann_abandon_binding_audit_v1_...`, `distann_abandoned_binding_set_v1_...`; packet `reviews/task-179/024-audited-predecessor-abandonment/`; a live end-to-end abandonment drill was not verified | ⚠️ Partial |
| FR-082-AC-16 | Test (TC-042, TC-050) | phys: `test_distann_participant_retire_reclaim_and_rollback`; fixtures: `distann_retire_decision_v1_fixture_decodes_independently_and_rejects_version_swap` | ✅ Covered |
| FR-082-AC-17 | Test (TC-042) | phys: `test_distann_decide_abort_guards`; fixtures: `distann_cancel_publish_audit_v1_...`; packet `reviews/task-179/034-cancelled-generation-recovery/`; cancellation-path breadth not individually verified | ⚠️ Partial |
| FR-082-AC-18 | Test (TC-042, TC-050) | packets `reviews/task-179/034-cancelled-generation-recovery/`, `reviews/task-179/042-cancelled-recovery-xmin/`; fixtures cancel audit; per-clause replay drill not individually verified | ⚠️ Partial |
| FR-082-CON-1 | State-machine and two-coordinator rejection test (TC-042) | phys: `test_distann_begin_build_competing_backend_busy`; unit:scan_registry `database_and_logical_uuid_namespace_tokens` | ✅ Covered |
| FR-082-CON-2 | Fault drill (TC-042) | unit:scan `restart_second_epoch_mismatch_errors` | ✅ Covered |
| FR-082-CON-3 | Endpoint integration test (TC-040, TC-042) | basic: `test_ec_distann_expand_nodes_rejects_epoch_mismatch`; phys: `test_distann_generation_topology_reports_ready_and_building` | ✅ Covered |
| FR-082-CON-4 | Crash-boundary drill (TC-042) | `reviews/task-179/018-publish-crash-window/` + phys recovery guards; commit-only property not drilled at every boundary | ⚠️ Partial |
| FR-082-CON-5 | Wire-format test (TC-040) | unit:epoch `fingerprint_bytes_round_trip`, `fingerprint_length_prefix_prevents_aliasing`; fixtures: `distann_epoch_manifest_v2_fixture_decodes_independently_and_rejects_swap` | ✅ Covered |

### FR-083 — DML Path (`spec/functional/distann/lifecycle/FR-083-distann-dml-path.md`)

| AC | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-083-AC-1 | Test | basic: `test_ec_distann_apply_record_writes_tombstones`, `test_ec_distann_tombstone_excludes_and_preserves_live_vectors` (tagged FR-083-AC-1) | ✅ Covered |
| FR-083-AC-2 | Test | basic: `test_ec_distann_reindex_drains_delta_buffer` (tagged FR-083-AC-2) | ✅ Covered |
| FR-083-AC-3 | Test | basic: `test_ec_distann_fold_delta_into_graph` (tagged FR-083-AC-3), `test_distann_control_metadata_and_fail_closed` (v5 fail-closed) | ✅ Covered |
| FR-083-AC-4 | Test (bench A/B) | Tier 2 not implemented | ❌ Planned (code-fix backlog / final milestone) |
| FR-083-AC-5 | Test (TC-040) | basic: `test_ec_distann_fold_delta_requires_read_committed`, `test_ec_distann_apply_record_writes_requires_read_committed`; the hardened-class (SECURITY DEFINER / revoke) half is a spec-flagged Task 214 gap | ⚠️ Partial |
| FR-083-AC-6 | Test (fault drill) | mc: mid-insert failure drill (TC-043, isolated table); covers the legacy fold path only — Tier-2 routed insert unimplemented | ⚠️ Partial |
| FR-083-AC-7 | Test (concurrency drill) | Tier 2 not implemented | ❌ Planned (code-fix backlog / final milestone) |
| FR-083-AC-8 | Test (TC-043) | Tier 2 not implemented | ❌ Planned (code-fix backlog / final milestone) |
| FR-083-AC-9 | Test (TC-043) | Tier 2 not implemented | ❌ Planned (code-fix backlog / final milestone) |

### FR-084 — Coordinator Traversal Replica (`spec/functional/distann/read/FR-084-distann-coordinator-traversal-replica.md`)

| AC | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-084-AC-1 | Test | unit:traversal_replica `content_digest_is_deterministic_and_identity_bound`, `content_digest_rejects_duplicate_order_shape_and_cardinality`; mc: `run_task199_replica_lifecycle_drills` | ✅ Covered |
| FR-084-AC-2 | Test | mc: `run_task199_replica_lifecycle_drills` (parity per `reviews/task-199/001-normal-selection-and-api/`); identical-counter assertion not individually verified | ⚠️ Partial |
| FR-084-AC-3 | Test | unit:traversal_replica `state_machine_allows_only_contract_transitions`, `suppression_is_scoped_to_index_and_build_identity`, `ready_presence_cache_distinguishes_unknown_and_known_absent` | ✅ Covered |
| FR-084-AC-4 | Test | mc: `run_task199_replica_lifecycle_drills`; specific mid-replica-fault owner-restart case not individually verified | ⚠️ Partial |
| FR-084-AC-5 | Test | mc drills + `reviews/task-199/002-operations-lifecycle-and-isolation/`; exactly-once `EC_REPLICA_INVALIDATED` semantics not individually verified | ⚠️ Partial |
| FR-084-AC-6 | Test | mc drills + unit:traversal_replica `full_relcache_reset_clears_ready_presence_and_suppression`; crash-replay idempotence not individually verified | ⚠️ Partial |
| FR-084-AC-7 | Test + suite audit | suite config-time replica screening + decision-role guard (per NFR-022 §Verification audit of `crates/ecaz-cli/src/commands/bench/suite.rs`); a default-off no-scan-selects-replica test was not individually verified | ⚠️ Partial |

### FR-085 — Domain Model (`spec/functional/distann/FR-085-distann-domain-model.md`)

All eight ACs are Inspection-verified statements about the spec text itself.

| AC | Verification (spec) | Covering evidence | Status |
| --- | --- | --- | --- |
| FR-085-AC-1..AC-8 | Inspection | Satisfied by the spec document as written (Task 214 remediation review); no test artifact applicable | ✅ Covered (8 rows) |

### FR-086 — Gateway Copies (`spec/functional/distann/read/FR-086-distann-gateway-copies.md`)

| AC / CON | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-086-AC-1 | Test | unit:gateway_copy `gateway_fill_and_rebatch_matches_the_owner_only_batch_semantics`, `split_request_separates_locally_answerable_ids_in_request_order` | ✅ Covered |
| FR-086-AC-2 | Test | unit:gateway_copy `gateway_copies_never_exceed_their_stated_capacity`, `reinserting_a_gateway_replaces_rather_than_growing` | ✅ Covered |
| FR-086-AC-3 | Test | none found — capacity-change discard is documented in a `gateway_copy.rs` code comment but no epoch-flip/capacity-discard test exists | ❌ Planned (code-fix backlog) |
| FR-086-AC-4 | Inspection + test | unit:gateway_copy `resident_bytes_is_bounded_by_capacity_not_by_corpus` (code-only entries); structural no-payload inspection not mechanized | ⚠️ Partial |
| FR-086-AC-5 | Analysis (bench) | `reviews/task-210/004-gateway-copies/` (−36%/−9%/−7% response bytes @10k/50k/100k, identical recall; cited by spec §Measured outcome) | ✅ Bench |
| FR-086-CON-1 | Analysis + unit test | unit:gateway_copy `resident_bytes_is_bounded_by_capacity_not_by_corpus` | ✅ Covered |

### FR-087 — Catalog Relations (`spec/functional/distann/storage/FR-087-distann-catalog-relations.md`)

| AC / CON | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| FR-087-AC-1 | Inspection | Spec Schema section transcribed from `sql/bootstrap.sql` (Task 214 audit) | ✅ Covered |
| FR-087-AC-2 | Test | no constraint-violation insert tests found for the state-machine CHECKs | ❌ Planned (code-fix backlog) |
| FR-087-AC-3 | Test | no membership-blob CHECK test found | ❌ Planned (code-fix backlog) |
| FR-087-AC-4 | Test | phys: `test_distann_multi_epoch_publish`, `test_distann_decide_abort_guards` exercise the T4a CAS; direct second-row / dangling-FK insert tests not found | ⚠️ Partial |
| FR-087-AC-5 | Test | spec-flagged known gap: four head relations missing from the REVOKE block | ❌ Planned (code-fix backlog) |
| FR-087-AC-6 | Test | spec-flagged known gap: no reclaim path exists for `ec_distann_head_shard_replica` / `ec_distann_head_replica_state` | ❌ Planned (code-fix backlog) |
| FR-087-AC-7 | Audit query (NFR-021) | full twenty-relation classification audit not yet run | ❌ Planned (code-fix backlog) |
| FR-087-AC-8 | Inspection + call-path test | phys tests call the physical endpoints by full signature; explicit overload-resolution test absent | ⚠️ Partial |
| FR-087-CON-1 | NFR-021 storage audit | not yet run over all twenty relations | ❌ Planned (code-fix backlog) |
| FR-087-CON-2 | NFR-021 clause 3 audit query | not yet run | ❌ Planned (code-fix backlog) |
| FR-087-CON-3 | Schema CHECK | no mis-sized-blob rejection test found | ❌ Planned (code-fix backlog) |
| FR-087-CON-4 | Schema (PK + FK) | phys: T4a tests exercise the pointer path; direct schema-rejection tests not found | ⚠️ Partial |
| FR-087-CON-5 | Schema (partial unique) | phys: `test_distann_begin_build_competing_backend_busy` exercises the single-gate property | ⚠️ Partial |
| FR-087-CON-6 | Security | spec-flagged: currently fails for the four head relations | ❌ Planned (code-fix backlog) |
| FR-087-CON-7 | Lifecycle | spec-flagged: currently fails for the head-replica pair | ❌ Planned (code-fix backlog) |

### FR-088 — Head Scaling Law (`spec/functional/distann/read/FR-088-distann-head-scaling-law.md`)

Unimplemented feature; all rows owned by Task 211
(`plan/tasks/211-ec-distann-head-scaling-law.md`).

| AC / CON | Verification (spec) | Status |
| --- | --- | --- |
| FR-088-AC-1 | Test | ✅ `head_scaling_attestation_is_deterministic_and_digest_bound`; `head_sizing_resolution_covers_untrained_and_trained_policies` |
| FR-088-AC-2 | Test | ✅ v3 attestation encode/decode and tamper refusal in `head_scaling_attestation_is_deterministic_and_digest_bound` |
| FR-088-AC-3 | Test | ✅ `head_sizing_reloption_validation_rejects_invalid_bounds`; resolver rejects invalid law bounds |
| FR-088-AC-4 | Test | ✅ `head_sizing_resolution_covers_untrained_and_trained_policies` exercises both policy branches |
| FR-088-AC-5 | Analysis (bench) | ✅ `reviews/task-211/002-head-scaling-law-implementation/artifacts/bench-run-law-fixed/`; selected rate 0.02 evidence at 10k/50k/100k; broader rate selection explicitly deferred to the 1M+ gate |
| FR-088-AC-6 | Test + storage audit | ⚠️ Partial: zero-byte head audit in `reviews/task-211/002-head-scaling-law-implementation/artifacts/bench-run-law-fixed/`; dedicated test half remains |
| FR-088-CON-1 | Unit test | ✅ resolver and build-option encode cross-checks in Task 211 validation |
| FR-088-CON-2 | Test | ✅ deterministic attestation/build-option digest assertion |

### FR-089 — Crown Cache (`spec/functional/distann/read/FR-089-distann-crown-cache.md`)

Unimplemented feature; all rows owned by Task 212
(`plan/tasks/212-ec-distann-crown-cache.md`).

| AC / CON | Verification (spec) | Status |
| --- | --- | --- |
| FR-089-AC-1 | Test | ✅ `test_distann_three_owner_physical_handoff` forces crown population failure, verifies fallback counters, and compares fallback results with the cache-off referent |
| FR-089-AC-2 | Test | ✅ `CrownCache::from_entries` validation and bounded serialized bytes |
| FR-089-AC-3 | Test | ✅ `test_distann_three_owner_physical_handoff` verifies capacity replacement and a changed epoch fingerprint after successor publication |
| FR-089-AC-4 | Inspection + test | ✅ codes-only storage and resident-byte bound in Task 212 storage rows |
| FR-089-AC-5 | Analysis (bench) | ✅ plain crown identity A/B at 10k/50k/100k; single-variable width-pruning A/B at 10k/50k/100k activates the path but prunes zero shards and shows no latency benefit; both outcomes are separately labeled |
| FR-089-CON-1 | Analysis + storage audit | ✅ bounded crown storage rows in Task 212 packet |
| FR-089-CON-2 | Test | ⚠️ Partial: identical cache entries produce identical serialized bytes/digest; cross-backend equivalence remains an unenforced operational invariant |

### FR-090 — Fused Head Hop (`spec/functional/distann/read/FR-090-distann-fused-head-hop.md`)

Unimplemented feature; all rows owned by Task 213
(`plan/tasks/213-ec-distann-fused-head-hop.md`).

| AC / CON | Verification (spec) | Status |
| --- | --- | --- |
| FR-090-AC-1 | Test | ⚠️ Partial: inherited FR-079 positional-contract coverage; dedicated forced fused-request fixture remains |
| FR-090-AC-2 | Test | ⚠️ Partial: shared expansion implementation preserves threshold path; dedicated fused/unfused semantic test remains |
| FR-090-AC-3 | Test | ✅ shared `test_distann_three_owner_physical_handoff` forces population failure and verifies identical fallback results; fused benchmark arms report zero fallbacks |
| FR-090-AC-4 | Test + fixture | ✅ Capacity 512/2048/4096 arms are explicitly `seed_set_change=true`; exact claims are restricted to the amended shared code-scored/full-membership condition |
| FR-090-AC-5 | Analysis (bench) | ✅ Post-fix Task 213 packet: fused 33.40/41.30/38.90 ms vs unfused 39.80/50.30/51.60 ms at 10k/50k/100k; fused requested-id accounting is 1600 latency / 6400 recall per arm |
| FR-090-CON-1 | Test + bench | ✅ Recall movement is labeled and measured; no exact-policy claim is made for the approximate 2048-capacity arms |

## Non-Functional Requirement Coverage

| Requirement | Verification (spec) | Covering test / evidence | Status |
| --- | --- | --- | --- |
| NFR-017 (latency/recall gate) | Pre-registered `ecaz bench suite` four-way comparison on the Task 146 protocol | Not executable yet: prerequisite branch merges (`task-138` distinct_recall emitter, `task-146` anchors `reviews/task-146/006-anchor-results/`) must land on the measuring branch; no gate packet exists | ❌ Planned (gate packet; blocked on task-138/task-146 merges) |
| NFR-018 (space amplification ≤4×) | Per-arm storage step + summed ratio row | `assert_distann_storage_ratio_rows` mechanically pairs storage rows with ratio rows (suite); the 4.0× budget comparison itself is manual review per the spec's 2026-08-01 enforcement audit | ⚠️ Partial (mechanized budget gate: code-fix backlog) |
| NFR-019 (per-query touch bound) | Counter emission + per-cell cap assertions | unit:scan `distann_orchestration_hop_rounds_cap_bounds_expansions`; unit:custom_scan `ranked_windows_are_deterministic_and_proven_bounded`; FR-090 fused counters now emit `fused_first_round_requested_ids` and the 10k/50k/100k packet reports it alongside `fused_head_hops` | ⚠️ Partial (first-round accounting is covered; full mechanical EXPLAIN/cap assertion regime remains) |
| NFR-020-AC-1 | Fault drill | mc: TC-042 fault matrix (fail-closed per class); basic: `test_ec_distann_fault_drills_distinct_classes` | ✅ Covered |
| NFR-020-AC-2 | Crash drill | phys abort/recovery guards + `reviews/task-179/018-publish-crash-window/`; not drilled at every pre-decision boundary | ⚠️ Partial |
| NFR-020-AC-3 | Crash drill | phys: `test_distann_multi_epoch_publish` recovery + crash-window packet; not drilled at every post-decision boundary | ⚠️ Partial |
| NFR-020-AC-4 | Replay test | phys: `test_distann_stage_batch_atomic_replay_and_directory`, `test_distann_participant_publish_status_replay_and_conflict` | ✅ Covered |
| NFR-020-AC-5 | Zero-mutation test | phys: `test_distann_stage_seal_zero_mutation_matrix`, `test_distann_participant_publish_negative_guards`, `test_distann_source_capture_mismatch_faults` | ✅ Covered |
| NFR-020-AC-6 | Inspection | No degraded-completion path exists in the implementation; requirement is a guard on future work | ✅ Covered |
| NFR-020-AC-7 | Test | unit:scan_registry `normal_owner_exit_releases_only_exact_incarnation`, `dead_backend_reaping_uses_pid_and_generation`, `abrupt_backend_reaps_operation_reference_before_recycling`; phys: `test_distann_participant_retire_reclaim_and_rollback` | ✅ Covered |
| NFR-021 (distribution invariant) | Suite topology/storage audit, per-node bytes in `results.jsonl` | Conformance emitter and topology audit exist (`reviews/task-210/001-conformance-emitter/`, suite verdict machinery); two spec-flagged audited gaps remain (unclassified-relation verdict shape; head-row/manifest mechanization) | ⚠️ Partial (both gaps: code-fix backlog) |
| NFR-022 (control validity) | Pre-registration screening + arm labeling | Config-time replica screening and the decision-role guard exist in the suite (per spec audit); general config-time screening and 100% `results.jsonl` labeling are spec-flagged gaps | ⚠️ Partial (screening/labeling: code-fix backlog) |

## Coverage Summary

| Status | Rows |
| --- | ---: |
| ✅ Covered | 100 |
| ✅ Bench (packet-backed) | 6 |
| ⚠️ Partial | 45 |
| ❌ Planned | 41 |
| **Total AC/CON rows** | **192** |

Planned-row owners: Task 211 (8), Task 212 (7), Task 213 (6), code-fix
backlog (19: FR-075-AC-9, FR-077-AC-3, FR-080-AC-3, FR-081-AC-5,
FR-083-AC-4/7/8/9, FR-086-AC-3, FR-087-AC-2/3/5/6/7 + CON-1/2/3/6/7,
NFR-017 gate packet is additionally blocked on branch merges).

## Coverage Discrepancies (spec claims vs. found tests)

ACs whose spec Verification column says **Test** today but for which no
existing test was found (coverage gaps that read as covered in the spec):

1. **FR-075-AC-9** — physical-lane inertness of `ec_distann.roster`/`.epoch`
   GUCs: no such assertion exists; the GUCs appear in tests only as
   legacy-lane inputs.
2. **FR-080-AC-3** — "every connected component represented in the head
   sample (property/BFS)": no component-representation test exists in
   `head_sample.rs` or elsewhere.
3. **FR-086-AC-3** — epoch-flip/capacity-change discard of the gateway set:
   asserted only in a code comment, never in a test.
4. **FR-087-AC-2 / AC-3 / CON-3** — catalog CHECK/state-machine violation
   inserts: no tests found (the spec's own gap notes cover AC-5/AC-6/CON-6/
   CON-7 but not these).
5. **FR-083-AC-4..AC-9** — labeled Test but sit under the spec's own
   "Tier 2 — Not Implemented" heading (spec-consistent, listed for
   completeness).

## AC-Id Mapping Notes (reported, not fixed)

1. **FR-083 renumbering** (documented in the spec itself, Task 214): an
   earlier revision carried two `FR-083-AC-5` rows; the second became AC-6 and
   former AC-6/7/8 became AC-7/8/9. Code tags citing the old ids map
   accordingly.
2. **`src/am/ec_distann/epoch_manifest.rs:89`** tags its publish helper
   `FR-082-AC-1` ("publish epoch as active") — this matches the pre-rewrite
   FR-082 numbering, not the current AC-1 ("queries never mix fingerprints").
   The multicluster drill tags (`FR-082-AC-1..AC-5` in
   `distann_multicluster.rs`) do match the current numbering.
3. **`spec/tests.md` (TM-001) is stale relative to the Task 214 spec
   remediation**: its TC-040 row cites `src/tests/ec_distann_remote.rs`, which
   does not exist (the tests live in `ec_distann_basic.rs` and
   `ec_distann_physical_lifecycle.rs`); its per-AC trace rows count
   FR-075-AC-1..6 (now 9), FR-076-AC-1..10 (now 14), FR-078-AC-1..16 (now 17),
   FR-082-AC-1..16 (now 18), FR-083-AC-1..8 (now 9); and FR-084..FR-090 have
   no rows there at all.
