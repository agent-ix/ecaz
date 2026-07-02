# Task 131: SPIRE Streaming Global Top-K Pruning

Status: **closed — shelved after accepted revised closeout** (2026-07-02; see
`reviews/task-131/028-revised-closeout-decision/feedback/2026-07-02-01-reviewer.md`):
packet 028 was accepted as the final closeout. The task shelves streaming
global-threshold pruning for the current SPIRE distributed surface, keeps
candidate-to-heap streaming as infrastructure, and files the duplicate-result
defect as Task 137.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 research follow-up if SPIRE distributed optimization continues.

## Why

Task 123's local multi-instance A/B closed two tempting but insufficient
hypotheses:

- shrinking remote heap payload bytes is measurable, but not the dominant
  latency driver;
- the dedupe-aware pre-materialization prune is correct plumbing, but does not
  produce a meaningful latency win in the representative b2/b4 matrix.

The production-read profile points instead at remote scan/scoring and remote
heap candidate production. In the measured shape, each query fans out to three
workers, waits for all candidate batches, then fans out again for heap
materialization. Workers do not share a global top-k frontier while scanning,
and the coordinator does not push a global kth-best threshold back to workers
to stop scans whose remaining candidates cannot compete.

This task researches that missing distributed algorithm: a coordinator-visible
global top-k threshold that can advance while remote workers are still scanning,
so workers can stop low-value work earlier or avoid heap work that cannot enter
the final top-k.

## Goal

Determine whether streaming/global top-k pruning can materially reduce SPIRE
distributed read latency at matched recall by cutting remote scan/scoring and
heap-materialization work before it is fully spent.

This task should produce a measured promote / iterate / shelve decision for the
algorithm. It must not claim a product win from static reasoning.

## Baseline Finding To Beat

Use the Task 123 packet 019/020 result as the starting point:

- `n1024 / b2 / nprobe64`: recall@10 `1.0000`, p50 about `0.73s-0.78s`;
- `n128 / b4 / nprobe96`: recall@10 `1.0000`, p50 about `5.1s-5.2s`;
- remote dispatch is healthy: no failed remote heap dispatches or degraded
  skips;
- `id,source` vs `id` payload bytes differ by roughly three orders of
  magnitude, but latency does not collapse with narrower payloads;
- current request shape has a candidate-phase barrier before heap receive.

The task must first re-establish the exact stage timing and candidate counts on
the branch under test before changing behavior.

## Scope

### Phase 0 - Timeline And Boundability Audit

Before implementing a pruning protocol, add or verify instrumentation for:

- per-worker candidate request start/end;
- per-worker heap request start/end;
- selected PID/list count;
- rows scanned or scored per selected PID/list;
- candidates retained locally;
- candidates shipped to coordinator;
- final global merge rank;
- current local kth score at each worker;
- whether each selected list/block has a sound upper bound usable for early
  stop.

The audit must distinguish:

- elapsed network/request time;
- remote scan/scoring time;
- heap/materialization time;
- payload decode time;
- coordinator merge/dedupe time.

If any required field is missing from `ecaz bench suite` output, extend the
runner or production-read profile first.

### Phase 1 - Coordinator Global Merge Before Heap

Implement the cheapest candidate reduction first: globally merge compact remote
candidate batches before requesting heap payload rows.

Current behavior can fetch roughly one local top-k worth of heap rows per
worker and then merge down to the query top-k. This phase should test whether
the coordinator can request heap only for globally surviving candidates.

Required evidence:

- heap rows avoided per query;
- payload bytes avoided;
- heap receive time before/after;
- recall/result identity against the baseline;
- latency p50/p95/p99 at 10k/50k/100k.

This phase is expected to be a limited win at best; it is still valuable because
it is simpler and de-risks global merge semantics before scan-time feedback.

### Phase 2 - Candidate-To-Heap Streaming

Remove the global candidate-phase barrier where safe.

When a worker finishes candidate production, allow heap resolution for that
worker's provisional globally competitive candidates while slower workers are
still scanning, subject to final merge correctness.

Required behavior:

- strict and degraded read semantics preserved;
- cancellation and statement-timeout cleanup preserved;
- connection-pool reuse preserved;
- no leaked remote sessions;
- final result identity or recall parity against the baseline.

Required evidence:

- skewed-fixture p95/p99 impact;
- normal-fixture p50/p95 impact;
- per-worker idle time removed;
- heap requests that later become globally unnecessary.

### Phase 3 - Streaming Global Threshold Feedback

Prototype a coordinator-to-worker global threshold protocol.

Candidate protocol shapes:

- workers stream candidate-score batches to the coordinator;
- coordinator maintains global kth-best exact or conservative approximate
  threshold;
- workers receive threshold updates and stop selected lists/blocks whose sound
  upper bound cannot beat the threshold;
- workers may overfetch conservatively when bounds are missing, stale, or too
  loose.

The threshold must be recall-safe by construction or explicitly gated as
diagnostic-only. Low-bit approximate scores are not sufficient as an unsafe
drop rule.

Required measurements:

- rows scanned/scored avoided;
- selected lists/blocks skipped or early-stopped;
- threshold-update count and bytes;
- extra coordinator/worker chatter;
- recall@10 and result identity where applicable;
- latency p50/p95/p99;
- storage or metadata added for bounds, if any.

### Phase 4 - Bound Strength And Metadata Decision

If Phase 3 needs better bounds, evaluate the minimum metadata needed for useful
early stop.

Candidate bound sources:

- existing route/leaf/block summaries;
- block-level score upper bounds;
- centroid/residual norm bounds;
- compact coarse-rerank summaries from Task 120;
- per-list max-score or norm envelopes.

Do not add durable metadata without a format/version plan and maintenance
invariants for insert, delete, vacuum, split, movement, remote version skew, and
stale summary fallback.

### Phase 5 - Closeout Decision

Publish a final packet that separately recommends promote, iterate, or shelve
for:

- global merge before heap;
- candidate-to-heap streaming;
- streaming global threshold feedback;
- any new bound metadata.

The closeout must explicitly state whether the algorithm beats the Task 123
baseline at matched recall and whether the win is large enough to justify the
added protocol complexity.

## Required Evidence

- Use `ecaz bench suite` for every benchmark matrix.
- Minimum local matrix before any promotion claim: 10k, 50k, 100k.
- Include `n1024/b2` and `n128/b4` unless Phase 0 proves one is irrelevant.
- Report recall@10, latency p50/p95/p99, storage, selected PID/list counts,
  rows scanned/scored, candidates retained, heap rows fetched, bytes shipped,
  threshold-update counts, cancellation/timeout behavior, and worker-count
  sensitivity.
- Store review artifacts under `reviews/task-131/`.
- Cite immutable benchmark packets when promoting or shelving a result.

## Non-Goals

- Do not optimize tuple payload encoding; Task 121 owns transport details.
- Do not replace topology routing; Task 120 owns route-set and coarse-rerank
  research.
- Do not reopen local multi-disk SPIRE as a product surface.
- Do not claim a win from reduced heap bytes alone; Task 123 showed bytes alone
  are not the dominant cost.
- Do not use an approximate threshold as a recall-unsafe drop rule.
- Do not write one-off benchmark sweepers.

## Acceptance Criteria

1. Phase 0 identifies where the baseline spends time and whether selected
   lists/blocks have sound early-stop bounds.
2. Phase 1 proves or rejects global merge-before-heap with recall/result
   identity and heap-row/byte/latency evidence.
3. Phase 2 proves or rejects candidate-to-heap streaming with skewed-fixture
   timing evidence and strict/degraded correctness coverage.
4. Phase 3 proves or rejects streaming global threshold feedback with
   rows-scanned/scored reduction and matched-recall latency evidence.
5. Any new bound metadata has a format/version and maintenance/fallback plan.
6. The final packet gives a clear promote / iterate / shelve decision for each
   investigated surface.

## References

- `plan/tasks/120-spire-coarse-rerank-measurement-program.md`
- `plan/tasks/121-spire-distributed-read-transport-efficiency.md`
- `plan/tasks/task30-phase13d-spire-read-efficiency-observability.md`
- `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`
- `spec/functional/spire/distributed/FR-058-spire-customscan-distributed-read.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
- `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`
- `src/am/ec_spire/scan/candidates.rs`
