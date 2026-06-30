# Task 125: TQ scorer LUT memory-traffic reduction (cache-blocking + smaller LUT)

Status: **proposed** (2026-06-30). Owner: coder (to be assigned). Priority: P2
TQ-scorer-speed follow-up to Task 124.

## Why

The dominant cost in the no-QJL 4-bit TQ scorer is streaming the ~96 KB per-query
LUT from L2 for every candidate-block. The arithmetic (one gather + one add per
(dim,candidate), `src/quant/lut32/neon.rs:140`) is trivial; the loop is memory
bound. This is the highest-value lever and directly explains the prior failures.

## Scope

- **Cache-block / tile over dimensions.** Restructure
  `score_width_cascade` (`src/am/common/candidate_batch/drivers.rs:35`) /
  `score_lut_no_qjl_4bit_block32` (`src/quant/lut32/mod.rs:89`) from
  "per candidate-block: walk all dims" to "per dim-tile: walk all candidate-blocks",
  keeping each LUT slice hot in L1 and accumulating per-candidate partial sums.
  Preserve per-candidate dim accumulation order so it stays bit-exact.
- **Shrink the LUT.** Build the prepared LUT as f16 (~48 KB) or int16 with f32
  accumulate (`build_prepared_query_lut`, `src/quant/prod.rs:1777`) to halve the
  bottleneck traffic. Measure recall (small LUT rounding feeds the exact f32
  final-15 pass, likely safe — but prove it).

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
