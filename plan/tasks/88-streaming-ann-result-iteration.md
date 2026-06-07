# Task 88: Streaming ANN Result Iteration (HNSW + DiskANN)

Status: proposed (2026-06-07)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (hybrid-search support; follows Task 87)

## Why

Task 86 investigation surfaced pgvectorscale's
`StreamingDiskANN` pattern (see
`/Users/peter/dev_bak/pgvectorscale/pgvectorscale/src/access_method/graph/mod.rs`
`greedy_search_streaming_init` + `greedy_search_iterate`).
The pattern yields candidates lazily to the PostgreSQL
executor in approximate-score order, with a resort buffer +
streaming stats for the approximate-vs-exact reranking
boundary.

Motivating workload: **hybrid search**.

```sql
SELECT id, body
  FROM document
 WHERE category = 'science'
 ORDER BY embedding <=> $1
 LIMIT 10;
```

Without streaming, the index AM must produce a full
candidate set, the executor applies the `WHERE` filter, and
work spent scoring filtered-out candidates is wasted. With
streaming, the AM yields candidates incrementally; the
executor applies the filter; the query stops after 10 matches
survive. For restrictive filters this is a substantial
end-to-end win — pgvectorscale's README explicitly cites
this as the motivation: *"The post-filtering implementation,
while slower, is streaming and correct, ensuring accurate
results without requiring the entire result set to be loaded
into memory."*

## Why this is hard

- Requires PG AM iterator semantics (`amgettuple`
  continuation across calls; index scan state must persist
  between executor invocations).
- Approximate-distance order ≠ exact-distance order — needs
  a resort buffer for the reranking boundary.
- Streaming stats (running mean/variance/max distance) tell
  the iterator when it's safe to pop from the resort buffer
  without missing a closer candidate.
- Only naturally applies to **graph-based AMs (HNSW +
  DiskANN)** which have best-first traversal. IVF + SPIRE
  scan partitions in arbitrary order and don't get the same
  post-filter win — explicitly out of scope.
- Must compose with Task 87's `CandidateBatch` abstraction
  without forcing a redesign of either.

## Goal

Land streaming result iteration on the two graph-based AMs
(HNSW + DiskANN), enabling post-filter early termination
for hybrid-search workloads. Demonstrate measured latency
wins at restrictive filter selectivities; preserve recall
on non-filtered baselines.

## Scope

### In scope

1. HNSW scan path supports `amgettuple` continuation.
2. DiskANN scan path supports `amgettuple` continuation.
3. Per-AM resort buffer + streaming stats matching
   pgvectorscale's pattern.
4. Per-AM hybrid-search benchmark workload.
5. Compose with Task 87's `CandidateBatch` so each iterate
   call can score a batch and push it into the resort
   buffer (the resort buffer IS the candidate batch from
   the executor's perspective).

### Out of scope

- IVF streaming (weak fit; no best-first traversal).
- SPIRE streaming (same — weak fit; no best-first
  traversal).
- New filter operator surface — use PostgreSQL's existing
  post-filter executor path.
- TurboQuant TQ+ — separate follow-up.

## Phase 1 — Design Packet

Land one design packet before any AM-level work:

- Streaming PG AM contract: `amgettuple` semantics,
  `IndexScanDesc` lifetime, how iterator state survives
  between calls.
- Resort buffer + streaming stats design (copy pgvectorscale's
  shape or adapt).
- Per-AM scoring-site contract: must compose with Task 87's
  `CandidateBatch`. Define how the iterator calls into the
  shared kernel and pushes results into the resort buffer.
- Hybrid-search benchmark methodology: which selectivities
  (e.g. 1 %, 10 %, 50 %, 100 %), which `LIMIT` values
  (10 / 100 / 1000), what counts as the baseline (today's
  full-materialize path).
- Pre-commit a `suite.json` shape so both AM slices use the
  same harness.

## Phase 2 — HNSW Streaming Slice

- HNSW scan supports `amgettuple` continuation.
- Resort buffer + streaming stats (per Phase 1 design).
- Validation:
  - **Hybrid-search workload**: latency improvement at every
    measured (selectivity × LIMIT) cell.
  - **Non-filtered baseline**: recall@10 + p50/p95/p99
    latency neutral (no regression).
  - **All existing HNSW pg_test surfaces pass.**
  - **Suite-driven per FR-038**: `suite.json` checked in,
    baseline vs streaming columns.

## Phase 3 — DiskANN Streaming Slice

- DiskANN scan supports `amgettuple` continuation.
- Resort buffer + streaming stats (per Phase 1 design;
  reuse HNSW's shape where applicable).
- Validation (same shape as HNSW):
  - **Hybrid-search workload**: latency improvement at every
    measured (selectivity × LIMIT) cell.
  - **Non-filtered baseline**: recall@10 + p50/p95/p99
    latency neutral (no regression).
  - **All existing DiskANN pg_test surfaces pass.**
  - **Suite-driven per FR-038.**

## Phase 4 — Closeout

- Both HNSW and DiskANN slices reviewer-approved.
- Aggregate measurement comparison.
- Closeout packet citing per-AM evidence.
- Status flip to `complete` referencing the closeout packet.

## Validation gate (per AM, every cell)

1. **Hybrid-search latency improves** at every measured
   selectivity × LIMIT cell. Measured against a real corpus
   (real10k / 50k / 100k) with a realistic filter column.
2. **Non-filtered baseline recall@10 byte-equal** vs
   pre-streaming.
3. **Non-filtered baseline latency** within ±5 % (resort-
   buffer overhead must not regress the no-filter path).
4. **All existing pg_test surfaces pass** for the AM under
   slice.
5. **Suite-driven per FR-038** — `ecaz bench suite` with
   checked-in `suite.json`, both columns committed.
6. **No new `unsafe`** outside the existing AM boundary;
   resort buffer is safe-Rust.

## Acceptance criteria

1. HNSW supports `amgettuple` continuation with resort
   buffer + streaming stats.
2. DiskANN supports `amgettuple` continuation with resort
   buffer + streaming stats.
3. Per-AM hybrid-search workload shows measured latency win
   at every restrictive-filter cell.
4. Per-AM non-filtered baseline shows recall byte-equal +
   latency neutral.
5. All existing pg_test surfaces pass for both AMs.
6. Closeout packet cites per-AM evidence + aggregate matrix.
7. `plan/tasks/88-…md` status flips to `complete` only
   referencing the closeout packet.

### Per-AM completion is non-negotiable

The task is **not** complete until both HNSW and DiskANN
have shipped + been reviewer-approved + met the per-AM
validation gate. A partial close (e.g. "shipped HNSW,
deferring DiskANN") requires:

- An explicit Stop Condition packet naming the per-AM
  blocker (e.g. "DiskANN's page-bounded scan can't preserve
  approximate-score order across page boundaries without
  buffering the whole page — overhead exceeds the
  post-filter win at our typical selectivities").
- Reviewer acceptance of the Stop Condition.
- A follow-up task explicitly scoped for the deferred AM.

Single-AM ship does **not** satisfy Task 88. The whole
point is providing streaming for both graph-based AMs;
shipping it on one and walking away makes the feature
asymmetric and surprising for operators choosing between
the two.

## Coordination

- **Depends on Task 87** (candidate-batched scoring) being
  merged. Task 88 reuses Task 87's `CandidateBatch`
  abstraction inside its resort buffer.
- **IVF + SPIRE are explicitly out of scope.** If
  streaming-for-IVF becomes a priority later (e.g. for a
  specific hybrid-search workload), file a new task with
  the post-filter use case justification — don't expand
  Task 88's scope.
- **pgvectorscale** is the reference implementation
  (`/Users/peter/dev_bak/pgvectorscale/`). Key files:
  - `pgvectorscale/src/access_method/scan.rs`
    (`resort_buffer`, `StreamingStats`)
  - `pgvectorscale/src/access_method/graph/mod.rs`
    (`greedy_search_streaming_init`)
  - Read-only reference; don't depend on it at build time.

## Coder workflow notes

- Phase 1 design packet must land before any AM slice
  starts. Reviewer approves Phase 1 explicitly.
- Each AM slice is its own packet with its own real-corpus
  hybrid-search measurement. No "batched landing" of both
  AMs in one packet.
- The PG `amgettuple` continuation contract is the
  trickiest part. Get it right in Phase 1; reuse across
  both AMs.
- Per memory `feedback_no_premature_task_close`, the
  reviewer drives to 100 % per AM.
- Per memory `feedback_dont_defer_safety_fixes`, any new
  unsafe ships with `# Safety` docs.
- The resort buffer is safe-Rust by construction (PG owns
  the iterator lifetime via the executor).

## Stop conditions

- Per AM, if the streaming + resort-buffer overhead causes
  the non-filtered baseline to regress > 5 %, back the AM
  out and document. Streaming must not cost non-streaming
  users.
- Per AM, if no measured post-filter latency win materializes
  at any reasonable selectivity × LIMIT cell, back the AM
  out and document. The whole point is the post-filter win.
- If the Task 87 `CandidateBatch` abstraction can't host
  the resort buffer cleanly, pause Task 88 and coordinate
  with Task 87 ownership.

## References

- Task 86 closeout: `reviews/task-86/010-closeout-audit/`
  (surfaced the streaming-vs-batching distinction)
- Task 87 (predecessor): `plan/tasks/87-candidate-batched-scoring-across-ams.md`
- pgvectorscale `StreamingDiskANN` reference:
  `/Users/peter/dev_bak/pgvectorscale/`
- pgvectorscale README streaming section: *"The
  post-filtering implementation, while slower, is streaming
  and correct…"*
- FR-038 (benchmark provenance)

## Estimated size

Medium-large. 2–4 weeks for one coder including Phase 1
design, two AM slices with hybrid-search workloads, and
closeout. Smaller than Task 87 because:

- Two AMs instead of four
- Both target AMs (HNSW + DiskANN) share the same graph-
  traversal shape — Phase 1's iterator design transfers
  across slices
- The hard PG `amgettuple` semantics are designed once and
  reused

But meaningfully harder than a pure refactor because the
iterator-lifetime contract is load-bearing for correctness,
and the resort-buffer-pop heuristic affects recall.
