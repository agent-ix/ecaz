# Task 94 Packet 007 Artifacts

- head SHA: `a14c73d24b9b3ba0a1661100ed38d20dd602915c`
- code checkpoint: `a14c73d24` (`Register grouped-PQ DiskANN and HNSW batch codecs`)
- task bucket: `reviews/task-94/007-grouped-pq-diskann-hnsw-codec-registration/`
- lane: coder-1 LUT lane
- fixture: local Rust unit tests plus matched local PG18 pg_test
- storage format / quant: DiskANN grouped-PQ prefilter codec, HNSW grouped-PQ scan codec
- rerank mode: not applicable for codec unit tests
- surface isolation: unit-test in-memory candidate batches, no database table surface
- timestamp: `2026-06-09T10:39:20-07:00`

## Artifacts

### `grouped-pq-codec-registration-tests.log`

- command: `cargo test grouped_pq --lib`
- result: pass
- key result lines:
  - `running 33 tests`
  - `test am::ec_diskann::quantizer::tests::diskann_grouped_pq_prefilter_codec_batch_uses_block_kernel_counters ... ok`
  - `test am::ec_hnsw::scan::tests::hnsw_grouped_pq_scan_codec_batch_uses_block_kernel_counters ... ok`
  - `test am::common::candidate_batch::tests::grouped_pq_batch_records_block_and_scalar_tail_counters ... ok`
  - `test tests::pg_test_pq_fastscan_binary_score_mode_bypasses_grouped_pq_scoring ... ok`
  - `test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 2017 filtered out; finished in 23.96s`

## Evidence Notes

- This is local-only evidence. No CI and no AWS/Graviton 4 run was performed.
- DiskANN and HNSW codec tests each score 39 grouped-PQ candidates through `QuantCodec::score_ip_batch`, exercising one block32 kernel path plus seven scalar-tail candidates.
- Direct block-kernel counter rows are verified under `(surface=diskann, quant=grouped_pq)` and `(surface=hnsw, quant=grouped_pq)`.
- The DiskANN traversal shell still scores discovered nodes one at a time; true traversal-level batching would require reshaping greedy descent and is not claimed by this packet.
