# Task 28 IVF Frontier Prune Trial

This packet records a negative trial for a safe post-score frontier prune in
the IVF posting scan. The trial kept the existing pooled dedup map, maintained
a streaming pre-rerank frontier, and skipped heap-TID/dedup work for postings
whose exact quantized score was already worse than the current frontier.

The trial was not landed. The code diff is preserved as
`artifacts/frontier_prune_trial.diff`, and the working tree was reverted before
this packet was committed.

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
- Storage format: `turboquant`.
- Rerank mode: `heap_f32`, `rerank_width=25`.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

| surface | nprobe | recall@10 | ndcg@10 | p50 latency | p95 latency |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k n64 w25 | 32 | 0.9800 | 0.9981 | 95.1 ms | 110.2 ms |
| 10k n64 w25 | 48 | 1.0000 | 1.0000 | 135.9 ms | 158.1 ms |
| 25k n64 w25 | 32 | 0.9840 | 0.9988 | 243.5 ms | 267.4 ms |
| 25k n64 w25 | 48 | 0.9990 | 1.0000 | 341.7 ms | 399.9 ms |

Recall remained byte-for-byte consistent with packet 30070. Latency did not
improve versus packet 30069/30070 and regressed slightly on the 25k surface
(`nprobe=32` p50 234.2 ms -> 243.5 ms; `nprobe=48` p50 329.9 ms -> 341.7 ms).

## Disposition

Do not land this post-score frontier prune. It avoids some heap-TID/dedup work
after the full quantized posting score is already computed, but the measured
bottleneck remains the full posting-score pass itself. This confirms the next
useful slice needs to reduce score-kernel cost or posting bytes scanned, not
only downstream hashmap churn.

## Artifacts

- `artifacts/frontier_prune_trial.diff`
- `artifacts/latency_10k_n64w25_nprobe32_48.log`
- `artifacts/latency_25k_n64w25_nprobe32_48.log`
- `artifacts/recall_10k_n64w25_nprobe32_48.log`
- `artifacts/recall_25k_n64w25_nprobe32_48.log`
- `artifacts/manifest.md`

## Validation

Validation run on the trial diff before reverting it:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_heap_f32`
- `git diff --check`

## Recommendation

Move to a real per-posting score-cost slice next: either a TurboQuant no-QJL
4-bit LUT/tiled-LUT IVF scan profile, or the first PQ-FastScan profile through
the existing quantizer dispatch seam. DiskANN remains task 29 and is not
included here.
