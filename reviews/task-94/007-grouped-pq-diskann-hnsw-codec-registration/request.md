# Task 94 Phase 6 Checkpoint: DiskANN and HNSW Grouped-PQ Codec Registration

## Summary

This checkpoint adds grouped-PQ `QuantCodec::score_ip_batch` registrations for the DiskANN grouped-PQ prefilter codec and the HNSW grouped-PQ scan codec.

Code checkpoint:

- `a14c73d24` - `Register grouped-PQ DiskANN and HNSW batch codecs`

Artifact checkpoint:

- `a654677ac` - `Add Task 94 DiskANN HNSW codec artifacts`

## What Changed

- Added `CandidateBatchScoringSurface::Diskann` to the direct block-kernel counter matrix.
- Preserved the old four-surface `candidate_batch_scoring_snapshots()` compatibility API for `[task87-counters]`.
- Updated `block_kernel_scoring_snapshots()` to include DiskANN rows.
- Implemented `QuantCodec::score_ip_batch` for `DiskannGroupedPqPrefilterCodec`.
- Implemented `QuantCodec::score_ip_batch` for `HnswGroupedPqScanCodec`.
- Added test-only serialization around global counter tests to prevent parallel unit tests from resetting the shared counter matrix mid-assertion.
- Added DiskANN and HNSW 39-candidate batch tests that verify bit-exact grouped-PQ scores and direct block-kernel counter attribution.

## Local Validation

Packet-local artifact:

- `artifacts/grouped-pq-codec-registration-tests.log`

Command:

```text
cargo test grouped_pq --lib
```

Result:

```text
running 33 tests
test am::ec_diskann::quantizer::tests::diskann_grouped_pq_prefilter_codec_batch_uses_block_kernel_counters ... ok
test am::ec_hnsw::scan::tests::hnsw_grouped_pq_scan_codec_batch_uses_block_kernel_counters ... ok
test am::common::candidate_batch::tests::grouped_pq_batch_records_block_and_scalar_tail_counters ... ok
test tests::pg_test_pq_fastscan_binary_score_mode_bypasses_grouped_pq_scoring ... ok
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 2017 filtered out; finished in 23.96s
```

Formatting:

```text
cargo fmt --check
```

Result: passed, with existing rustfmt warnings about nightly-only import grouping settings.

## Evidence Limits

- This is local-only evidence. No CI and no AWS/Graviton 4 run was performed.
- This packet proves codec-level batch registration and counter attribution for DiskANN and HNSW grouped-PQ.
- DiskANN traversal currently scores discovered graph nodes one at a time through the existing greedy-descent prefilter closure. Reshaping that traversal into true block32 discovery batches is larger than this checkpoint and is not claimed here.
- The suite-latency `[block-kernel-counters]` result-emission gap remains the next Phase 6 checkpoint.
