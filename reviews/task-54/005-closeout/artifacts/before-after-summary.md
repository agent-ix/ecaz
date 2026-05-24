# Task 54 Bench Window — Before/After vs Post-Task-50 M5 Baseline

Date: 2026-05-23.
Host: Peters-MBP (Apple M5 Pro, 64 GiB, macOS 26.4.1).
PG: 18 on pgrx socket `/Users/peter/.pgrx`, port 28818.
Baseline: `benchmarks/task-50-m5-hnsw-baseline/artifacts/` (HEAD `18acf379a`).
Task 54 candidate HEAD: this packet's parent commit.

Same 8-step suite shape as the baseline (load + recall + latency +
storage at both 10k and 100k, same prefixes / `m` / ef_construction /
sweep / recall trials / latency trials).

## §10k corpus

### Recall@10 (`ec_real_10k_hnsw`, 200 queries × 2000 trials per ef)

| ef | Baseline recall@k | Task 54 recall@k | Δ |
| --- | ---: | ---: | ---: |
| 40  | 0.9040 | 0.9040 | 0.0000 |
| 80  | 0.9530 | 0.9530 | 0.0000 |
| 120 | 0.9605 | 0.9605 | 0.0000 |
| 200 | 0.9775 | 0.9775 | 0.0000 |
| 400 | 0.9950 | 0.9950 | 0.0000 |

**Identical to four decimals.** ndcg@k also identical to four decimals.

### Latency (`ec_real_10k_hnsw`, 1000 trials per ef, ms)

| ef | Base mean | T54 mean | Base p50 | T54 p50 | Δ% | Base p95 | T54 p95 | Δ% | Base p99 | T54 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.59 | 0.56 | 0.57 | 0.54 | **-5.3** | 0.74 | 0.68 | -8.1 | 0.88 | 0.82 |
| 80  | 0.93 | 0.91 | 0.89 | 0.88 | -1.1 | 1.15 | 1.14 | -0.9 | 1.37 | 1.29 |
| 120 | 0.85 | 0.83 | 0.82 | 0.79 | **-3.7** | 1.08 | 1.05 | -2.8 | 1.28 | 1.27 |
| 200 | 1.09 | 1.04 | 1.05 | 1.00 | **-4.8** | 1.36 | 1.30 | -4.4 | 1.58 | 1.55 |
| 400 | 1.72 | 1.65 | 1.69 | 1.61 | **-4.7** | 2.02 | 1.96 | -3.0 | 2.26 | 2.26 |

**Task 54 is faster or equal on every bucket.** No regression.

### Storage (`ec_real_10k_hnsw`)

| Field | Baseline | Task 54 | Δ |
| --- | --- | --- | --- |
| `m=16` idx per row | 1366.4 B | 1366.4 B | 0 |
| `m=8` idx per row | 1235.4 B | 1235.4 B | 0 |

**Bit-for-bit identical** at the B/row.

### Build wall-clock (`ec_real_10k_hnsw`, rebuilt from scratch)

| Index | Baseline | Task 54 | Δ% |
| --- | ---: | ---: | ---: |
| m=8 | 1.45s | 1.40s | -3.4 |
| m=16 | 1.79s | 1.65s | -7.8 |

Both within the established noise band for HNSW build at this scale;
slight improvement consistent with the scan-path inlining wins
(scan paths share buffer-guard primitives with the build path).

## §100k corpus

### Recall@10 (`ec_real_100k_hnsw`, 1000 queries × 10000 trials per ef)

| ef | Baseline recall@k | Task 54 recall@k | Δ | Inside ci95? |
| --- | ---: | ---: | ---: | --- |
| 40  | 0.7426 | 0.7392 | -0.0034 | yes (T54 ci95 0.7305-0.7477) |
| 80  | 0.8506 | 0.8480 | -0.0026 | yes (T54 ci95 0.8408-0.8549) |
| 120 | 0.8973 | 0.8972 | -0.0001 | yes |
| 200 | 0.9414 | 0.9396 | -0.0018 | yes (T54 ci95 0.9348-0.9441) |
| 400 | 0.9676 | 0.9669 | -0.0007 | yes (T54 ci95 0.9632-0.9702) |

**All deltas inside ci95.** Sub-0.004 jitter from worker-scheduling
stochasticity during parallel build (same pattern as Task 53's bench
window).

### Latency (`ec_real_100k_hnsw`, 1000 trials per ef, ms)

| ef | Base mean | T54 mean | Base p50 | T54 p50 | Δ% | Base p95 | T54 p95 | Δ% | Base p99 | T54 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 1.00 | 0.89 | 0.92 | 0.81 | **-12.0** | 1.61 | 1.45 | -9.9 | 2.25 | 2.14 |
| 80  | 1.67 | 1.47 | 1.57 | 1.36 | **-13.4** | 2.56 | 2.32 | -9.4 | 3.50 | 3.10 |
| 120 | 2.09 | 1.86 | 1.96 | 1.73 | **-11.7** | 3.11 | 2.83 | -9.0 | 4.19 | 3.96 |
| 200 | 2.92 | 2.50 | 2.80 | 2.39 | **-14.6** | 4.03 | 3.45 | -14.4 | 5.54 | 4.87 |
| 400 | 5.02 | 4.22 | 4.92 | 4.10 | **-16.7** | 6.38 | 5.47 | -14.3 | 7.98 | 7.14 |

**Task 54 is faster on every bucket at 100k, 12-17% on p50.** The
wins compound at higher ef and at the larger corpus where the
scan-path inner loop dominates. Same effect-size as Task 53's
post-P6-wrapper window (which saw 7-13% on the same lanes).

### Storage (`ec_real_100k_hnsw`)

| Field | Baseline | Task 54 | Δ |
| --- | --- | --- | --- |
| `m=16` idx per row | 1365.4 B | 1365.4 B | 0 |
| corpus_pkey per row | 45.1 B | 45.1 B | 0 |

**Bit-for-bit identical** at the B/row.

### Build wall-clock (`ec_real_100k_hnsw`, rebuilt from scratch)

| Index | Baseline | Task 54 | Δ% |
| --- | ---: | ---: | ---: |
| m=16 | 34.50s | 32.23s | -6.6 |

Within the established 5%-ish noise band for HNSW build at this
scale; small improvement consistent with the scan/buffer-guard
inlining wins.

## §Disposition

- **Recall**: bit-for-bit identical at 10k; within ci95 at 100k.
  No regression.
- **Latency**: faster or equal on every bucket at 10k; **12-17%
  faster on p50 at 100k**. Same wrapper-inlining mechanism as Task
  53's source.rs migration; effect scales with corpus size.
- **Storage**: bit-for-bit identical at both 10k and 100k. The P3
  wrappers do not change on-disk format.
- **Build wall-clock**: within noise band on both corpora (10k m=8
  -3.4 %, 10k m=16 -7.8 %, 100k m=16 -6.6 %), all improvements.

§Exit Criterion #3 (no regression vs post-Task-50 baseline): **met**.

## §Artifacts cited

- `recall-ec_real_10k-hnsw.log`, `latency-ec_real_10k-hnsw.log`,
  `storage-ec_real_10k-hnsw.log`, `corpus-load-ec_real_10k-hnsw.log`
- `recall-ec_real_100k-hnsw.log`, `latency-ec_real_100k-hnsw.log`,
  `storage-ec_real_100k-hnsw.log`, `corpus-load-ec_real_100k-hnsw.log`
- `results.jsonl` (suite-run merged results)
- `suite-manifest.json` (suite-run timestamps + step provenance)
- `suite-run.log` (CLI mirror)
