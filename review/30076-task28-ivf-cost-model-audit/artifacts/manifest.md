# Artifacts Manifest

Packet: `30076-task28-ivf-cost-model-audit`

Head SHA: `223e7c0350f1735682f3f0b74f47cabee53b1269`

Timestamp: `2026-04-27T19:02:40-07:00`

Lane: Task 28 A1 IVF planner-cost audit.

Fixture: local PG18, existing 10k DBPedia-derived `task28_ivf_postopt10k_n128w25` surface for planner matrix; local AVX2+FMA release binary for kernel microbenchmark.

Storage format: TurboQuant / no-QJL 4-bit LUT posting scorer for scan-cost evidence.

Rerank mode: heap-f32 on the existing n128 surface.

Isolation: existing isolated one-index-per-table IVF surface.

## `simd_bench_cost_kernels.log`

- Command: `cargo run --release --bin simd_bench -- --iterations 20000 --log-output review/30076-task28-ivf-cost-model-audit/artifacts/simd_bench_cost_kernels.log`
- Key lines:
  - `backend=avx2+fma`
  - `f32_inner_product/d1536: total=26.122695ms ns_per_iter=1306.1`
  - `score_ip_lut_no_qjl_4bit/d1536: total=26.620532ms ns_per_iter=1331.0`

## `cost_model_matrix.sql`

- SQL script used for planner matrix.
- Uses `LOAD 'ecaz'`, PG18 default cost constants, `enable_seqscan=on`, and `ec_ivf.nprobe` settings of `8`, `32`, and `64`.

## `cost_model_matrix.log`

- Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30076-task28-ivf-cost-model-audit/artifacts/cost_model_matrix.sql --raw --log-output review/30076-task28-ivf-cost-model-audit/artifacts/cost_model_matrix.log`
- Key lines:
  - `Index Scan using task28_ivf_postopt10k_n128w25_idx ... (cost=105.92..710.75 rows=10000 width=12)` for KNN `LIMIT 10`, `nprobe=8`.
  - `Execution Time: 89.656 ms`
  - `Index Scan using task28_ivf_postopt10k_n128w25_idx ... (cost=105.92..1022.27 rows=10000 width=12)` for KNN `LIMIT 10`, `nprobe=32`.
  - `Execution Time: 50.073 ms`
  - `Index Scan using task28_ivf_postopt10k_n128w25_idx ... (cost=105.92..1437.62 rows=10000 width=12)` for KNN `LIMIT 1000`, `nprobe=64`.
  - `Execution Time: 80.885 ms`
  - `Index Only Scan using task28_ivf_postopt10k_n128w25_corpus_pkey ... (cost=0.29..6.05 rows=101 width=0)` for non-KNN `count(*) WHERE id <= 100`.
  - `Execution Time: 0.671 ms`
  - `Index Scan using task28_ivf_postopt10k_n128w25_corpus_pkey ... (cost=0.29..49.30 rows=1001 width=12)` for mixed predicate `WHERE id <= 1000 ORDER BY embedding <#> query LIMIT 10`.
  - `Execution Time: 540.382 ms`

## Validation Commands

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::cost --no-default-features --features pg18`
- `cargo test --bin simd_bench --no-default-features --features pg18`
- `git diff --check`
