# Task 122: TurboQuant Performance and Rerank Pipeline Optimization

Status: closeout requested (2026-06-27; outcome: keep experimental / promote follow-up, pending review in `reviews/task-122/010-closeout-keep-experimental/`)
Owner: coder (to be assigned). One coder, one branch.
Priority: 2 (post-Task 89 TurboQuant performance follow-up)

## Why

Task 89 closed TQ+ as deferred: calibration did not produce a durable
real-corpus recall win and should not be promoted or ported across AMs in its
current form. That does not settle TurboQuant itself. TurboQuant still has a
large installed implementation surface, but product performance has struggled
against the current strong baseline: very fast compressed candidate generation
with precise f32 rerank.

The next TurboQuant work should stop treating TQ as a standalone final scorer
and instead optimize where TQ can improve the end-to-end rerank pipeline:
comparable SIMD/block scoring, fused candidate handling, reduced f32 rerank
width, and fewer heap/vector fetches.

## Goal

Determine whether TurboQuant can improve product latency at matched recall
against the best RaBitQ + f32 rerank shape. The winning claim is not
"TurboQuant approximate scores are better"; it is:

- same recall;
- lower p50/p95/p99 latency;
- fewer f32 rerank/vector fetches or less IO;
- storage overhead justified by the latency/recall result.

If the measurements show no such path, close with evidence and keep RaBitQ +
f32 rerank as the preferred direction.

## Priority Order

Work in this order:

1. **Comparable TQ SIMD/block scoring** (highest priority).
2. **Fuse score + top-k + materialization**.
3. **Use TQ as a candidate reducer before f32 rerank**.
4. **Add block-level pruning**.
5. **Improve TQ quality only if it reduces rerank width**.
6. **Exploit storage/IO cases**.
7. **Benchmark against the right opponent**.

## Phase 1 — Comparable TQ SIMD / Block Scoring

Audit and close the hot-path gaps where TurboQuant still falls back to scalar or
non-comparable scoring while RaBitQ/f32 pipelines use batched/block paths.

Required work:

- Inventory TurboQuant no-QJL, QJL, and exact/rerank scoring paths across IVF,
  SPIRE, HNSW, and DiskANN.
- Mark each path as SIMD/block, scalar fallback, or structurally unbatched.
- Prioritize the production hot paths first: IVF/SPIRE TQ no-QJL and QJL, then
  graph AM traversal/rerank surfaces.
- Add or wire comparable block scorers before using latency as a promotion or
  rejection signal.
- Keep correctness gates byte-identical or recall-identical against the current
  scalar/reference scorer.

Exit evidence:

- Code or explicit inventory showing every measured TQ lane has comparable
  scorer status.
- Focused unit tests for scorer equivalence.
- `ecaz bench suite` evidence for any behavior-changing scorer path before
  claiming latency improvement.

## Phase 2 — Fuse Score + Top-K + Materialization

Avoid paying for candidate arrays and row materialization before the candidate
has survived local top-k pressure.

Required work:

- Identify scan loops that score a large batch, materialize all candidate
  metadata, and only then truncate.
- Score in blocks, update local top-k/frontier, and discard losing candidates
  before heap TID / row locator / rerank payload materialization.
- Preserve planner-visible and counter-visible behavior.
- Add counters for candidates scored, candidates retained, materializations
  avoided, and f32 rerank rows emitted.

Exit evidence:

- A/B evidence showing equal recall and lower materialization or rerank fetch
  counts.
- Latency evidence at 10k/50k/100k for the touched AM/quant lane.

## Phase 3 — TQ as Candidate Reducer Before f32 Rerank

Evaluate TQ as an intermediate compressed reranker that can reduce f32 rerank
width rather than replace f32 rerank.

Required work:

- Compare RaBitQ -> f32, TQ -> f32, and RaBitQ -> TQ -> f32 where the AM
  supports the pipeline.
- Sweep candidate budgets and f32 rerank widths at matched recall.
- Report whether TQ reduces f32 fetches enough to pay for its scoring cost.

Exit evidence:

- Recall/latency/storage/f32-fetch matrix at 10k/50k/100k.
- Decision: promote, iterate, or abandon TQ-as-stage-2 for the measured AM.

## Phase 4 — Block-Level Pruning

Use block summaries and score upper bounds to skip whole blocks before
per-candidate scoring.

Required work:

- Define recall-safe or explicitly bounded pruning rules.
- Start with IVF/SPIRE page/list/leaf blocks before graph traversal.
- Add counters for blocks considered, blocks pruned, candidates skipped, and
  top-k threshold at prune time.
- Prove pruned and unpruned results are byte-identical when the rule is claimed
  recall-safe.

Exit evidence:

- Correctness proof or bounded-risk contract.
- A/B suite showing candidate-surface reduction and latency effect.

## Phase 5 — Quality Improvements Only When They Reduce Rerank Width

Quality tweaks are in scope only when they reduce f32 rerank width or improve
candidate retention at matched recall.

Candidate ideas:

- OPQ/rotation variants.
- Per-list or per-cluster calibration instead of global TQ+.
- Query-aware or corpus-family-specific calibration.
- Better residual normalization.
- Score-error diagnostics before implementing new durable metadata.

Exit evidence:

- Matched-recall rerank-width reduction, not just approximate-score improvement.
- No public format or metadata change without ADR and benchmark evidence.

## Phase 6 — Storage / IO Cases

Measure cases where TQ might win by avoiding f32 vector reads or remote IO even
if raw scoring is not faster.

Required work:

- Cold-cache, remote, distributed, and index-side rerank-payload cases where
  f32 reads dominate.
- Track bytes read, heap/vector fetch count, and storage layout pressure.
- Do not claim a product win from warm local-only latency if the win depends on
  IO avoidance.

Exit evidence:

- Suite-driven hot/cold or local/remote comparison.
- Explicit storage/IO tradeoff table.

## Phase 7 — Correct Comparator Matrix

Benchmark against the product-relevant opponent:

- RaBitQ + f32 rerank width N.
- TQ + f32 rerank width N.
- RaBitQ -> TQ -> f32.
- TQ -> f32 with reduced rerank width.

Required metrics:

- recall@k and NDCG where available;
- p50/p95/p99 latency;
- candidate count and scored-candidate count;
- f32 fetch/rerank count;
- storage per row / per index;
- query-prep and materialization counters.

## Closeout Outcomes

One of:

- **Promote a TQ pipeline slice**: matched recall, lower latency, and clear
  fetch/materialization/storage rationale.
- **Keep experimental**: promising but limited to one AM, one corpus, or one
  cache/IO condition.
- **Redesign**: evidence points to a different TQ scoring/storage shape.
- **Defer**: TQ still cannot beat RaBitQ + f32 rerank on the product-relevant
  matrix.

## References

- Task 89 closeout: branch `task-89-ivf-tqplus-profile`, commit
  `e157af931`, packet `reviews/task-89/008-closeout-deferred/` on that branch.
- Task 87 candidate batching: `plan/tasks/87-candidate-batched-scoring-across-ams.md`
- Task 97 QJL block kernels: `plan/tasks/97-tq-qjl-block-kernel-family.md`
- Task 99 block-kernel closeout: `plan/tasks/99-cross-am-quant-isa-block-kernel-closeout.md`
- Task 111h persisted rerank format sweep: `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md`
- Task 112 lazy f32 rerank: `plan/tasks/112-ivf-lazy-heap-f32-rerank.md`
- Task 120 SPIRE coarse-rerank measurement program: `plan/tasks/120-spire-coarse-rerank-measurement-program.md`
