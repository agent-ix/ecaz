# SPIRE Diagnostics

SPIRE diagnostics are read-only SQL functions for inspecting the currently
published index state. They are operator-facing triage surfaces, not recall or
latency measurements.

Start with:

- `ec_spire_index_health_snapshot(index_oid)` for a compact health summary and
  recommended next action.
- `ec_spire_index_active_snapshot_diagnostics(index_oid)` for active epoch,
  placement, object, assignment, and byte-count cardinalities.
- `ec_spire_index_scan_sanity_snapshot(index_oid)` when a query appears slow
  or unexpectedly approximate.
- `ec_spire_index_relation_storage_snapshot(index_oid)` when old epoch or
  relation-object cleanup debt is suspected.
- `ec_spire_index_maintenance_scheduler_plan(index_oid)` before running a
  periodic maintenance job.
- `ec_spire_index_epoch_cleanup_summary(index_oid)` when old-epoch retention
  and physical cleanup status need one operator row.
- `ec_spire_index_epoch_cleanup_run(index_oid)` when the cleanup summary reports
  eligible old-epoch tuple debt.
- `ec_spire_remote_pipeline_steps(...)` when remote search spans multiple
  libpq/manifest/result diagnostic surfaces and you need one step list.

## Function Map

| Function | Audience | Use when |
| --- | --- | --- |
| `ec_spire_index_health_snapshot(index_oid)` | operator | You need the quickest health label and recommendation. |
| `ec_spire_index_active_snapshot_diagnostics(index_oid)` | operator | You need active epoch cardinalities and byte totals. |
| `ec_spire_index_options_snapshot(index_oid)` | operator | You need resolved reloptions, session overrides, and effective scan settings. |
| `ec_spire_index_scan_sanity_snapshot(index_oid)` | operator | You need deterministic scan preconditions such as exact leaf coverage and rerank mode. |
| `ec_spire_index_relation_storage_snapshot(index_oid)` | operator | You need relation object tuple counts, active referenced bytes, and cleanup-candidate debt. |
| `ec_spire_index_epoch_snapshot(index_oid)` | operator | You need active, retired, failed, superseded, and cleanup-eligibility epoch rows. |
| `ec_spire_index_epoch_cleanup_summary(index_oid)` | operator | You need retained-epoch blockers and cleanup-candidate tuple debt in one row. |
| `ec_spire_index_epoch_cleanup_run(index_oid)` | operator | You need to reclaim cleanup-eligible old-epoch object tuples under the SPIRE publish lock. |
| `ec_spire_index_placement_snapshot(index_oid)` | operator | You need per-store placement counts, availability counts, and object bytes by kind. |
| `ec_spire_index_scan_placement_snapshot(index_oid, query)` | operator/debug | You need the stores, leaf PIDs, delta PIDs, and candidate rows touched by one query. |
| `ec_spire_index_root_routing_snapshot(index_oid)` | debug | You need root centroid-to-child routing rows and child placement metadata. |
| `ec_spire_index_hierarchy_snapshot(index_oid)` | operator/debug | You need the current hierarchy shape and single-level foundation capability flags. |
| `ec_spire_index_object_snapshot(index_oid)` | debug | You need one row per active manifest object PID with kind, version, placement, and readability. |
| `ec_spire_index_leaf_snapshot(index_oid)` | operator/debug | You need per-leaf base, delta, effective assignment counts, maintenance labels, and object bytes. |
| `ec_spire_index_delta_snapshot(index_oid)` | debug | You need readable delta object rows with parent leaf, version, and insert/delete counts. |
| `ec_spire_index_insert_debt_snapshot(index_oid)` | operator | You need active delta fanout and whether insert batching is recommended. |
| `ec_spire_index_maintenance_plan_snapshot(index_oid)` | operator/debug | You need an unlocked split/merge maintenance preview. |
| `ec_spire_index_locked_maintenance_run_plan(index_oid)` | operator/debug | You need the publish-lock rechecked split/merge plan without publishing an epoch. |
| `ec_spire_index_maintenance_scheduler_plan(index_oid)` | operator | You need to decide whether an operator-controlled periodic job should call maintenance. |
| `ec_spire_index_maintenance_scheduler_run(index_oid)` | operator | You need a periodic-job entrypoint that reuses the normal maintenance publish path. |
| `ec_spire_index_allocator_snapshot(index_oid, warn_within)` | operator | You need PID and local vec_id cursor distance-to-exhaustion warnings. |
| `ec_spire_remote_pipeline_steps(index_oid, requested_epoch, query, selected_pids, top_k, consistency_mode)` | operator | You need one consolidated remote-search pipeline row per dispatch, connection, candidate, heap, manifest, and result step. |

## Stable Labels

Diagnostic label strings are part of the operator-facing contract. Do not reuse
an existing label for a new meaning; add a new label instead.

`ec_spire_index_options_snapshot(index_oid)` reports assignment payload
scannability with these `assignment_payload_status` values:

| Status | Meaning |
| --- | --- |
| `supported` | The configured assignment payload format can be scored by current SPIRE scans. Today this covers TurboQuant and RaBitQ. |
| `deferred_model_metadata` | The configured format is recognized, but SPIRE does not yet persist the additional grouped-PQ model metadata needed to scan it. Today this covers PQ-FastScan. |

`ec_spire_index_options_snapshot(index_oid)` also reports
`effective_nprobe_per_level` and `nprobe_policy_per_level`. Single-level
indexes report one `single_level` entry. Recursive indexes report one entry per
active routing level, ordered from level 1 upward. Phase 3 recursive routing is
conservative: relation or session `nprobe` applies at level 1, and levels above
1 probe one child until durable per-level nprobe configuration lands.

`ec_spire_index_options_snapshot(index_oid)` reports `local_store_count` and
`local_store_tablespaces` as the requested local placement surface. Repeated
tablespace names are allowed so same-device baseline runs can be configured and
reported honestly. Phase 4 supports auxiliary partition-store relations for
local multi-store indexes, while multi-store REINDEX remains explicitly
rejected until a full auxiliary-store rebuild lifecycle lands.

`ec_spire_index_options_snapshot(index_oid)` reports Phase 5 boundary
replication planning state through `boundary_replica_count`,
`boundary_replication_enabled`, and `scan_dedupe_mode`. The default
`boundary_replica_count = 0` keeps primary-only assignment and reports
`scan_dedupe_mode = none`; replica-capable indexes report `vec_id` so operators
can see when scan plans must deduplicate replicated vector identities.

`ec_spire_remote_pipeline_steps(...)` reports six stable `step_name` values:
`dispatch_plan`, `connection_check`, `candidates`, `heap_candidates`,
`manifest_apply`, and `coordinator_result`. Each row carries the existing
status string for that step plus counts and a next blocker/recommendation, so
operators can start from one ordered pipeline view before opening narrower
remote-search diagnostics.

## Maintenance And Cleanup

SPIRE maintenance uses epoch publication. The operator-controlled periodic-job
path is:

1. Read `ec_spire_index_maintenance_scheduler_plan(index_oid)`.
2. If `scheduler_status = 'due'`, call
   `ec_spire_index_maintenance_scheduler_run(index_oid)`.
3. Inspect `maintenance_status`, `planned_action`, `published`, and
   `maintenance_message`.

The scheduler entrypoint does not implement a separate split/merge algorithm.
It delegates to `ec_spire_index_maintenance_run(index_oid)`, which takes the
SPIRE publish lock, reloads active state, rechecks the selected action, and
publishes through the normal maintenance path.

Use `ec_spire_index_epoch_cleanup_summary(index_oid)` for old-epoch cleanup
triage. `physical_cleanup_status = 'not_required'` means there is no old-epoch
tuple debt to reclaim. `physical_cleanup_status = 'blocked_by_retention'` means
cleanup debt is visible, but no epoch is currently eligible after retention and
active-query checks. `physical_cleanup_status = 'supported'` means
`ec_spire_index_epoch_cleanup_run(index_oid)` can reclaim old object tuples. The
cleanup run removes only unprotected tuples for cleanup-eligible epochs under
the SPIRE publish lock. Schedule cleanup from an operator-controlled job during
an acceptable pause window for publish-path work.

## Reading Notes

- These functions inspect SPIRE partition-object storage, not PostgreSQL
  declarative table partitions.
- Empty indexes commonly return zero rows for row-oriented snapshots; use
  `ec_spire_index_health_snapshot` or
  `ec_spire_index_active_snapshot_diagnostics` for a single-row overview.
- Strict local single-store mode treats stale, unavailable, or skipped
  placements as errors in scan-equivalent diagnostics. Degraded mode is
  configurable future-facing behavior for local multi-store and remote
  deployments.
- Repeated columns such as effective `nprobe`, rerank labels, or epoch values
  are intentional. They make each diagnostic row self-describing when exported
  independently.
