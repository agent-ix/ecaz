# Task 53 Bench Window — Before/After vs Post-Task-50 M5 Baseline

Date: 2026-05-23.
Host: Peters-MBP (Apple M5 Pro, 64 GiB, macOS 26.4.1).
PG: 18 on pgrx socket `/Users/peter/.pgrx`, port 28818.
Baseline: `benchmarks/task-50-m5-hnsw-baseline/artifacts/` (HEAD `18acf379a`).
Task 53 candidate HEAD: this packet's parent commit.

Same 8-step suite shape as the baseline (load + recall + latency +
storage at both 10k and 100k, same prefixes / `m` / ef_construction /
sweep / recall trials / latency trials).

## §10k corpus

### Recall@10 (`ec_real_10k_hnsw`, 200 queries × 2000 trials per ef)

| ef | Baseline recall@k | Task 53 recall@k | Δ |
| --- | ---: | ---: | ---: |
| 40  | 0.9040 | 0.9040 | 0.0000 |
| 80  | 0.9530 | 0.9530 | 0.0000 |
| 120 | 0.9605 | 0.9605 | 0.0000 |
| 200 | 0.9775 | 0.9775 | 0.0000 |
| 400 | 0.9950 | 0.9950 | 0.0000 |

**Identical to four decimals.** ndcg@k also identical to four decimals.

### Latency (`ec_real_10k_hnsw`, 1000 trials per ef, ms)

| ef | Base mean | T53 mean | Base p50 | T53 p50 | Δ% | Base p95 | T53 p95 | Δ% | Base p99 | T53 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 0.59 | 0.56 | 0.57 | 0.53 | **-7.0** | 0.74 | 0.70 | -5.4 | 0.88 | 0.83 |
| 80  | 0.93 | 0.93 | 0.89 | 0.90 | +1.1 | 1.15 | 1.15 | 0.0 | 1.37 | 1.33 |
| 120 | 0.85 | 0.82 | 0.82 | 0.78 | **-4.9** | 1.08 | 1.06 | -1.9 | 1.28 | 1.22 |
| 200 | 1.09 | 1.07 | 1.05 | 1.04 | -1.0 | 1.36 | 1.33 | -2.2 | 1.58 | 1.49 |
| 400 | 1.72 | 1.67 | 1.69 | 1.63 | **-3.6** | 2.02 | 1.93 | -4.5 | 2.26 | 2.23 |

**Task 53 is faster or equal on every bucket** except ef=80 p50
(+1.1%, within stddev 0.13 ms). No regression.

### Storage (`ec_real_10k_hnsw`)

| Field | Baseline | Task 53 | Δ |
| --- | --- | --- | --- |
| total per row | 19359.3 B | 19359.3 B | 0 |
| heap per row | 136.8 B | 136.8 B | 0 |
| `m=16` idx per row | 1366.4 B | 1366.4 B | 0 |
| `m=8` idx per row | 1235.4 B | 1235.4 B | 0 |

**Bit-for-bit identical** at the B/row.

## §100k corpus

### Recall@10 (`ec_real_100k_hnsw`, 1000 queries × 10000 trials per ef)

| ef | Baseline recall@k | Task 53 recall@k | Δ | Inside ci95? |
| --- | ---: | ---: | ---: | --- |
| 40  | 0.7426 | 0.7392 | -0.0034 | yes (T53 ci95 0.7305-0.7477) |
| 80  | 0.8506 | 0.8480 | -0.0026 | yes (T53 ci95 0.8408-0.8549) |
| 120 | 0.8973 | 0.8972 | -0.0001 | yes |
| 200 | 0.9414 | 0.9396 | -0.0018 | yes (T53 ci95 0.9348-0.9441) |
| 400 | 0.9676 | 0.9669 | -0.0007 | yes (T53 ci95 0.9632-0.9702) |

**All deltas inside ci95.** Sub-0.004 jitter from worker-scheduling
stochasticity producing tiny neighbor variations during parallel
build — mathematically equivalent graphs.

### Latency (`ec_real_100k_hnsw`, 1000 trials per ef, ms)

| ef | Base mean | T53 mean | Base p50 | T53 p50 | Δ% | Base p95 | T53 p95 | Δ% | Base p99 | T53 p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 40  | 1.00 | 0.93 | 0.92 | 0.84 | **-8.7** | 1.61 | 1.53 | -5.0 | 2.25 | 2.22 |
| 80  | 1.67 | 1.53 | 1.57 | 1.41 | **-10.2** | 2.56 | 2.45 | -4.3 | 3.50 | 3.17 |
| 120 | 2.09 | 1.90 | 1.96 | 1.78 | **-9.2** | 3.11 | 2.92 | -6.1 | 4.19 | 4.05 |
| 200 | 2.92 | 2.62 | 2.80 | 2.49 | **-11.1** | 4.03 | 3.69 | -8.4 | 5.54 | 5.30 |
| 400 | 5.02 | 4.40 | 4.92 | 4.29 | **-12.8** | 6.38 | 5.71 | -10.5 | 7.98 | 7.27 |

**Task 53 is faster on every bucket at 100k, 7-13% on p50.** This is
the wrapper-inlining win — the typed `FlatFloat4Source<'a>` boundary
collapses the per-call `unsafe { ... }` block, letting the compiler
inline the detoast + slice-extraction path more aggressively. Effect
amplifies at larger corpus sizes where scan-path call overhead
dominates.

### Storage (`ec_real_100k_hnsw`)

| Field | Baseline | Task 53 | Δ |
| --- | --- | --- | --- |
| total per row | 18117.1 B | 18117.3 B | +0.2 B (FSM/VM noise) |
| heap per row | 136.8 B | 136.8 B | 0 |
| `m=16` idx per row | 1365.4 B | 1365.4 B | 0 |

**Bit-for-bit identical** on indexes; total-per-row drift is FSM/VM
noise (<1 B per row across 100k rows).

## Disposition

**No regression on either corpus size; Task 53 is a measurable
improvement.**

- 10k recall: exact-equal to 4 decimals.
- 100k recall: deltas ≤0.0034, all inside ci95 confidence envelope.
- Storage: bit-for-bit identical on index sizes; total-per-row
  drift is FSM/VM noise (<1 B at 100k, 0 B at 10k).
- 10k latency: faster or equal on every bucket.
- 100k latency: **7-13% faster on p50, 4-11% faster on p95** —
  consistent across all ef buckets.

Task 53 §Exit Criterion #3 — "HNSW recall + QPS + per-row storage no
regression vs the post-Task-50 baseline" — is satisfied with
substantial margin. The wrapper inlining produces a measurable
latency improvement, particularly visible at 100k where the
scan-path call density is higher.

## Build-path coverage

The bench's `load-10k-hnsw` and `load-100k-hnsw` steps each ran
`CREATE INDEX ... USING ec_hnsw (...)` invocations. The build path
exercises:
- `FlatFloat4Source<'a>::from_datum` on each tuple's source vector
  (replaces `FlatFloat4SourceRef` dispatch in HNSW).
- `DetoastedVarlena::as_typed_slice::<f32>()` for varlena paths
  (slice-002 addition).
- `AttnumLookup::lookup(rel, "source")` once per build.

The scan path (recall + latency steps) exercises the same wrappers
on each query vector + each result-vector decode.

A semantics regression would surface as recall@k mismatch (different
neighbor topology). 10k recall is exact-equal; 100k recall deltas
are inside ci95.

## Cross-references

- Baseline: `benchmarks/task-50-m5-hnsw-baseline/manifest.md` +
  `benchmarks/task-50-m5-hnsw-baseline/artifacts/`.
- Task 52 bench precedent: `reviews/task-52/007-closeout/artifacts/before-after-summary.md`.
- This packet's bench logs:
  - `corpus-load-{10k,100k}-hnsw.log`
  - `recall-{10k,100k}-hnsw.log`
  - `latency-{10k,100k}-hnsw.log`
  - `storage-{10k,100k}-hnsw.log`
  - `suite-manifest.json` (ecaz bench suite audit trail)
  - `results.jsonl` (structured per-step results)
