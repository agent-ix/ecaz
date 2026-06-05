# Task 81: SPIRE Leaf Block Summary Format

Status: complete (2026-06-05)
Owner: coder (to be assigned). One coder, one branch.
Priority: 0 (Task 80 closeout follow-up)

## Why

Task 80 proved that local `leaf_block_rows=16` block-cap tuning can look
promising at 100k, but does not preserve the latency/candidate win at AWS 1M.
The successful AWS 1M continuation row for global block cap `2048` improved
recall to `0.9914-0.9918`, but scored about `16.38M` candidates over 500
queries and p50 stayed at `308.087-334.867 ms`, worse than the old tg96
comparator's `268.824 ms`.

The remaining bottleneck is still inside selected leaves: selected routes expose
too many rows before SPIRE has a compact query-aware way to skip row blocks.

## Scope

Implement ADR-074-style leaf-local block summaries as an explicit SPIRE format
and build contract.

The implementation should:

- persist compact scoreable summaries for deterministic row blocks inside each
  SPIRE leaf;
- score block summaries before reading or scoring row payload blocks;
- select row blocks per query using an explicit block or row budget;
- preserve full-leaf fallback for old formats, disabled GUCs, diagnostics, and
  malformed summary metadata;
- expose selected block counts, skipped block counts, summary-score time,
  row-score time, summary bytes, and row bytes;
- include an explicit format/version plan before changing durable layout.

## Required Evidence

- Use `ecaz bench suite` for all matrix runs.
- Local PG18 100k RaBitQ evidence must compare old full-leaf behavior against
  block-summary pruning at matched recall.
- AWS 1M follow-up should run only after a local row clears the Task 80
  candidate and latency gate shape.
- Packet evidence must live under `reviews/task-81/` with
  `artifacts/manifest.md`.

## Gates

- Recall@10 should stay at or above `0.9925`, or the packet must prove a better
  Pareto point.
- Scored candidates must materially beat Task 79/80's retained high-recall
  surface and should target `<=4.0M` over the 100k / 200-query lane.
- p50 must be `<=45 ms` locally or at least 25% better than the Task 78
  `60.256 ms` baseline.
- AWS 1M accepted rows must improve recall over the old tg96 row without
  increasing the candidate surface relative to the old `9,213,846` q500 shape.

## Exit Criteria

- Format/version decision recorded in an ADR or ADR update.
- One block-summary implementation lands with PG18 validation and packet-backed
  local candidate reduction at matched recall.
- AWS 1M packet captured only after the local gate clears.
- Closeout updates this task status and cites the accepted packet.

## Closeout

Task 81 is closed on the corrected acceptance bar clarified during closeout:
beat Task 79's optimized candidate surface on latency while preserving the same
recall and candidate count. The old full-leaf `15.5M` row is retained only as a
mechanism comparator, not as the success baseline.

Accepted evidence:

- Format/version decision: `spec/adr/ADR-074-spire-leaf-local-block-pruning.md`
  and packet `reviews/task-81/001-block-summary-diagnostics/`.
- Local corrected comparison:
  `reviews/task-81/004-local-nprobe-block-summary-gate/` shows the Task 81
  tg256/nprobe96 row at `3,672,619` candidates, p50 `32.212 ms`, recall@10
  `0.9945`, beating Task 79's accepted local `global1152` row
  (`3,673,383` candidates, p50 `35.293 ms`, recall@10 `0.9940`).
- AWS corrected comparison:
  `reviews/task-81/007-aws-100k-task79-comparison/` reruns Task 79's retained
  AWS 100k/q200 surface under the Task 81 branch and records `3,672,619`
  candidates, p50 `32.023 ms`, p95 `32.940 ms`, p99 `33.315 ms`, recall@10
  `0.9945`, beating Task 79's accepted AWS row (`35.199 ms` p50, `36.203 ms`
  p95, `36.591 ms` p99) at unchanged recall and candidates.

The earlier retained q500 1M attempts in packets 003 and 005 remain negative
scale evidence: they did not improve recall over `0.9832` at the old q500
candidate shape. Packet 006 records why the attempted q50 rank attribution path
was abandoned before this task pivoted to the Task 79 acceptance comparison.
