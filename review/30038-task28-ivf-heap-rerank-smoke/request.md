# Review Request: Task 28 IVF Heap Rerank Smoke

## Summary

This packet records the first IVF `rerank = 'heap_f32'` smoke after commit
`9b42a71`. It reuses the DBPedia-derived 10k x 1536 fixture from packets 30036
and 30037, creates a fresh copied table with only the heap-rerank IVF index, and
checks full-probe recall against a seq-scan exact top-10 table.

No DiskANN implementation or measurement is included.

## Fixture

- source table: `task28_ivf_anchor10k1536_corpus`
- copied table: `task28_ivf_anchor10k1536_heap_corpus`
- rows: 10,000
- dimensions: 1536
- index: `ec_ivf`, `nlists = 32`, `nprobe = 32`
- training sample rows: 2,000
- storage: `turboquant`
- rerank: `heap_f32`
- cache state: normal local scratch state; not cold-cache controlled

## Results

Build/storage from `artifacts/pg18-ivf-anchor10k1536-heap-rerank-smoke-rerun.log`:

| metric | result |
|---|---:|
| table copy | `00:03.040` |
| build time | `00:25.130` |
| index size | `9,379,840` bytes (`9160 kB`) |
| heap size | `1,048,576` bytes (`1024 kB`) |

One-query full-probe heap-rerank EXPLAIN:

| metric | result |
|---|---:|
| returned rows | 10 |
| execution time | `706.722 ms` |
| buffers | `shared hit=40097 read=1146` |

20-query recall against seq-scan exact top-10:

| returned | exact hits | recall@10 |
|---:|---:|---:|
| 200 | 200 | `1.0000` |

Every query returned 10 rows.

20-query latency loop from `artifacts/pg18-ivf-anchor10k1536-heap-rerank-latency.log`:

| p50 | p95 | p99 | avg |
|---:|---:|---:|---:|
| `686.339 ms` | `714.122 ms` | `748.029 ms` | `685.006 ms` |

## Interpretation

`heap_f32` fixes the full-probe scorer-alignment gap from packet 30037 on this
10k x 1536 anchor slice: full probe now reaches `1.0000` recall@10 against the
SQL `<#>` exact order.

The cost is high because this first implementation reranks the whole selected
candidate frontier. At `nprobe = 32` / `nlists = 32`, that means heap-fetching
and raw-f32 scoring all 10,000 candidates before emitting top 10.

## Next Slice Recommendation

Add a bounded IVF rerank frontier knob and sweep it before broadening the
centroid-count grid. The immediate target should compare `rerank = off` and
`rerank = heap_f32` at fixed `nlists = 32` with rerank widths such as 50, 100,
200, 500, and 1000, measuring recall recovery versus latency.

## Artifacts

- `artifacts/pg18-anchor-exists.log`
- `artifacts/pg18-ivf-anchor10k1536-heap-rerank-smoke.sql`
- `artifacts/pg18-ivf-anchor10k1536-heap-rerank-smoke.log`
- `artifacts/pg18-ivf-anchor10k1536-heap-rerank-smoke-rerun.log`
- `artifacts/pg18-ivf-anchor10k1536-heap-rerank-latency.sql`
- `artifacts/pg18-ivf-anchor10k1536-heap-rerank-latency.log`
- `artifacts/manifest.md`

## Validation

Code validation before this packet:

- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
- `cargo test --lib test_ec_ivf_gettuple_emits_probe_candidates_with_scores --no-default-features --features pg18`
- `cargo test --lib test_ec_ivf_full_probe_matches_simple_exact_oracle_top1 --no-default-features --features pg18`
- `git diff --check`

The broader `cargo test --lib ec_ivf --no-default-features --features pg18`
compiled and passed the non-concurrent IVF tests reached before failing the
three existing concurrent-insert tests because the pg_test backend could not
spawn `psql` from its process environment.
