# Task 81: SPIRE Leaf Block Summary Format

Status: active (2026-06-04)
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
