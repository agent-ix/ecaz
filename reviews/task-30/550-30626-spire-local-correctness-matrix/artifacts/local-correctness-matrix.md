# SPIRE Local Correctness Matrix

Head SHA: `a1665209`
Packet/topic: `30626-spire-local-correctness-matrix`

This matrix records the local PG18 lanes that should stay narrow and focused
while Phase 8 work continues. It is intentionally a selection map rather than a
full-test mandate; run the smallest lane that covers the changed behavior.

| Lane | Focus | Representative PG18 filters |
| --- | --- | --- |
| Empty/populated build and scan | AM bootstrapping, scan no-row behavior, populated round trips | `test_ec_spire_empty_build_scan_no_rows`, `test_ec_spire_empty_pq_fastscan_build_scan_no_rows`, `test_ec_spire_large_routing_object_builds_and_scans`, `test_ec_spire_tqvector_populated_build_scans_with_heap_rerank` |
| Recursive/top-graph routing | Recursive fanout contracts, top graph metadata, recursive-vs-flat result parity | `test_ec_spire_recursive_fanout_build_hierarchy`, `test_ec_spire_top_graph_snapshot_sql`, `test_ec_spire_flat_recursive_same_candidate` |
| Insert/update epochs | Delta epoch publication, dimension/null rejection, empty-index bootstrap | `test_ec_spire_insert_after_build_delta_epoch`, `test_ec_spire_insert_after_build_multi_row_epoch_progression`, `test_ec_spire_insert_after_build_rejects_dimension_mismatch`, `test_ec_spire_insert_after_build_rejects_null_value`, `test_ec_spire_insert_bootstraps_empty_index_epoch` |
| VACUUM and delete visibility | Delete-delta suppression, compaction no-op, insert/mixed delta compaction | `test_ec_spire_vacuum_delete_delta_suppresses_visible_row`, `test_ec_spire_vacuum_cleanup_no_delta_is_noop`, `test_ec_spire_vacuum_cleanup_compacts_insert_delta`, `test_ec_spire_vacuum_cleanup_compacts_mixed_delta_on_leaf` |
| Maintenance publish safety | Empty/no-candidate behavior, lock-time run plan, split/merge publication, recursive rejection | `test_ec_spire_maintenance_run_empty_sql`, `test_ec_spire_locked_maintenance_run_plan_no_write_sql`, `test_ec_spire_maintenance_run_no_candidate_sql`, `test_ec_spire_maintenance_run_merge_publish_sql`, `test_ec_spire_maintenance_run_split_publish_sql`, `test_ec_spire_recursive_maintenance_run_rejected` |
| Storage and old-epoch debt | Relation object tuple accounting, epoch retention state, cleanup summary | `test_ec_spire_relation_storage_snapshot_sql`, `test_ec_spire_epoch_snapshot_sql` |
| Placement/replica diagnostics | Scan placement state, boundary replica build/dedupe, scan sanity | `test_ec_spire_scan_placement_snapshot_sql`, `test_ec_spire_boundary_replica_build_writes_and_dedupes_scan`, `test_ec_spire_recursive_boundary_replica_build_dedupes`, `test_ec_spire_scan_sanity_snapshot_sql` |
| Remote local-contract drift guards | Local storage endpoint, coordinator-local fanout, degraded/strict mismatch and placement-state errors | `test_ec_spire_remote_search_sql_scores_selected_leaf_pids`, `test_ec_spire_remote_search_coord_local_matches_storage`, `test_ec_spire_remote_search_mode_mismatch`, `test_ec_spire_remote_search_strict_unavailable_leaf`, `test_ec_spire_remote_search_degraded_stale_leaf` |
| Remote executor contract drift guards | Request/readiness/libpq/receive/heap/result contract rows and status precedence | `test_ec_spire_remote_search_request_plan_contract`, `test_ec_spire_remote_search_receive_contract`, `test_ec_spire_remote_search_final_contract`, `test_ec_spire_remote_search_local_heap_resolution_plan`, `test_ec_spire_remote_search_coordinator_gate_summary`, `test_ec_spire_remote_phase7_policy_contracts` |
| Distributed manifest contracts | Publish planning, stale descriptor handling, persistence ready/blocked, catalog summary | `test_ec_spire_remote_epoch_publish_plan_missing`, `test_ec_spire_remote_epoch_publish_manifest_stale_descriptor`, `test_ec_spire_remote_epoch_manifest_persist_ready`, `test_ec_spire_remote_epoch_manifest_persist_blocked`, `test_ec_spire_remote_epoch_manifest_catalog_summary_missing` |
| Planner and operator surfaces | Cost snapshot, planner callback, suite explain rows, allocator and options diagnostics | `cargo test ec_spire::cost`, `test_ec_spire_allocator_snapshot_sql`, `test_ec_spire_recursive_fanout_build_hierarchy` |

## Coverage Rule

- PostgreSQL callback or SQL-visible behavior: use a focused `cargo pgrx test pg18 <filter>`.
- Pure Rust planner/cost/parser behavior: use the focused `cargo test <filter>`.
- Cross-surface changes touching publish, VACUUM, or scan visibility: run the relevant lane plus `git diff --check`.
- PG17 remains optional compatibility coverage unless the change is PG17-facing.
