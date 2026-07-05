# Task 94 Phase 6 Checkpoint: Grouped-PQ IVF Batch Registration

## Summary

This checkpoint wires the canonical grouped-PQ block32 kernel into the shared candidate-batch path and registers it for IVF `PqFastScan` through `QuantCodec::score_ip_batch`.

Code checkpoint:

- `e3bc6c621` - `Register grouped-PQ IVF batch scoring`

Artifact checkpoint:

- `a47c65845` - `Add Task 94 IVF batch artifacts`

## What Changed

- Added `score_grouped_pq_batch_for(...)` in `src/am/common/candidate_batch.rs`.
- Routes `batch.len() >= 32` through `src/quant/grouped_pq_block::score_grouped_pq_block32`.
- Scores scalar tails with `score_grouped_pq_scalar`.
- Records grouped-PQ block-kernel counters under the caller-provided surface and dispatched ISA.
- Records scalar tails under `(surface, grouped_pq, scalar)` via `record_block_scalar_score_for`.
- Registered IVF `PqFastScan` in `IvfQuantCodec::score_ip_batch`.
- Expanded the IVF grouped-PQ codec parity test to 39 candidates so it exercises one block plus one scalar tail.

## Local Validation

Packet-local artifact:

- `artifacts/grouped-pq-batch-tests.log`

Command:

```text
cargo test grouped_pq_batch --lib
```

Result:

```text
running 6 tests
test am::common::candidate_batch::tests::grouped_pq_batch_records_block_and_scalar_tail_counters ... ok
test am::ec_ivf::quantizer::tests::common_quant_codec_grouped_pq_batch_is_bit_exact_with_scalar ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2042 filtered out; finished in 0.08s
```

Formatting:

```text
cargo fmt --check
```

Result: passed, with existing rustfmt warnings about nightly-only import grouping settings.

## Evidence Limits

- This is local-only evidence. No CI and no AWS/Graviton 4 run was performed.
- This packet covers IVF registration plus the shared grouped-PQ candidate-batch helper.
- DiskANN, HNSW applicability, and the suite-latency `[block-kernel-counters]` result emission gap remain Phase 6 follow-up work.
