# Task 70 Packet 013 Zero-Overhead Profile Split Summary

- Code commit: `261186cf7ee217ab51e5b061c1ee8e5e1c8c95bc`
- Review driver: packet 012 reviewer follow-up requesting a zero-overhead default scan path
- Suite: `task70-diskann-zero-overhead-profile-split-real10k`
- Fixture: real10K DBPedia staged corpus, PG18, `pq_fastscan`, `graph_degree=32`, `build_list_size=100`, `alpha=1.2`, `rerank_budget=64`, `top_k=10`
- Storage surface: isolated packet-local prefix `task70_013_diskann`

## Code Change

The packet 012 review identified that the default scan path still passed through profile-aware helpers with `Option<&mut FrontierProfile>` checks in the frontier inner loop. This slice splits the paths:

- `vamana_scan_with` now validates params, calls `greedy_descent_with`, and reranks.
- `greedy_descent_with` is the non-profile implementation and contains no profile option parameter or profile branches.
- `vamana_scan_with_frontier_profile` calls `greedy_descent_with_frontier_profile`, whose timing/counter updates are unconditional.
- Shared validation and exact-rerank finalization were extracted to avoid duplicating non-hot-path logic.

No new `unsafe` blocks were introduced.

## Validation

| command | result |
| --- | --- |
| `cargo fmt --check` | pass |
| `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` | pass, 20/20 scan tests |
| `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | pass |

## Measurement

The packet-local suite repeats the clean real10K load, recall, and pgvectorscale compare at L64/L200. It intentionally omits scan-profile NOTICE steps because this follow-up targets unprofiled default-path overhead.

Recall:

| list_size | recall@10 | mean q-time |
| ---: | ---: | ---: |
| 64 | 0.9965 | 0.63 ms |
| 200 | 0.9975 | 0.79 ms |

Clean compare:

| engine | sweep | recall@10 | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` | 64 | 0.9965 | 0.66 ms | 0.65 ms | 0.83 ms | 1.10 ms |
| `pgvectorscale` | 64 | 0.9955 | 0.63 ms | 0.62 ms | 0.81 ms | 1.04 ms |
| `ec_diskann` | 200 | 0.9975 | 0.81 ms | 0.80 ms | 1.01 ms | 1.11 ms |
| `pgvectorscale` | 200 | 1.0000 | 1.22 ms | 1.21 ms | 1.57 ms | 1.81 ms |

Acceptance gate from packet 012 review:

- Target: L200 clean compare `ec_diskann` mean <= `0.83 ms`.
- Result: L200 clean compare `ec_diskann` mean `0.81 ms`.
- Packet 012 closeout was `0.88 ms`; this slice improves it by `0.07 ms` (`8.0%`) and clears the Phase 1 baseline target.
