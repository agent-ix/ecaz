# Review Request: Task 120 Phase 2 RaBitQ Block Pruning

- task: Task 120
- packet: `reviews/task-120/008-phase2-rabitq-block-pruning`
- measured head SHA: `f919874bfa6591c7b6722b7cc27b5aafa64cf0ef`
- code change under review: none in this packet; this is Phase 2 measurement evidence

## Summary

This packet runs the local Phase 2 recursive RaBitQ SPIRE block-pruning matrix
on staged real corpora at `10k`, `50k`, and `100k`.

The initial flat SPIRE attempt did not exercise leaf block summaries
(`leaf_block_available_count=0`), so the final decision-grade matrix uses
recursive SPIRE (`recursive_fanout=8`) with `leaf_block_rows=64` and
`leaf_block_summary_representatives=2`.

The A/B compares:

- `full`: no leaf block pruning caps.
- `l2`: `ec_spire.leaf_block_pruning_max_blocks_per_leaf=2`, with global caps
  disabled.

Final suite status:

```text
completed=25 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

`l2` does prune selected leaf blocks and candidates, but it is not recall-safe.
At `nprobe=32`:

```text
10k  full recall=0.9965 mean=8.49 ms p95=9.51 ms candidates=520143
10k  l2   recall=0.9855 mean=8.12 ms p95=8.99 ms candidates=483392
50k  full recall=0.9725 mean=16.0 ms p95=18.4 ms candidates=2326779
50k  l2   recall=0.5505 mean=11.8 ms p95=13.0 ms candidates=784248
100k full recall=0.9310 mean=26.8 ms p95=30.4 ms candidates=5165224
100k l2   recall=0.5060 mean=14.9 ms p95=16.3 ms candidates=806794
```

The 50k and 100k candidate cuts are large (`66.29%` and `84.38%` at
`nprobe=32`), but the recall collapse is also large. This rules out
`leaf_block_pruning_max_blocks_per_leaf=2` as a recall-preserving Phase 2 path.

Storage for the recursive RaBitQ surfaces was:

```text
10k index=9.7 MiB total=168.8 MiB per_row_total=17695.5 B
50k index=42.1 MiB total=837.0 MiB per_row_total=17553.0 B
100k index=82.5 MiB total=1.6 GiB per_row_total=17534.0 B
```

## Recommendation

Do not promote the `l2` per-leaf block cap as a SPIRE recall-sensitive policy.
For Phase 2, keep the recursive RaBitQ `full` surface as the baseline and treat
leaf-block pruning as an unsafe candidate-reduction knob until a softer or
adaptive policy proves otherwise.

The next useful Phase 2 experiment is a less aggressive block policy, such as a
higher per-leaf cap, a recall-aware recovery/global fallback, or a repaired
target-block-rank diagnostic that can explain which true-neighbor blocks are
being discarded before final recall is measured again.

## Evidence

See `artifacts/manifest.md` for commands, provenance, and key result lines.

Primary summary artifacts:

- `artifacts/recall-latency-summary.txt`
- `artifacts/block-pruning-summary.txt`
- `artifacts/block-pruning-comparison.txt`
- `artifacts/storage-summary.txt`
- `artifacts/suite-results.jsonl`
- `artifacts/suite-manifest.json`
- `artifacts/suite-status.log`

The raw per-query SPIRE pipeline JSONL dumps were pruned after reviewer
feedback because the cited decision evidence is preserved in the compact
summary files above plus `suite-results.jsonl`. The aborted flat `g128` run is
not cited as decision-grade evidence.

## Caveat

This packet deliberately does not claim route-vs-leaf-vs-block truth attribution
from target-block-rank output. The Phase 1 reviewer flagged that diagnostic as
non-decision-grade. The Phase 2 conclusion here rests on A/B final recall,
latency, storage, and selected-leaf-block candidate counts.

The suite ran through `target/debug/ecaz`, so the latency numbers are local
debug-build reference data only. The no-go recommendation rests on the measured
recall collapse; any future promotion packet must remeasure latency with a
release backend/binary.

## Closeout Status

This satisfies the local Phase 2 A/B evidence for the tested recursive RaBitQ
leaf-block policy only. Task 120 remains open for Phase 3 budget policy, the
Phase 1 attribution fix/rerun, Phases 4 through 6, and AWS 1M evidence before
any SPIRE product-default or product-claim decision.
