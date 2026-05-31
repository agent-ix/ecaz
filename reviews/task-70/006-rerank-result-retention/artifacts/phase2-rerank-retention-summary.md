# Task 70 Phase 2 Rerank Result Retention Summary

- Measured code SHA: `4f499e27910399760f8c535588a8fdab805bc1b6`
- Shelve/revert SHA: `3f3fa8bfe6f6e103e4219a8a54c1c8c879fb20fb`
- Task bucket: `reviews/task-70/`
- Packet path: `reviews/task-70/006-rerank-result-retention/`
- Timestamp: `2026-05-31T19:58:12Z`
- Phase 1 backreference: `reviews/task-70/003-phase1-suite-config/artifacts/phase1-profile-summary.md`
- Previous retained-frontier packet: `reviews/task-70/005-retained-frontier-heap/artifacts/phase2-retained-frontier-summary.md`
- Slice: exact rerank / final result retention P0.

## Code Change Measured

Commit `4f499e27910399760f8c535588a8fdab805bc1b6` changed final exact-rerank result retention from materializing all `rerank_budget` exact results and sorting them to retaining only the best `top_k` results in a bounded max-heap, then sorting that retained set before return.

The change preserved rerank call count and heap-TID rerank call order, but added heap maintenance on every exact result. Measurement showed that overhead was not worthwhile for `rerank_budget=64` / `top_k=10`.

The slice was shelved by revert commit `3f3fa8bfe6f6e103e4219a8a54c1c8c879fb20fb`.

No new `unsafe` blocks were introduced by the measured slice.

## Validation

Commands:

```sh
cargo fmt --check
cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::
cargo check --all-targets --no-default-features --features pg18
./target/debug/ecaz dev install ecaz-pg-test --pg 18 --database tqvector_bench --log-file reviews/task-70/006-rerank-result-retention/artifacts/install-ecaz-pg-test.log
./target/debug/ecaz bench suite run --config reviews/task-70/006-rerank-result-retention/artifacts/suite.json --dry-run --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/006-rerank-result-retention/artifacts/suite-dry-run-manifest.json --log-file reviews/task-70/006-rerank-result-retention/artifacts/suite-dry-run.log
./target/debug/ecaz bench suite run --config reviews/task-70/006-rerank-result-retention/artifacts/suite.json --database tqvector_bench --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-70/006-rerank-result-retention/artifacts/suite-manifest.json --results-output reviews/task-70/006-rerank-result-retention/artifacts/results.jsonl --log-file reviews/task-70/006-rerank-result-retention/artifacts/suite-run.log
cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::
```

Result: pass. The scan test module ran 18 tests before measurement and again after the revert. The full suite generated `results.jsonl`, EXPLAIN logs, pgvectorscale comparison, and 200 scan profile NOTICE rows for each L value.

## Recall And Latency

The baseline below is packet 005, after the retained-frontier heap slice.

| list_size | packet 005 recall@10 | measured recall@10 | packet 005 latency mean | measured latency mean | packet 005 p95 | measured p95 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 0.9965 | 0.9965 | 0.64 ms | 0.63 ms | 0.73 ms | 0.74 ms |
| 200 | 0.9975 | 0.9975 | 0.90 ms | 0.96 ms | 1.10 ms | 1.23 ms |

## Cross-Engine Comparison

| engine | L/search_list | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` | 64 | 0.9965 | 0.64 ms | 0.62 ms | 0.78 ms | 0.90 ms |
| `pgvectorscale` | 64 | 0.9960 | 0.61 ms | 0.60 ms | 0.75 ms | 0.98 ms |
| `ec_diskann` | 200 | 0.9975 | 0.80 ms | 0.80 ms | 0.96 ms | 1.07 ms |
| `pgvectorscale` | 200 | 1.0000 | 1.17 ms | 1.14 ms | 1.45 ms | 1.57 ms |

## Phase Split Delta

The profile rows are 200 `ec_diskann_scan_profile` NOTICEs per L value from:

- `profile-notices-diskann-real10k-l64.log`
- `profile-notices-diskann-real10k-l200.log`

The baseline below is packet 005.

| list_size | metric | packet 005 | measured | delta |
| ---: | --- | ---: | ---: | ---: |
| 64 | total mean_us | 366.32 | 386.21 | +5.43% |
| 64 | frontier mean_us | 261.37 | 266.97 | +2.14% |
| 64 | exact rerank mean_us | 84.58 | 97.64 | +15.44% |
| 200 | total mean_us | 641.93 | 668.97 | +4.21% |
| 200 | frontier mean_us | 531.88 | 544.10 | +2.30% |
| 200 | exact rerank mean_us | 88.69 | 101.02 | +13.90% |

Graph/prefilter visit counts are unchanged: mean `758.37` at L=64 and `1,585.36` at L=200.

## Interpretation

The bounded top-k result-retention heap is a negative slice at the current rerank budget. It preserves recall, but the exact rerank phase worsens by roughly 14-15%, and the L=200 latency table also regresses. This slice is shelved by revert commit `3f3fa8bfe6f6e103e4219a8a54c1c8c879fb20fb`.
