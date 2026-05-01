# Task 29d Build Distance SIMD

Status: ready for review
Branch: `task29-diskann-initial-tuning`
Measurement/code commit: `0cd4baf9`
Plan update commit: `67042297`

## Question

Task 29d-3 asked for a build-performance attack with a concrete stop
condition: get DiskANN build within 3x of the strongest local reference
(`pgvectorscale` at 5.82 s, so target <= 17.5 s), or document why the next
attack is not worth landing in this slice.

The structured build counters showed tens of millions of exact 1536-d f32
source-vector distance calls per real-10k build. The helper benchmark measured
the scalar 1536-d inner product at `1238.8 ns` per call on this machine.

## Change

Commit `0cd4baf9` adds a runtime-gated AVX2+FMA implementation for
`ec_diskann` ambuild's source-vector inner product helper, with the existing
scalar loop as fallback. The distance wrapper still computes
`max(0, 1 - inner_product)`.

## Result

The release-mode real-10k DROP+CREATE build result is:

| Build | Total | Build/persist | Core graph | Pass 0 | Pass 1 | Size |
|---|---:|---:|---:|---:|---:|---:|
| `11104` active-mask baseline | 70.678 s | 69.000 s | 67.571 s | 20.737 s | 46.832 s | 4,939,776 B |
| `11108` SIMD distance | 14.493 s | 12.855 s | 12.639 s | 4.452 s | 8.185 s | 4,939,776 B |

That is a 79.5% total build-time reduction from the active-mask baseline and
meets the Task 29d build stop condition: 14.493 s is below the 17.5 s target.

Recall and latency stayed in family after rebuilding with the SIMD distance
path:

| L | Recall@10 | NDCG@10 | Recall mean query | Latency mean | p50 | p95 | p99 | HWM |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 0.9965 | 0.9999 | 7.75 ms | 7.57 ms | 7.45 ms | 8.34 ms | 9.81 ms | 61020 KiB |
| 200 | 0.9970 | 0.9999 | 8.24 ms | 7.91 ms | 7.80 ms | 8.66 ms | 10.6 ms | 61820 KiB |
| 800 | 0.9975 | 0.9999 | 9.78 ms | 9.33 ms | 9.19 ms | 10.4 ms | 13.1 ms | 62892 KiB |

Recommendation: land the SIMD build-distance change and treat 29d-3 as
complete. Further build work should move to the final readiness sweep and then
to a follow-up only if merge discussion asks for parallel build or algorithmic
changes.

## Validation

- `cargo test --lib am::ec_diskann::ambuild::tests -- --nocapture`
- `cargo test --lib am::ec_diskann::vamana::tests::build_recall_at_10_meets_baseline -- --nocapture`
- `cargo pgrx test pg18 test_ec_diskann_sql_ordered_index_scan_executes`
- `cargo pgrx test pg18 test_ec_diskann_empty_index_bootstrap_insert_executes`
- `cargo pgrx test pg18 test_ec_diskann_vacuum_unlinks_and_tombstones_dead_node`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo fmt --check`
- `git diff --check`

## Artifacts

See `artifacts/manifest.md`.
