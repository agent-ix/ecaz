# Task 121: SPIRE Distributed Read Transport Efficiency

Status: **proposed**.
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 follow-up for renewed distributed SPIRE work. Run after Task 120
has identified the candidate/rerank budget shape, or earlier only if
production-read profiles show executor or transport overhead dominates.

## Why

Task 30 Phase 13d/13e delivered the production distributed SPIRE read path:
production read profiles, remote CustomScan fanout, candidate/heap session
reuse, heap receive fanout, and connection pooling evidence. The accepted
pooling packet already removed the obvious connect/socket-open bottleneck.

Task 107 then made the product call: drop local multi-disk SPIRE as a product
surface, keep multinode SPIRE only as a narrow research/regression surface, and
do not market SPIRE RaBitQ/TurboQuant as product-competitive without separate
latency/storage observability and product-scale evidence.

Task 120 owns the algorithmic SPIRE research frontier: stage containment,
coarse rerank, explicit candidate budgets, topology route-set refinement, and
distributed near-data rerank. This task must not duplicate that scope.

The remaining unowned area is the distributed read executor and transport once
the route/candidate/rerank policy has already been chosen. The current code and
bench evidence leave several measurable questions open:

- typed tuple payloads are returned by the endpoint as `bytea[]`, but the libpq
  coordinator path hex-encodes them into text and decodes them back into bytes;
- typed payload metadata is repeated per returned row;
- projection narrowing falls back to full tuple payloads for expression targets
  instead of collecting the input Vars safely;
- the current candidate/heap request driver reuses sessions, but still has a
  global candidate phase before heap receive starts;
- existing distributed benchmarks mostly projected `id`, so they did not
  measure tuple-heavy payload transport costs;
- the existing timeline proves heap receive fanout within the heap phase, but
  does not reliably prove candidate-to-heap streaming because heap phase start
  timestamps are not recorded separately.

## Goal

Determine whether SPIRE's distributed read executor and tuple transport have
measurable latency, byte, or CPU wins after Task 120's candidate/rerank policy
has fixed the algorithmic shape.

This task should produce evidence-backed executor decisions, not a broad SPIRE
optimization bucket.

## Scope

This is a phased measurement-first task. Each phase should produce a packet
under `reviews/task-121/` with its own go/no-go result before the next phase
lands behavior changes.

### Phase 0 - Evidence Gate and Instrumentation Audit

Audit the existing production-read profile and timeline surfaces before making
transport changes.

Required checks:

- confirm which profile fields are reliable for connect, endpoint identity,
  regclass probe, candidate receive, heap receive, payload decode, merge, total
  elapsed, rows, bytes, and socket opens;
- add or fix true per-node phase start/end timestamps before making any
  overlap or streaming claim;
- confirm `ecaz bench suite` can drive the required distributed tuple-heavy
  read profiles and store the production-read profile/timeline artifacts;
- record result source, worker count, projection shape, storage format,
  route/fanout settings, rows shipped, bytes shipped, payload decode bytes, and
  payload decode elapsed time in packet-local artifacts.

If the runner cannot express a required diagnostic, extend `ecaz bench suite`
first instead of writing a per-packet sweeper.

### Phase 1 - Tuple-Heavy Distributed Baseline

Run a distributed SPIRE baseline with one coordinator and two workers, using
`ecaz bench suite`.

Required cells:

- an `id`-only projection control matching the Task 107 style;
- a tuple-heavy projection with several scalar/text columns;
- at least one SPIRE storage format that Task 120 or Task 107 evidence keeps
  relevant;
- 100k as the first decision scale;
- 1M only if the 100k profile shows a plausible executor/transport bottleneck
  or if the task is being used to support a product-scale SPIRE claim.

Required measurements:

- recall@10 and latency p50/p95/p99;
- candidate receive, heap receive, payload decode, merge, and total elapsed
  profile rows;
- rows and bytes shipped by worker;
- payload decode row count and byte count;
- socket-open and connection-pool behavior;
- endpoint identity and regclass probe cost after pooling;
- projection width and targetlist shape.

Phase 1 should rank the executor/transport bottlenecks, or close the task early
if payload and scheduling overhead are not material.

### Phase 2 - Direct Typed Payload Receive

Evaluate replacing the coordinator libpq typed-payload hex text path with
direct `bytea[]` receive.

This phase is gated by Phase 1. Do not implement it unless payload bytes,
payload decode elapsed time, or tuple-heavy distributed latency makes the hex
path worth testing.

Required behavior:

- preserve the existing typed tuple payload contract and JSON/older-remote
  fallback behavior;
- keep malformed typed payloads fail-closed;
- validate nulls, type OIDs, typmods, collations, and binary receive exactly as
  the current typed path does;
- prove recall/result identity against the baseline.

### Phase 3 - Metadata and Projection Width

Evaluate reducing tuple payload width and repeated metadata cost.

Candidate slices:

- send typed metadata once per result batch instead of repeating it per row, if
  the wire/parse profile shows metadata repetition is material;
- add a conservative expression-target Var dependency collector so expressions
  such as `title || suffix` request only their input columns instead of the
  whole relation payload;
- preserve full-payload fallback for whole-row Vars, system columns, dropped
  columns, unsupported targetlist shapes, or any case where dependency
  extraction is not obviously safe.

This phase must keep the Task 30 expression-payload correctness fix intact: an
expression must never be evaluated with a missing input column.

### Phase 4 - Candidate-to-Heap Scheduling

Evaluate whether the global candidate phase before heap receive is a real
latency bottleneck.

Required first step:

- use the fixed timeline fields from Phase 0 and a skewed distributed fixture
  to measure whether slow candidate or heap phases inflate p95 latency.

Only if the measurement shows material p95 impact, evaluate streaming heap
receive for nodes whose candidates are ready while slower candidate sessions
continue.

Required behavior:

- preserve strict vs degraded read semantics;
- preserve cancellation, timeout, and cleanup behavior;
- preserve connection-pool reuse and do not leak sessions;
- keep final merge/dedupe correctness byte-identical to the baseline.

### Phase 5 - Closeout Decision

Publish a closeout packet that recommends one decision for each investigated
surface:

- promote;
- iterate with a narrower follow-up;
- shelve with evidence.

The final decision must separate tuple transport, metadata/projection width,
and phase scheduling. A win in one does not justify broad SPIRE product claims.

## Required Evidence

- Use `ecaz bench suite` for every benchmark matrix.
- Store review artifacts under `reviews/task-121/` and immutable benchmark
  evidence under `benchmarks/` only when the packet cites it explicitly.
- Include the exact commit SHA, topology, worker roles, storage format,
  projection shape, route/fanout settings, corpus scale, and runner config in
  every manifest.
- Include paired before/after rows for every behavior change.
- Include result identity or recall evidence for every transport/projection
  change.

## Non-Goals

- Do not own routing, candidate-surface reduction, coarse rerank, route-set
  refinement, or budget policy. Those belong to Task 120.
- Do not reopen the Task 30 Phase 13e connection-pooling implementation unless
  fresh profile evidence shows pooling is still the bottleneck.
- Do not reopen local multi-disk SPIRE as a product surface; Task 107 dropped
  it.
- Do not make a distributed SPIRE product/default claim from executor evidence
  alone.
- Do not add a fused endpoint or new RPC shape unless the earlier phases prove
  the current candidate/heap split is the dominant cost.
- Do not write one-off benchmark sweepers.

## Acceptance Criteria

1. Phase 0 records which production-read profile and timeline fields are
   reliable, and fixes true per-node phase start/end timestamps if needed.
2. Phase 1 records tuple-heavy distributed read evidence against an `id`-only
   control and ranks the executor/transport bottlenecks.
3. Any direct typed-payload receive change has before/after tuple-heavy
   evidence and preserves result identity or recall.
4. Any metadata or projection-width change has before/after byte/latency
   evidence and preserves expression-payload correctness.
5. Any candidate-to-heap scheduling change has skewed-fixture evidence,
   strict/degraded behavior coverage, cancellation/timeout cleanup coverage,
   and connection-pool reuse evidence.
6. Every benchmark matrix is suite-driven and packet-local.
7. The closeout packet separately recommends promote, iterate, or shelve for
   direct payload receive, metadata/projection width, and phase scheduling.

## References

- `plan/tasks/task30-phase13d-spire-read-efficiency-observability.md`
- `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`
- `plan/tasks/75-spire-latency-routing-envelope.md`
- `plan/tasks/107-spire-multidisk-multinode-value-prop.md`
- `plan/tasks/120-spire-coarse-rerank-measurement-program.md`
- `plan/design/spire-typed-tuple-transport.md`
- `spec/functional/spire/distributed/FR-058-spire-customscan-distributed-read.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
- `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs`
- `src/am/ec_spire/coordinator/remote_candidates/payload_limits.rs`
- `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`
- `src/am/ec_spire/custom_scan/begin_exec.rs`
