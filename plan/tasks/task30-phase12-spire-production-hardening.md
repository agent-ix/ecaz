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
- [ ] Add endpoint negotiation so CustomScan prefers typed tuple transport when
  the remote advertises support and falls back to JSON only during the
  migration window.
  - [x] Endpoint identity advertises `tuple_transport_capabilities`,
    `tuple_transport_default`, and `tuple_transport_status` for
    `pg_binary_attr_v1`.
  - [ ] Exit criterion: the JSON fallback may remain production-reachable for
    one minor-version compatibility window after the typed endpoint release;
    removal requires all scalar, array, composite, NULL, and domain fixture
    classes to pass through typed transport and a reviewer-accepted packet that
    names any unsupported type gaps.
- [ ] Switch `EcSpireDistributedScan` tuple slot delivery from
  `serde_json`/text input to typed binary datum construction.
- [ ] Remove or retire the scalar-only JSON gate for arrays/composites once the
  typed transport covers them.
- [ ] Add fixtures proving typed transport round-trips scalar, array,
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
- [ ] Measure tuple-heavy read throughput before and after typed transport.
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
- [ ] Add a per-snapshot relation-context cache keyed by heap/index relcache
  invalidation state.
- [ ] Register relcache invalidation callbacks for the heap relation and its
  `ec_spire` indexes.
- [ ] Calibrate the symbolic CustomScan cost constants from local benchmark
  measurements across fanout counts, placement counts, output row counts, and
  tuple payload widths.
- [ ] Store the benchmark logs packet-locally and update the cost constants
  only when the measurement explains the chosen values.
- [ ] Clean up `custom_private` layout by replacing JSON string lists with
  native PostgreSQL node lists or another typed representation.
- [ ] Replace trivial per-row/per-statement PK-byte `Vec<u8>` allocations with
  stack `[u8; 8]` or caller-owned buffers where profiling shows pressure.

## Phase 12.4: Coordinator-Routed Write and 2PC Hardening

- [ ] Add a concurrent INSERT descriptor-generation-race fixture:
  - [ ] fire N parallel coordinator INSERTs targeting the same descriptor;
  - [ ] assert exactly one succeeds under the current v1 guard or implement
    accepted retry behavior;
  - [ ] assert failed attempts leave no orphaned remote prepared transactions.
- [ ] Pin a stable SQLSTATE for descriptor refresh races and document the safe
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
- [ ] Consider and, if accepted, add
  `ec_spire_recover_orphaned_prepared_xacts(node_id)` for operator recovery.
- [ ] Bring INSERT 2PC dispatch cancellation to parity with Stage C read
  cancellation:
  - [ ] bridge local `InterruptPending` / `QueryCancelPending` to the remote
    libpq/tokio cancellation path;
  - [ ] fixture local cancel or statement timeout during slow remote prepare;
  - [ ] assert remote prepared transactions are rolled back, not orphaned.
- [ ] Add `max_prepared_transactions` readiness:
  - [x] document it as required on every remote;
  - [x] check or warn during descriptor registration;
  - [x] wrap `PREPARE TRANSACTION` exhaustion with a SPIRE-named hint.
- [ ] Add a multi-row INSERT trigger fixture proving per-row trigger dispatch
  lands every row on its owning remote and commits all remote prepared
  transactions on local commit.
- [ ] Add a placement-table write-contention fixture with many parallel
  distinct-PK INSERTs and DELETEs, asserting no deadlocks and bounded latency
  growth.
- [ ] Evaluate partitioning `ec_spire_placement` by `index_oid` if contention
  evidence shows shared-table hot pages.
  - [ ] Exit criterion: choose the fixture's writer count and p99 latency
    threshold before running it; partition only if packet-local evidence crosses
    that threshold or shows lock waits/deadlocks attributable to shared-table
    placement pages.
- [ ] Migrate wide-fanout INSERT 2PC dispatch to the same async/pipelined
  transport pattern as the read path, so M remote prepares do not serialize
  into M round trips.
- [x] Document the 2PC latency tradeoff and the bulk-load escape hatch for
  applications that can tolerate post-write placement registration.

## Phase 12.5: Schema Drift, DDL, and Type Round-Trip Hardening

- [ ] Fingerprint the user table column shape on the coordinator at trigger
  installation or descriptor registration time.
- [ ] Store or bind the column-shape fingerprint to the remote descriptor.
- [ ] Before INSERT dispatch, compare current coordinator shape to the
  descriptor fingerprint and fail closed on drift with a clear "schema drift"
  error and remediation hint.
- [ ] Add a fixture that `ALTER TABLE`s the coordinator only, attempts INSERT,
  and asserts the schema-drift error fires before remote dispatch.
- [ ] Add round-trip fixtures for non-trivial trigger payload types:
  - [ ] `numeric` / `decimal` precision;
  - [ ] `timestamptz` timezone and value preservation;
  - [ ] `json` and `jsonb` nested payloads;
  - [ ] `text` with embedded edge characters, documenting unsupported values
    if PostgreSQL/JSON cannot round-trip them;
  - [ ] domain-over-base-type validation;
  - [ ] nullable columns with SQL NULL;
  - [ ] NOT NULL violation paths;
  - [ ] DEFAULT-valued columns after PostgreSQL has materialized `NEW`.
- [ ] Document the v1 DDL ordering contract: pause writes, apply DDL to the
  coordinator, apply matching DDL to every remote, refresh descriptors, then
  resume writes.
- [ ] Decide whether a lightweight DDL window guard is needed to block writes
  while remote schemas are known to be inconsistent.

## Phase 12.6: Isolation, EvalPlanQual, and Negative DML Coverage

- [ ] Document the v1 distributed read isolation contract in ADR-068/ADR-069:
  distributed CustomScan virtual tuples do not get PostgreSQL's normal
  EvalPlanQual rerun semantics.
- [ ] Add fixtures for SERIALIZABLE / REPEATABLE READ / READ COMMITTED
  distributed PK SELECT behavior and pin the expected v1 outcome.
- [ ] Decide whether `ec_spire_custom_scan_recheck` should remain
  unconditional for v1 or re-run the primitive for a subset of read paths.
  - [ ] Exit criterion: the decision packet must include an isolation fixture
    that demonstrates the stale-row/EvalPlanQual behavior under at least
    SERIALIZABLE and states the accepted v1 contract in ADR-068 or ADR-069.
- [ ] Add negative classifier fixtures for unsupported PK predicate shapes:
  - [ ] `$1::numeric` outside int8 range;
  - [ ] `$1::int8 IS NULL` with stable SQLSTATE;
  - [ ] `WHERE id IN (...)`;
  - [ ] `WHERE id = $1 OR id = $2`;
  - [ ] any non-bigint equality accepted accidentally by coercion.
- [ ] Ensure unsupported DML shapes against a SPIRE-fronted table continue to
  fail closed rather than falling through to the empty coordinator heap.

## Phase 12.7: Multi-Instance Placement, Epoch, and Replica Readiness

- [ ] Add or extend an `ecaz`-owned local one-coordinator/two-remote setup and
  teardown command for repeated Stage E and readiness runs.
- [ ] Publish and inspect placement metadata that maps selected PIDs to remote
  nodes and local store IDs.
- [ ] Verify strict mode never mixes incompatible epochs across nodes.
- [ ] Verify degraded mode reports every skipped or stale remote node with node
  identity, count, and first skip category.
- [ ] Add remote-node multi-instance proof that boundary replicas carry the
  same global original-vector identity across leaves, stores, and remotes.
- [ ] Add boundary-replica manifest freshness fixtures using
  `ec_spire_remote_epoch_manifest_freshness()`.
- [ ] Add operator diagnostics for stale, missing, or unavailable boundary
  replica placements and their degraded-mode reporting.
- [ ] Preserve and periodically rerun the full Stage E fault/lifecycle matrix
  against the current CustomScan path while this hardening proceeds.

## Phase 12.8: Local Multi-Store and Multi-NVMe Readiness

- [ ] Preserve `(node_id, local_store_id)` as the scheduling and diagnostic
  unit.
- [ ] Prove local store lookup remains indexed or otherwise bounded for the
  configured maximum store count.
- [ ] Add a repeatable local multi-store read-overlap harness with per-store
  route, candidate, object-byte, read-batch, and delta-decode counters.
- [ ] If PostgreSQL backend constraints keep execution sequential, expose the
  limitation in diagnostics and document the exact future primitive needed to
  improve it.
- [ ] Confirm delta decode reuse remains covered under multi-store and remote
  candidate paths.

## Phase 12.9: Local Production Harness and Runbook

- [ ] Extend `ecaz` with setup, load, query, teardown, and benchmark commands
  for the local distributed SPIRE fixture when shell scripts become repeated
  operator workflows.
- [ ] Add or extend `ecaz bench spire-pipeline` for distributed recall,
  latency, and counter capture across local instances.
- [ ] Capture recall, latency p50/p95/p99, object bytes, route counts,
  candidate counts, heap rows, remote fanout, timeout/cancel counts,
  strict-failure counts, degraded-skip counts, placement contention, and typed
  tuple transport counters in packet-local artifacts.
- [ ] Publish local capacity targets for maximum remotes, maximum concurrent
  coordinator queries, maximum concurrent writers, maximum work per remote,
  maximum PIDs per node, and expected overload/degraded behavior.
- [ ] Include libpq security and operations in the runbook:
  `sslmode` preservation, raw-conninfo non-exposure, sanitized
  auth/certificate failures, credential-rotation deferral, audit-log deferral,
  `max_prepared_transactions`, and orphaned-prepared-xact recovery.
- [ ] Distinguish local functionality, local production-readiness smoke, and
  AWS/RDS product-scale evidence in the docs.
- [ ] Produce a final local production-readiness bundle from clean setup
  through distributed read/write, fault/degraded checks, multi-store checks,
  and harness artifact capture.

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
