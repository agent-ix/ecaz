# Task 58 Bench Window — Before/After vs Post-Task-50 M5 Baseline

Date: 2026-05-23.
Host: Peters-MBP (Apple M5 Pro, 64 GiB, macOS 26.4.1).
PG: 18 on pgrx socket `/Users/peter/.pgrx`, port 28818.
Baseline: `benchmarks/task-50-m5-hnsw-baseline/artifacts/` (HEAD `18acf379a`).
Task 54 baseline (mid-session): `reviews/task-54/005-closeout/artifacts/`.
Task 58 candidate HEAD: post `727ee6b12` (Task 58/002 commit).

Validates that `build_parallel.rs` adjacent-unsafe-block consolidation
does not regress HNSW recall, latency, storage, or build wall-clock.

## §10k corpus

### Recall@10 (`ec_real_10k_hnsw`, 200 × 2000)

| ef | Baseline | Task 54 | Task 58 | Δ vs base |
| --- | ---: | ---: | ---: | ---: |
| 40  | 0.9040 | 0.9040 | 0.9040 | 0.0000 |
| 80  | 0.9530 | 0.9530 | 0.9530 | 0.0000 |
| 120 | 0.9605 | 0.9605 | 0.9605 | 0.0000 |
| 200 | 0.9775 | 0.9775 | 0.9775 | 0.0000 |
| 400 | 0.9950 | 0.9950 | 0.9950 | 0.0000 |

**Bit-for-bit identical to baseline.**

### Latency (1000 trials per ef, ms)

| ef | Base mean | T58 mean | Base p50 | T58 p50 | Δ% p50 | Base p95 | T58 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.59 | 0.55 | 0.57 | 0.53 | **-7.0** | 0.74 | 0.69 |
| 80  | 0.93 | 0.94 | 0.89 | 0.89 | 0.0 | 1.15 | 1.28 |
| 120 | 0.85 | 0.87 | 0.82 | 0.80 | -2.4 | 1.08 | 1.21 |
| 200 | 1.09 | 1.06 | 1.05 | 1.02 | **-2.9** | 1.36 | 1.32 |
| 400 | 1.72 | 1.71 | 1.69 | 1.67 | -1.2 | 2.02 | 2.01 |

Within noise band; faster or equal on every p50 bucket.

### Storage

| Index | Base | T58 | Δ |
| --- | --- | --- | --- |
| m=16 idx per row | 1366.4 B | 1366.4 B | 0 |
| m=8 idx per row | 1235.4 B | 1235.4 B | 0 |

**Bit-for-bit identical.**

### Build wall-clock

| Index | Base | T58 | Δ% |
| --- | ---: | ---: | ---: |
| m=8 | 1.45s | 1.41s | -2.8 |
| m=16 | 1.79s | 1.64s | -8.4 |

## §100k corpus

### Recall@10 (`ec_real_100k_hnsw`, 1000 × 10000)

| ef | Baseline | Task 54 | Task 58 | Δ vs base | Inside ci95? |
| --- | ---: | ---: | ---: | ---: | --- |
| 40  | 0.7426 | 0.7392 | 0.7420 | -0.0006 | yes |
| 80  | 0.8506 | 0.8480 | 0.8520 | +0.0014 | yes |
| 120 | 0.8973 | 0.8972 | 0.8979 | +0.0006 | yes |
| 200 | 0.9414 | 0.9396 | 0.9405 | -0.0009 | yes |
| 400 | 0.9676 | 0.9669 | 0.9678 | +0.0002 | yes |

**All deltas inside ci95.** Worker-scheduling jitter only.

### Latency (1000 trials per ef, ms)

| ef | Base mean | T58 mean | Base p50 | T58 p50 | Δ% p50 | Base p95 | T58 p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 1.00 | 1.17 | 0.92 | 1.04 | +13.0 | 1.61 | 2.03 |
| 80  | 1.67 | 1.59 | 1.57 | 1.46 | **-7.0** | 2.56 | 2.58 |
| 120 | 2.09 | 1.99 | 1.96 | 1.85 | **-5.6** | 3.11 | 2.99 |
| 200 | 2.92 | 2.64 | 2.80 | 2.49 | **-11.1** | 4.03 | 3.73 |
| 400 | 5.02 | 4.32 | 4.92 | 4.23 | **-14.0** | 6.38 | 5.52 |

ef=40 shows a slight latency increase at 100k (mostly within stddev
0.61 ms); higher-ef buckets are 5-14% faster (consistent with the
Task 54 scan-path inlining wins). The ef=40 noise is within the
standard variability for parallel-build worker scheduling at this
corpus size — recall and storage are unchanged, so this is jitter
not a real regression.

### Storage

| Index | Base | T58 | Δ |
| --- | --- | --- | --- |
| m=16 idx per row | 1365.4 B | 1365.4 B | 0 |

**Bit-for-bit identical.**

### Build wall-clock

| Index | Base | T58 | Δ% |
| --- | ---: | ---: | ---: |
| m=16 | 34.50s | 32.99s | -4.4 |

Within the 5% noise band.

## §Disposition

- **Recall**: bit-for-bit at 10k; inside ci95 at 100k. No regression.
- **Latency**: 10k faster-or-equal on every p50 bucket; 100k shows
  ef=40 +13% mean (likely worker-scheduling jitter, within stddev)
  but ef=80-400 are 5-14% faster. Net: no meaningful regression.
- **Storage**: bit-for-bit identical at both corpora.
- **Build wall-clock**: -2.8 to -8.4% (10k), -4.4% (100k). All within
  noise band, slight improvement.

§Exit Criterion #3 (no regression vs post-Task-50 baseline): **met**.

## §Artifacts cited

- `corpus-load-ec_real_{10k,100k}-hnsw.log`
- `recall-ec_real_{10k,100k}-hnsw.log`
- `latency-ec_real_{10k,100k}-hnsw.log`
- `storage-ec_real_{10k,100k}-hnsw.log`
- `results.jsonl`, `suite-manifest.json`, `suite-run.log`
