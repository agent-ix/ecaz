# Task 30 Phase 11: SPIRE Distributed Production Parity

Status: in progress
Owner: coder1 / SPIRE distributed production track
Priority: 1 after Phase 9 and Phase 10 local architecture closeout

## Goal

Move SPIRE from local architecture parity to production-ready distributed
parity with the SPIRE paper shape: routed hierarchical index data, near-data
remote scoring, bounded distributed fanout, stable cross-node identity, and one
coordinator-visible ordered result stream.

Phase 11 is the work that must be solid before the deferred AWS/RDS-class scale
packet is worth running. It should not claim product performance. It should make
the distributed path correct, bounded, observable, and repeatably testable on
local multi-instance fixtures first.

## Scope

- Implement or explicitly block production remote query execution rather than
  relying on SQL-visible diagnostic-only libpq surfaces.
- Finish writer-side global vector identity so cross-node and boundary-replica
  dedupe is end-to-end safe.
- Finish origin-node remote heap resolution, then deliver remote rows through
  the ADR-067 `EcSpireDistributedScan` CustomScan node instead of the index AM
  materialization path.
- Extend the Stage B endpoint contract with the ADR-068 tuple-payload
  side-channel needed by CustomScan.
- Land the ADR-069 v1 distributed write contract: coordinator-routed
  INSERT/UPDATE/DELETE/PK-read, placement directory, embedding-UPDATE
  rejection, and bulk-load primitives only where required by that ADR.
- Add local multi-instance fixtures that exercise coordinator, remote nodes,
  epoch handling, failure/degradation behavior, and merged result ordering.
- Tighten multi-NVMe/local-store readiness with production diagnostics and local
  reproducible harnesses, while deferring AWS hardware measurement.
- Keep RaBitQ as the supported first quantized scoring path. PQ/PQFastScan
  remains out of scope unless a slice only improves unsupported/reserved
  wording.

## Non-Goals

- AWS/RDS-class product scale runs. Those remain deferred until Phase 11 proves
  the production path is internally ready.
- Billion-scale performance claims.
- PQ/PQFastScan implementation.
- A generic distributed SQL engine. Phase 11 is SPIRE-specific and should reuse
  PostgreSQL/libpq only where it serves the SPIRE coordinator path.
- Coordinator high availability and multi-coordinator consensus. Phase 11 may
  allow a coordinator to also serve local SPIRE partitions, but failover and
  coordinator election are deferred.
- Bulk-load tooling and CLI orchestration. ADR-069 keeps classification and
  batch-registration primitives in scope but schedules high-throughput bulk
  ingestion as a separate task.
- Cross-shard non-vector scatter-gather, DDL propagation, cross-shard
  embedding UPDATE moves, and multi-coordinator deployments. ADR-069 defers
  each to future ADRs.
- Credential rotation, audit-log schema, and a full TLS runbook. Phase 11 keeps
  the narrower libpq contract: preserve `sslmode` through
  `conninfo_secret_name`, keep raw conninfo hidden, and define sanitized
  strict/degraded auth or certificate-verification failure behavior.

## Entry Rules

- Phase 9 local graph architecture is complete:
  `plan/tasks/task30-phase9-spire-graph-architecture.md`.
- Phase 10 local execution architecture is complete:
  `plan/tasks/task30-phase10-spire-execution-performance.md`.
- The Phase 8 scale packet remains open and explicitly deferred until Phase 11
  exit criteria are met.
- Each implementation slice needs its own review packet and narrow validation.

## Phase 11.1: Paper-Parity Checklist and Production Gate

- [x] Create a packet-local SPIRE paper parity checklist mapping each paper
  mechanism to current implementation status, evidence, and required follow-up.
- [x] Use a stable traceability artifact format:
  `paper section/mechanism -> implementation status -> evidence packet ->
  blocker/deferral -> owner phase`, so reviewers can audit parity without
  reconstructing the history from chat or task prose.
- [x] Split "diagnostic-only" surfaces from production surfaces in docs and
  code comments so operators cannot mistake diagnostic libpq helpers for the
  production distributed AM path.
- [x] Define the Phase 11 production-readiness gate that must pass before AWS
  scale is scheduled.
- [x] Record explicit deferrals for paper features we intentionally skip in v1,
  including PQ/PQFastScan and any non-RaBitQ scoring path.

Acceptance artifact:
`plan/design/spire-phase11-paper-parity-production-gate.md`.

## Phase 11.2: Writer-Side Global Vector Identity

- [x] Add an explicit assignment-builder identity source hook so writer code can
  allocate either default local `0x01` IDs or caller-provided global `0x02`
  payload IDs without advancing the local sequence for global rows.
- [x] Extend the SQL-visible vector identity contract with writer allocation,
  stable source identity, Leaf V2 base-storage blocker, and row-encoded delta
  storage status rows.
- [x] Emit durable global `0x02` `SpireVecId` values from the writer/build path
  when a stable source identity is available.
- [x] Define the stable source-identity input contract for build/insert paths;
  heap TID alone is not a cross-node stable identity. The Phase 11 writer
  contract is a fixed 16-byte source payload, documented in
  `plan/design/spire-stable-source-identity-contract.md`.
- [x] Choose and implement the first live source-identity provider, such as an
  explicit identity column or expression contract, then plumb it into build and
  insert assignment inputs. ADR-063 selects the v1 provider as
  `source_identity = 'include'` with one included UUID or exact-16-byte `bytea`
  column.
- [x] Replace or extend Leaf V2 base-object storage so global `0x02` IDs are not
  rejected by the local-only fixed-width vec-id column format. Leaf V2 now
  supports per-object `GlobalBytes` columns when every row has the same global
  ID byte length; see `plan/design/spire-leaf-v2-vector-id-layout.md`.
- [x] Preserve compatibility with existing node-local `0x01` IDs, scoped by
  origin `node_id`, without silently mixing namespaces.
- [ ] Ensure boundary replicas carry the same global original-vector identity
  across leaves, stores, and remote nodes.
  - [x] Packet `30775` adds
    `ec_spire_index_boundary_replica_identity_snapshot()`, an index-level
    diagnostic that groups primary and boundary-replica assignments by
    `vec_id`, reports global vs node-local scope, leaf/store/node span, and
    readiness. The PG18 fixture covers global source IDs across boundary
    replica leaves and a multi-store placement directory. Remote-node
    multi-instance proof remains open under Stage E.
- [x] Add migration/rewrite or compatibility diagnostics for indexes that still
  contain only node-local IDs.
- [x] Add tests proving unrelated node-local IDs do not dedupe, while replicated
  global IDs do dedupe.

## Phase 11.3: Remote Search Endpoint Contract

- [ ] Promote the remote search SQL contract from diagnostic proof surface to a
  production endpoint, or add the production endpoint beside the diagnostic
  one.
- [ ] The remote endpoint must accept requested epoch, selected PIDs, query
  vector, top-k/candidate budget, consistency mode, and scoring/rerank options.
- [ ] Remote nodes must score near their partition objects and return compact
  candidate rows with served epoch, node identity, vector identity, row locator,
  score, flags, and diagnostics.
- [ ] For the ADR-067 CustomScan path, remote nodes must also return requested
  tuple columns through the ADR-068 side-channel keyed by `(node_id, vec_id)`.
  The existing 18-column envelope remains stable.
- [ ] Bind remote candidate rows to the served quantizer/index identity:
  RaBitQ profile, code length, training-stat fingerprint, index build format,
  and served epoch must be compatible before coordinator merge accepts scores.
  - [x] First production receive guard: compact-candidate receive requests
    carry expected `remote_index_identity`, and returned candidate-row
    `profile_fingerprint` bytes must match before the batch can enter
    production merge state.
- [ ] Add protocol, extension-version, and opclass-binary version negotiation.
  Strict mode must reject incompatible remotes; degraded mode must report the
  skipped node and the exact mismatch.
- [ ] Remote nodes must reject stale/unavailable epochs explicitly in strict
  mode and surface degraded behavior explicitly when allowed.
  - [x] First production receive strict stale-epoch guard: stale served epochs
    are categorized as `served_epoch_mismatch` instead of generic candidate
    batch validation failure.
- [ ] Add PG18 tests for nonempty remote candidates, stale epoch rejection, and
  empty/top-k-zero behavior.
  - [x] PG18 production receive coverage now includes nonempty loopback
    candidates, top-k-zero ready-empty behavior, and stale served-epoch
    rejection.
- [ ] Add local version-skew tests with two remotes at different advertised
  contract versions.

## Phase 11.4: Production Libpq Coordinator Executor

- [ ] Replace the serial diagnostic executor shape with a production coordinator
  executor that overlaps remote work.
- [ ] Use libpq pipeline mode or async send/receive for batched remote calls.
- [ ] Bound remote fanout per query and expose truncation/degradation
  diagnostics when the bound applies.
  - [x] First per-query bound: session caps gate ready libpq dispatch rows and
    report `remote_executor_overload` before secret lookup.
- [ ] Cache validated remote index identity and avoid per-query repeated
  `to_regclass` lookups where safe.
- [ ] Add connection reuse or a clear bounded connection lifecycle with connect
  and statement timeouts.
  - [x] First timeout surface: session connect/statement timeout GUCs are
    applied by the diagnostic executor connection helper when nonzero.
- [ ] Define the narrow Phase 11 libpq security contract: preserve `sslmode`
  through `conninfo_secret_name` resolution, do not expose raw conninfo, reject
  libpq authentication or certificate-verification failures in strict mode with
  sanitized errors, and report skipped remotes in degraded mode.
- [ ] Add resource governance across queries, not only per query: coordinator
  global connection/work limits, per-remote-node concurrency caps, overload
  shedding behavior, and diagnostics for backpressure decisions.
  - [x] Production async transport and compact-candidate receive acquire the
    global/per-node advisory governance permit before conninfo parsing or
    socket open and report `remote_executor_overload` on saturation.
- [ ] Propagate local query cancellation to outstanding remote work.
  - [x] First adapter primitive: production receive and transport use a
    `tokio-postgres` cancel token and map local cancellation to global
    dispatch cleanup.
  - [x] Before final executor integration, pin and implement the PostgreSQL
    backend interrupt bridge from actual local query cancel / statement timeout
    into the adapter cancel token; test-only triggers are not production
    evidence.
    - [x] First PostgreSQL interrupt bridge: production transport and compact
      candidate receive now poll backend `InterruptPending` /
      `QueryCancelPending` while remote work is in flight and map the signal to
      the adapter cancel token as `local_query_cancelled`; timer-triggered
      cancellation remains test-only.
    - [x] Local statement-timeout provenance: when PostgreSQL has query cancel
      pending and `get_timeout_indicator(STATEMENT_TIMEOUT, false)` is set, the
      adapter cancels remote work and reports `local_statement_timeout` without
      resetting PostgreSQL's timeout indicator.
- [ ] Define fail-closed strict mode and explicit degraded mode behavior for
  partial remote failures.
  - [x] First production degraded-state slice: production executor state can
    mark transport, secret-resolution, and compact-candidate receive failures
    as `degraded_skipped`, preserve the first skip category, and merge only
    ready candidate batches in degraded mode while strict mode remains
    fail-closed.
  - [ ] Before CustomScan integration, pin executor-boundary consistency-mode
    policy: source of strict/degraded mode, single per-query threading into
    executor state, and final diagnostics or warning that name skipped nodes or
    at least count plus first skip category.
    - [x] First executor-boundary source-of-truth slice: session GUC
      `ec_spire.remote_search_consistency_mode` feeds a production executor
      session summary without a per-call free-form consistency string.
    - [x] Production executor state rows now carry `consistency_mode_source`
      and `consistency_mode`; the SQL full-state surface remains at its pgrx
      row-width limit, so SQL mode attribution is still exposed through the
      compact session summary.
    - [x] Active-epoch consistency-policy mismatch is visible as a named
      `consistency_mode_mismatch` diagnostic category before C5 depends on the
      dispatch-planning surface.
  - [x] Add a strict/degraded fault matrix table covering connect, secret,
    statement timeout, backend termination, query cancellation, validation,
    identity, version, stale epoch, and heap-resolution failures.
    - [x] `ec_spire_remote_search_production_fault_matrix()` now exposes the
      dry production policy rows, including `connect_failed`,
      `requires_conninfo_secret_resolution`, remote/local statement timeout,
      remote/local query cancellation, validation, identity, version,
      stale/served epoch, and reserved Stage D heap-resolution categories.
    - [x] Include `consistency_mode_mismatch` alongside the transport and
      receive categories when the matrix lands.
    - [x] Reviewer P3 follow-up: final executor integration must consume this
      matrix, or a Rust-side equivalent generated from the same category list,
      as the executor-boundary source of truth. Reserved Stage D
      heap-resolution rows are category names only until the heap executor
      emits them.
- [x] Add local multi-instance tests proving tail latency is not serialized
  across ready remotes under an instrumented slow-remote fixture.
  - [x] First multi-instance timing proof: packet `30752` adds a PG18 harness
    with one coordinator plus two separate remote PostgreSQL clusters, resolves
    conninfo through secret names, and proves the fast ready remote completes
    before the deliberately slow remote under the production async transport
    adapter. This is transport-overlap evidence only; the broader Stage E
    epoch/lifecycle/fault matrix remains open.

## Phase 11.5: CustomScan Distributed Read and v1 Write Contract

Pivot note, 2026-05-10: ADR-067 / ADR-068 / ADR-069 supersede the
ADR-064 / ADR-065 / ADR-066 index-AM materialization path. The Stage C
executor, origin-node heap visibility resolution, diagnostics, and Stage E
fault/lifecycle matrices remain reusable. The final distributed read path now
returns tuples directly from `EcSpireDistributedScan`; the index AM remains the
local-only path.

- [x] Extend the Stage B production endpoint with the ADR-068 tuple-payload
  side-channel while preserving the existing 18-column candidate envelope,
  fingerprint, identity, version, and status fields.
  - [x] Let the coordinator declare required projected columns at request build
    time.
  - [x] Key tuple payloads by `(node_id, vec_id)` so the executor can attach
    payloads to heap-visible candidates without changing the candidate
    envelope shape.
  - [x] Add focused PG18 coverage for tuple-payload responses. Existing Stage B
    endpoint and identity tests must stay valid.
- [ ] Register the `EcSpireDistributedScan` CustomScan provider and planner
  path.
  - [ ] Hook planner path generation for tables with an `ec_spire` index and
    active remote placements when the query shape is
    `ORDER BY <vector-distance-op> LIMIT k`.
  - [ ] Cost the CustomPath so remote-placement scans choose CustomScan over
    the local-only AM path, and declare path keys so ordered output does not
    require an extra sort.
  - [ ] Implement Begin/Exec/End callbacks that invoke the existing
    `SpireRemoteFanoutExecutor` state machine and return tuples directly via
    the CustomScan tuple interface.
    - [x] Packet `30810` wires the callbacks through the production executor
      result stream and PostgreSQL scan/projection machinery.
    - [x] Packet `30814` adds remote-origin tuple-payload slot delivery.
    - [x] Packet `30815` extends the tuple-payload endpoint with heap
      coordinates needed by the production executor decode path and proves a
      coordinator CustomScan can return the projected remote tuple.
  - [x] Preserve local-only `ec_spire` index AM behavior unchanged.
    - [x] Packet `30821` adds PG18 plan coverage proving a local-only
      `ORDER BY <#> ... LIMIT 1` query does not use
      `Custom Scan (EcSpireDistributedScan)` and still plans through the
      `ec_spire` index AM path.
  - [x] Add `EXPLAIN` coverage showing `Custom Scan (EcSpireDistributedScan)`.
- [x] Add the read-path end-to-end PG18 fixture: coordinator with remote-only
  placements, remote shard rows, SPIRE index, and
  `SELECT cols FROM tbl ORDER BY embedding <-> $1 LIMIT k` returning the
  correct remote rows through CustomScan.
  - [x] Packet `30815` adds a PG18 loopback-remote fixture with coordinator
    placements rewritten to a remote node, an active remote descriptor, and
    `SELECT id, title ... ORDER BY embedding <#> ... LIMIT 1` returning the
    remote shard row through `EcSpireDistributedScan`.
  - [x] The fixture must not call
    `ec_spire_register_remote_row_materialization(...)`.
  - [x] Broaden the fixture to the final multi-instance distributed read lane
    before calling Stage D reads feature-complete.
    - [x] Packet `30820` adds
      `scripts/run_spire_multicluster_customscan_read_pg18.sh`, which starts
      separate coordinator and remote PG18 clusters, registers the real remote
      endpoint fingerprint, asserts `Custom Scan (EcSpireDistributedScan)`,
      and verifies the returned projected tuple is the remote shard row.
- [ ] Land the ADR-069 placement directory and coordinator-routed INSERT.
  - [x] Add `ec_spire_placement(index_oid, pk_value, node_id, centroid_id,
    served_epoch, source_identity)` with the required primary key and identity
    index.
    - [x] Packet `30817` adds the catalog table to bootstrap and upgrade SQL
      and wires it into remote catalog orphan/index cleanup diagnostics.
    - [x] Packet `30824` relaxes the placement directory `node_id` constraint
      to `node_id >= 0`, preserving ADR-068 local node `0` for
      coordinator-local shard rows.
  - [x] Add `ec_spire_classify_centroid(embedding, index_oid)`.
    - [x] Packet `30818` adds the coordinator-side classifier helper using
      active routing centroids and returns `(node_id, centroid_id, epoch)`.
    - [x] Packet `30826` pins `centroid_id` as the active-epoch routing leaf
      pid, adds recursive classifier coverage, and bounds routing traversal by
      the root routing level.
  - [x] Add `ec_spire_register_placement_batch(index_oid,
    ec_spire_placement_entry[])` for bulk-load post-write registration.
    - [x] Packet `30819` adds the composite entry type and SQL batch
      registration primitive.
    - [x] Packet `30825` hardens the primitive with explicit NULL-entry
      rejection, empty-batch/duplicate/constraint regression tests, and ADR
      notes for transaction and v1 entry-shape semantics.
  - [ ] Route coordinator INSERT by classifying the embedding, forwarding the
    row to the target remote, and atomically updating the placement directory
    with remote `PREPARE TRANSACTION` / local commit / remote commit.
    - [x] Packet `30829` adds the side-effect-free
      `ec_spire_plan_coordinator_insert_dispatch(...)` readiness primitive for
      the classified remote target. It reuses the Stage C
      `ec_spire_remote_node_descriptor` and external conninfo-secret contract,
      checks the remote descriptor's served-epoch window, and reports the
      libpq/2PC dispatch action without opening a connection or mutating
      placement state.
  - [ ] PG18 fixture: INSERT at the coordinator, verify the row lands on the
    target remote, verify placement-directory state, and verify CustomScan
    SELECT returns the row.
- [ ] Land coordinator-routed non-embedding UPDATE, DELETE, and PK-keyed SELECT.
  - [ ] UPDATE non-embedding columns by placement-directory lookup and remote
    forwarding.
  - [ ] DELETE through remote prepared DELETE plus placement-directory delete.
  - [ ] PK-keyed SELECT through placement-directory lookup and remote SELECT.
  - [ ] Reject embedding-changing UPDATE with the exact ADR-069 error and hint.
  - [ ] Add PG18 fixtures per operation.
- [ ] Migrate Stage E fault and lifecycle fixtures onto the CustomScan path.
  - [ ] Preserve the existing 11 fault-matrix and 6 lifecycle-matrix cases
    where they already assert executor state and diagnostic SQL surfaces.
  - [ ] Replace only the subset that exercised the AM cursor or
    materialization-specific blocker.
  - [ ] Run and attach packet-local logs for the full Stage E matrix against
    the CustomScan path.
- [ ] Cleanup after CustomScan read and v1 writes are feature-complete.
  - [ ] Remove or migrate away from the vestigial
    `ec_spire_remote_row_materialization` table and
    `ec_spire_register_remote_row_materialization` function in the next
    extension upgrade script.
  - [ ] Remove dead AM cursor references to
    `requires_remote_row_materialization`; the local-only AM path must keep its
    classifier logic but no longer reference the superseded materialization
    catalog.
  - [ ] Keep the catalog/register function until this cleanup packet rather
    than deleting it in the Step 0 docs rewrite, because the repository still
    has in-flight Shape-A code and an untracked `30802` mirror-sync packet in
    the local worktree.

## Phase 11.6: Multi-Instance Epoch and Placement Readiness

- [ ] Add a local multi-instance fixture with at least one coordinator and two
  remote PostgreSQL nodes.
- [ ] Publish and inspect placement metadata that maps selected PIDs to remote
  nodes and local store IDs.
- [ ] Verify strict mode does not mix incompatible epochs across nodes.
- [ ] Verify degraded mode reports every skipped or stale remote node.
- [ ] Define and test online lifecycle behavior when a remote index is dropped,
  reindexed, or created concurrently while fanout is planned or in flight.
  - [x] Packet `30772` adds
    `ec_spire_remote_search_stage_e_lifecycle_matrix()`, covering DROP INDEX,
    REINDEX INDEX CONCURRENTLY, and CREATE INDEX CONCURRENTLY before fanout,
    during in-flight receive, and before descriptor registration. Packet-local
    DDL fixture evidence remains open.
- [ ] Add replica manifest freshness diagnostics: identify which remote node is
  serving each boundary replica, whether its manifest is current, and how
  degraded mode reports stale or missing replica placement.
  - [x] Packet `30774` adds
    `ec_spire_remote_epoch_manifest_freshness()`, a per-remote-node assertion
    surface that composes the current manifest plan, persisted manifest entry,
    catalog summary, and publication action into `freshness_status` and
    `next_action`. This covers node-level current-vs-persisted freshness for
    Stage E evidence. Per-boundary-replica fixture evidence remains open.
- [ ] Add a fault matrix for connection reset mid-batch, remote backend
  termination, remote statement timeout, local statement timeout/cancel,
  simulated network partition, version skew, and remote OOM. Each case must
  state strict and degraded expected behavior.
  - [x] Packet `30770` adds
    `ec_spire_remote_search_stage_e_fault_matrix()`, a fixture-facing matrix
    that names each Stage E local multi-instance fault case, maps it to the
    existing production failure category, and states strict/degraded action,
    status, next executor step, expected counter delta, and required evidence.
    It also captured the now-superseded packet `30765` row-materialization
    mapping matrix follow-up; CustomScan migration must replace any
    materialization-specific assertions with tuple-payload and executor-state
    assertions.
  - [x] Packet `30773` documents the Stage E per-case artifact convention and
    network-partition simulation mechanism before fixture implementation.
- [x] Add operator diagnostics that show remote node readiness, served epoch,
  remote fanout, candidate batches, heap resolution state, and merge status in
  one packet-friendly path.
  - [x] Packet `30771` adds
    `ec_spire_remote_search_operator_diagnostics()`, a single-row rollup over
    remote capability readiness, remote last-served epoch range, production
    fanout/candidate batches, heap resolution, merge/result source, and AM
    delivery blocker state.
  - [x] Packet `30773` names this rollup as the preferred Stage E fixture
    assertion surface in the production coordinator executor design.

## Phase 11.7: Local Multi-NVMe and Store Execution Hardening

- [ ] Keep `(node_id, local_store_id)` as the scheduling and diagnostic unit.
- [ ] Confirm local store lookup remains indexed or otherwise bounded for the
  configured maximum store count.
- [ ] Add a local repeatable harness for multi-store read overlap and per-store
  route/candidate/object-byte counters, without requiring AWS.
- [ ] If PostgreSQL backend constraints keep execution sequential, make the
  limitation explicit in diagnostics and identify the exact future primitive
  needed to improve it.
- [ ] Ensure delta decode reuse remains covered under multi-store and remote
  candidate paths.

## Phase 11.8: Production Harness and Operator Runbooks

- [ ] Extend `ecaz` with local multi-instance setup, load, query, and teardown
  commands when an operator workflow repeats across packets.
- [ ] Extend `ecaz bench spire-pipeline` or add a sibling command for
  distributed recall/latency/counter capture across local instances.
- [ ] Capture recall, latency p50/p95/p99, object bytes, route counts,
  candidate counts, heap rows, remote fanout, timeout/cancel counts, and
  degraded/strict failure counts in packet-local artifacts.
- [ ] Add a production-readiness runbook that states exactly when AWS scale
  can be scheduled.
- [ ] Include the Phase 11 libpq security boundary in the runbook: `sslmode`
  preservation, raw-conninfo non-exposure, sanitized strict/degraded
  auth/certificate failure behavior, and explicit deferral of credential
  rotation plus audit-log schema.
- [ ] Publish local capacity targets for the pre-AWS gate: maximum remotes,
  maximum concurrent coordinator queries, maximum concurrent work per remote,
  maximum PIDs per node, and expected overload/degraded-mode behavior. These
  numbers are local production-readiness targets, not AWS product claims.
- [ ] Add docs that distinguish local functionality, local production-readiness
  smoke, and AWS/RDS product-scale evidence.

## Phase 11.9: AWS Scale Entry Gate

- [ ] Do not schedule AWS/RDS-class scale until Phase 11.1-11.8 have either
  passed or have explicit accepted deferrals.
- [ ] Before AWS, run a final local production-readiness bundle from clean
  setup through multi-instance query, failure/degradation checks, and harness
  artifact capture.
- [ ] Prepare the AWS packet manifest only after the local bundle is reviewed.

## Production Landing Sequence

This section is the broad quality plan for finishing Phase 11. Each stage must
land as narrow reviewed packets with packet-local evidence. A later stage cannot
claim production readiness by relying on a diagnostic-only surface from an
earlier stage.

### Production Readiness Ladder After Packet 30747

The remaining work should land in this order. The plan is intentionally broad,
but each item still needs a narrow code checkpoint and a packet-local review
request before the next item depends on it.

1. **C4/C5 executor-boundary preflight.** Finish the consistency and
   cancellation gate before CustomScan depends on it: expose active-epoch
   consistency-policy mismatch as a named row-returning category, document that
   the session GUC is the stable contract, and keep future query-level options
   as statement-local overrides of that GUC rather than replacements.
2. **C2 backend interrupt bridge.** Connect actual PostgreSQL local cancel and
   local statement timeout to the production adapter cancel token. Evidence
   must prove cancelled work releases global and per-node governance slots and
   that local cancel, local statement timeout, remote statement timeout, remote
   query cancellation, and remote backend termination stay separate categories.
3. **C3 production identity-cache handoff.** Move the executor-local endpoint
   identity cache from diagnostic proof into the production handoff used by
   compact receive and Stage D remote heap resolution. Evidence must show one
   validated identity decision can be reused without caching raw conninfo or
   silently accepting a live fingerprint mismatch.
4. **Stage B tuple-payload extension.** Keep the existing 18-column candidate
   envelope and add the ADR-068 side-channel that returns requested tuple
   columns for CustomScan delivery.
5. **Stage D CustomScan read path.** Register `EcSpireDistributedScan`, add
   planner path selection for `ORDER BY <vector-distance-op> LIMIT k` on
   indexes with active remote placements, invoke the existing production
   executor from CustomScan `Exec`, and return ordered tuples directly. No
   mirror sync, no materialization catalog, and no register calls are allowed on
   this path.
6. **Stage D read-path end-to-end fixture.** Prove a coordinator with
   remote-only placements returns remote rows through CustomScan with the
   requested columns.
7. **ADR-069 v1 write contract.** Add the placement directory,
   coordinator-routed INSERT with remote 2PC, coordinator-routed
   non-embedding UPDATE / DELETE / PK-keyed SELECT, and embedding-UPDATE
   rejection. Bulk-load tooling stays separate.
8. **Stage E CustomScan matrix migration.** Re-run the 11 fault-matrix and 6
   lifecycle-matrix fixtures against the CustomScan path, replacing only
   AM-cursor-specific assertions.
9. **Cleanup.** Remove the superseded materialization catalog/register
   function and dead AM materialization blocker references after the CustomScan
   read path and ADR-069 writes are feature-complete.
10. **Stage E local multi-instance matrix.** Build the one-coordinator /
   two-remote local fixture and run the strict/degraded matrix for epoch,
   version, fingerprint, connection, backend termination, timeout, cancel,
   network partition, OOM, and remote index lifecycle faults.
11. **Stage F/G local readiness bundle.** Add the repeatable multi-store and
   distributed harness commands, publish numeric local capacity targets, and
   capture recall, latency, fanout, heap, route, candidate, byte, timeout,
   cancel, strict-failure, and degraded-skip counters. AWS/RDS-class scale
   remains blocked until this bundle is reviewed.

### Stage A: Writer Identity Provider

Goal: make real build/insert writers capable of emitting stable global IDs.

- [ ] Accept ADR-063 or revise it based on reviewer feedback before wide AM
  callback changes.
- [x] Enable the `source_identity = 'include'` reloption and the AM INCLUDE
  capability behind strict validation:
  one vector key column, zero or one included identity column, no partial index,
  no expression identity in v1.
- [x] Canonicalize included `uuid` and exact-16-byte `bytea` values to
  `StableFixedGlobalPayload([u8; 16])`; reject NULL, unsupported types, and
  malformed bytea values.
- [x] Thread source identity through populated build, empty-index insert
  bootstrap, live insert deltas, boundary replicas, and scheduled replacement
  paths without advancing local ID sequence for global rows.
- [x] Add diagnostics for three index classes: local-only, global-capable but
  not yet remote-published, and global-writer active.
- [x] Verification: PG18 DDL tests for accepted/rejected index shapes; build
  and insert tests proving global IDs land in Leaf V2/delta rows; tests proving
  replicas share one global ID and local-only indexes remain node-scoped.
- [x] Verification landed for accepted/rejected index shapes plus populated
  build, empty-index bootstrap, live insert delta global IDs, scheduled
  replacement preservation, boundary-row global ID reuse, and node-local
  namespace scoping:
  `cargo test source_identity --lib`,
  `cargo test remote_candidate_batch_validation --lib`,
  `cargo test global_vec_ids --lib`,
  `cargo test local_vec_ids_by_node --lib`,
  `cargo pgrx test pg18 test_ec_spire_srcid`, and
  `cargo pgrx test pg18 test_ec_spire_include_requires_srcid_reloption`.

### Stage B: Production Remote Endpoint

Goal: turn remote scoring from diagnostic proof into a production candidate
endpoint.

- [ ] Define the endpoint request/response contract for selected PIDs, requested
  epoch, query vector, candidate budget, strict/degraded mode, scoring profile,
  and rerank settings.
  - [x] First contract slice: add
    `ec_spire_remote_search_endpoint_contract()` so the SQL-visible endpoint
    reports the current request/response shape, RaBitQ-only v1 protocol, delta
    PID semantics, and the remaining non-ready scoring/fingerprint/opclass
    binding blockers without claiming production readiness.
  - [x] Document the direct-call diagnostic posture in the endpoint contract:
    `ec_spire_remote_search` may expose non-ready rows for operators, while
    production libpq receive accepts `endpoint_status = ready` only.
  - [x] Add `ec_spire_remote_search_endpoint_identity()` so a remote-serving
    SPIRE index exposes protocol version, extension version, opclass identity,
    storage/assignment payload format, RaBitQ profile, scoring profile, and a
    deterministic profile fingerprint; non-RaBitQ endpoint identities are
    blocked until rebuilt with `storage_format = 'rabitq'`.
- [ ] Return compact candidate rows with served epoch, node identity, vector ID,
  row locator, score, assignment flags, quantizer/index fingerprint,
  protocol/extension/opclass version, and packet-friendly diagnostics.
  - [x] Extend the `ec_spire_remote_search` row envelope with protocol version,
    extension version, opclass identity, storage/assignment payload format,
    quantizer profile, scoring profile, profile fingerprint, and endpoint
    status columns. The coordinator still needs a follow-up production gate
    before treating non-ready endpoint rows as mergeable.
  - [x] Document the v1 profile fingerprint input order, NUL-separated FNV-1a
    encoding, active-epoch semantics, and future training-stat extension rule
    in `plan/design/spire-remote-node-model.md`.
- [ ] Bind RaBitQ profile, code length, training-stat fingerprint, storage
  format, served epoch, extension version, and opclass identity before merge.
  - [x] Libpq candidate decode now validates endpoint protocol/version and
    rejects non-ready endpoint identity rows before candidates can enter the
    merge path; loopback executor coverage uses a RaBitQ remote-serving index.
  - [x] Strict fail-closed PG18 loopback coverage proves a non-RaBitQ remote
    endpoint is rejected with
    `endpoint_status requires_rabitq_storage_format is not ready` before
    remote candidates can enter the merge path.
  - [x] Libpq dispatch now preflights `ec_spire_remote_search_endpoint_identity`
    before both compact candidate receive and origin-node remote heap
    candidate receive, so empty batches and heap rows cannot bypass the ready
    endpoint identity gate.
  - [x] Libpq dispatch now compares the descriptor `remote_index_identity`
    against the live endpoint `profile_fingerprint` bytes before receive, so a
    ready remote index with the wrong advertised identity cannot enter merge.
- [ ] Reject stale or incompatible remotes in strict mode; report exact skip
  reasons in degraded mode.
  - [x] Add per-node libpq receive-attempt diagnostics that preserve the
    production strict `fail_closed` action while reporting the degraded
    `skip_node` action and exact endpoint mismatch reason for non-ready
    remotes.
  - [x] PG18 coordinator-result coverage proves the remote heap path rejects a
    non-RaBitQ endpoint before final row delivery.
  - [x] PG18 strict-mode coverage proves descriptor/endpoint fingerprint
    mismatch reports `endpoint_identity_mismatch` and fails closed before
    compact candidate merge.
  - [x] Search readiness and libpq dispatch now import
    `ec_spire_remote_node_capability_plan` status, so stale epoch,
    retention-gap, and extension-version blockers stop dispatch before
    pipeline mode, conninfo secret lookup, receive validation, or merge.
  - [x] PG18 strict/degraded coverage proves stale epoch and extension-version
    skew report exact `remote_epoch_window` / `remote_extension_version`
    blockers with strict `fail_closed` and degraded `skip_node` receive-attempt
    actions.
- [ ] Verification: PG18 loopback tests for nonempty candidates, empty/top-k-zero
  behavior, stale epoch rejection, fingerprint mismatch, version skew, and
  malformed candidate rejection.
  - [x] Stale epoch and extension-version skew are covered on the libpq search
    path before dispatch, including target readiness, execution, bind, secret,
    work, executor readiness, receive attempts, coordinator gate, and heap
    resolution summaries.

### Stage C: Production Libpq Coordinator

Goal: execute remote fanout with bounded concurrent or pipelined work.

Stage C status note, 2026-05-10: packets 30724 through 30736 have made the
C0/C1 executor layer materially composable. Production state now covers dry
admission, per-query budget/governance gates, overlapped transport probes,
per-node transport and compact-receive isolation, remote-side regclass
resolution, executor-owned receive request state, state-owned compact receive
execution, cancel-clears-batch invariants, strict compact merge preconditions,
and a routing-only selected-leaf PID handoff for the future CustomScan
integration.
This is not a production-ready distributed scan claim. The remaining blockers
are C2 cancellation propagation to in-flight remote work, C3/C4 production use
and strict/degraded normalization at the CustomScan executor boundary, C5
CustomScan read integration, Stage D tuple delivery, and the local
multi-instance fault/readiness bundle.

Stage C update, packet 30753: the production async transport and compact
candidate receive adapters now enter the same global/per-node advisory
governance surface as the blocking diagnostic executor. Saturated governance
slots produce the first-class `remote_executor_overload` production category
before conninfo parsing or socket open, and local cancellation releases both
global and per-node permits on the tested production paths.

- [x] Define the production coordinator executor state, landing sequence,
  cancellation contract, counter set, and validation gates in
  `plan/design/spire-production-coordinator-executor.md`.
- [ ] Implement production executor state separate from diagnostic SQL
  functions; keep raw conninfo hidden behind `conninfo_secret_name`.
  - [x] Add `SpireRemoteFanoutExecutor` / `SpireRemoteDispatch` state structs
    that can be built from the existing request, target, descriptor, budget,
    and governance planning data without opening sockets.
  - [x] Expose production-state summaries without invoking live diagnostic
    SQL helpers or opening extra sockets.
  - [x] Add dry-state verification for admitted and pre-dispatch-blocked
    production dispatches; PG18 coverage proves the SQL summary does not
    resolve conninfo secrets, open sockets, or query endpoint identity.
- [ ] Use libpq async or pipeline mode for overlapping remote work.
  - [x] Pin the C1 connection lifetime contract to per-query connect /
    per-dispatch close, with pooling deferred to a measured optimization after
    cancellation and strict/degraded failure semantics are stable.
  - [x] Add a narrow Tokio/tokio-postgres transport adapter boundary for
    production fanout probes, separate from blocking `postgres::Client`.
  - [x] Prove at least two ready remotes can make progress independently under
    an instrumented slow-remote fixture.
  - [x] Normalize transport probe parse/connect/query failures into per-node
    result rows so one failed remote cannot abort the whole fanout batch.
  - [x] Wire transport result rows into the production executor state machine
    with explicit pending, ready, and failed transport counters.
  - [x] Add a test-facing async compact-candidate receive adapter that uses
    `tokio-postgres`, decodes existing `ec_spire_remote_search(...)` rows, and
    validates the candidate batch contract.
  - [x] Add multi-node receive isolation coverage so a ready remote can return
    candidates while failed remotes preserve per-row failure categories.
  - [x] Wire compact-candidate receive results into production executor state
    with explicit pending, ready, failed, and candidate-row counters.
  - [x] Document the production stage-extension pattern and the
    `CandidateReceiveReady` handoff contract into Stage D heap resolution.
  - [x] Store ready compact-candidate batches inside production executor state
    and merge only `CandidateReceiveReady` batches for the Stage D handoff.
  - [x] Resolve remote index regclass on the remote connection in the
    production candidate-receive adapter instead of requiring coordinator-local
    remote OIDs.
  - [x] Make production executor dispatch state retain the selected PIDs,
    `conninfo_secret_name`, and remote index regclass needed to build compact
    candidate receive requests from `TransportReady` state without re-reading
    diagnostic rows or maintaining parallel AM scan bookkeeping.
  - [x] Wire the async adapter into compact candidate receive production state:
    build requests from `TransportReady` dispatches, apply adapter results back
    into the executor, and preserve per-dispatch failure isolation.
  - [x] Add a routing-only AM scan precursor that extracts selected leaf PIDs
    from the scan plan without reading remote leaf payload objects locally.
  - [x] Wire compact candidate receive production state into an AM-scan
    handoff summary: packet `30754` proves scan-derived selected PIDs can feed
    live production compact receive and validated compact-candidate merge while
    final row readiness remains `requires_remote_heap_resolution`.
  - [ ] Move the handoff from the summary/proof surface into the final
    `amrescan`/`amgettuple` execution path after Stage D can resolve remote
    heap rows.
- [ ] Add per-query fanout caps, global coordinator work limits, per-remote
  concurrency caps, connect/statement timeouts, cancellation propagation, and
  overload-shedding diagnostics.
  - [x] Define the first Stage C budget contract in
    `plan/design/spire-libpq-executor-budget.md`: session caps use `0` as
    unlimited, dispatch rows are the admission unit, over-budget rows report
    `remote_executor_overload`, and overload blocks before secret lookup or
    socket open.
  - [x] Add session GUCs for per-query remote node, total PID, per-node PID,
    connect-timeout, and statement-timeout limits.
  - [x] Enforce per-query dispatch admission before secret lookup and expose
    `ec_spire_remote_search_libpq_executor_budget_summary(...)` with admitted
    and budget-blocked dispatch/PID counts plus the active limits.
  - [x] Apply nonzero connect/statement timeout session settings in the
    diagnostic executor connection helper while keeping raw conninfo hidden.
  - [x] Add global cross-query coordinator work limits and per-remote-node
    concurrency caps.
    - [x] First governance surface: session GUCs cap concurrent libpq
      dispatches globally and per remote node using nonblocking PostgreSQL
      advisory locks; saturated slots report `remote_executor_overload` with
      `remote_executor_governance` before secret lookup or socket open.
    - [x] Document the advisory-lock namespace in
      `plan/design/spire-libpq-executor-budget.md` and ADR-058 so operator
      scripts and future extension features avoid the reserved governance
      class ranges.
    - [x] Add PG18 coverage that saturating one per-node governance slot does
      not block a second node with its own per-node slot.
    - [x] Packet `30753` extends the governance surface into production async
      transport and compact-candidate receive, with PG18 wrapper evidence that
      saturated global slots fail before remote connection parsing.
  - [ ] Propagate PostgreSQL cancellation into in-flight remote work.
    - [x] On local cancel, stop accepting new remote work, cancel or close all
      in-flight remote libpq work, release advisory governance slots, and
      report cancellation counters without raw remote error text.
      - [x] Packet `30753` proves local cancellation releases the production
        adapter's global and per-node governance slots for both transport and
        compact-candidate receive.
      - [x] Packet `30754` routes the AM scan handoff proof through this
        production adapter instead of the diagnostic executor; final tuple
        production still waits for Stage D remote heap resolution.
    - [x] Lock the pre-C5 batch ownership rule: local cancellation clears any
      retained `CandidateReceiveReady` compact batch and reports
      `remote_executor_cancelled` / `local_query_cancelled`, so cancelled
      dispatches cannot enter compact merge or Stage D heap resolution.
    - [x] First C2 failure-taxonomy slice: async production transport and
      compact-candidate receive classify remote statement timeout separately
      from generic remote query failure, with reserved categories for remote
      query cancellation and remote backend termination.
    - [x] Classify closed in-flight async remote query connections as
      `remote_backend_terminated` on the production transport path.
    - [x] Add PG18 fault-taxonomy coverage for remote query cancellation on
      production transport and compact-candidate receive, and remote backend
      termination on compact-candidate receive.
    - [x] First local-cancel remote-cancel primitive: the async production
      adapter can request `tokio-postgres` remote query cancellation through a
      cancel token under a deterministic local-cancel trigger, and executor
      state maps `local_query_cancelled` outcomes to global
      `remote_executor_cancelled` instead of ordinary per-node transport or
      receive failure.
    - [ ] Keep local cancel, local statement timeout, remote statement timeout,
      connect timeout, and remote backend termination as distinct diagnostic
      categories.
- [ ] Cache validated remote index identity where safe and invalidate on epoch,
  descriptor, or version changes.
  - [x] Define the cache contract in
    `plan/design/spire-libpq-identity-cache.md`: key by coordinator index,
    node, remote index, descriptor identity, and served epoch; bind descriptor
    generation plus endpoint protocol/version/opclass/storage/profile
    fingerprint; never store raw conninfo.
  - [x] Pin invalidation triggers before implementation: descriptor
    register/update/delete, descriptor generation or identity change,
    served/retained epoch-window change, live fingerprint mismatch, extension
    version change, opclass identity change, storage/assignment/profile change,
    remote regclass change, and local extension upgrade.
  - [x] Pin mismatch behavior before implementation: a live fingerprint change
    invalidates the entry and reports `endpoint_identity_mismatch` rather than
    silently reseating descriptor identity from the remote endpoint.
  - [x] Implement the first bounded executor-local identity cache: descriptor
    generation is part of the libpq dispatch key, the cache stores validated
    endpoint identity fields but no raw conninfo, and
    `ec_spire_remote_search_libpq_identity_cache_summary(...)` proves compact
    candidate receive and remote-heap receive reuse one validation in a single
    executor state.
  - [x] Expand PG18 cache-matrix coverage before further reuse work: ready
    loopback proves one miss/one hit and compact/heap parity; the contract probe
    proves descriptor-generation, descriptor-identity, and served-epoch changes
    miss; capability blockers prove stale epoch, retention gap, and
    extension-version skew do not touch the cache in strict or degraded mode.
  - [x] Add degraded live-fingerprint mismatch coverage: a descriptor/endpoint
    identity mismatch under degraded consistency reports `skip_node` with
    `remote_endpoint_identity`, leaves compact/heap candidates empty, and keeps
    the executor-local identity cache unpopulated.
- [ ] Verification: local multi-instance slow-remote fixture proves ready
  remotes are not serialized behind slow remotes; strict/degraded tests cover
  auth/cert failure, connection reset, remote timeout, backend termination, and
  local cancel.
  - [x] Add packet-local evidence for governance-slot release after local
    cancel.
    - [x] Packet `30753` runs PG18 test wrappers for
      `test_ec_spire_prod_transport_local_cancel_remote_cancel` and
      `test_ec_spire_prod_receive_local_cancel_remote_cancel`; both acquire
      global/per-node governance slots, trigger local cancellation, and then
      prove a separate backend can acquire and unlock the same advisory keys.
  - [x] Add packet-local timing evidence for first ready remote result arriving
    before a deliberately slow remote completes.
    - [x] Packet `30752` records `fast_completed_after_ms=3` and
      `slow_completed_after_ms=304` from separate local PG18 remote clusters
      through the production transport adapter.

### Stage D: CustomScan Distributed Read Path and v1 Writes

Goal: make the coordinator-visible result stream production-correct by using
the ADR-067 CustomScan integration point. The previous index-AM
materialization direction is superseded: ADR-064, ADR-065, ADR-066 and packets
`30761`, `30762`, `30765`, `30796`, `30797`, `30798`, `30799`, `30801`, and
the in-flight mirror-sync work are Shape-A history. Do not extend that path.

Preserved evidence from the superseded path:

- [x] Packet `30755` proves origin-node heap visibility resolution with opaque
  row locators and `remote_heap_resolution_failed` for missing remote heap
  rows.
- [x] Packet `30756` introduces `SpireRemoteProductionScanResultStream` as a
  Rust-side handoff/result-stream state.
- [x] Packets `30757`, `30758`, `30760`, `30761`, `30762`, `30768`, `30769`,
  and `30796` are retained as historical AM-boundary evidence but no longer
  define the production distributed read path.
- [x] Packets `30770` through `30795` remain reusable Stage E executor,
  diagnostic, fault-matrix, and lifecycle-matrix evidence.

CustomScan read-path work:

- [ ] Extend the production endpoint with ADR-068 tuple payloads.
  - [x] Add the first tuple-payload side-channel endpoint.
    - [x] Packet `30807` adds
      `ec_spire_remote_search_tuple_payload(...)`, which reuses the existing
      local heap candidate visibility path and returns JSON payload rows keyed
      by `(node_id, vec_id)` without changing the existing 18-column
      `ec_spire_remote_search(...)` envelope.
  - [x] Coordinator request building declares the exact target-column list
    required by the CustomScan projection at the endpoint boundary.
    - [x] Packet `30807` requires `requested_columns text[]`, validates the
      names against the indexed heap relation, rejects duplicate/empty names,
      and returns only those columns in the payload JSON object.
  - [x] Remote endpoint returns payloads keyed by `(node_id, vec_id)` alongside
    the unchanged 18-column candidate envelope.
    - [x] Packet `30807` exposes `payload_key = 'node_id_vec_id'` and leaves
      the Stage B envelope endpoint unchanged.
  - [x] Existing identity, fingerprint, version, and endpoint-status fields are
    unchanged and still gate merge eligibility.
  - [x] Add a focused tuple-payload PG18 fixture.
    - [x] Packet `30807` covers `id,title` projection payloads and proves the
      unrequested `embedding` column is omitted.
  - [x] Make missing heap rows explicit and avoid per-candidate SPI heap fetches.
    - [x] Packet `30812` batches tuple-payload CTID resolution through one SPI
      query, adds `tuple_payload_missing`, reports
      `remote_tuple_payload_missing`, and covers the missing-row signal. The
      JSON side-channel remains an endpoint/diagnostic bridge; production
      CustomScan slot delivery remains open below.
- [ ] Register `EcSpireDistributedScan`.
  - [x] Register the CustomScan provider.
    - [x] Packet `30805` registers `EcSpireDistributedScan` in `_PG_init`,
      chains the `set_rel_pathlist_hook`, exposes
      `ec_spire_custom_scan_status()`, and fails closed if executor callbacks
      are reached before planner path generation and tuple payload wiring
      land.
  - [x] Add planner path generation for tables with an `ec_spire` index and
    active remote placements when the query shape is
    `ORDER BY <vector-distance-op> LIMIT k`.
    - [x] Packet `30806` adds
      `ec_spire_custom_scan_index_eligibility()` so planner-path work can
      identify active remote placements without using the superseded AM-local
      materialization-gated placement snapshot. Query-shape detection and actual
      path creation remain open.
    - [x] Packet `30808` narrows that eligibility read to the placement
      directory object tuple, adds available-node / unavailable-placement /
      all-available signals for planner gating, and covers `no_active_epoch`
      plus `no_available_remote_placements` SQL states.
    - [x] Packet `30809` adds the first CustomPath generator for base
      relations with an eligible remote-placement `ec_spire` index and
      `ORDER BY ... LIMIT` pathkeys. It builds a `CustomScan` plan for EXPLAIN
      and keeps execution fail-closed until the executor wiring slice.
  - [x] Add a first planner cost model that chooses CustomScan when active
    remote placements exist.
    - [x] Packet `30809` assigns startup cost `0` and total cost based on the
      LIMIT row goal so the remote-placement path wins this query shape. This
      is intentionally provisional.
    - [x] Replace the provisional cost with a production model that accounts
      for coordinator routing traversal, per-remote dispatch latency by fanout
      count, and bounded heap-rerank/tuple-delivery cost before treating
      CustomScan costing as production-ready.
      - [x] Packet `30827` replaces `startup=0,total=LIMIT` with a calibrated
        fanout-aware model and Rust unit coverage for fanout/output-row cost
        monotonicity.
  - [x] Declare path keys so PostgreSQL can consume the ordered output without
    adding an explicit sort.
    - [x] Packet `30809` carries the planner sort pathkeys onto the CustomPath.
      Packet `30810` makes this true-by-construction by gating path generation
      on the vector-distance `ORDER BY ... LIMIT` query shape.
  - [ ] Implement Begin/Exec/End callbacks that invoke
    `SpireRemoteFanoutExecutor` directly and return tuples through the
    CustomScan tuple interface.
    - [x] Packet `30810` adds serializable CustomScan plan-private state for
      the selected index, constant `real[]` query, and `LIMIT`, allocates
      provider-owned executor state, routes `ExecCustomScan` through
      PostgreSQL `ExecScan`, and calls the production
      `SpireRemoteFanoutExecutor` result-stream entry point.
    - [x] Packet `30810` proves a remote-placement `SELECT ... ORDER BY ... LIMIT`
      reaches the production executor and fails on the real remote transport
      gate instead of the old scaffold "not wired" error.
    - [x] Packet `30811` extends the query-vector contract from constant
      `real[]` expressions to prepared-statement parameters, covering
      `ORDER BY embedding <#> $1 LIMIT k` through the same production executor
      path.
    - [x] Packet `30814` threads ADR-068 tuple-payload column requests through
      the production remote heap receive path, carries payload JSON on
      remote-origin output rows, and stores typed coordinator-visible values in
      a CustomScan virtual tuple slot.
    - [x] Packet `30815` adds the first end-to-end PG18 loopback-remote fixture
      proving remote-origin output rows return through CustomScan without the
      materialization catalog.
    - [x] Packet `30816` closes the immediate JSON payload-slot review
      blockers by rejecting projected array/composite payload columns before
      dispatch, rejecting non-scalar JSON values before type input, and caching
      per-attribute input functions for scalar slot delivery.
    - [x] Add the final multi-instance fixture proving the same path across
      separate local PostgreSQL instances.
      - [x] Packet `30820` adds
        `scripts/run_spire_multicluster_customscan_read_pg18.sh`; packet-local
        evidence shows `plan=Limit -> Custom Scan (EcSpireDistributedScan)`,
        `read_row=10,remote alpha`, and tuple-payload probe
        `ready,2,{"id": 10, "title": "remote alpha"}` across separate
        coordinator/remote clusters.
  - [x] Keep the existing index AM unchanged for local-only scans.
    - [x] Packet `30821` adds
      `test_ec_spire_customscan_does_not_replace_local_only_index_plan`, which
      disables seqscan, leaves indexscan enabled, and asserts the local-only
      plan contains `Index Scan` but not `Custom Scan (EcSpireDistributedScan)`.
  - [x] Add `EXPLAIN` coverage for `Custom Scan (EcSpireDistributedScan)`.
    - [x] Packet `30809` covers a remote-placement `ORDER BY <#> ... LIMIT 1`
      PG18 query and asserts `Custom Scan (EcSpireDistributedScan)`.
- [x] Add the end-to-end distributed read fixture.
  - [x] Packet `30815` fixture creates a coordinator with remote-only
    placements and loopback-remote
    shard rows.
  - [x] Fixture issues
    `SELECT cols FROM tbl ORDER BY embedding <-> $1 LIMIT k`.
  - [x] Fixture proves the selected remote rows and projected columns are
    returned through CustomScan.
  - [x] Fixture contains no
    `ec_spire_register_remote_row_materialization(...)` calls.
  - [x] Packet `30820` extends this from loopback evidence to a local
    multi-cluster PG18 script with separate coordinator and remote data
    directories and socket endpoints.

v1 write contract from ADR-069:

- [x] Add `ec_spire_placement(index_oid, pk_value, node_id, centroid_id,
  served_epoch, source_identity)` plus required indexes.
  - [x] Packet `30817` lands the table, primary key, identity index, and
    cleanup diagnostics coverage.
  - [x] Packet `30824` allows `node_id = 0` so ADR-069 placement rows can
    represent coordinator-local shard ownership as well as remote ownership.
- [x] Add `ec_spire_classify_centroid(embedding, index_oid)` using the same
  placement decision coordinator-routed INSERT will use.
  - [x] Packet `30818` classifies against the active routing hierarchy and
    returns the selected placement node, leaf-centroid id, and epoch.
  - [x] Packet `30826` documents the leaf-pid semantics for `centroid_id` and
    adds a recursive routing fixture before write paths consume the helper.
- [x] Add `ec_spire_register_placement_batch(index_oid,
  ec_spire_placement_entry[])` for bulk-load post-write registration.
  - [x] Packet `30819` inserts validated placement entries into the
    coordinator-local placement directory.
  - [x] Packet `30825` makes malformed array entries explicit, pins
    all-or-nothing batch behavior, and covers empty, duplicate, and invalid
    entries before coordinator-routed writes consume the primitive.
- [ ] Coordinator-routed INSERT:
  - [ ] classify embedding to target `node_id`;
    - [x] Packet `30828` adds the side-effect-free
      `ec_spire_plan_coordinator_insert(...)` primitive that validates
      `pk_value`/`source_identity`, calls the active classifier, and returns the
      placement tuple fields the 2PC INSERT path will persist.
  - [ ] forward remote INSERT through the Stage C transport;
    - [x] Packet `30829` adds the Stage C descriptor/secret/epoch-window
      readiness gate the mutating transport call will consume.
    - [x] Packet `30831` adds the remote-side
      `ec_spire_remote_insert_tuple_payload(index_oid, row_payload,
      requested_columns)` endpoint. The endpoint derives the heap relation from
      the remote SPIRE index, validates the explicit column list, projects JSON
      through PostgreSQL type input, and inserts the named tuple columns.
  - [ ] use remote `PREPARE TRANSACTION`, local placement-directory write, and
    remote commit for atomicity;
    - [x] Packet `30830` adds the internal remote-prepare primitive: it opens
      Stage C libpq transport, runs remote INSERT SQL inside a remote
      transaction, issues `PREPARE TRANSACTION`, registers coordinator
      transaction callbacks for remote `COMMIT PREPARED` / `ROLLBACK
      PREPARED`, and stages the local placement row only after remote prepare
      succeeds. The generic tuple-to-remote-INSERT builder and transparent DML
      hook remain open.
    - [x] Packet `30832` wires the remote-prepare primitive to the typed
      tuple-payload endpoint from packet `30831`, deriving the remote endpoint
      call from descriptor `remote_index_regclass`, JSON tuple payload, and an
      explicit column list before staging the local placement row in PG18
      coverage.
    - [x] Packet `30833` adds
      `ec_spire_prepare_coordinator_insert_tuple_payload(...)`, which composes
      classification, remote tuple-payload prepare, and placement-directory
      staging into the production operation the transparent INSERT hook will
      call after it builds tuple JSON, canonical primary-key bytes, and
      ADR-063 source identity from the executor tuple.
    - [x] Packet `30834` advances the helper's post-staging SQL status to
      `remote_insert_prepared_pending_local_commit` / `await_local_commit`, so
      callers do not mistake a prepared remote transaction for a committed,
      durable remote row.
    - [x] Packet `30835` adds the trigger-based transparent INSERT front door:
      `ec_spire_enable_coordinator_insert(...)` installs a `BEFORE INSERT`
      row trigger for the v1 bigint-PK / `ecvector` / bytea source-identity
      table shape, forwards through the helper, stages placement, and
      suppresses the coordinator heap row for remote-owned inserts.
  - [x] add PG18 coverage for remote row, placement row, and CustomScan
    read-after-insert.
    - [x] Packet `30834` adds
      `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh`,
      which runs separate coordinator and remote PG18 clusters, invokes
      `ec_spire_prepare_coordinator_insert_tuple_payload(...)`, verifies the
      remote row and coordinator placement row, and confirms
      `Custom Scan (EcSpireDistributedScan)` returns the inserted remote row.
  - [ ] refresh remote descriptor epoch/identity automatically after the
    prepared remote INSERT commits, so a subsequent CustomScan read does not
    require manual descriptor re-registration.
  - [ ] add PG18 multicluster coverage for `INSERT INTO coordinator_table ...`
    through the trigger front door, including remote row, placement row, and
    CustomScan read-after-insert once descriptor refresh is automatic.
- [ ] Coordinator-routed non-embedding UPDATE:
  - [ ] lookup `node_id` from the placement directory;
  - [ ] forward UPDATE to the owning remote;
  - [ ] add PG18 coverage.
- [ ] Coordinator-routed DELETE:
  - [ ] lookup `node_id` from the placement directory;
  - [ ] use remote prepared DELETE plus local placement-directory delete;
  - [ ] add PG18 coverage.
- [ ] PK-keyed SELECT:
  - [ ] lookup `node_id` from the placement directory;
  - [ ] forward SELECT to the owning remote;
  - [ ] add PG18 coverage.
- [ ] Reject embedding-changing UPDATE with the exact ADR-069 error and hint.
- [ ] Bulk-load tooling, cross-shard embedding moves, cross-shard non-vector
  scatter-gather, DDL propagation, and multi-coordinator deployments remain out
  of Phase 11 scope unless a later accepted ADR reopens them.

Cleanup decision:

- [ ] Keep `ec_spire_remote_row_materialization` and
  `ec_spire_register_remote_row_materialization` temporarily for migration and
  to avoid deleting in-flight Shape-A code before the CustomScan path exists.
- [ ] Remove the materialization catalog/register function and dead
  `requires_remote_row_materialization` CustomScan-adjacent references in the
  cleanup packet after CustomScan read and ADR-069 writes are feature-complete.

### Stage E: Multi-Instance Epoch, Lifecycle, and Fault Matrix

Goal: prove distributed correctness locally before AWS.

- [ ] Add local one-coordinator/two-remote setup and teardown through `ecaz`.
  - [x] First shell harness exists for the transport-overlap slice:
    `scripts/run_spire_multicluster_transport_overlap_pg18.sh` starts one
    coordinator and two remote PG18 clusters. The `ecaz` operator command and
    the full epoch/lifecycle/fault fixture remain open.
  - [x] Packet `30776` adds
    `ecaz dev spire-multicluster transport-overlap-pg18`, a CLI-owned wrapper
    for the one-coordinator/two-remote PG18 transport-overlap fixture with
    pgrx install discovery, packet-local artifact/log arguments, explicit port
    overrides, and setup/teardown delegated to the reviewed fixture script.
    The full epoch/lifecycle/fault fixture remains open.
  - [x] Packet `30777` runs the transport-overlap fixture through the new
    `ecaz` entrypoint with packet-local logs. The fixture started one
    coordinator plus two remotes and captured ready rows for both remotes, with
    the fast remote completing before the deliberately slow remote. This is CLI
    runtime evidence for the existing transport-overlap case, not coverage of
    the full Stage E strict/degraded matrix.
- [ ] Publish remote placement readiness and replica manifest freshness
  diagnostics.
  - [x] Packet `30771` adds the packet-friendly operator diagnostic rollup for
    readiness, served epoch range, fanout, candidate batches, heap resolution,
    merge, and AM delivery status. Explicit replica manifest freshness fixture
    evidence remains open with the broader Stage E lifecycle work.
  - [x] Packet `30774` adds node-level manifest freshness diagnostics through
    `ec_spire_remote_epoch_manifest_freshness()`. Boundary-replica fixture
    evidence and local multi-instance logs remain open.
- [x] Define online lifecycle behavior for DROP, REINDEX, and CREATE INDEX
  CONCURRENTLY while fanout is planned or in flight.
  - [x] Packet `30772` adds the SQL-visible Stage E lifecycle matrix for
    remote index DROP, REINDEX CONCURRENTLY, and CREATE INDEX CONCURRENTLY
    strict/degraded behavior. The later fixture still needs packet-local logs
    proving each lifecycle case.
  - [x] Packet `30789` starts lifecycle runtime evidence with
    `drop_remote_index_before_fanout`: the fixture drops the remote index
    before candidate-receive fanout, strict fails closed with
    `remote_index_unavailable`, and degraded skips that dispatch while keeping
    the ready remote moving. Remaining lifecycle rows still need fixture logs.
  - [x] Packet `30790` extends lifecycle runtime evidence with
    `drop_remote_index_in_flight`: the fixture builds production candidate
    receive requests while the remote index exists, injects `DROP INDEX`
    before receive, and expects the same strict fail-closed / degraded skip
    contract at the `remote_index_unavailable` boundary. Remaining REINDEX and
    CREATE INDEX CONCURRENTLY lifecycle rows still need fixture logs.
  - [x] Packet `30791` adds `reindex_remote_index_before_fanout` runtime
    evidence: the fixture captures the planned remote endpoint identity,
    runs `REINDEX INDEX CONCURRENTLY` before fanout, and verifies strict
    fail-closed / degraded skip behavior at the `endpoint_identity_mismatch`
    boundary. Remaining in-flight REINDEX and CREATE INDEX CONCURRENTLY
    lifecycle rows still need fixture logs.
  - [x] Packet `30792` adds `reindex_remote_index_in_flight` runtime
    evidence: the fixture builds candidate receive requests against the
    pre-REINDEX endpoint identity, injects `REINDEX INDEX CONCURRENTLY` before
    receive, and verifies strict fail-closed / degraded skip behavior at
    `endpoint_identity_mismatch`. CREATE INDEX CONCURRENTLY lifecycle rows
    still need fixture logs.
  - [x] Packet `30793` adds
    `create_index_concurrently_missing_descriptor` runtime evidence: the
    fixture creates a remote index before descriptor registration, rewrites a
    coordinator placement to that remote node without registering a descriptor,
    and verifies strict blocks at `requires_remote_node_descriptor` while
    degraded skips the pre-dispatch blocker without opening sockets. The new
    descriptor deferral row still needs fixture logs.
  - [x] Packet `30794` processes lifecycle reindex reviewer follow-up by
    documenting the filenode contribution to endpoint fingerprints and
    confirming the existing identity-cache matrix still covers filenode-driven
    fingerprint changes.
  - [x] Packet `30795` adds
    `create_index_concurrently_new_descriptor` runtime evidence: the fixture
    builds receive requests against an old descriptor, injects CREATE INDEX
    CONCURRENTLY plus descriptor generation advancement before receive, and
    verifies the already-planned old descriptor remains the receive target.
- [ ] Run a strict/degraded fault matrix: epoch mismatch, version skew,
  fingerprint mismatch, connection reset, backend termination, remote and local
  statement timeout, local cancel, simulated network partition, remote OOM, and
  missing/reindexed remote index.
  - [x] Every fault case must state expected strict outcome, expected degraded
    outcome, required status string, next blocker, failure action, and counter
    delta before the fixture is implemented.
    - [x] Packet `30770` lands the SQL-visible Stage E fault matrix and
      operator entrypoint contract for the one-coordinator/two-remote fixture.
    - [x] Packet `30773` pins review artifact names:
      `stage_e_fault_{fault_case}_{mode}.log` and
      `stage_e_lifecycle_{lifecycle_case}_{mode}.log`, and chooses the
      non-root unreachable-conninfo connect-failure mechanism for simulated
      network partitions.
- [ ] Verification: packet-local logs for every fault case, with explicit
  strict failure and degraded skip counts.
  - [x] Packet `30778` starts the runtime matrix with the simulated network
    partition case: one ready remote plus one unreachable conninfo, strict
    fail-closed transport summary, degraded skip-node transport summary, and
    packet-local strict/degraded logs. Remaining Stage E fault and lifecycle
    rows still need fixture logs.
  - [x] Packet `30779` adds the version-skew pre-dispatch runtime row:
    strict mode blocks the incompatible remote descriptor at
    `remote_extension_version`, degraded mode records one
    `incompatible_extension_version` skip while the ready remote remains
    pending for production transport, and strict/degraded logs are
    packet-local. Remaining Stage E fault and lifecycle rows still need
    fixture logs.
  - [x] Packet `30780` adds the epoch-mismatch pre-dispatch runtime row:
    strict mode blocks a stale remote served-epoch window at
    `remote_epoch_window`, degraded mode records one `stale_epoch` skip while
    the ready remote remains pending for production transport, and
    strict/degraded logs are packet-local. Remaining Stage E fault and
    lifecycle rows still need fixture logs.
  - [x] Packet `30781` adds the missing/reindexed remote-index candidate
    receive runtime row: strict mode records one
    `remote_index_unavailable` candidate-receive failure while a ready
    loopback candidate batch still decodes, degraded mode records one
    `remote_index_unavailable` skip and advances the ready batch to
    `remote_heap_resolution`, and strict/degraded logs are packet-local.
    Remaining Stage E fault and lifecycle rows still need fixture logs.
  - [x] Packet `30782` adds the endpoint fingerprint mismatch candidate
    receive runtime row: strict mode records one
    `endpoint_identity_mismatch` candidate-receive failure while a ready
    loopback candidate batch still decodes, degraded mode records one
    `endpoint_identity_mismatch` skip and advances the ready batch to
    `remote_heap_resolution`, and strict/degraded logs are packet-local.
    Remaining Stage E transport, timeout/cancel, OOM, and lifecycle rows still
    need fixture logs.
  - [x] Packet `30783` adds the remote statement-timeout transport runtime
    row: strict mode records one `remote_statement_timeout` transport failure
    while a ready remote still completes, degraded mode records one
    `remote_statement_timeout` skip and advances the ready remote to
    `compact_candidate_receive`, and strict/degraded logs are packet-local.
    Remaining Stage E backend termination, connection reset, local
    timeout/cancel, OOM, and lifecycle rows still need fixture logs.
  - [x] Packet `30784` adds the remote backend-termination transport runtime
    row: strict mode records one `remote_backend_terminated` transport failure
    while a ready remote still completes, degraded mode records one
    `remote_backend_terminated` skip and advances the ready remote to
    `compact_candidate_receive`, and strict/degraded logs are packet-local.
    Remaining Stage E connection reset, local timeout/cancel, OOM, and
    lifecycle rows still need fixture logs.
  - [x] Packet `30785` adds the local-cancel transport runtime row: local
    cancellation cancels every in-flight remote and reports
    `remote_executor_cancelled` in both strict and degraded mode. The executor
    summary now treats `local_statement_timeout` as the same query-wide
    cancellation control path while preserving its distinct category.
    Remaining Stage E connection reset, local statement-timeout, OOM, and
    lifecycle rows still need fixture logs.
  - [x] Packet `30786` adds the local statement-timeout transport runtime row:
    the PG interrupt bridge reports `local_statement_timeout` as a query-wide
    cancellation category, cancels every in-flight remote, and reports
    `remote_executor_cancelled` in both strict and degraded mode.
    Remaining Stage E connection reset, OOM, and lifecycle rows still need
    fixture logs.
  - [x] Packet `30787` adds the connection-reset-mid-batch transport runtime
    row: the fault remote starts a result stream and then terminates its own
    backend, which is classified as `remote_backend_terminated`; strict fails
    closed while degraded skips that dispatch and advances the ready remote to
    `compact_candidate_receive`.
    Remaining Stage E OOM and lifecycle rows still need fixture logs.
  - [x] Packet `30788` adds the remote-OOM transport runtime row: the fault
    remote raises SQLSTATE `53200`, the coordinator surfaces only the
    sanitized `remote_query_failed` category, strict fails closed, and degraded
    skips the failed dispatch while keeping the ready remote moving.
    Remaining Stage E lifecycle rows still need fixture logs.

### Stage F: Multi-Store / Multi-NVMe Hardening

Goal: keep local store scheduling production-observable before external scale.

- [ ] Preserve `(node_id, local_store_id)` as the scheduling and diagnostic
  unit.
- [ ] Prove local store lookup and read scheduling are bounded for configured
  maximum store count.
- [ ] Add repeatable local multi-store counters: route counts, candidate counts,
  object bytes, read batches, delta decode reuse, and scheduling limits.
- [ ] Verification: local harness evidence, not AWS hardware claims.

### Stage G: Production Harness and AWS Gate

Goal: make the final local readiness bundle reproducible.

- [ ] Extend `ecaz` with setup/load/query/teardown and distributed benchmark
  commands for the local multi-instance fixture.
  - [x] Packet `30759` fixes installed `ecaz dev install ecaz-pg-test --pg 18`
    repo-root discovery, so packet and reviewer workflows can use the operator
    install surface instead of falling back to direct `cargo pgrx install`.
- [ ] Publish a runbook with numeric local targets for max remotes, concurrent
  coordinator queries, per-remote work, PIDs per node, and overload behavior.
- [ ] Capture recall, latency p50/p95/p99, fanout, heap rows, timeout/cancel,
  strict failure, degraded skip, route, candidate, and byte counters.
- [ ] Open the AWS packet only after Stage A-F are reviewed or explicitly
  deferred.

## Validation

- Prefer focused PG18 tests for remote endpoint, coordinator, identity, and
  heap-resolution contracts.
- Use local multi-instance fixtures before external scale work.
- Use packet-local raw logs for every benchmark, latency, recall, or fanout
  claim.
- For each Phase 11 production slice, record the verification contract before
  or in the same patch as the code, then land the narrowest PG18/local fixture
  that can falsify the claim.
- Do not mark a Stage C/D/E item production-ready using only
  `ec_spire_remote_search_libpq_*` diagnostic SQL output; diagnostic output may
  support a claim only when it reflects production executor state or a live
  diagnostic probe explicitly labeled as such.
- Use `git diff --check` for planning-only packets.
- Keep AWS/RDS-class results out of Phase 11 until the explicit entry gate is
  satisfied.

## Exit Criteria

- Cross-node candidate dedupe uses stable global IDs end to end.
- Coordinator can query at least two remote PostgreSQL SPIRE nodes and merge
  compact candidates into one ordered stream.
- Remote heap resolution is production-defined and tested.
- Strict and degraded modes have explicit, tested behavior.
- Strict mode rejects incompatible epoch, quantizer/index fingerprint,
  extension/protocol version, and opclass-binary combinations before merge.
- Remote fanout is concurrent or pipelined, bounded, timed out, cancellable,
  and observable.
- Remote fanout preserves libpq `sslmode`, keeps raw conninfo hidden, reports
  sanitized auth/cert failures under strict/degraded semantics, and has a
  bounded resource-governance story across concurrent queries.
- Local multi-instance harness can reproduce recall/latency/counter evidence
  without AWS.
- AWS scale packet is ready to run, but not yet claimed as complete.
