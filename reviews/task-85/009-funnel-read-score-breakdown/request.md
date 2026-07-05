# Task 85 Review Request: Funnel Read/Score Breakdown

## Request

Review this Task 85 implementation checkpoint.

This is the first reopened Task 85 slice after packet 008. It does not claim a
product Pareto win. It widens the benchmark evidence surface needed for the
comprehensive same-recall latency program, specifically the object-read and
summary-scoring workstreams.

## Change

`ecaz bench spire-pipeline --funnel-output` now includes fields that the
extension already exposed through
`ec_spire_index_scan_leaf_candidate_snapshot` but the CLI was not projecting:

- `leaf_object_bytes`
- `leaf_summary_object_bytes`
- `leaf_row_object_bytes`
- `leaf_block_available_count`
- `leaf_block_selected_count`
- `leaf_block_skipped_count`
- `leaf_summary_score_nanos`
- `leaf_row_score_nanos`

The existing `leaf_object_read_nanos`, `candidate_score_nanos`,
`candidate_materialize_nanos`, and `candidate_heap_append_nanos` remain.

## Why This Is Task 85 Work

Task 85 needs to improve latency at the retained recall point, not by lowering
recall or hiding candidate growth. The current evidence points to object-read
and summary-scoring cost as the next place to work, but the previous funnel
JSONL could not separate:

- object bytes vs summary bytes vs row bytes;
- available/selected/skipped block counts;
- summary score time vs row score time.

This checkpoint makes those measurements durable in packet-local
`ecaz bench suite` artifacts so the next AWS 1M/q500 retained-recall run can
prove whether a layout/read-path change actually moves the correct bottleneck.

## Validation

Packet-local logs:

- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-test-ecaz-cli-spire-pipeline.log`
- `artifacts/aws-ec2-status-final.log`

Results:

- `cargo fmt --check`: passed, with stable-rustfmt warnings already present in
  this repo.
- `cargo test -p ecaz-cli spire_pipeline`: passed, `21 passed; 0 failed`.
- AWS `1m` instance `i-06ace3e95ab942623` is `stopped`.

## Next Required Task 85 Work

Run an AWS 1M/q500 retained block16/global1152 suite with this widened funnel
output, then use the per-query read/score breakdown to choose the first actual
read-path or summary-scoring optimization. This packet only makes that next
decision reviewable.
