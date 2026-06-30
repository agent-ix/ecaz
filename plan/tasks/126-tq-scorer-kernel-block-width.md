# Task 126: TQ scorer kernel block/batch width

Status: **proposed** (2026-06-30). Owner: coder (to be assigned). Priority: P2
TQ-scorer-speed follow-up to Task 124.

## Why

The kernel block is hard-coded `BLOCK_WIDTH = 32` (`src/quant/lut32/mod.rs:15`).
Each dim's 64-byte LUT sub-table is loaded once per 32 candidates
(`src/quant/lut32/neon.rs:197`); a larger block amortizes each table load over
more candidates. Task 124 pkt 030 swept the *caller* flush width (the 256 cap),
not the kernel block.

## Scope

- Parameterize / raise `BLOCK_WIDTH` (e.g. 64, 128) and sweep it against the
  existing 32, keeping octet/tail correctness (`drivers.rs:53`, tail
  `mod.rs:595`). Report the per-width `ns/candidate` curve.

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
