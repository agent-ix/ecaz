# Task 30 Phase 11: SPIRE Distributed Production Parity

Status: proposed
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
- [ ] Emit durable global `0x02` `SpireVecId` values from the writer/build path
  when a stable source identity is available.
- [ ] Define the stable source-identity input contract for build/insert paths;
  heap TID alone is not a cross-node stable identity.
- [ ] Replace or extend Leaf V2 base-object storage so global `0x02` IDs are not
  rejected by the local-only fixed-width vec-id column format.
- [ ] Preserve compatibility with existing node-local `0x01` IDs, scoped by
  origin `node_id`, without silently mixing namespaces.
- [ ] Ensure boundary replicas carry the same global original-vector identity
  across leaves, stores, and remote nodes.
- [ ] Add migration/rewrite or compatibility diagnostics for indexes that still
  contain only node-local IDs.
- [ ] Add tests proving unrelated node-local IDs do not dedupe, while replicated
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
- [ ] Add protocol, extension-version, and opclass-binary version negotiation.
  Strict mode must reject incompatible remotes; degraded mode must report the
  skipped node and the exact mismatch.
- [ ] Remote nodes must reject stale/unavailable epochs explicitly in strict
  mode and surface degraded behavior explicitly when allowed.
- [ ] Add PG18 tests for nonempty remote candidates, stale epoch rejection, and
  empty/top-k-zero behavior.
- [ ] Add local version-skew tests with two remotes at different advertised
  contract versions.

## Phase 11.4: Production Libpq Coordinator Executor

- [ ] Replace the serial diagnostic executor shape with a production coordinator
  executor that overlaps remote work.
- [ ] Use libpq pipeline mode or async send/receive for batched remote calls.
- [ ] Bound remote fanout per query and expose truncation/degradation
  diagnostics when the bound applies.
- [ ] Cache validated remote index identity and avoid per-query repeated
  `to_regclass` lookups where safe.
- [ ] Add connection reuse or a clear bounded connection lifecycle with connect
  and statement timeouts.
- [ ] Define the narrow Phase 11 libpq security contract: preserve `sslmode`
  through `conninfo_secret_name` resolution, do not expose raw conninfo, reject
  libpq authentication or certificate-verification failures in strict mode with
  sanitized errors, and report skipped remotes in degraded mode.
- [ ] Add resource governance across queries, not only per query: coordinator
  global connection/work limits, per-remote-node concurrency caps, overload
  shedding behavior, and diagnostics for backpressure decisions.
- [ ] Propagate local query cancellation to outstanding remote work.
- [ ] Define fail-closed strict mode and explicit degraded mode behavior for
  partial remote failures.
- [ ] Add local multi-instance tests proving tail latency is not serialized
  across ready remotes under an instrumented slow-remote fixture.

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

## Validation

- Prefer focused PG18 tests for remote endpoint, coordinator, identity, and
  heap-resolution contracts.
- Use local multi-instance fixtures before external scale work.
- Use packet-local raw logs for every benchmark, latency, recall, or fanout
  claim.
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
