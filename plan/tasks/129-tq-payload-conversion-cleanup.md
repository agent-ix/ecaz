# Task 129: TQ payload conversion / format-swap cleanup

Status: **proposed** (2026-06-30). Owner: coder (to be assigned). Priority: P2
TQ-scorer-speed follow-up to Task 124.

## Why

On the no-QJL path the payload still parses an unused f32 gamma per candidate
(`split_turboquant_payload`, `src/am/ec_ivf/rerank.rs:721`) and builds a
`gammas: Vec<f32>` (`rerank.rs:597`) the kernel never reads
(`src/am/common/candidate_batch/mod.rs:903`). The same payload is re-wrapped as
`Vec<&[u8]>` ~4x per call (`scan.rs:2681` -> `rerank.rs:604` ->
`quantizer.rs:791` -> `mod.rs:557`).

## Scope

- Drop the gamma parse + `gammas` allocation on the no-QJL path.
- Collapse the ~4 pointer-vector rebuilds into a single pass.

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
