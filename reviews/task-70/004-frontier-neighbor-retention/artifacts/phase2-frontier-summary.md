# Task 70 Phase 2 Frontier Slice Summary

- Code head SHA: `dd42450f7fd0215d9c7385dd9cc1b25c0443b769`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/004-frontier-neighbor-retention/`
- Timestamp: `2026-05-31T18:51:14Z`
- Phase 1 backreference: `reviews/task-70/003-phase1-suite-config/artifacts/phase1-profile-summary.md`
- Slice: frontier / candidate management P0.

## Code Change

The scan frontier now moves the decoded tuple's existing neighbor vector into each queued frontier entry instead of collecting a second neighbor vector. It also caps the retained best-candidate vector at `list_size` after each sorted insertion, preserving the top-L invariant without carrying unused tail candidates.

Changed source:

- `src/am/ec_diskann/scan.rs`: `FrontierEntry` stores `neighbor_count`; `neighbors_from_tuple` consumes `VamanaNodeTuple`; `insert_visited_sorted` truncates to `list_size`.

No new `unsafe` blocks were introduced.

## Validation

Commands:

```sh
cargo fmt --check
cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::
cargo check --all-targets --no-default-features --features pg18
./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file reviews/task-70/004-frontier-neighbor-retention/artifacts/install-ecaz-pg-test.log
./target/debug/ecaz bench suite run --config reviews/task-70/004-frontier-neighbor-retention/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/004-frontier-neighbor-retention/artifacts/suite-manifest.json --results-output reviews/task-70/004-frontier-neighbor-retention/artifacts/results.jsonl --log-file reviews/task-70/004-frontier-neighbor-retention/artifacts/suite-run.log
```

Result: pass. The scan test module ran 18 tests. The full suite generated `results.jsonl`, EXPLAIN logs, pgvectorscale comparison, and 200 scan profile NOTICE rows for each L value.

## Recall And Latency

| list_size | baseline recall@10 | new recall@10 | baseline latency mean | new latency mean | baseline p95 | new p95 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 0.9965 | 0.9965 | 0.65 ms | 0.64 ms | 0.75 ms | 0.73 ms |
| 200 | 0.9975 | 0.9975 | 0.96 ms | 0.91 ms | 1.18 ms | 1.10 ms |

## Cross-Engine Comparison

| engine | L/search_list | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` | 64 | 0.9965 | 0.60 ms | 0.59 ms | 0.72 ms | 0.86 ms |
| `pgvectorscale` | 64 | 0.9960 | 0.60 ms | 0.59 ms | 0.71 ms | 0.88 ms |
| `ec_diskann` | 200 | 0.9975 | 0.77 ms | 0.77 ms | 0.92 ms | 1.02 ms |
| `pgvectorscale` | 200 | 1.0000 | 1.13 ms | 1.11 ms | 1.37 ms | 1.44 ms |

## Phase Split Delta

The profile rows are 200 `ec_diskann_scan_profile` NOTICEs per L value from:

- `profile-notices-diskann-real10k-l64.log`
- `profile-notices-diskann-real10k-l200.log`

| list_size | metric | baseline | new | delta |
| ---: | --- | ---: | ---: | ---: |
| 64 | total mean_us | 369.68 | 372.50 | +0.76% |
| 64 | frontier mean_us | 269.62 | 263.60 | -2.23% |
| 64 | frontier share | 72.94% | 70.77% | -2.17 pp |
| 64 | exact rerank mean_us | 80.71 | 88.36 | +9.48% |
| 200 | total mean_us | 661.41 | 635.61 | -3.90% |
| 200 | frontier mean_us | 553.04 | 527.90 | -4.55% |
| 200 | frontier share | 83.61% | 83.05% | -0.56 pp |
| 200 | exact rerank mean_us | 87.29 | 87.31 | +0.02% |

Graph/prefilter visit counts are unchanged: mean `758.37` at L=64 and `1,585.36` at L=200.

## Interpretation

The slice preserves recall and gives a measurable L=200 scan-time win, with a smaller L=64 latency improvement in the benchmark rows. The raw per-query phase totals are noisy at L=64 and show rerank variance offsetting the frontier improvement, so this is a useful but not sufficient frontier slice. Frontier remains the top remaining P0 area after this change.
