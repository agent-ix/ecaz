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
- `ec_spire_remote_search_production_executor_state_summary(...)` when you need
  the dry production fanout state and C0/C1 counters without conninfo secret
  lookup or socket opens.
- `ec_spire_remote_pipeline_steps(...)` when remote search spans multiple
  libpq/manifest/result diagnostic surfaces and you need one cheap step list.

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
| `ec_spire_index_scan_routing_snapshot(index_oid, query)` | operator/debug | You need per-routing-level frontier widths, expansion counts, deduped route counts, and truncation reasons for one query. |
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
| `ec_spire_remote_search_production_executor_state_summary(index_oid, requested_epoch, query, selected_pids, top_k, consistency_mode)` | operator | You need the planned production fanout state plus dry C0/C1 counters without resolving conninfo secrets or opening remote libpq sockets. |
| `ec_spire_remote_pipeline_steps(index_oid, requested_epoch, query, selected_pids, top_k, consistency_mode)` | operator | You need one consolidated remote-search pipeline row per dispatch, connection, candidate, heap, manifest, and result step without opening remote libpq connections. |
| `ec_spire_remote_pipeline_steps_live(index_oid, requested_epoch, query, selected_pids, top_k, consistency_mode)` | operator | You have already inspected the dry pipeline row and explicitly want live libpq connection, candidate, heap, and coordinator-result probes. |

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
conservative by default: relation or session `nprobe` applies at level 1, and
levels above 1 probe one child unless the index is configured with
`nprobe_per_level`. That reloption is a comma-separated list ordered from level
2 upward.

The same options snapshot reports the effective Phase 9 route-budget guardrails
as `recursive_beam_width`, `max_leaf_routes`, and `max_routing_expansions`.
These are derived from active leaf count and effective `nprobe`; they cap the
global recursive routing frontier while `nprobe_per_level` remains the local
per-parent input.

Phase 10 adds a `max_candidate_rows` scan cap. `max_candidate_rows = 0` means
`auto`, which resolves to the hard SPIRE candidate ceiling. This cap applies
before exact heap rerank, including scans with `rerank_width = 0`.

`ec_spire_index_scan_routing_snapshot(index_oid, query)` reports one row per
routing level touched by the query. `input_frontier_width` is the number of
parent routes entering that level; `expanded_parent_count` is the number
actually expanded before the route-expansion guard; `selected_child_count` is
the local per-parent routing output before global dedupe; and
`deduped_route_count` is the route count left after global dedupe and the level
cap. `truncation_reason` uses stable labels: `none`,
`max_routing_expansions`, `beam_width`, and `max_leaf_routes`.

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
`ec_spire_index_scan_placement_snapshot(index_oid, query)` then reports the
runtime side of that contract with primary versus boundary-replica candidate
rows, vec-id duplicate candidates suppressed by scan dedupe, and final
candidate winners after dedupe and candidate limits. The aggregate
`candidate_row_count` is the pre-dedupe total; its role split is
`primary_candidate_row_count + boundary_replica_candidate_row_count`. Candidate
rows retained by the bounded collection path are reported as
`candidate_winner_count`; rows dropped only by the candidate-row cap are
reported as `truncated_candidate_row_count` with matching primary versus
boundary-replica role splits.

`ec_spire_remote_pipeline_steps(...)` reports six stable `step_name` values:
`dispatch_plan`, `connection_check`, `candidates`, `heap_candidates`,
`manifest_apply`, and `coordinator_result`. The default surface is dry: it can
read conninfo-secret presence, but it does not open remote libpq connections or
execute remote candidate/coordinator probes. When the dry
`connection_check` row reports `requires_libpq_executor`, use
`ec_spire_remote_pipeline_steps_live(...)` only if live probe load is expected.
Both surfaces carry step-local counts, status, next blocker, and
recommendation; counts are not comparable across step names.

`ec_spire_remote_search_vector_identity_contract()` records the Phase 9 vector
identity contract. Global `0x02` vec IDs dedupe across nodes. Existing local
`0x01` vec IDs remain valid but remote merge scopes them by `node_id`, so
unrelated local sequences from different nodes cannot silently collapse into one
candidate.

## Distributed CustomScan Compatibility

SPIRE 0.1.2 uses `EcSpireDistributedScan` as the production distributed read
integration point. Remote-origin rows are delivered through CustomScan tuple
payloads, not through coordinator-side AM mirror rows.

Operator status labels changed with that pivot:

| Superseded label | Current label | Meaning |
| --- | --- | --- |
| `requires_remote_row_materialization` | `requires_custom_scan_tuple_delivery` | A remote-origin row reached a path that cannot deliver it as a coordinator heap TID; use the CustomScan tuple delivery path. |
| `remote_row_materialization` | `custom_scan_tuple_delivery` | The next blocker is the CustomScan tuple-delivery integration point, not a mirror-row catalog. |

The row-materialization and mirror-sync SQL contract entrypoints were removed
with the Shape-A AM mirror path. Operators should no longer expect rows for
`ec_spire_remote_search_row_materialization_contract`,
`ec_spire_remote_search_row_materialization_mapping_contract`, or the
operator-owned mirror-sync contract in
`ec_spire_remote_operator_entrypoint_contract()`. The surviving entrypoint
contract rows cover descriptor, manifest, libpq executor, pipeline, and
CustomScan-compatible read/write diagnostics.

Remote catalog cleanup functions keep the `row_materialization_*` result
columns as a 0.1.x compatibility shim, but they always report `0` after the
0.1.1 -> 0.1.2 upgrade drops `ec_spire_remote_row_materialization`. A future
0.2.x cleanup may remove those zero-valued columns once operator consumers have
had a full minor-version window to stop reading them.

Packet `30895` reran the full Stage E CustomScan matrix after the cleanup. The
matrix definitions remain anchored in packets `30770` (fault matrix), `30772`
(lifecycle matrix), and `30773` (per-case artifact convention), with packet
`30895` providing the current CustomScan evidence trail.

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

## Prepared Transaction Recovery

Coordinator-routed SPIRE writes use remote PostgreSQL prepared transactions.
Every remote PostgreSQL instance used for coordinator-routed writes must set
`max_prepared_transactions` above zero and leave enough free slots for peak
concurrent SPIRE remote prepares plus any other prepared transactions on that
instance. Changing the setting requires a PostgreSQL restart. When a remote
`PREPARE TRANSACTION` fails because prepared transactions are disabled or the
slot pool is exhausted, SPIRE wraps the remote error with a hint naming this
readiness requirement. Remote descriptor registration performs a nonblocking
preflight when `conninfo_secret_name` is resolvable: it warns if the remote
cannot be reached, if `SHOW max_prepared_transactions` cannot be read, or if
the value is zero. The warning does not reject the descriptor because
secret-resolution and remote availability are already separately visible
operator surfaces, but it must be treated as a write-readiness blocker before
enabling coordinator-routed writes.

If a coordinator backend crashes after remote prepare and before the xact
callback resolves the remote transaction, inspect the remote:

```sql
SELECT gid, prepared, owner, database
  FROM pg_prepared_xacts
 WHERE gid LIKE 'ec_spire_insert_%'
 ORDER BY prepared;
```

SPIRE GIDs have the stable form
`ec_spire_insert_<index_oid>_<node_id>_<served_epoch>_<top_xid>`. The
`ec_spire_insert` prefix is historical and currently covers both remote INSERT
and DELETE prepares; do not use the prefix to infer the operation type. There
is no backend pid in the GID; `top_xid` is the coordinator transaction identity
to correlate with logs and coordinator-side evidence while the resolution
decision remains based on the known coordinator transaction outcome and the
placement row state for the affected key.

Resolve only after the affected primary key and coordinator outcome are known:

1. On the coordinator, inspect `ec_spire_placement` for the parsed
   `index_oid`, `node_id`, `served_epoch`, and the affected primary key.
2. For INSERT recovery, commit the remote prepared transaction only when the
   coordinator transaction committed and the expected placement row exists.
   Roll it back when the coordinator transaction aborted or the placement row
   is absent after the outcome is known.
3. For DELETE recovery, commit the remote prepared transaction only when the
   coordinator transaction committed and the placement row was removed. Roll it
   back when the coordinator transaction aborted and the placement row remains.
4. After `COMMIT PREPARED` or `ROLLBACK PREPARED`, re-query the remote row and
   the coordinator placement row to verify the two sides match the intended
   outcome.

If the coordinator transaction outcome or affected primary key cannot be
established, leave the prepared transaction unresolved and escalate with the
GID, remote node id, and coordinator index OID. Do not bulk-resolve SPIRE GIDs
from the remote side alone.

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
