# Task 127: TQ dimension-progressive SIMD candidate pruning

Status: **proposed** (2026-06-30). Owner: coder (to be assigned). Priority: P2
TQ-scorer-speed follow-up to Task 124.

## Why

Stage-2 fully scores all ~1536 dims for every ~7,500 candidates even though only
`stage2_final_rerank_width` (~15) survive. A bound-prune cutoff exists ONLY on the
scalar fallback (`src/am/ec_ivf/scan.rs:1925`), never the SIMD batch path. This
cuts the *frequency* of the full calc.

## Scope

- Score a dimension prefix for all candidates, compute a partial-sum bound, drop
  candidates that cannot reach the running top-k threshold, and finish only the
  survivors. This is distinct from Task 124 pkt 012 (which fused *selection* and
  regressed) — this prunes *dimensions*.
- Recall is the gate: a correct bound makes this exact. Prove recall@10/NDCG@10 is
  unchanged on a real index and report the pruned fraction.

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
