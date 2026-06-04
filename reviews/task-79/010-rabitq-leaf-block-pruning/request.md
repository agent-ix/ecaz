# Review Request: RaBitQ Leaf Block Pruning

Code commit under review: `b27202e08d02dda7fee8f81dd9f81d83e5c86a8f`

This checkpoint implements the first direct Task 79 candidate-surface reduction
path for the primary/default RaBitQ lane. It does not claim benchmark success
yet: the next packet must run `ecaz bench suite` on the 100k RaBitQ nprobe96
surface and compare candidate count, recall, and latency against the Task 78
baseline.

## What Changed

- Added build-time `ec_spire.leaf_block_rows`; `0` disables block summary
  materialization.
- Added scan-time `ec_spire.leaf_block_pruning_max_blocks_per_leaf`; `0`
  disables block pruning.
- Materialized V3 leaf block summaries during recursive RaBitQ leaf build by
  averaging source vectors for fixed-size row blocks and encoding each block
  mean with the existing assignment payload format.
- Added a RaBitQ-only scanner block selector that scores leaf summaries, keeps
  the top N blocks per leaf, and only scores rows inside those selected row
  ranges.
- Routed the top-graph candidate path through the common validated quantized
  leaf collector so the default high-recall top-graph lane can actually use
  the pruning path.
- Preserved fallback behavior: V2 leaves, leaves without summaries, unsupported
  scorers, or disabled GUCs continue to scan the full leaf.

TurboQuant is intentionally not enabled for this pruning path in this slice. It
remains a comparison/control target after the RaBitQ candidate-surface sweep
finds a viable operating point.

## Expected Cost Model

The intended sweep remains block sizes 32, 64, and 128 rows. With 64-row blocks,
summary scoring adds roughly one extra summary score per 64 candidate row scores
before pruning. To beat the Task 79 strong candidate target, the useful operating
point must prune substantially more row scores than that summary overhead while
keeping recall inside the task's target band.

## Validation

See `artifacts/manifest.md`.

- `cargo check -p ecaz`: pass
- `cargo test -p ecaz leaf_block_summaries_cover_rabitq_row_blocks`: pass
- `cargo test -p ecaz select_leaf_block_row_ranges_keeps_best_rabitq_blocks`:
  pass
- `cargo test -p ecaz prepare_single_level_snapshot_scan_candidates_uses_top_graph_when_enabled`:
  pass
- `cargo test -p ecaz collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer`:
  pass

## Next Required Evidence

Run `ecaz bench suite` for the 100k RaBitQ nprobe96 high-recall lane with
`ec_spire.leaf_block_rows` across 32/64/128 and scan budgets across a small
`leaf_block_pruning_max_blocks_per_leaf` grid. The acceptance decision should
use Task 79 gates: recall at or above the target band, candidates at or below
5.2M with strong target <=4.0M, and p50 latency <=45 ms or at least 25% better
than the Task 78 baseline.
