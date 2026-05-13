# Task 30 Phase 12: SPIRE Production Hardening

Status: planned
Owner: coder1 / SPIRE distributed production-hardening track
Priority: 1 after Phase 11 CustomScan functional delivery

## Goal

Turn the working ADR-067 CustomScan read path and ADR-069 v1 distributed write
path into a locally hardened production candidate before any AWS/RDS-class
verification. Phase 12 is not an AWS scale phase. It is the exhaustive
non-happy-path, performance, operator-readiness, and local-capacity phase
described by reviewer packet `30896`.

## Entry State

- Phase 9 local graph architecture is complete.
- Phase 10 local execution architecture is complete.
- Phase 11 functional delivery is complete:
  - `SELECT ... ORDER BY embedding <op> $query LIMIT k` can return remote rows
    through `EcSpireDistributedScan`.
  - Coordinator-routed INSERT, non-embedding UPDATE, DELETE, PK SELECT, and
    embedding-UPDATE rejection are live.
  - The materialization catalog/register path and AM materialization blocker
    are removed.
  - Stage E fault matrix (11 cases) and lifecycle matrix (6 cases) pass
    against the CustomScan build in packet `30895`.

## Non-Goals

- AWS/RDS-class scale verification. That is Phase 13.
- Billion-scale product claims.
- Multi-coordinator HA, coordinator election, and cross-shard embedding UPDATE
  moves. These remain Phase 12+ or later ADR scope unless explicitly reopened.
- ADR-069 deferred items, including cross-shard non-vector queries, DDL
  propagation, foreign keys, sequences, rebalance, and multi-coordinator
  deployments, remain later ADR scope unless explicitly reopened.
- Bulk-load product tooling beyond primitives or fixtures needed to harden the
  v1 coordinator-routed write path.

## Phase 12.1: Tracker and Operator-Compatibility Reconciliation

- [x] Reconcile stale parent checkboxes in the Phase 11 task file where child
  evidence is complete, especially Stage B endpoint, Stage C executor, and
  Stage D CustomScan parent rows.
  - [x] Reviewer packet `30910` closes Phase 11 and records the disposition of
    every remaining open box as done, moved to Phase 12, or moved to Phase 13.
- [x] Add a 0.1.1 -> 0.1.2 migration comment explaining why
  `ec_spire_remote_row_materialization` was created in the previous migration
  and dropped after the Shape-A -> Shape-B CustomScan pivot.
  - [x] `ecaz--0.1.1--0.1.2.sql` now explains the Shape-A AM mirror origin and
    the Shape-B CustomScan removal.
- [x] Document the diagnostic status-string rename:
  `requires_remote_row_materialization` ->
  `requires_custom_scan_tuple_delivery`.
  - [x] `docs/SPIRE_DIAGNOSTICS.md` records the old and current labels plus the
    `remote_row_materialization` -> `custom_scan_tuple_delivery` blocker rename.
- [x] Document dropped mirror-sync / row-materialization operator-entrypoint
  rows so operator monitoring can adjust expected row counts.
  - [x] `docs/SPIRE_DIAGNOSTICS.md` records the removed row-materialization and
    mirror-sync contract rows in `ec_spire_remote_operator_entrypoint_contract()`.
- [x] Schedule the 0.2.x compatibility cleanup that removes zero-valued
  `row_materialization_*` shim columns from remote catalog cleanup diagnostics.
  - [x] `docs/SPIRE_DIAGNOSTICS.md` documents the 0.1.x shim window and the
    future 0.2.x removal point.
- [x] Cross-link packet `30895` to the earlier Stage E matrix definition
  packets (`30770`, `30772`, `30773`) so each fault/lifecycle case has a
  durable definition and evidence trail.
  - [x] `docs/SPIRE_DIAGNOSTICS.md` links `30895` evidence to `30770`,
    `30772`, and `30773`.

## Phase 12.2: Typed Tuple Transport and JSON Retirement

- [x] Design the typed tuple-payload protocol that replaces the current JSON
  bridge for remote-origin tuple delivery.
  - [x] `plan/design/spire-typed-tuple-transport.md` selects per-attribute
    PostgreSQL binary I/O, defines endpoint metadata, negotiation, JSON
    fallback, and removal criteria before executor changes.
- [x] Add a typed remote endpoint beside the JSON endpoint, preferably using
  PostgreSQL binary composite/record transport or per-attribute `typsend` /
  `typreceive` bytes.
- [x] Add endpoint negotiation so CustomScan prefers typed tuple transport when
  the remote advertises support and falls back to JSON only during the
  migration window.
  - [x] Endpoint identity advertises `tuple_transport_capabilities`,
    `tuple_transport_default`, and `tuple_transport_status` for
    `pg_binary_attr_v1`.
  - [x] Exit criterion: the JSON fallback may remain production-reachable for
    one minor-version compatibility window after the typed endpoint release;
    removal requires all scalar, array, composite, NULL, and domain fixture
    classes to pass through typed transport and a reviewer-accepted packet that
    names any unsupported type gaps.
  - Evidence: packet `30963` switches the production libpq heap receive path
    to request `ec_spire_remote_search_tuple_payload_typed(...)` whenever the
    endpoint identity advertises ready `pg_binary_attr_v1`, with JSON retained
    only for older endpoints that lack the typed capability metadata.
- [x] Switch `EcSpireDistributedScan` tuple slot delivery from
  `serde_json`/text input to typed binary datum construction.
  - Evidence: packet `30963` adds typed payload rows to the production scan
    output model and materializes remote-origin CustomScan slots through
    PostgreSQL binary receive functions. The PG18 CustomScan fixture returns a
    remote row with `text[]`, domain, and named-composite payload columns via
    `Custom Scan (EcSpireDistributedScan)`.
- [x] Remove or retire the scalar-only JSON gate for arrays/composites once the
  typed transport covers them.
  - Evidence: packet `30963` removes the CustomScan array/composite planner
    rejection gate and proves the production CustomScan path can deliver
    `text[]` and named-composite values through typed payload bytes. JSON
    remains as a compatibility fallback and production retirement is still
    tracked separately below.
- [x] Add fixtures proving typed transport round-trips scalar, array,
  composite, NULL, and domain values where supported.
  - [x] Scalar JSON-parity fixture covers `bigint` and `text` payload bytes via
    `ec_spire_remote_search_tuple_payload_typed(...)`.
  - [x] NULL and array fixture covers out-of-band NULL flags plus `text[]`
    binary payload bytes via `array_send(...)`.
  - [x] Domain and composite fixture covers domain metadata with base binary
    bytes via `textsend(...)` and named composite bytes via `record_send(...)`.
  - [x] Empty projection fixture proves typed transport returns aligned empty
    metadata/value arrays without falling back to JSON.
  - [x] All v1 distributed table column classes that blocked the JSON bridge
    now have endpoint-level typed transport coverage; negotiation can proceed
    without a known scalar/array/composite/domain/NULL type-class gap.
  - Evidence: packets `30915`, `30916`, `30917`, and `30918` cover scalar
    JSON-parity, NULL, `text[]`, domain, named composite, and empty-projection
    endpoint fixtures through `ec_spire_remote_search_tuple_payload_typed(...)`;
    the remaining Phase 12.2 rows still track negotiation, CustomScan typed
    receive, throughput measurement, and production JSON retirement.
- [x] Measure tuple-heavy read throughput before and after typed transport.
  - [x] Add `ec_spire.remote_tuple_transport` plus
    `ecaz bench spire-pipeline --remote-tuple-transport` so the measurement
    can force JSON fallback versus `pg_binary_attr_v1` without editing fixture
    scripts.
  - [x] Add `ecaz bench spire-pipeline --query-metric-projection-columns` so
    the benchmark can project payload columns during coordinator KNN latency
    measurement instead of timing id-only scans.
  - [x] Packet `30975` measures a two-index loopback fixture with `rabitq`
    storage, 20 coordinator KNN queries, `nprobe=8`, `k=10`, and tuple-heavy
    `title,body` projection. The warm pass returned the same 200 rows and
    31,510 payload bytes for both modes: JSON fallback p50/p95/p99
    `28.988/30.459/35.103 ms` at `34.081 qps`, typed `pg_binary_attr_v1`
    p50/p95/p99 `33.892/35.179/41.313 ms` at `29.160 qps`. This local
    loopback shape proves the transport switch and typed capability readiness
    but does not claim a typed speedup for the small scalar payload fixture.
- [ ] After compatibility is sufficient, remove the JSON endpoint from the
  production path and drop the `serde_json` dependency if no other runtime path
  needs it.

## Phase 12.3: Planner, Metadata, and Cost Hardening

- [x] Replace the `ec_spire_placement` planner eligibility seqscan with an
  indexed existence check against the placement primary key or a dedicated
  `index_oid` index.
  - [x] Added `ec_spire_placement_by_index_oid` and changed the DML PK-select
    CustomScan planner gate to use an explicit index scan for index presence.
- [x] Add regression coverage proving planner eligibility remains O(log N) or
  otherwise bounded as placement rows grow.
  - [x] `test_ec_spire_placement_index_oid_lookup_uses_index_sql` inserts
    unrelated placement rows and asserts the `index_oid` lookup uses
    `ec_spire_placement_by_index_oid`; the existing DML PK-select CustomScan
    test still verifies planner replacement.
- [x] Add a per-snapshot relation-context cache keyed by heap/index relcache
  invalidation state.
- [x] Register relcache invalidation callbacks for the heap relation and its
  `ec_spire` indexes.
  - [x] DML frontdoor catalog relation context now uses a backend-local
    relcache-invalidated cache, watches the heap plus its index relids, and
    exposes `ec_spire_dml_frontdoor_relation_context_cache()` for hit/miss and
    invalidation diagnostics.
    `test_ec_spire_dml_context_cache_invalidation_sql`
    warms a no-index context, verifies a cache hit, creates the `ec_spire`
    index, and verifies the refreshed context is not stale.
- [ ] Calibrate the symbolic CustomScan cost constants from local benchmark
  measurements across fanout counts, placement counts, output row counts, and
  tuple payload widths.
- [ ] Store the benchmark logs packet-locally and update the cost constants
  only when the measurement explains the chosen values.
- [x] Clean up `custom_private` layout by replacing JSON string lists with
  native PostgreSQL node lists or another typed representation.
  - [x] DML CustomScan `custom_private` now stores updated/projected column
    metadata as native PostgreSQL `String` nodes with explicit column counts,
    including zero-count empty projected-column metadata.
    `test_ec_spire_custom_scan_dml_plan_private_copyobject_sql` covers
    copyObject round-trip behavior.
- [x] Replace trivial per-row/per-statement PK-byte `Vec<u8>` allocations with
  stack `[u8; 8]` or caller-owned buffers where profiling shows pressure.
  - [x] DML frontdoor and CustomScan bigint PK bytes now use `[u8; 8]`
    through plan construction, runtime parameter evaluation, invocation
    metadata, and executor state. The remaining `Vec<u8>` conversion is at the
    PostgreSQL `bytea` boundary for SPI/pg_extern calls. Focused PG18 checks:
    `test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send` and
    `test_ec_spire_dml_frontdoor_primitive_plan_from_decision`.

## Phase 12.4: Coordinator-Routed Write and 2PC Hardening

- [x] Add a concurrent INSERT descriptor-generation-race fixture:
  - [x] fire N parallel coordinator INSERTs targeting the same descriptor;
  - [x] assert exactly one succeeds under the current v1 guard or implement
    accepted retry behavior;
  - [x] assert failed attempts leave no orphaned remote prepared transactions.
    - [x] `test_ec_spire_insert_descriptor_race_sql`
      holds one coordinator INSERT transaction open after remote prepare,
      drives a second same-descriptor INSERT to the documented
      `serialization_failure` retry path, and asserts only the winner's remote
      row and placement row remain with no SPIRE prepared xacts.
- [x] Pin a stable SQLSTATE for descriptor refresh races and document the safe
  retry contract in ADR-069.
- [x] Decide the concurrent DELETE collision policy in ADR-069:
  `DELETE-not-found is success` versus `DELETE-not-found is error`.
- [x] Implement and test the chosen DELETE collision behavior, including
  placement-directory idempotence.
- [x] Remove volatile backend pid from SPIRE prepared-transaction GIDs; use a
  stable `(index_oid, node_id, served_epoch, xid)`-style identity instead.
- [x] Add an orphaned prepared-transaction recovery runbook:
  - [x] identify SPIRE GIDs on a remote via `pg_prepared_xacts`;
  - [x] decide commit vs rollback from coordinator placement-directory state;
  - [x] verify the recovered remote row or cleanup result.
- [x] Consider and, if accepted, add
  `ec_spire_recover_orphaned_prepared_xacts(node_id)` for operator recovery.
  - [x] Decision: defer the helper for v1. ADR-069 and
    `docs/SPIRE_DIAGNOSTICS.md` now state that remote `pg_prepared_xacts`
    alone does not contain the affected primary key or coordinator transaction
    outcome needed to safely choose `COMMIT PREPARED` versus
    `ROLLBACK PREPARED`; operators must use the explicit placement-directory
    runbook until SPIRE records durable prepared-transaction intent metadata.
- [x] Bring INSERT 2PC dispatch cancellation to parity with Stage C read
  cancellation:
  - [x] bridge local `InterruptPending` / `QueryCancelPending` to the remote
    libpq/tokio cancellation path;
  - [x] fixture local cancel or statement timeout during slow remote prepare;
  - [x] assert remote prepared transactions are rolled back, not orphaned.
  - [x] `coordinator_insert_prepare_remote_sql` now routes through the async
    prepare adapter, which polls local cancel flags, sends the remote cancel
    token, rolls back open remote transactions, and rolls back any
    just-prepared remote transaction before local commit callbacks are
    registered. PG18 fixture
    `test_ec_spire_insert_prepare_local_cancel_rolls_back` has the loopback
    remote cancel the coordinator while remote prepare SQL is sleeping, then
    asserts `local_query_cancelled`, no matching `pg_prepared_xacts`, and no
    visible remote row.
  - [x] Reviewer note: the fixture covers the hardest post-prepare cleanup
    window directly; the earlier remote-SQL, descriptor-metadata, and
    `PREPARE TRANSACTION` failure windows rely on the same rollback path by
    construction.
- [x] Add `max_prepared_transactions` readiness:
  - [x] document it as required on every remote;
  - [x] check or warn during descriptor registration;
  - [x] wrap `PREPARE TRANSACTION` exhaustion with a SPIRE-named hint.
- [x] Add a multi-row INSERT trigger fixture proving statement-batched trigger
  dispatch lands every row on its owning remote and commits all remote prepared
  transactions on local commit.
- [x] Add a placement-table write-contention fixture with many parallel
  distinct-PK INSERTs and DELETEs, asserting no deadlocks and bounded latency
  growth.
  - [x] `test_pg18_ec_spire_placement_write_contention_distinct_pk_dml`
    releases 8 distinct-PK INSERT+DELETE workers behind an advisory barrier,
    asserts worker success, row counts, no placement lock waiters, no new
    deadlocks, and p99 completion below a predeclared 20s threshold.
    A fixture comment now pins the same-transaction app-table plus
    placement-table write shape so it keeps mirroring the production
    coordinator INSERT trigger path.
- [x] Evaluate partitioning `ec_spire_placement` by `index_oid` if contention
  evidence shows shared-table hot pages.
  - [x] Exit criterion: choose the fixture's writer count and p99 latency
    threshold before running it; partition only if packet-local evidence crosses
    that threshold or shows lock waits/deadlocks attributable to shared-table
    placement pages.
  - [x] Current decision: keep the shared table for now because the PG18
    fixture passed with 8 writers, no deadlocks, no placement lock waiters, and
    p99 below the 20s partition threshold.
- [x] Migrate wide-fanout INSERT 2PC dispatch to the same async/pipelined
  transport pattern as the read path, so M remote prepares do not serialize
  into M round trips.
  - [x] `ec_spire_enable_coordinator_insert(...)` now installs a BEFORE ROW
    queue trigger plus an AFTER STATEMENT flush trigger. The flush calls
    `ec_spire_prepare_coordinator_insert_tuple_payload_batch(...)`, which
    routes all queued remote prepares through the Tokio fanout batch adapter
    and rolls back any prepared remotes if one dispatch fails before local
    callbacks are registered. PG18 fixtures
    `test_ec_spire_enable_coordinator_insert_trigger_sql` and
    `test_ec_spire_trigger_multirow_commits_prepares_sql` cover trigger
    installation, queue drain, and multi-row commit behavior.
  - Follow-up notes from review packet `30973`: keep any future read/write
    fanout adapter consolidation and same-node batch micro-optimization
    evidence-driven; current P9 closure does not require either refactor.
- [x] Document the 2PC latency tradeoff and the bulk-load escape hatch for
  applications that can tolerate post-write placement registration.

## Phase 12.5: Schema Drift, DDL, and Type Round-Trip Hardening

- [x] Fingerprint the user table column shape on the coordinator at trigger
  installation or descriptor registration time.
- [x] Store or bind the column-shape fingerprint to the remote descriptor.
- [x] Before INSERT dispatch, compare current coordinator shape to the
  descriptor fingerprint and fail closed on drift with a clear "schema drift"
  error and remediation hint.
- [x] Add a fixture that `ALTER TABLE`s the coordinator only, attempts INSERT,
  and asserts the schema-drift error fires before remote dispatch.
  - [x] `ec_spire_register_remote_node_descriptor` binds
    `coordinator_insert_shape_fingerprint` from the coordinator heap shape, and
    coordinator-routed INSERT validates it before remote libpq dispatch.
  - [x] `test_ec_spire_schema_drift_fails_before_dispatch_sql` alters only the
    coordinator table and asserts the schema-drift error leaves no remote row
    and no SPIRE prepared transaction.
- [x] Extend descriptor-bound schema-drift coverage to coordinator-routed
  UPDATE and DELETE payload paths, or document why INSERT-only remains the
  accepted v1 boundary.
  - [x] The pre-dispatch guard now runs for remote UPDATE and DELETE as well
    as INSERT; `test_ec_spire_update_delete_schema_drift_guard_sql` asserts
    remote UPDATE leaves the row unchanged and remote DELETE leaves the remote
    row, placement row, and prepared transaction state untouched.
- [x] Add round-trip fixtures for non-trivial trigger payload types:
  - [x] `numeric` / `decimal` precision;
  - [x] `timestamptz` timezone and value preservation;
  - [x] `json` and `jsonb` nested payloads;
  - [x] `text` with embedded edge characters, documenting unsupported values
    if PostgreSQL/JSON cannot round-trip them;
  - [x] domain-over-base-type validation;
  - [x] nullable columns with SQL NULL;
  - [x] NOT NULL violation paths;
  - [x] DEFAULT-valued columns after PostgreSQL has materialized `NEW`.
- [x] Document the v1 DDL ordering contract: pause writes, apply DDL to the
  coordinator, apply matching DDL to every remote, refresh descriptors, then
  resume writes.
- [x] Decide whether a lightweight DDL window guard is needed to block writes
  while remote schemas are known to be inconsistent.
  - [x] Decision: no separate v1 DDL-window guard GUC/catalog flag; operators
    must pause writes during DDL, and the descriptor-bound Phase 12.5
    schema-drift fingerprint is now the fail-closed safety net for violated
    ordering.

## Phase 12.6: Isolation, EvalPlanQual, and Negative DML Coverage

- [x] Document the v1 distributed read isolation contract in ADR-068/ADR-069:
  distributed CustomScan virtual tuples do not get PostgreSQL's normal
  EvalPlanQual rerun semantics.
- [x] Add fixtures for SERIALIZABLE / REPEATABLE READ / READ COMMITTED
  distributed PK SELECT behavior and pin the expected v1 outcome.
- [x] Decide whether `ec_spire_custom_scan_recheck` should remain
  unconditional for v1 or re-run the primitive for a subset of read paths.
  - [x] Exit criterion: the decision packet must include an isolation fixture
    that demonstrates the stale-row/EvalPlanQual behavior under at least
    SERIALIZABLE and states the accepted v1 contract in ADR-068 or ADR-069.
- [x] Add negative classifier fixtures for unsupported PK predicate shapes:
  - [x] `$1::numeric` outside int8 range;
  - [x] `$1::int8 IS NULL` with stable SQLSTATE;
  - [x] `WHERE id IN (...)`;
  - [x] `WHERE id = $1 OR id = $2`;
  - [x] any non-bigint equality accepted accidentally by coercion.
- [x] Ensure unsupported DML shapes against a SPIRE-fronted table continue to
  fail closed rather than falling through to the empty coordinator heap.
  - [x] PK predicate edge fixture asserts unsupported SELECT/PREPARE shapes
    raise `feature_not_supported` through the planner hook before execution.

## Phase 12.7: Multi-Instance Placement, Epoch, and Replica Readiness

- [x] Add or extend an `ecaz`-owned local one-coordinator/two-remote setup and
  teardown command for repeated Stage E and readiness runs.
  - Evidence: `ecaz dev spire-multicluster` now covers the repeated fixture
    setup/teardown paths used by this phase: the existing
    `transport-overlap-pg18` command owns the one-coordinator/two-remote
    transport fixture, and the new `customscan-read-pg18` command owns the
    CustomScan readiness setup/read/teardown path. The CustomScan wrapper
    forwards packet-local artifact, run-dir, log-dir, smoke-log, port, run-id,
    and `--skip-install` options to the existing PG18 fixture; parser coverage
    pins the CLI surface without starting PostgreSQL.
- [x] Publish and inspect placement metadata that maps selected PIDs to remote
  nodes and local store IDs.
  - Evidence: `ec_spire_index_selected_pid_placement_snapshot(index_oid,
    selected_pids)` returns one row per selected PID with `pid`, `node_id`,
    `local_store_id`, `store_relid`, placement state, object version, and
    object bytes; the PG18 fixture rewrites one selected PID to a remote node
    and verifies the selected PID map reports both the local and remote
    `(node_id, local_store_id)` pairs.
- [x] Verify strict mode never mixes incompatible epochs across nodes.
  - Evidence: packet `30895` Stage E `epoch_mismatch` strict artifact runs a
    two-dispatch coordinator/remote fixture where one remote advertises a stale
    epoch window. Strict mode reports `status = stale_epoch`,
    `blocked_before_dispatch_count = 1`,
    `degraded_skipped_dispatch_count = 0`, and
    `next_executor_step = remote_epoch_window`, proving the coordinator does
    not continue by mixing the ready remote with the incompatible-epoch remote.
- [x] Verify degraded mode reports every skipped or stale remote node with node
  identity, count, and first skip category.
  - Evidence: `ec_spire_remote_search_degraded_skip_report(...)` returns one
    row per degraded-skipped remote dispatch with `node_id`,
    `skipped_pid_count`, `first_skip_category`, and `status`; unit coverage
    proves stale-epoch and incompatible-version pre-dispatch blockers are
    reported as separate skipped nodes in degraded mode.
- [x] Add remote-node multi-instance proof that boundary replicas carry the
  same global original-vector identity across leaves, stores, and remotes.
  - Evidence: `ec_spire_index_boundary_replica_identity_snapshot(index_oid)`
    groups primary and boundary-replica assignments by global `vec_id` and
    reports their node/local-store span; the PG18 fixture rewrites one leaf
    placement to remote node `2` and verifies at least one ready global
    identity spans node IDs `0..2` while preserving one
    primary plus one boundary-replica assignment per source identity.
- [x] Add boundary-replica manifest freshness fixtures using
  `ec_spire_remote_epoch_manifest_freshness()`.
  - Evidence: `test_ec_spire_boundary_replica_manifest_freshness_sql` builds a
    boundary-replica index with global source identity, rewrites one leaf
    placement to remote node `2`, verifies freshness requires manifest
    persistence before `ec_spire_persist_remote_epoch_manifest(...)`, verifies
    ready freshness after persistence, then drifts the persisted entry and
    verifies `stale_remote_epoch_manifest` with
    `refresh_remote_epoch_manifest`.
- [x] Add operator diagnostics for stale, missing, or unavailable boundary
  replica placements and their degraded-mode reporting.
  - Evidence:
    `ec_spire_index_boundary_replica_placement_diagnostics(index_oid)` groups
    boundary-replica placement health by global `vec_id`, reports missing
    replica assignments plus stale, unavailable, and skipped replica placement
    counts, and labels the degraded-mode action as either `fail_closed` or
    `skip_and_report`; the PG18 fixture rewrites a boundary-replica leaf
    through unavailable, skipped, and stale states and verifies each diagnostic
    status/action.
- [x] Preserve and periodically rerun the full Stage E fault/lifecycle matrix
  against the current CustomScan path while this hardening proceeds.
  - Evidence: packet `30961` reruns the CustomScan read proof plus the full
    Stage E matrix against the current branch: 11/11 fault cases report
    `stage_e_fault_<case>_passed=true`, all 6 lifecycle cases report
    `stage_e_lifecycle_<case>_passed=true`, and the CustomScan read proof
    reports `Custom Scan (EcSpireDistributedScan)` plus
    `SPIRE multicluster CustomScan read passed`.

## Phase 12.8: Local Multi-Store and Multi-NVMe Readiness

- [x] Preserve `(node_id, local_store_id)` as the scheduling and diagnostic
  unit.
- [x] Prove local store lookup remains indexed or otherwise bounded for the
  configured maximum store count.
  - Evidence: `ec_spire_index_placement_snapshot(index_oid)` exposes
    `(node_id, local_store_id, store_relid)` and
    `ec_spire_index_scan_placement_snapshot(index_oid, query)` exposes
    scan-touched `(node_id, local_store_id)` groups. The PG18 two-store SQL
    VACUUM fixture now asserts those diagnostic keys along with the
    post-delete/post-insert delta-cleanup counts.
  - Evidence: in-memory local object stores resolve by a prebuilt
    `local_store_id -> stores[index]` map, and relation-backed object stores
    resolve by `(local_store_id, store_relid) -> stores[index]`; packet `30678`
    reviewed the indexed lookup implementation and its non-contiguous store-id
    coverage.
- [x] Add a repeatable local multi-store read-overlap harness with per-store
  route, candidate, object-byte, read-batch, and delta-decode counters.
  - Evidence:
    `ec_spire_index_scan_local_store_read_overlap_harness(index_oid, query)`
    reports one row per touched `(node_id, local_store_id)` with route counts,
    candidate rows, prefetched object bytes, read-batch count, and
    delta-decode count. The PG18 multi-store SQL fixture asserts two touched
    store groups, one read batch per store group, positive object bytes, and
    one selected delta decode after a post-build insert.
- [x] If PostgreSQL backend constraints keep execution sequential, expose the
  limitation in diagnostics and document the exact future primitive needed to
  improve it.
  - Evidence:
    `ec_spire_index_scan_local_store_execution_snapshot(index_oid, query)` now
    reports `local_store_execution_mode = 'sequential_backend'`,
    `local_store_read_ahead_primitive`, and
    `local_store_parallelism_next_step =
    'async_or_parallel_store_group_executor'`; the PG18 scan-placement SQL
    fixture asserts those labels, and the diagnostics/design docs define the
    read-ahead versus true parallel execution boundary.
- [x] Confirm delta decode reuse remains covered under multi-store and remote
  candidate paths.
  - Evidence: packet `30677` added `SpireLoadedDeltaObjectRoute` and
    `load_delta_rows_for_routes_reads_each_delta_object_once`, proving selected
    delta routes are decoded once and reused for delete suppression plus
    delta-insert candidate scoring. Remote candidate and tuple-payload
    endpoints call the same selected-leaf collector before origin-node heap or
    payload resolution; the PG18 remote local heap resolution fixture now
    covers a post-build delta row returned through
    `ec_spire_remote_search_local_heap_candidates(...)`.

## Phase 12.9: Local Production Harness and Runbook

- [x] Extend `ecaz` with setup, load, query, teardown, and benchmark commands
  for the local distributed SPIRE fixture when shell scripts become repeated
  operator workflows.
  - [x] `ecaz dev spire-multicluster` now exposes the baseline PG18
    multicluster smoke fixture and INSERT-after-CustomScan read fixture through
    `smoke-pg18` and `insert-read-after-customscan-pg18`, including
    packet-local artifact/log controls and skip-install/run-dir options.
  - [x] Together with the accepted `customscan-read-pg18`,
    `transport-overlap-pg18`, `fault-pg18`, and `lifecycle-pg18` wrappers, the
    local distributed fixture CLI covers baseline setup/teardown, repeated
    reads, write-then-read, two-remote transport overlap, and Stage E fault /
    lifecycle reruns without direct shell-script invocation. The CLI README
    command tree now lists all six wrappers.
- [x] Add or extend `ecaz bench spire-pipeline` for distributed recall,
  latency, and counter capture across local instances.
  - [x] `ecaz bench spire-pipeline` can now optionally emit coordinator query
    latency p50/p95/p99 and recall@k alongside routing/local/remote pipeline
    counters through `--include-query-metrics` and `--include-recall`; it can
    also aggregate local-store object bytes, read batches, and delta-decode
    counts through `--include-local-store-overlap`.
  - [x] `ecaz bench spire-pipeline` now records endpoint tuple-transport
    readiness plus remote degraded-skip node/count/category counters when
    remote diagnostics are enabled.
  - [x] Reviewer packet `30968` accepted the combined 30965-30967 bench batch
    as the completed Phase 12.9 harness surface; live packet-local artifact
    capture remains a separate open row below.
- [ ] Capture recall, latency p50/p95/p99, object bytes, route counts,
  candidate counts, heap rows, remote fanout, timeout/cancel counts,
  strict-failure counts, degraded-skip counts, placement contention, and typed
  tuple transport counters in packet-local artifacts.
- [x] Publish local capacity targets for maximum remotes, maximum concurrent
  coordinator queries, maximum concurrent writers, maximum work per remote,
  maximum PIDs per node, and expected overload/degraded behavior.
  - Evidence: `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md` publishes the local
    production-readiness smoke profile, including explicit remote fanout caps,
    conservative one-at-a-time read/write concurrency targets, per-remote work
    limits, required GUC settings, and strict/degraded overload behavior. The
    readiness boundary doc now requires packets to cite the active capacity
    profile and forbids raising targets without packet-local benchmark or
    contention logs.
- [x] Include libpq security and operations in the runbook:
  `sslmode` preservation, raw-conninfo non-exposure, sanitized
  auth/certificate failures, credential-rotation deferral, audit-log deferral,
  `max_prepared_transactions`, and orphaned-prepared-xact recovery.
  - Evidence: `docs/SPIRE_LIBPQ_RUNBOOK.md` defines the connection security
    contract, sanitized strict/degraded failure behavior, prepared-transaction
    readiness, orphaned prepared xact recovery, credential-rotation deferral,
    and audit-log deferral; `docs/SPIRE_DIAGNOSTICS.md` links operators to it.
- [x] Distinguish local functionality, local production-readiness smoke, and
  AWS/RDS product-scale evidence in the docs.
  - Evidence: `docs/SPIRE_LOCAL_READINESS.md` defines the three evidence labels,
    allowed claims, disallowed claims, artifact requirements, and the Phase 13
    entry boundary.
- [ ] Produce a final local production-readiness bundle from clean setup
  through distributed read/write, fault/degraded checks, multi-store checks,
  and harness artifact capture.
  - [ ] Include the endpoint tuple-transport capability summary, especially
    `pg_binary_attr_v1_ready`, in the bundle header so throughput/latency
    artifacts expose whether the remote typed-receive path was ready.

## Suggested Packet Sequence

Reviewer packet `30896` suggested `30897`-`30908` for the first hardening wave.
Treat the numbers below as a planning sequence; exact packet numbers may shift
if feedback packets land first.

1. Typed tuple transport design and first endpoint (`P1`, `P3`).
2. Placement planner gate indexed lookup (`P2`).
3. 2PC GID cleanup and orphaned-transaction runbook (`H3`).
4. Concurrency test matrix: INSERT race, DELETE collision, cancel
   mid-prepare, multi-row INSERT, placement contention (`H1`, `H2`, `H4`,
   `H11`, `H12`).
5. Schema-drift detection and type round-trip fixtures (`H6`, `H7`, `H10`).
6. Async INSERT dispatch (`P9`).
7. `max_prepared_transactions` preflight and error wrapping (`H9`).
8. EvalPlanQual / isolation contract fixtures (`H5`).
9. Catalog-backed relation-context cache (`P4`).
10. Cost-model calibration (`P6`).
11. Negative classifier coverage (`H8`).
12. `custom_private` and PK-byte allocation cleanup (`P7`, `P8`).
13. Multi-instance placement/replica readiness.
14. Local multi-store / multi-NVMe readiness.
15. Local production harness and runbook.

## Exit Criteria

- All H1-H12 hardening items from packet `30896` are implemented, explicitly
  deferred with reviewer acceptance, or moved to a later ADR with rationale.
- All P1-P9 performance hotspots from packet `30896` are implemented,
  measured, or explicitly deferred with reviewer acceptance.
- Local multi-instance, multi-store, strict/degraded, write-contention,
  schema-drift, type-round-trip, and isolation fixtures have packet-local
  evidence.
- The operator runbook defines required GUCs, recovery steps, local capacity
  targets, and the boundary between local readiness and AWS verification.
- Phase 13 AWS verification is allowed to open only after this file is
  complete or has accepted deferrals for every remaining open item.
