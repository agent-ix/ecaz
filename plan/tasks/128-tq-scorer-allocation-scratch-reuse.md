# Task 128: TQ scorer allocation + scratch reuse

Status: **proposed** (2026-06-30). Owner: coder (to be assigned). Priority: P2
TQ-scorer-speed follow-up to Task 124.

## Why

Each scorer call allocates `CandidateBatch` (2 Vecs), `mse_codes`, `gammas`,
`codes`, `scores`, `estimates` (`src/am/ec_ivf/quantizer.rs:646`,
`src/am/ec_ivf/rerank.rs:597-605`); stage-1 flushes many times per query (256
cap, `src/am/ec_ivf/scan.rs:363`), re-allocating each flush. `estimates` is
collected then negated/copied in a separate pass (`rerank.rs:605`). Task 124 pkt
030 removed one Vec; the rest remain.

## Scope

- Park reusable scratch (batch Vecs, codes, scores, estimates) in the scan opaque
  or a thread-local, reused across flushes instead of per-call allocation.
- Fold the `estimates` negate into the kernel output or the comparator to remove a
  full N-element pass.

## Out of Scope (hard)

- Do NOT introduce or extend any new rerank format/mode (`turboquant2`,
  `turboquant_binary`, reduced-dimension formats, etc.). This task optimizes the
  EXISTING production `turboquant` 4-bit no-QJL scorer only.
- Do NOT answer with f32/source comparisons, storage size, promotion, or
  product-competitiveness verdicts, or `nprobe`/frontier tuning. Those are not
  TQ-scorer speed and are out of scope.

## Required Evidence

- A TQ-internal before/after delta on the existing 4-bit no-QJL scorer
  (`ns/candidate` via the packet-028/030 profiler harness), plus the in-engine
  TQ scorer-elapsed delta on a real `ec_ivf turboquant` index.
- Recall safety: bit-exactness test vs the scalar reference where the change is
  exact; otherwise recall@10/NDCG@10 on a real index showing no regression.

Context: Task 124 reopen + hot-path analysis
(`reviews/task-124/035-post-scorer-product-suite/feedback/2026-06-30-03-reviewer.md`).
Key finding: the 4-bit TQ scorer is **memory/LUT-bound, not compute-bound** — the
per-query LUT is ~96 KB at 1536D (`src/quant/prod.rs:1779`), larger than L1,
streamed from L2 per candidate-block. That is why the inner-kernel compute
rewrites (Task 124 pkt 028) and naive prefetch (pkt 033) produced no win.
