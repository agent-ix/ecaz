# Task 94 Packet 026: F8 + Width-Cascade Integration

This packet covers the reopened Task 94 F8 slice as integrated with Task 101's shared width-cascade driver.

Code checkpoint: `11f8fc38113c08614c8ddca2073e54adcb018d81` (`Unify batch width cascade for quant kernels`)

## What Changed

- Replaced grouped-PQ AVX2 per-lane `_mm256_i32gather_ps` scoring with a f32 register-LUT path:
  - loads the two 8-entry f32 halves for each group;
  - uses `_mm256_permutevar8x32_ps` plus `_mm256_blendv_ps` to select arbitrary f32 LUT entries;
  - preserves the scalar f32 LUT anchor and existing bit-exact parity tests.
- Added grouped-PQ partial dispatch by padding sub-32 tails into a block and copying live lanes back.
- Updated grouped-PQ AM counter tests for full AVX2 partial coverage: on AVX2 hosts, the 39-candidate test cases now attribute all candidates to kernel rows; scalar-only hosts still permit 32 kernel + 7 scalar fallback.

## Local Evidence

Artifacts: `reviews/task-94/026-f8-width-cascade-integration/artifacts/`

- `cargo-test-candidate-batch.log`: `18 passed; 0 failed`
- `cargo-test-grouped-pq.log`: `35 passed; 0 failed`, including the PG18 grouped-PQ pg_test
- `suite-status-cli.log`: `[suite:task94-local-pqfastscan-matrix] completed=14 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `results.jsonl` / `results-report.jsonl`: 38 parsed result rows from the local suite

## Matrix Summary

Recall parity held exactly for IVF batch-off vs batch-on at all measured cells:

| Fixture | nprobe | batch-off recall | batch-on recall |
| --- | ---: | ---: | ---: |
| 10k IVF PqFastScan | 32 | 0.4620 | 0.4620 |
| 10k IVF PqFastScan | 64 | 0.4660 | 0.4660 |
| 25k IVF PqFastScan | 32 | 0.4870 | 0.4870 |
| 25k IVF PqFastScan | 64 | 0.4900 | 0.4900 |
| 100k IVF PqFastScan | 32 | 0.6350 | 0.6350 |
| 100k IVF PqFastScan | 64 | 0.6360 | 0.6360 |

Direct counter evidence is present in the suite results:

- IVF batch-on rows: `surface=ivf quant=grouped_pq isa=avx2 scalar_candidates=0`
- DiskANN forced grouped-PQ rows: `surface=diskann quant=grouped_pq isa=avx2 scalar_candidates=0`

## Latency Result

This packet does not claim a local end-to-end latency win. The local IVF batch-on cells were slower than batch-off in this run:

| Fixture | Sweep | batch-off p50 | batch-on p50 |
| --- | --- | ---: | ---: |
| 10k IVF | nprobe=32 | 41.6 ms | 47.4 ms |
| 10k IVF | nprobe=64 | 69.0 ms | 80.4 ms |
| 25k IVF | nprobe=32 | 89.2 ms | 103.3 ms |
| 25k IVF | nprobe=64 | 152.1 ms | 180.8 ms |
| 100k IVF | nprobe=32 | 280.5 ms | 341.4 ms |
| 100k IVF | nprobe=64 | 568.0 ms | 697.7 ms |

The value of this packet is correctness, full partial-width AVX2 counter coverage, and suite-local recall parity. The Graviton 4 SVE2/vector-length evidence remains deferred until AWS testing is approved.

Please review Task 94 packet 026 together with Task 101 packet 001.
