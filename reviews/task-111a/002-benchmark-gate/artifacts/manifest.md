# Task 111a Benchmark Gate Manifest

- head SHA: `2d2b69444a5aa7ba0754930bba36fefbcfec9cfb`
- branch: `task-111-ivf-dense-posting-block-layout`
- task bucket: `reviews/task-111a/002-benchmark-gate`
- timestamp: `2026-06-17T06:10:15Z`
- backend install SHA: `ed390829194fa23752aa0e235888fbb097da23cbbf57fcdd4e61bf300165568a`
- database: `task111a_dense_bench`
- host / port: `/home/peter/.pgrx` / `28818`
- PostgreSQL: `18.3`
- suite config: `artifacts/task111a-dense-coalescing-suite.json`
- suite config sha256: `d4adf4a127a6d6c2ede8af31f764a58ac6cea6706df9e19ae63b5aadf93b342e`
- suite manifest: `artifacts/suite/suite-manifest.json`
- structured results: `artifacts/suite/results.jsonl`
- parsed report output: `artifacts/suite/results-report.jsonl`
- lane: local PG18 benchmark gate, warm cache, concurrency 1
- fixture scales: real 50k and real 100k
- storage formats: TurboQuant and RaBitQ `quant_bits=1`
- surfaces:
  - row: `dense_posting_blocks=0`
  - dense-old: `dense_posting_blocks=1`, `ec_ivf.dense_posting_coalescing=off`
  - dense+A: `dense_posting_blocks=1`, `ec_ivf.dense_posting_coalescing=on`
- suite status: 60 completed, 0 failed, 0 skipped, 0 missing artifacts, 0 stale
- isolated one-index-per-table surfaces: yes

## Corpus Inputs

50k fixture was derived from the existing 100k prepared TSV via:

```text
cargo run -q -p ecaz-cli -- corpus subset --profile ec_real_50k --source-corpus-file data/task106_full_sweep_100k/ec_real_100k_corpus.tsv --source-manifest-file data/task106_full_sweep_100k/ec_real_100k_manifest.json --output-dir data/task111a_real50k
```

The corpus TSVs are regenerable local inputs and are not committed in this
packet.

| Scale | Corpus rows | Query rows | Corpus SHA256 | Query SHA256 |
| --- | ---: | ---: | --- | --- |
| 50k | 50000 | 1000 | `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` |
| 100k | 100000 | 1000 | `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` |

The suite created local truth caches under `artifacts/suite/truth-*.json` for
recall reuse. They are regenerable ground-truth caches and are intentionally
not part of the committed packet.

## Commands

```text
target/release/ecaz --log-file reviews/task-111a/002-benchmark-gate/artifacts/suite-audit.log bench suite audit --config reviews/task-111a/002-benchmark-gate/artifacts/task111a-dense-coalescing-suite.json --database task111a_dense_bench --host /home/peter/.pgrx --port 28818
```

```text
target/release/ecaz --log-file reviews/task-111a/002-benchmark-gate/artifacts/suite-dry-run.log bench suite run --dry-run --config reviews/task-111a/002-benchmark-gate/artifacts/task111a-dense-coalescing-suite.json --database task111a_dense_bench --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111a/002-benchmark-gate/artifacts/suite-dry-run-manifest.json
```

```text
target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-111a/002-benchmark-gate/artifacts/install-ecaz-pg18-release.log
```

```text
target/release/ecaz --log-file reviews/task-111a/002-benchmark-gate/artifacts/suite-run.log bench suite run --config reviews/task-111a/002-benchmark-gate/artifacts/task111a-dense-coalescing-suite.json --database task111a_dense_bench --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-111a/002-benchmark-gate/artifacts/suite/suite-manifest.json --results-output reviews/task-111a/002-benchmark-gate/artifacts/suite/results.jsonl
```

```text
target/release/ecaz --log-file reviews/task-111a/002-benchmark-gate/artifacts/suite-status.log bench suite status --manifest reviews/task-111a/002-benchmark-gate/artifacts/suite/suite-manifest.json
```

```text
target/release/ecaz --log-file reviews/task-111a/002-benchmark-gate/artifacts/suite-report.log bench suite report --manifest reviews/task-111a/002-benchmark-gate/artifacts/suite/suite-manifest.json --results-output reviews/task-111a/002-benchmark-gate/artifacts/suite/results-report.jsonl
```

## Nprobe 32 Latency

| Step | p50 | p95 | p99 | Mean |
| --- | ---: | ---: | ---: | ---: |
| 50k TQ row | 14.4 ms | 15.6 ms | 17.0 ms | 14.5 ms |
| 50k TQ dense-old | 17.2 ms | 19.2 ms | 22.5 ms | 17.4 ms |
| 50k TQ dense+A | 13.1 ms | 18.3 ms | 19.0 ms | 13.7 ms |
| 50k RB1 row | 7.11 ms | 7.60 ms | 7.97 ms | 7.12 ms |
| 50k RB1 dense-old | 5.80 ms | 6.63 ms | 9.18 ms | 5.92 ms |
| 50k RB1 dense+A | 6.10 ms | 6.69 ms | 8.38 ms | 6.15 ms |
| 100k TQ row | 38.7 ms | 49.2 ms | 50.6 ms | 37.9 ms |
| 100k TQ dense-old | 37.2 ms | 42.2 ms | 47.8 ms | 37.4 ms |
| 100k TQ dense+A | 26.7 ms | 31.6 ms | 34.2 ms | 27.1 ms |
| 100k RB1 row | 14.5 ms | 16.3 ms | 17.8 ms | 14.5 ms |
| 100k RB1 dense-old | 11.6 ms | 13.4 ms | 16.7 ms | 11.9 ms |
| 100k RB1 dense+A | 12.3 ms | 14.1 ms | 15.4 ms | 12.4 ms |

## Nprobe 32 Recall / NDCG

| Step | Recall@10 | NDCG@10 | Mean query time |
| --- | ---: | ---: | ---: |
| 50k TQ row | 0.9420 | 0.9994 | 16.22 ms |
| 50k TQ dense-old | 0.9420 | 0.9994 | 17.34 ms |
| 50k TQ dense+A | 0.9420 | 0.9994 | 12.97 ms |
| 50k RB1 row | 0.7750 | 0.9896 | 7.28 ms |
| 50k RB1 dense-old | 0.7750 | 0.9896 | 6.02 ms |
| 50k RB1 dense+A | 0.7750 | 0.9896 | 6.32 ms |
| 100k TQ row | 0.9370 | 0.9966 | 31.52 ms |
| 100k TQ dense-old | 0.9370 | 0.9966 | 36.69 ms |
| 100k TQ dense+A | 0.9370 | 0.9966 | 26.60 ms |
| 100k RB1 row | 0.7630 | 0.9875 | 15.34 ms |
| 100k RB1 dense-old | 0.7630 | 0.9875 | 11.59 ms |
| 100k RB1 dense+A | 0.7630 | 0.9875 | 12.09 ms |

## Build Time And Index Size

| Step | Build time | EC-IVF index size | Per row |
| --- | ---: | ---: | ---: |
| 50k TQ row | 4.15 s | 44.1 MiB | 925.2 B |
| 50k TQ dense-old | 3.84 s | 39.8 MiB | 835.1 B |
| 50k TQ dense+A | 5.10 s | 39.8 MiB | 835.1 B |
| 50k RB1 row | 5.49 s | 15.2 MiB | 319.8 B |
| 50k RB1 dense-old | 4.34 s | 11.6 MiB | 243.3 B |
| 50k RB1 dense+A | 3.70 s | 11.6 MiB | 243.3 B |
| 100k TQ row | 7.36 s | 87.6 MiB | 918.2 B |
| 100k TQ dense-old | 7.48 s | 78.9 MiB | 827.1 B |
| 100k TQ dense+A | 7.21 s | 78.9 MiB | 827.1 B |
| 100k RB1 row | 7.07 s | 29.7 MiB | 311.3 B |
| 100k RB1 dense-old | 8.60 s | 22.5 MiB | 235.8 B |
| 100k RB1 dense+A | 8.02 s | 22.5 MiB | 235.8 B |

## SIMD Width Histogram At Nprobe 32

| Step | Flushes | Candidates | width_lt8 | width_8_15 | width_16_31 | width_ge32 | Kernel elapsed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k TQ row | 9147 | 2328863 | 1 | 3 | 9 | 9134 | 548.718 ms |
| 50k TQ dense-old | 233854 | 2328462 | 1693 | 232161 | 0 | 0 | 1094.184 ms |
| 50k TQ dense+A | 10699 | 2328864 | 177 | 88 | 45 | 10389 | 575.681 ms |
| 50k RB1 row | 9147 | 2328864 | 1 | 3 | 9 | 9134 | 171.810 ms |
| 50k RB1 dense-old | 66103 | 2328864 | 379 | 861 | 1556 | 63307 | 188.208 ms |
| 50k RB1 dense+A | 10699 | 2328864 | 177 | 88 | 45 | 10389 | 173.882 ms |
| 100k TQ row | 20379 | 5203807 | 4 | 5 | 7 | 20363 | 1460.020 ms |
| 100k TQ dense-old | 521755 | 5203613 | 2405 | 519350 | 0 | 0 | 2506.077 ms |
| 100k TQ dense+A | 21778 | 5203752 | 53 | 142 | 140 | 21443 | 1240.876 ms |
| 100k RB1 row | 20380 | 5203809 | 5 | 5 | 7 | 20363 | 384.695 ms |
| 100k RB1 dense-old | 146299 | 5203809 | 810 | 837 | 1266 | 143386 | 434.139 ms |
| 100k RB1 dense+A | 21778 | 5203809 | 53 | 142 | 140 | 21443 | 411.345 ms |

## EXPLAIN Scan Counters At Nprobe 32

| Step | Posting pages | Postings visited | Dense blocks | Dense coalesced flushes | Dense payload bytes copied | Approx scan elapsed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k TQ row | 2668 | 23904 | 0 | 0 | 0 | 20153 us |
| 50k TQ dense-old | 2404 | 23904 | 2404 | 0 | 0 | 21435 us |
| 50k TQ dense+A | 2404 | 23904 | 2404 | 109 | 18358272 | 18315 us |
| 50k RB1 row | 903 | 23904 | 0 | 0 | 0 | 10296 us |
| 50k RB1 dense-old | 677 | 23904 | 677 | 0 | 0 | 7467 us |
| 50k RB1 dense+A | 677 | 23904 | 677 | 109 | 4876416 | 7650 us |
| 100k TQ row | 4700 | 42171 | 0 | 0 | 0 | 40362 us |
| 100k TQ dense-old | 4233 | 42171 | 4233 | 0 | 0 | 41324 us |
| 100k TQ dense+A | 4233 | 42171 | 4233 | 178 | 32387328 | 34313 us |
| 100k RB1 row | 1577 | 42171 | 0 | 0 | 0 | 17535 us |
| 100k RB1 dense-old | 1189 | 42171 | 1189 | 0 | 0 | 12659 us |
| 100k RB1 dense+A | 1189 | 42171 | 1189 | 178 | 8602884 | 13121 us |

## Interpretation

The TurboQuant dense latency regression was not caused by worse selected lists
or recall drift. Dense-old scans the same candidate count as row/dense+A, but
flushes almost every dense block as a tiny 8-15 candidate batch. At 100k nprobe
32 it produced 521755 AVX2 flushes for 5203613 candidates, with 519350 of
those flushes in `width_8_15` and no `width_ge32` flushes.

Dense+A coalesces across dense blocks before scoring. At the same 100k
TurboQuant nprobe 32 point, it reduced AVX2 flushes to 21778 and restored
21443 `width_ge32` flushes. That lowers p50/p95/p99 from
37.2/42.2/47.8 ms to 26.7/31.6/34.2 ms while preserving recall@10 and NDCG@10.

RaBitQ also benefits from dense posting blocks, but its scoring kernel is much
faster and the old dense path was less sensitive to small batches. Dense+A
keeps the RaBitQ win: 100k RB1 dense+A is p50 12.3 ms versus row p50 14.5 ms,
with the same recall@10/NDCG@10.

Approach A is sufficient for this task gate. Approach B, an on-disk multi-page
dense packing change, is dominated for this slice because it would alter the
storage format to solve a scan batching problem already fixed without changing
the dense on-disk representation. Dense posting blocks remain default-off for
index creation; the new scan-side coalescing GUC only controls how existing
dense posting blocks are scored.
