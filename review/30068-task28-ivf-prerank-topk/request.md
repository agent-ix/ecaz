# Task 28 IVF Pre-Rerank Top-K Candidate Bound

This packet records commit `2ea678c`, which bounds candidate sorting before
heap-f32 rerank. When `rerank=heap_f32` and `rerank_width > 0`, the scan now
keeps only the best `rerank_width` deduped candidates by quantized score before
running exact heap rerank. This preserves the previous semantics because the old
path sorted all deduped candidates and reranked only that same prefix.

## Measurement Result

Fixture:

- Local PG18 scratch database `postgres`.
- Existing isolated n64 DBPedia-derived surfaces from packet 30052:
  - `task28_ivf_postopt10k_n64w25`
  - `task28_ivf_postopt25k_n64w25`
- `ecaz bench latency`, profile `ec_ivf`, `k=10`, `concurrency=1`,
  `iterations=100`, sweep `nprobe=32,48`.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

| surface | nprobe | packet 30052 p50 | new p50 | packet 30052 p95 | new p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k n64 w25 | 32 | 98.1 ms | 95.4 ms | 105.9 ms | 104.3 ms |
| 10k n64 w25 | 48 | 140.2 ms | 140.4 ms | 148.2 ms | 157.4 ms |
| 25k n64 w25 | 32 | 246.2 ms | 240.9 ms | 261.0 ms | 254.1 ms |
| 25k n64 w25 | 48 | 351.4 ms | 340.3 ms | 383.2 ms | 357.3 ms |

The 10k signal is effectively neutral. The 25k signal is directionally useful:
p50 improves by about 2.2% at `nprobe=32` and about 3.2% at `nprobe=48`, with
p95 also lower at both points. This is not the large missing lever, but it
removes unnecessary full-candidate sorting in the path where exact rerank will
truncate the candidate set anyway.

## Artifacts

- `artifacts/latency_10k_n64w25_nprobe32_48.log`
- `artifacts/latency_25k_n64w25_nprobe32_48.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_heap_f32`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18,pg_test --no-default-features`
- `git diff --check`

## Recommendation

Keep this bounded-sort cleanup, but do not treat it as closing the Task 28
performance gap. The next scan slice should reduce posting-list scoring volume
or posting tuple handling itself. DiskANN remains task 29 and is not included.
