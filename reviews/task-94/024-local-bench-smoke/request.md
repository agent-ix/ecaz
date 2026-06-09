# Task 94 Packet 024: Local IVF PqFastScan Bench Smoke

## Summary

This packet fixes and proves a production IVF call-site gap for Task 94.

Earlier local latency runs against IVF PqFastScan emitted zero direct block-kernel counters because the production IVF posting scan only used the scratch SoA batch drain for TurboQuant and RaBitQ. `IvfQuantCodec::score_ip_batch` was registered, but the real PqFastScan scan path was still per-posting scalar.

Code checkpoint `187be1af1` wires PqFastScan into the existing IVF scratch SoA batch decode path:

- `IvfQuantizer::score_grouped_pq_batch_from_payloads(...)` builds a `CandidateBatch` from scratch payloads and calls `score_grouped_pq_batch_for(surface=ivf, quant=grouped_pq)`.
- `use_scratch_soa_batch_decode_for_format(...)` now admits `StorageFormat::PqFastScan`.
- `process_scratch_soa_postings(...)` drains PqFastScan scratch batches through the block-kernel helper before falling back to scalar per-posting scoring.

## Local Bench Evidence

All evidence is local PG18 only. No AWS and no GitHub CI were run.

Suite config: `artifacts/task94-local-ivf-pqfastscan-suite.json`

Suite result: `artifacts/suite-run-cli.log`, `artifacts/suite-manifest.json`, `artifacts/results.jsonl`

Fixture: `task94_local_pqfs10k_roff`, copied locally from the existing 10k IVF PqFastScan corpus with `rerank=off`.

Recall equality:

| nprobe | batch off recall@k | batch on recall@k | batch off ndcg@k | batch on ndcg@k |
| --- | ---: | ---: | ---: | ---: |
| 32 | 0.4275 | 0.4275 | 0.9022 | 0.9022 |
| 64 | 0.4325 | 0.4325 | 0.9038 | 0.9038 |

Latency smoke:

| nprobe | batch off p50/p95/p99 | batch on p50/p95/p99 |
| --- | --- | --- |
| 32 | 2.88 / 3.17 / 3.41 ms | 2.83 / 3.16 / 3.50 ms |
| 64 | 4.56 / 5.04 / 5.62 ms | 4.55 / 5.02 / 5.81 ms |

Direct `[block-kernel-counters]` rows preserved through the suite runner:

| nprobe | surface | quant | isa | kernel candidates | scalar candidates |
| --- | --- | --- | --- | ---: | ---: |
| 32 | ivf | grouped_pq | avx2 | 579392 | 0 |
| 32 | ivf | grouped_pq | scalar | 0 | 1775 |
| 64 | ivf | grouped_pq | avx2 | 1198080 | 0 |
| 64 | ivf | grouped_pq | scalar | 0 | 1920 |

## Validation

- `cargo test -p ecaz --lib pq_fastscan_payload_batch_scores_match_scalar_and_records_counters --no-default-features --features pg18`: passed.
- `cargo test -p ecaz --lib scratch_soa_batch_decode_gate_admits --no-default-features --features pg18`: passed.
- `cargo fmt --check`: passed, with the repo's existing stable-rust warnings for unstable rustfmt config keys.
- `ecaz bench suite audit`: passed, 4 steps.
- `ecaz bench suite run`: completed 4, failed 0.

## Remaining Scope

This packet proves the local IVF PqFastScan production batch path on AVX2. It does not claim final Task 94 closeout:

- Graviton 4 NEON/SVE2 runtime dispatch and vector-length evidence still need approved AWS execution.
- DiskANN end-to-end bench evidence still needs the final benchmark pass.
- Full real10k/50k/100k matrix remains closeout scope.
