# Task 118: HNSW Quantized Recall Attribution

Status: **complete** (2026-06-24; closeout
`reviews/task-118/006-final-attribution-matrix/`, M5 release evidence commit
`af1845658` plus score-correlation baseline fix `e94ba88d1`).
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 for HNSW quantized-format decisions.

## Why

The current HNSW quantized-format benchmark rows are fast, but the recall
profile is not good enough to treat the result as a product conclusion.
Task 63's HNSW RaBitQ evidence shows the issue most clearly:

- at 100k / `ef_search=200`, TurboQuant and PqFastScan reach roughly
  `0.95` recall@10, while RaBitQ is materially lower;
- RaBitQ does not show the expected index-size win in the current HNSW layout;
- final rerank cannot recover a true neighbor that HNSW traversal never places
  in the candidate frontier.

Before adding more HNSW profiles, this task must attribute where recall is
lost: graph construction, approximate traversal scoring, candidate truncation,
final rerank ordering, or result emission.

## Goal

Build a measurement-first HNSW attribution workflow that answers, per query and
per format, whether the exact top-k neighbors are present in the HNSW candidate
frontier before final output truncation.

The task should produce a grounded decision about which follow-up is justified:

- fix graph build quality;
- fix approximate scorer ordering or metric handling;
- widen or exact-rerank the frontier;
- repair final rerank/output boundaries;
- or shelve a format because the measured frontier cannot support high recall.

## Scope

### Phase 1 - Candidate Containment Diagnostic

Add a debug or bench-suite mode that captures the HNSW candidate frontier before
final top-k emission.

For each query, report:

- requested `ef_search`;
- actual visited count;
- pre-final frontier size;
- exact f32 score for frontier candidates;
- whether exact truth top-10 and top-100 rows are present in the frontier;
- final emitted top-k rows;
- approximate score rank vs exact score rank for retained candidates.

The diagnostic must distinguish "truth row never entered the frontier" from
"truth row was present but final rerank/output lost it".

### Phase 2 - Final Rerank Boundary Audit

Prove that exact/source rerank, when enabled, is applied over the intended
candidate set rather than only polishing an already-truncated approximate top-k.

Add counters for:

- approximate traversal candidates visited;
- frontier candidates retained for rerank;
- exact-reranked candidates;
- candidates dropped before exact rerank;
- final emitted rows.

Add focused tests or diagnostic fixtures where a true neighbor is present in
the frontier but has a worse approximate score, so the test can catch rerank
being applied too late.

### Phase 3 - Build Graph Quality A/B

Measure whether graph construction itself is degrading topology.

Run an A/B with the same corpus, HNSW parameters, and query set:

- source-f32 neighbor selection during build;
- compressed or quantized neighbor selection during build;
- same scan-time format where feasible.

Confirm that `build_source_column` actually changes build scoring and/or graph
edges. If it does not, treat that as a bug or missing implementation rather
than a benchmark result.

### Phase 4 - Approx-Score Correlation Audit

For TurboQuant, PqFastScan, and RaBitQ:

- dump approximate score vs exact f32 score over visited candidates;
- verify score sign, metric handling, normalization, and distance ordering;
- report correlation and rank-error summaries;
- include synthetic fixtures with known exact ordering for scorer sanity.

This phase should catch wrong-sign, wrong-metric, stale-query, or format-dispatch
bugs that can still pass ordinary scan smoke tests.

### Phase 5 - Decision Packet

Publish a packet that classifies the dominant recall-loss stage by format:

- graph build quality;
- traversal scorer quality;
- frontier width;
- rerank boundary;
- visibility/output behavior;
- benchmark harness issue.

The packet should recommend the next HNSW task, or explicitly state that no
implementation follow-up is justified.

## Required Evidence

- Use `ecaz bench suite` for all benchmark matrices and durable diagnostics.
- Required scales: 10k, 50k, and 100k.
- Required formats: TurboQuant, PqFastScan, and RaBitQ.
- Required outputs: recall, latency, storage, candidate containment, frontier
  size, visited count, and rerank counters.
- 1M is optional until 10k/50k/100k identify a promising or suspicious path.
- Packet-local artifacts must live under `reviews/task-118/` or an immutable
  `benchmarks/` packet cited by `reviews/task-118/`.

## Non-Goals

- Do not change HNSW defaults in this task.
- Do not add a new RaBitQ profile in this task.
- Do not redesign HNSW storage format before attribution identifies the loss
  stage.
- Do not close on static review alone; recall, latency, and storage evidence at
  10k/50k/100k are required.

## Acceptance Criteria

1. Candidate-containment diagnostics exist and report whether exact truth rows
   reached the HNSW frontier before final output.
2. Rerank-boundary counters prove how many candidates are exact-reranked and
   when truncation occurs.
3. Source-f32 build vs compressed-build A/B evidence exists, or the task proves
   that the current code cannot express that A/B and files the narrow blocker.
4. Approx-score correlation evidence exists for TurboQuant, PqFastScan, and
   RaBitQ.
5. A final packet states the dominant recall-loss stage and recommends the next
   action for each format.

## References

- `plan/tasks/63-hnsw-rabitq-storage-format.md`
- `benchmarks/task63-hnsw-rabitq-format/`
- `spec/adr/ADR-018-hnsw-quantized-graph-quality.md`
- `spec/adr/ADR-030-fastscan-grouped-subvector-scoring.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
