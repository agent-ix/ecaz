# Task 70 Phase 2 Retained Frontier Heap Summary

- Code head SHA: `9bbef9ecf718bab30ef543b5f21d4728267136d0`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/005-retained-frontier-heap/`
- Timestamp: `2026-05-31T19:33:01Z`
- Phase 1 backreference: `reviews/task-70/003-phase1-suite-config/artifacts/phase1-profile-summary.md`
- Previous frontier slice: `reviews/task-70/004-frontier-neighbor-retention/artifacts/phase2-frontier-summary.md`
- Slice: frontier / candidate management P0.

## Code Change

The scan frontier now keeps the retained best candidate set in a bounded max-heap instead of inserting every expanded candidate into a sorted vector and shifting the tail. The top of the retained heap is the current worst retained candidate; traversal can stop when the next frontier candidate cannot improve that bounded set. The final retained heap is sorted before return, preserving the existing result ordering contract.

Changed source:

- `src/am/ec_diskann/scan.rs`: replace `insert_visited_sorted` with `push_retained_candidate`; use `BinaryHeap<ScanCandidate>` for retained best candidates.

No new `unsafe` blocks were introduced.

## Validation

Commands:

```sh
cargo fmt --check
cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::
cargo check --all-targets --no-default-features --features pg18
./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file reviews/task-70/005-retained-frontier-heap/artifacts/install-ecaz-pg-test.log
./target/debug/ecaz bench suite run --config reviews/task-70/005-retained-frontier-heap/artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/005-retained-frontier-heap/artifacts/suite-dry-run-manifest.json --log-file reviews/task-70/005-retained-frontier-heap/artifacts/suite-dry-run.log
./target/debug/ecaz bench suite run --config reviews/task-70/005-retained-frontier-heap/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/005-retained-frontier-heap/artifacts/suite-manifest.json --results-output reviews/task-70/005-retained-frontier-heap/artifacts/results.jsonl --log-file reviews/task-70/005-retained-frontier-heap/artifacts/suite-run.log
```

Result: pass. The scan test module ran 18 tests. The suite generated `results.jsonl`, EXPLAIN logs, pgvectorscale comparison, and 200 scan profile NOTICE rows for each L value.

## Recall And Latency

The baseline below is packet 004, after the neighbor-retention frontier slice.

| list_size | packet 004 recall@10 | new recall@10 | packet 004 latency mean | new latency mean | packet 004 p95 | new p95 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 0.9965 | 0.9965 | 0.64 ms | 0.64 ms | 0.73 ms | 0.73 ms |
| 200 | 0.9975 | 0.9975 | 0.91 ms | 0.90 ms | 1.10 ms | 1.10 ms |

## Cross-Engine Comparison

| engine | L/search_list | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` | 64 | 0.9965 | 0.63 ms | 0.61 ms | 0.79 ms | 0.90 ms |
| `pgvectorscale` | 64 | 0.9955 | 0.61 ms | 0.60 ms | 0.74 ms | 0.98 ms |
| `ec_diskann` | 200 | 0.9975 | 0.80 ms | 0.80 ms | 0.95 ms | 1.04 ms |
| `pgvectorscale` | 200 | 1.0000 | 1.16 ms | 1.16 ms | 1.44 ms | 1.61 ms |

## Phase Split Delta

The profile rows are 200 `ec_diskann_scan_profile` NOTICEs per L value from:

- `profile-notices-diskann-real10k-l64.log`
- `profile-notices-diskann-real10k-l200.log`

The baseline below is packet 004.

| list_size | metric | packet 004 | new | delta |
| ---: | --- | ---: | ---: | ---: |
| 64 | total mean_us | 372.50 | 366.32 | -1.66% |
| 64 | frontier mean_us | 263.60 | 261.37 | -0.85% |
| 64 | frontier share | 70.77% | 71.35% | +0.58 pp |
| 64 | exact rerank mean_us | 88.36 | 84.58 | -4.28% |
| 200 | total mean_us | 635.61 | 641.93 | +0.99% |
| 200 | frontier mean_us | 527.90 | 531.88 | +0.75% |
| 200 | frontier share | 83.05% | 82.86% | -0.19 pp |
| 200 | exact rerank mean_us | 87.31 | 88.69 | +1.58% |

Graph/prefilter visit counts are unchanged: mean `758.37` at L=64 and `1,585.36` at L=200.

## Interpretation

The slice preserves recall and gives a tiny L=200 latency-table improvement, but the phase profile does not show a convincing frontier win. L=64 improves slightly, while L=200 is slightly slower in raw NOTICE timings. Treat this as a semantics-preserving candidate-management cleanup with neutral performance rather than a retired P0 bottleneck. Frontier remains the dominant P0 area.
