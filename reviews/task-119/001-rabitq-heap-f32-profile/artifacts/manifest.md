# Task 119 HNSW RaBitQ Coarse-Rerank Profile Artifacts

- Task bucket: `reviews/task-119/001-rabitq-heap-f32-profile`
- Code/config head for final counter probe: `dd7154bb65fb7a4be2bd549dfed2d1fa71d2453a`
- Release benchmark head: `b67deea0356fbf7e7380df409e35ae406110425b`
- Host/lane: M5 laptop local PG18, socket `/Users/peter/.pgrx`, port `28818`
- Release suite database: `tqvector_task119_m5_release2`
- Surface isolation: one loaded corpus/query/index prefix per scale and storage format; the 50k/100k counter probe reused the release-built one-index-per-table HNSW RaBitQ indexes.
- Storage format under decision: `storage_format = 'rabitq'`
- Rerank modes under decision: `ec_hnsw.rerank_format=quantized` baseline and `ec_hnsw.rerank_format=heap_f32` with explicit `ec_hnsw.rerank_width`.
- Durable storage layout changes: none.

## Task 118 Gate

Task 118 packet `reviews/task-118/006-final-attribution-matrix` unblocks this task but does not justify promotion by itself. Its M5 release closeout says RaBitQ's dominant loss is candidate containment/traversal rather than source-vs-compressed build, final exact rerank, or scorer ordering. The Task 119 go/no-go condition is therefore: run a true RaBitQ candidate-frontier profile with explicit overfetch and exact/source rerank; promote only if recall/latency/storage jointly improve enough to justify the new profile.

## Release Suite

Command:

```text
./target/release/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-coarse-rerank-profile.json --manifest-output reviews/task-119/001-rabitq-heap-f32-profile/artifacts/suite-manifest.release2.json --results-output reviews/task-119/001-rabitq-heap-f32-profile/artifacts/suite-results.release2.jsonl --log-file reviews/task-119/001-rabitq-heap-f32-profile/artifacts/suite-run.release2.log
```

Artifacts:

- `precheck-release2-extension.log`: `ecaz_build_profile()` returned `release` before the release benchmark run.
- `suite-manifest.release2.json`: `44/44` suite steps succeeded.
- `suite-results.release2.jsonl`: normalized recall, latency, storage, and load result rows.
- `suite-run.release2.log`: full runner output.
- `load-*`, `recall-*`, `latency-*`, and `storage-*` logs cited by the JSONL rows.

Key `ef_search=1000` release results:

| Scale | Lane | Recall@10 | Recall mean q-time | Latency mean / p95 / p99 | Total storage | HNSW index |
| --- | --- | ---: | ---: | --- | ---: | ---: |
| 10k | TurboQuant | 0.9720 | 5.24 ms | n/a | 172.1 MiB | 13.0 MiB |
| 10k | PqFastScan | 0.9965 | 5.34 ms | n/a | 172.2 MiB | 13.1 MiB |
| 10k | RaBitQ quantized | 0.9535 | 6.84 ms | 6.65 / 7.09 / 7.28 ms | 172.1 MiB | 13.0 MiB |
| 10k | RaBitQ heap_f32 w1000 | 0.9765 | 9.46 ms | 9.45 / 10.4 / 10.9 ms | 172.1 MiB | 13.0 MiB |
| 50k | TurboQuant | 0.9405 | 6.74 ms | n/a | 860.0 MiB | 65.1 MiB |
| 50k | PqFastScan | 0.9855 | 6.89 ms | n/a | 860.1 MiB | 65.2 MiB |
| 50k | RaBitQ quantized | 0.9380 | 8.71 ms | 8.07 / 9.09 / 9.80 ms | 860.0 MiB | 65.1 MiB |
| 50k | RaBitQ heap_f32 w1000 | 0.9885 | 11.94 ms | 12.6 / 15.1 / 16.9 ms | 860.0 MiB | 65.1 MiB |
| 100k | TurboQuant | 0.9450 | 8.34 ms | n/a | 1.7 GiB | 130.2 MiB |
| 100k | PqFastScan | 0.9890 | 10.44 ms | n/a | 1.7 GiB | 130.3 MiB |
| 100k | RaBitQ quantized | 0.9420 | 9.74 ms | 10.2 / 12.5 / 15.0 ms | 1.7 GiB | 130.2 MiB |
| 100k | RaBitQ heap_f32 w1000 | 0.9850 | 21.36 ms | 21.0 / 27.2 / 30.7 ms | 1.7 GiB | 130.2 MiB |

## Candidate Counters

10k full frontier diagnostic command:

```text
./target/release/ecaz --database tqvector_task119_m5_pgtest --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-coarse-rerank-frontier.json --manifest-output reviews/task-119/001-rabitq-heap-f32-profile/artifacts/frontier-diagnostics/suite-manifest.frontier.json --results-output reviews/task-119/001-rabitq-heap-f32-profile/artifacts/frontier-diagnostics/suite-results.frontier.jsonl --log-file reviews/task-119/001-rabitq-heap-f32-profile/artifacts/frontier-diagnostics/suite-run.frontier.log
```

The full 10k diagnostic was stopped after the 10k rows because the old full containment helper was too slow for 50k/100k. The cited completed 10k artifacts are:

- `frontier-diagnostics/frontier-10k-hnsw-rabitq-quantized.log`
- `frontier-diagnostics/frontier-10k-hnsw-rabitq-heap-f32-w1000.log`
- matching JSONL files for both rows.

10k `ef_search=1000` counters:

| Mode | Queries | Truth@10 in pool | Emitted pool | Exact rerank | Quantized rerank | Dropped before exact |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| quantized | 200 | 0.9765 | 1000 | 0 | 1000 | 1000 |
| heap_f32 w1000 | 200 | 0.9765 | 1000 | 1000 | 0 | 0 |

50k/100k counters-only command:

```text
./target/debug/ecaz --database tqvector_task119_m5_release2 --host /Users/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task119-hnsw-rabitq-coarse-rerank-frontier-reuse.json --manifest-output reviews/task-119/001-rabitq-heap-f32-profile/artifacts/frontier-release-index-diagnostics/suite-manifest.counter20.json --results-output reviews/task-119/001-rabitq-heap-f32-profile/artifacts/frontier-release-index-diagnostics/suite-results.counter20.jsonl --log-file reviews/task-119/001-rabitq-heap-f32-profile/artifacts/frontier-release-index-diagnostics/suite-run.counter20.log
```

Artifacts:

- `frontier-release-index-diagnostics/register-counter-function.log`: registered the pg_test counter wrapper in the release benchmark database.
- `frontier-release-index-diagnostics/suite-manifest.counter20.json`: `4/4` counter steps succeeded.
- `frontier-release-index-diagnostics/suite-results.counter20.jsonl`: normalized counter rows.
- `frontier-release-index-diagnostics/frontier-50k-hnsw-rabitq-{quantized,heap-f32-w1000}.{log,jsonl}`
- `frontier-release-index-diagnostics/frontier-100k-hnsw-rabitq-{quantized,heap-f32-w1000}.{log,jsonl}`

50k/100k `ef_search=1000` counters:

| Scale | Mode | Queries | Emitted pool | Exact rerank | Quantized rerank | Dropped before exact | Pool == emitted |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| 50k | quantized | 20 | 1000 | 0 | 1000 | 1000 | true |
| 50k | heap_f32 w1000 | 20 | 1000 | 1000 | 0 | 0 | true |
| 100k | quantized | 20 | 1000 | 0 | 1000 | 1000 | true |
| 100k | heap_f32 w1000 | 20 | 1000 | 1000 | 0 | 0 | true |

## Validation Logs

- `cargo-check-ecaz-cli-counter-probe.log`: `cargo check -p ecaz-cli` succeeded.
- `cargo-check-ecaz-pg18-pgtest-counter-probe.log`: `cargo check -p ecaz --lib --no-default-features --features pg18,pg_test` succeeded.
- `cargo-test-ecaz-cli-hnsw-frontier-counter-probe.log`: `cargo test -p ecaz-cli hnsw_frontier -- --nocapture` succeeded with `3 passed`.
- `suite-audit-frontier-reuse-counter20.log`: counters-only suite audit passed with `4 steps`.
