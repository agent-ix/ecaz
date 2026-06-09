# Task 94 Phase 6 Checkpoint: DiskANN Grouped-PQ Traversal Batching

## Summary

This checkpoint wires grouped-PQ block-kernel batching into the DiskANN Vamana traversal path, not just the codec unit surface.

Code checkpoint:

- `bc0133f4` - `Batch DiskANN grouped-PQ traversal prefilter`

Artifact checkpoint:

- `b6f0271a2` - `Add Task 94 DiskANN traversal artifacts`

## What Changed

- Added a `VamanaPrefilter` trait to the DiskANN scan shell with a default scalar `score_batch` implementation for existing closure-based callers.
- Updated greedy-descent neighbor expansion to collect newly discovered neighbors for an expansion and score them through `score_batch` before pushing into the frontier.
- Implemented the batch override for `DiskannPreparedPrefilter`.
- Grouped-PQ DiskANN prefilter batches now route through `score_grouped_pq_batch_for(surface=Diskann, quant=GroupedPq)` and then negate scores to preserve DiskANN distance ordering.
- Updated normal scan, insert planning, vacuum repair, and profiled scan paths to use the batch-capable prefilter surface. The profiled path records prefilter timing/counts around the batch call.
- Added a scan-shell test proving greedy descent invokes the batch prefilter override.
- Added a prepared grouped-PQ prefilter test proving bit-exact negated scoring and DiskANN block-kernel counter attribution.

## Local Validation

Packet-local artifacts:

- `artifacts/diskann-greedy-batch-prefilter-test.log`
- `artifacts/diskann-grouped-pq-batch-tests.log`

Commands:

```text
cargo test greedy_descent_uses_batch_prefilter_for_neighbor_expansions --lib
cargo test diskann_grouped_pq --lib
```

Results:

```text
test am::ec_diskann::scan::tests::greedy_descent_uses_batch_prefilter_for_neighbor_expansions ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2051 filtered out; finished in 0.00s
```

```text
test am::ec_diskann::quantizer::tests::diskann_grouped_pq_prefilter_codec_batch_uses_block_kernel_counters ... ok
test am::ec_diskann::quantizer::tests::diskann_grouped_pq_prepared_prefilter_batch_scores_and_records_counters ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2049 filtered out; finished in 0.00s
```

Formatting:

```text
cargo fmt --check
```

Result: passed, with existing rustfmt warnings about nightly-only import grouping settings.

## Evidence Limits

- This is local-only unit evidence. No CI and no AWS/Graviton 4 run was performed.
- This proves DiskANN traversal-level batch dispatch structurally and counter attribution for the grouped-PQ prefilter path.
- End-to-end recall/latency benchmark evidence remains deferred until the approved final local/host closeout pass.
