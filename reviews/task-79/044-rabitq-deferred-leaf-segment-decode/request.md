# Task 79 Review Request: RaBitQ Deferred Leaf Segment Decode

## Summary

This packet reviews source checkpoint `129a00fab`, which defers RaBitQ leaf assignment segment decoding until after global leaf-block pruning selects the final row ranges.

The change is narrow:

- adds summary-only V2/V3/V4 leaf reads;
- adds selected-row-range segment reads that validate the segment chain but decode only intersecting assignment segments;
- changes non-sampled global block pruning to load summaries first, select global row ranges, then decode only selected ranges;
- leaves sampled global probing on the previous full-object path;
- adds a focused storage test for selected segment filtering.

## Result

This is a real Task 79 pass for the local RaBitQ lane.

Best row:

| storage | global blocks | candidates | p50 ms | p95 ms | recall@10 | returned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rabitq | 1152 | 3,673,383 | 35.293 | 40.600 | 0.9940 | 2000 |

Against the Task 78 RaBitQ nprobe96 baseline (`15,506,227` candidates, p50 `60.256 ms`, recall `0.9975`), this cuts candidates by `76.3%`, clears the strong `<=4.0M` candidate target, clears the `<=45 ms` latency target, and stays above the `0.9925` recall floor.

TurboQuant comparison was run on an isolated copied surface. It remains unreduced at `15,506,227` candidates and p50 `141.561 ms`; the endpoint identity explicitly reports `requires_rabitq_storage_format`. That matches the intended primary/default RaBitQ scope.

## Evidence

- Packet manifest: `reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/manifest.md`
- Suite config, RaBitQ: `reviews/task-79/044-rabitq-deferred-leaf-segment-decode/suite-rabitq-deferred-leaf-segment-decode.json`
- Suite config, TurboQuant: `reviews/task-79/044-rabitq-deferred-leaf-segment-decode/suite-turboquant-comparison.json`
- Compact results: `reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/compact-results.tsv`
- RaBitQ low-cap manifest: `reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-manifest.json`
- RaBitQ high-cap manifest: `reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-manifest-high-caps.json`
- TurboQuant manifest: `reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-manifest-turboquant-isolated.json`

```text
row	storage	global_blocks	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	gate
task78_baseline	rabitq	0	15506227	60.256	NA	0.9975	baseline
deferred_global1024	rabitq	1024	3265373	33.023	39.633	0.9920	fail_recall_by_0.0005
deferred_global1152	rabitq	1152	3673383	35.293	40.600	0.9940	pass_best
deferred_global1216	rabitq	1216	3877368	35.812	41.836	0.9940	pass_reference_cap
turboquant_global1152	turboquant	1152	15506227	141.561	153.951	0.9975	comparison_not_candidate_reduced
```

## Validation

- `cargo test leaf_partition_object_v2_selected_segment_reader_filters_by_row_range --no-default-features --features pg18`: passed.
- `cargo test global_leaf_block_row_ranges --no-default-features --features pg18`: passed, 4 tests.
- `cargo fmt --check`: passed with existing stable-rustfmt warnings.
- Local PG18 install/restart completed; installed backend SHA256 `c7bae4e16804615d8e853b7308d782c5a38741711e62fbcf1a68e73edc645ee8`.
- `ecaz bench suite audit/status/report` completed for RaBitQ and TurboQuant packet-local suites.
- All benchmark work was local PG18 only. No AWS was used.

## Reviewer Focus

Please review:

- whether the deferred summary/segment read split preserves storage validation invariants;
- whether the scan path handles empty selected ranges and delta routes correctly;
- whether the packet evidence is sufficient to treat Task 79's RaBitQ candidate/latency gates as met.
