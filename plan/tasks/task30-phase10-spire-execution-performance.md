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
- [x] Surface diagnostics for candidate rows seen, deduped, retained, and
  truncated.
- [x] Preserve deterministic ordering and boundary-replica tie-break behavior.

## Phase 10.1a: Routing Diagnostic Drift Guard

- [x] Remove the parallel production-vs-diagnostic recursive routing loop, or
  extract the traversal behind a shared collector/helper. Deferred for Phase 10
  after review packet `30669` accepted the fallback drift-guard path; packets
  `30669` and `30674` now guard selected/deduped counts across recursive depth.
- [x] If a shared traversal helper is too invasive, add a property test proving
  diagnostic selected/deduped route counts match production route sets on a
  recursive fixture.
- [x] Extend the routing diagnostic drift guard to a depth > 2 fixture or a
  parameterized hierarchy so intermediate recursive levels are covered.

## Phase 10.2: Streaming AM Scan Shape

- [x] Decide whether the AM should remain eager in `amrescan` with bounded work
  or move toward incremental `amgettuple` production. ADR-056 keeps Phase 10 on
  the eager bounded path.
- [x] If staying eager, document the memory and latency ceiling and enforce
  limits. ADR-056 records the route/candidate ceilings and the first-tuple
  latency tradeoff.
- [x] If streaming, define snapshot/object-store ownership so scan state can
  advance safely across `amgettuple` calls. ADR-056 rejects streaming for this
  phase and requires a separate ownership ADR before that change.
- [x] Keep PostgreSQL executor semantics simple: forward scan only until a
  broader AM contract is intentionally added. `amcanbackward = false` and
  `amgettuple` rejects non-forward scan directions.

## Phase 10.3: Heap Rerank I/O

- [x] Batch exact heap rerank TIDs before fetching source vectors.
- [x] Add heap prefetch where PostgreSQL APIs permit it.
- [ ] Measure rerank width against recall and latency floors.
- [x] Ensure missing/dead heap rows do not perturb candidate ordering beyond
  the existing visibility contract.

## Phase 10.4: Multi-NVMe Read Overlap

- [x] Keep the existing `(node_id, local_store_id)` grouping as the scheduling
  unit.
- [x] Add per-store route/candidate/read diagnostics.
- [x] Split chained top-graph diagnostics enough for I/O attribution, including
  meta-tuple versus segment-tuple counts or equivalent block-level visibility.
- [x] Overlap local-store reads where PostgreSQL backend constraints allow it,
  or make the sequential limitation explicit if read-stream prefetch is the
  only safe primitive. ADR-057 accepts PostgreSQL relation prefetch/read-stream
  as the Phase 10 overlap primitive and keeps decoding/scoring sequential
  inside one backend.
- [x] Decode delta objects once per leaf/query and reuse the decoded rows for
  delete suppression and insert candidate scoring.
- [x] Replace linear local-store lookup with an indexed map if store counts
  grow beyond the current small bounded surface.

## Phase 10.5: Remote Libpq Executor

- [x] Decide whether the current SQL-visible libpq executor remains diagnostic
  only or becomes the production AM remote query path. ADR-058 keeps it
  diagnostic/operator-only.
- [x] If production-bound, implement the following. ADR-058 chooses
  diagnostic-only for the current executor, so these production requirements
  are deferred to a future production remote executor ADR/checkpoint:
  - concurrent dispatch across ready remote nodes;
  - libpq pipeline mode or async receive;
  - bounded remote connection fanout;
  - connect and statement timeouts;
  - cancel propagation on local query cancellation;
  - cached remote index identity validation;
  - clear degraded/fail-closed behavior for partial remote failure.
- [x] Keep raw conninfo out of SQL-visible surfaces for the diagnostic
  executor. ADR-058 preserves the `conninfo_secret_name` indirection and
  executor-owned secret lookup.
- [x] Preserve receive-batch validation before merge. The diagnostic executor
  decodes rows into the result contract and calls
  `validate_remote_search_candidate_batch` before global merge.

## Phase 10.6: Remote Heap Resolution

- [x] Define whether remote heap candidates are resolved on the origin node or
  returned as opaque locators to a higher-level executor. ADR-059 assigns
  remote heap resolution to the origin node; the coordinator keeps
  `row_locator` opaque.
- [x] Add or require writer-side global vector ID allocation before claiming
  cross-node boundary-replica dedupe is end-to-end safe. ADR-059 requires
  writer-side global `0x02` vec IDs before a production cross-node
  replica-dedupe claim.
- [x] If resolving remotely, return only heap-visible rows from the origin node
  under the requested epoch/consistency contract. ADR-059 makes this mandatory
  for any future production remote-heap-ready state; the current production
  state remains blocked rather than partially ready.
- [x] If resolving later, expose that state as an explicit blocked/deferred
  result rather than a partially ready candidate set. Existing coordinator
  summaries expose `requires_remote_heap_resolution`, and ADR-059 keeps that as
  the production state until origin-node resolution lands.

## Phase 10.7: Performance Harness

- [ ] Extend `ecaz` benchmark commands for Phase 9/10 routing budgets and
  remote fanout.
- [x] Add a unified local scan pipeline snapshot that orders routing,
  placement, candidate, and heap-rerank steps, mirroring the remote
  `ec_spire_remote_pipeline_steps` operator shape. Added
  `ec_spire_index_scan_pipeline_snapshot`.
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
