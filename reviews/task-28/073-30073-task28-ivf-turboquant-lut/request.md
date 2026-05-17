# Task 28 IVF TurboQuant LUT Scan

This packet records the first successful per-posting score-cost slice after
the borrowed posting scan. Commit `fd7e115` routes IVF TurboQuant scans through
the existing no-QJL 4-bit LUT scorer when the index dimensions make that lane
applicable. The change reuses the checked-in `ProdQuantizer` LUT kernel and
does not alter storage format or rerank semantics.

## Measurement Result

Fixture:

- Local PG18 scratch database `postgres`.
- Existing isolated n64 DBPedia-derived surfaces:
  - `task28_ivf_postopt10k_n64w25`
  - `task28_ivf_postopt25k_n64w25`
- `ecaz bench latency`, profile `ec_ivf`, `k=10`, `iterations=100`, sweep
  `nprobe=32,48`.
- `ecaz bench recall`, profile `ec_ivf`, `k=10`, `queries-limit=100`, sweep
  `nprobe=32,48`.
- Storage format: `turboquant`, using no-QJL 4-bit LUT scoring.
- Rerank mode: `heap_f32`, `rerank_width=25`.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

| surface | nprobe | recall@10 | ndcg@10 | p50 latency | p95 latency | prior p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k n64 w25 | 32 | 0.9800 | 0.9981 | 60.5 ms | 70.5 ms | 93.4 ms |
| 10k n64 w25 | 48 | 1.0000 | 1.0000 | 84.5 ms | 122.7 ms | 136.1 ms |
| 25k n64 w25 | 32 | 0.9840 | 0.9988 | 141.6 ms | 162.3 ms | 234.2 ms |
| 25k n64 w25 | 48 | 0.9990 | 1.0000 | 197.2 ms | 234.2 ms | 329.9 ms |

Prior p50 is from packet 30069/30070 after borrowed posting scan. Recall is
unchanged from packet 30070 at all four operating points.

## Interpretation

This confirms the main current bottleneck is the per-posting TurboQuant score
kernel, not heap rerank or downstream candidate management. The LUT scorer
cuts p50 by roughly 35-40% on both 10k and 25k while preserving recall.

## Artifacts

- `artifacts/latency_10k_n64w25_nprobe32_48.log`
- `artifacts/latency_25k_n64w25_nprobe32_48.log`
- `artifacts/recall_10k_n64w25_nprobe32_48.log`
- `artifacts/recall_25k_n64w25_nprobe32_48.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_heap_f32`
- `git diff --check`

## Recommendation

Keep the n64/nprobe48 point as the current high-recall local reference:
10k p50 84.5 ms at recall 1.0000, and 25k p50 197.2 ms at recall 0.9990.
The next IVF slice should compare this LUT lane with a tiled-LUT variant or
start PQ-FastScan through the existing quantizer dispatch seam. DiskANN remains
task 29 and is not included here.
