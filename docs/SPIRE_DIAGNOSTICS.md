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

## Function Map

| Function | Audience | Use when |
| --- | --- | --- |
| `ec_spire_index_health_snapshot(index_oid)` | operator | You need the quickest health label and recommendation. |
| `ec_spire_index_active_snapshot_diagnostics(index_oid)` | operator | You need active epoch cardinalities and byte totals. |
| `ec_spire_index_options_snapshot(index_oid)` | operator | You need resolved reloptions, session overrides, and effective scan settings. |
| `ec_spire_index_scan_sanity_snapshot(index_oid)` | operator | You need deterministic scan preconditions such as exact leaf coverage and rerank mode. |
| `ec_spire_index_relation_storage_snapshot(index_oid)` | operator | You need relation object tuple counts, active referenced bytes, and cleanup-candidate debt. |
| `ec_spire_index_epoch_snapshot(index_oid)` | operator | You need active, retired, failed, superseded, and cleanup-eligibility epoch rows. |
| `ec_spire_index_placement_snapshot(index_oid)` | operator | You need per-store placement counts, availability counts, and object bytes by kind. |
| `ec_spire_index_scan_placement_snapshot(index_oid, query)` | operator/debug | You need the stores, leaf PIDs, delta PIDs, and candidate rows touched by one query. |
| `ec_spire_index_root_routing_snapshot(index_oid)` | debug | You need root centroid-to-child routing rows and child placement metadata. |
| `ec_spire_index_hierarchy_snapshot(index_oid)` | operator/debug | You need the current hierarchy shape and single-level foundation capability flags. |
| `ec_spire_index_object_snapshot(index_oid)` | debug | You need one row per active manifest object PID with kind, version, placement, and readability. |
| `ec_spire_index_leaf_snapshot(index_oid)` | operator/debug | You need per-leaf base, delta, effective assignment counts, maintenance labels, and object bytes. |
| `ec_spire_index_delta_snapshot(index_oid)` | debug | You need readable delta object rows with parent leaf, version, and insert/delete counts. |
| `ec_spire_index_insert_debt_snapshot(index_oid)` | operator | You need active delta fanout and whether insert batching is recommended. |
| `ec_spire_index_allocator_snapshot(index_oid, warn_within)` | operator | You need PID and local vec_id cursor distance-to-exhaustion warnings. |

## Stable Labels

Diagnostic label strings are part of the operator-facing contract. Do not reuse
an existing label for a new meaning; add a new label instead.

`ec_spire_index_options_snapshot(index_oid)` reports assignment payload
scannability with these `assignment_payload_status` values:

| Status | Meaning |
| --- | --- |
| `supported` | The configured assignment payload format can be scored by current SPIRE scans. Today this covers TurboQuant and RaBitQ. |
| `deferred_model_metadata` | The configured format is recognized, but SPIRE does not yet persist the additional grouped-PQ model metadata needed to scan it. Today this covers PQ-FastScan. |

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
