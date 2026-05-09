# Task 30 Phase 10: SPIRE Execution and Performance Architecture

Status: proposed
Owner: coder1 / SPIRE execution track
Priority: 1 after Phase 9 graph contracts

## Goal

Turn the corrected Phase 9 graph design into a scalable query execution path.
Phase 10 should improve remote fanout, local multi-store execution, candidate
materialization, and heap rerank behavior without changing the graph semantics
that Phase 9 establishes.

## Scope

- Replace serial remote libpq query execution with an executor that overlaps
  remote work.
- Make candidate collection bounded and less eager.
- Improve multi-NVMe read overlap and diagnostics.
- Batch or prefetch exact heap rerank.
- Make top-graph query overhead proportional to the searched frontier, not to
  avoidable per-query setup.
- Keep all product performance claims gated on packet-local artifacts.

## Dependencies

- Phase 9 top-graph frontier contract.
- Phase 9 global recursive beam / route budget.
- Phase 9 global vector identity for remote merge and boundary replicas.
- Phase 8 scale packet for product-scale claims, unless explicitly waived by
  the operator for a narrower local claim.

## Phase 10.1: Bounded Candidate Collection

- [x] Add a hard candidate-row budget that applies even when
  `rerank_width = 0`.
- [x] Keep a bounded heap while scanning leaf and delta rows rather than
  materializing all routed candidates before ranking.
- [ ] Surface diagnostics for candidate rows seen, deduped, retained, and
  truncated.
- [ ] Preserve deterministic ordering and boundary-replica tie-break behavior.

## Phase 10.1a: Routing Diagnostic Drift Guard

- [ ] Remove the parallel production-vs-diagnostic recursive routing loop, or
  extract the traversal behind a shared collector/helper.
- [x] If a shared traversal helper is too invasive, add a property test proving
  diagnostic selected/deduped route counts match production route sets on a
  recursive fixture.

## Phase 10.2: Streaming AM Scan Shape

- [ ] Decide whether the AM should remain eager in `amrescan` with bounded work
  or move toward incremental `amgettuple` production.
- [ ] If staying eager, document the memory and latency ceiling and enforce
  limits.
- [ ] If streaming, define snapshot/object-store ownership so scan state can
  advance safely across `amgettuple` calls.
- [ ] Keep PostgreSQL executor semantics simple: forward scan only until a
  broader AM contract is intentionally added.

## Phase 10.3: Heap Rerank I/O

- [ ] Batch exact heap rerank TIDs before fetching source vectors.
- [ ] Add heap prefetch where PostgreSQL APIs permit it.
- [ ] Measure rerank width against recall and latency floors.
- [ ] Ensure missing/dead heap rows do not perturb candidate ordering beyond
  the existing visibility contract.

## Phase 10.4: Multi-NVMe Read Overlap

- [ ] Keep the existing `(node_id, local_store_id)` grouping as the scheduling
  unit.
- [ ] Add per-store route/candidate/read diagnostics.
- [ ] Split chained top-graph diagnostics enough for I/O attribution, including
  meta-tuple versus segment-tuple counts or equivalent block-level visibility.
- [ ] Overlap local-store reads where PostgreSQL backend constraints allow it,
  or make the sequential limitation explicit if read-stream prefetch is the
  only safe primitive.
- [ ] Decode delta objects once per leaf/query and reuse the decoded rows for
  delete suppression and insert candidate scoring.
- [ ] Replace linear local-store lookup with an indexed map if store counts
  grow beyond the current small bounded surface.

## Phase 10.5: Remote Libpq Executor

- [ ] Decide whether the current SQL-visible libpq executor remains diagnostic
  only or becomes the production AM remote query path.
- [ ] If production-bound, implement:
  - concurrent dispatch across ready remote nodes;
  - libpq pipeline mode or async receive;
  - bounded remote connection fanout;
  - connect and statement timeouts;
  - cancel propagation on local query cancellation;
  - cached remote index identity validation;
  - clear degraded/fail-closed behavior for partial remote failure.
- [ ] Keep raw conninfo out of SQL-visible surfaces.
- [ ] Preserve receive-batch validation before merge.

## Phase 10.6: Remote Heap Resolution

- [ ] Define whether remote heap candidates are resolved on the origin node or
  returned as opaque locators to a higher-level executor.
- [ ] If resolving remotely, return only heap-visible rows from the origin node
  under the requested epoch/consistency contract.
- [ ] If resolving later, expose that state as an explicit blocked/deferred
  result rather than a partially ready candidate set.

## Phase 10.7: Performance Harness

- [ ] Extend `ecaz` benchmark commands for Phase 9/10 routing budgets and
  remote fanout.
- [ ] Add a unified local scan pipeline snapshot that orders routing,
  placement, candidate, and heap-rerank steps, mirroring the remote
  `ec_spire_remote_pipeline_steps` operator shape.
- [ ] Record local development numbers separately from AWS/RDS-class numbers.
- [ ] Include one-index-per-table fixtures for cross-AM comparisons unless a
  packet explicitly measures shared-table planner behavior.
- [ ] Capture recall, latency p50/p95/p99, object bytes, route counts,
  candidate counts, heap rerank rows, and remote fanout counts.

## Validation

- Use focused PG18 tests for executor contracts and failure behavior.
- Use packet-local benchmark logs for every performance claim.
- `cargo clippy` remains optional unless the touched slice is lint-risky or the
  user explicitly asks for it.

## Exit Criteria

- Query execution has hard candidate and route budgets.
- Local multi-store reads either overlap or expose why they cannot.
- Delta rows are not decoded twice in the hot path.
- Remote fanout is concurrent or explicitly diagnostic-only.
- Heap rerank has a bounded, measured I/O path.
- The AM scan path has a documented eager-vs-streaming contract and enforces
  its memory/latency limits.
