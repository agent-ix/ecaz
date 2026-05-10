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
- Finish origin-node remote heap resolution and final row delivery semantics.
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
- Distributed writes across remote nodes. Phase 11 defines read fanout and
  origin-node heap resolution first; cross-node insert routing and
  read-after-write semantics remain a later distributed-write phase unless a
  slice explicitly narrows and lands the contract.
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
  - [x] Before C5 AM integration, pin and implement the PostgreSQL backend
    interrupt bridge from actual local query cancel / statement timeout into
    the adapter cancel token; test-only triggers are not production evidence.
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
  - [ ] Before C5 AM integration, pin AM-boundary consistency-mode policy:
    source of strict/degraded mode, single per-query threading into executor
    state, and final diagnostics or warning that name skipped nodes or at
    least count plus first skip category.
    - [x] First AM-boundary source-of-truth slice: session GUC
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
    - [x] Reviewer P3 follow-up: C5 must consume this matrix, or a Rust-side
      equivalent generated from the same category list, as the AM-boundary
      source of truth. Reserved Stage D heap-resolution rows are category names
      only until the heap executor emits them.
- [x] Add local multi-instance tests proving tail latency is not serialized
  across ready remotes under an instrumented slow-remote fixture.
  - [x] First multi-instance timing proof: packet `30752` adds a PG18 harness
    with one coordinator plus two separate remote PostgreSQL clusters, resolves
    conninfo through secret names, and proves the fast ready remote completes
    before the deliberately slow remote under the production async transport
    adapter. This is transport-overlap evidence only; the broader Stage E
    epoch/lifecycle/fault matrix remains open.

## Phase 11.5: Remote Heap Resolution and Final Row Delivery

- [ ] Implement origin-node heap visibility filtering for remote candidates
  before the coordinator claims final SQL row readiness.
- [ ] Keep coordinator row locators opaque except through the documented remote
  heap resolution path.
- [ ] Return one coordinator-visible ordered result stream after local and
  remote candidate merge/dedupe.
- [ ] Preserve deterministic tie-break ordering across local rows, remote rows,
  and boundary replicas.
- [ ] Add tests for missing/dead remote rows, stale locators, and duplicate
  replicated candidates across nodes.

## Phase 11.6: Multi-Instance Epoch and Placement Readiness

- [ ] Add a local multi-instance fixture with at least one coordinator and two
  remote PostgreSQL nodes.
- [ ] Publish and inspect placement metadata that maps selected PIDs to remote
  nodes and local store IDs.
- [ ] Verify strict mode does not mix incompatible epochs across nodes.
- [ ] Verify degraded mode reports every skipped or stale remote node.
- [ ] Define and test online lifecycle behavior when a remote index is dropped,
  reindexed, or created concurrently while fanout is planned or in flight.
- [ ] Add replica manifest freshness diagnostics: identify which remote node is
  serving each boundary replica, whether its manifest is current, and how
  degraded mode reports stale or missing replica placement.
- [ ] Add a fault matrix for connection reset mid-batch, remote backend
  termination, remote statement timeout, local statement timeout/cancel,
  simulated network partition, version skew, and remote OOM. Each case must
  state strict and degraded expected behavior.
- [ ] Add operator diagnostics that show remote node readiness, served epoch,
  remote fanout, candidate batches, heap resolution state, and merge status in
  one packet-friendly path.

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

1. **C4/C5 AM-boundary preflight.** Finish the pre-AM consistency and
   cancellation gate: expose active-epoch consistency-policy mismatch as a
   named row-returning category, document that the session GUC is the stable
   contract, and keep future query-level options as statement-local overrides
   of that GUC rather than replacements.
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
4. **C5 candidate-only AM integration.** Wire production compact candidate
   receive into the coordinator AM scan path behind the readiness gate. Until
   Stage D lands, the scan may prove ordered compact-candidate merge, but final
   SQL row readiness must still report `requires_remote_heap_resolution`.
   Packet `30754` lands the first handoff surface:
   `ec_spire_remote_search_production_scan_handoff_summary` derives selected
   PIDs from the scan router, runs live production compact receive, merges
   validated compact candidates, and keeps remote SQL row readiness blocked on
   Stage D.
5. **Stage D remote heap finalization.** Resolve remote heap visibility on the
   origin node, keep coordinator locators opaque, and produce one
   coordinator-visible ordered result stream only after local and remote
   candidates are visibility-correct.
   Packet `30755` lands the first heap-resolution proof surface:
   `ec_spire_remote_search_production_scan_heap_resolution_summary` gates
   origin-node heap receive on production compact-candidate readiness, resolves
   remote heap visibility under the origin PostgreSQL snapshot, exact-reranks
   visible heap rows, fails strict mode for missing remote heap rows, and merges
   ready local plus remote heap candidates. The actual `amrescan`/`amgettuple`
   tuple stream still remains open.
6. **Stage E local multi-instance matrix.** Build the one-coordinator /
   two-remote local fixture and run the strict/degraded matrix for epoch,
   version, fingerprint, connection, backend termination, timeout, cancel,
   network partition, OOM, and remote index lifecycle faults.
7. **Stage F/G local readiness bundle.** Add the repeatable multi-store and
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
and a routing-only selected-leaf PID handoff for the future AM scan integration.
This is not a production-ready distributed scan claim. The remaining blockers
are C2 cancellation propagation to in-flight remote work, C3/C4 production use
and strict/degraded normalization at the AM boundary, C5 AM scan integration,
Stage D remote heap resolution, and the local multi-instance fault/readiness
bundle.

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

### Stage D: Remote Heap Resolution and Final Rows

Goal: make the coordinator-visible result stream production-correct.

- [x] Keep remote row locators opaque at the coordinator for the first heap
  proof surface; packet `30755` asks the origin node to interpret locators and
  only receives heap block/offset diagnostics after origin-side visibility
  resolution.
- [x] Resolve remote heap visibility on the origin node before claiming
  heap-resolution summary readiness; packet `30755` fails strict mode with
  `remote_heap_resolution_failed` when a remote indexed locator no longer
  resolves to a visible heap row.
- [x] Merge local and remote heap-resolved candidates into one ordered stream
  with the existing deterministic tie-breaks across score, role, epoch, node,
  PID, object version, row index, and locator.
- [x] Introduce a narrow Rust-side production scan handoff/result-stream state
  that `amrescan` / `amgettuple` can consume directly, with SQL summary
  functions serializing from that state rather than becoming the internal AM
  contract. Packet `30756` adds
  `SpireRemoteProductionScanResultStream`, keeps the existing SQL summary
  stable, and preserves ordered heap-resolved output rows for the final AM
  cursor slice.
- [ ] Move the heap-resolved stream from the summary/proof surface into
  `amrescan` / `amgettuple` final tuple delivery.
  - [x] Packet `30757` classifies stream outputs into local coordinator heap
    TIDs that are safe for `xs_heaptid` and remote-origin outputs that must
    block on `remote_row_materialization`; it explicitly prevents treating
    remote origin heap coordinates as local heap TIDs.
  - [x] Packet `30758` gates the local manifest loader used by `amrescan` and
    other coordinator-local heap consumers so active remote placements report
    `remote_row_materialization` before any legacy local `xs_heaptid` path can
    consume them.
  - [x] Packet `30760` reuses the shared `remote_row_materialization` executor
    step constant in the AM remote-placement gate, closing the `30758` reviewer
    P2 before cursor wiring depends on the symbol.
  - [x] Packet `30761` defines the SQL-visible remote row materialization
    contract: origin-node heap coordinates are never coordinator `xs_heaptid`s;
    remote-origin AM delivery requires a same-indexed-heap shadow/proxy row,
    while FDW/custom-executor tuple paths are future non-AM integrations.
  - [ ] Implement the remote row materialization mechanism required
    before remote-origin outputs can be returned by a PostgreSQL index scan.
  - [x] Packet `30762` cursors AM-deliverable production-stream outputs through
    scan opaque state: `amrescan` now feeds the production heap-resolution
    result stream into an AM output cursor, and `amgettuple` only receives local
    coordinator heap TIDs while remote-origin outputs keep blocking on
    `remote_row_materialization`.
  - [x] ADR-064 proposes the shadow/proxy lifecycle required by the `30761`
    reviewer P2: v1 AM materialization must be a pre-existing coordinator heap
    row in the same scanned relation, not a per-query temp/scratch/proxy tuple.
  - [x] Add the materialized-row mapping contract required by ADR-064 before
    storage-provider implementation: the mapping key must preserve requested
    and served epoch, origin node, global vec-id, and opaque row locator; the
    mapped TID must belong to the same scanned heap relation and be visible to
    the scan snapshot; `amrescan` / `amgettuple` may validate existing mappings
    only and must keep missing or stale mappings blocked as
    `remote_row_materialization`.
  - [ ] Verification: tests for dead/missing remote rows, stale locators,
  duplicate cross-node replicas, local-only node-scoped IDs, and global-ID
  dedupe.
  - [x] Packet `30755` covers visible remote heap rows and missing remote heap
    rows in a focused PG18 loopback fixture.
  - [ ] Add stale locator, duplicate cross-node replica, local-only
    node-scoped ID, and global-ID dedupe coverage on the final AM tuple path.
  - [x] Run a broader PG18 pgrx pass across coordinator fanout call sites once
    the packet `30753` sandbox loader issue is resolved.
    - [x] Packet `30766` fixes pg_test-only advisory-governance test
      namespacing so production governance/cancel tests do not interfere under
      the default parallel pgrx runner, then validates
      `cargo pgrx test pg18 test_ec_spire_prod_` with 20 passed tests.

### Stage E: Multi-Instance Epoch, Lifecycle, and Fault Matrix

Goal: prove distributed correctness locally before AWS.

- [ ] Add local one-coordinator/two-remote setup and teardown through `ecaz`.
  - [x] First shell harness exists for the transport-overlap slice:
    `scripts/run_spire_multicluster_transport_overlap_pg18.sh` starts one
    coordinator and two remote PG18 clusters. The `ecaz` operator command and
    the full epoch/lifecycle/fault fixture remain open.
- [ ] Publish remote placement readiness and replica manifest freshness
  diagnostics.
- [ ] Define online lifecycle behavior for DROP, REINDEX, and CREATE INDEX
  CONCURRENTLY while fanout is planned or in flight.
- [ ] Run a strict/degraded fault matrix: epoch mismatch, version skew,
  fingerprint mismatch, connection reset, backend termination, remote and local
  statement timeout, local cancel, simulated network partition, remote OOM, and
  missing/reindexed remote index.
  - [ ] Every fault case must state expected strict outcome, expected degraded
    outcome, required status string, next blocker, failure action, and counter
    delta before the fixture is implemented.
- [ ] Verification: packet-local logs for every fault case, with explicit
  strict failure and degraded skip counts.

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
