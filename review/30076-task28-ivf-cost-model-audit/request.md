# Task 28 IVF Cost Model Audit

## Scope

This packet records the A1 follow-up for Task 28: audit the IVF planner-cost constants so they reflect measured work instead of planner-selection tuning.

Code checkpoint: `223e7c0350f1735682f3f0b74f47cabee53b1269` (`ivf: ground cost constants in kernel timing`).

## What Changed

- Added `--iterations` and `--log-output` flags to `simd_bench` so cost-kernel microbenchmarks can be packet-local artifacts without shell redirection.
- Added benchmark coverage for:
  - 1536D f32 inner product, matching centroid scoring shape.
  - 1536D no-QJL 4-bit LUT posting scoring, matching the current IVF TurboQuant scan path.
- Changed `IVF_CENTROID_SCORING_DIMENSION_SCALE` from `0.03` to `0.01`.
- Left `IVF_POSTING_SCORING_DIMENSION_SCALE` at `0.01`.
- Changed `IVF_INDEX_PAGE_COST_SCALE` from `0.25` to `1.0`, using `seq_page_cost` without an unsupported sub-sequential discount.

## Evidence

Release microbenchmark on local AVX2+FMA:

- `f32_inner_product/d1536`: `1306.1 ns_per_iter`
- `score_ip_lut_no_qjl_4bit/d1536`: `1331.0 ns_per_iter`

The centroid f32 kernel and the current posting LUT kernel are effectively the same per-vector cost at 1536 dimensions, so the planner model now uses the same dimension scale for both.

Planner matrix after the cost change on the existing 10k n128 IVF surface:

- KNN `LIMIT 10`, `nprobe=8`: selected `task28_ivf_postopt10k_n128w25_idx`, cost `105.92..710.75`, execution `89.656 ms`.
- KNN `LIMIT 10`, `nprobe=32`: selected `task28_ivf_postopt10k_n128w25_idx`, cost `105.92..1022.27`, execution `50.073 ms`.
- KNN `LIMIT 1000`, `nprobe=64`: selected `task28_ivf_postopt10k_n128w25_idx`, cost `105.92..1437.62`, returned the available 25 candidates, execution `80.885 ms`.
- Non-KNN `count(*) WHERE id <= 100`: selected the primary-key index-only scan, not IVF, execution `0.671 ms`.
- Mixed predicate `WHERE id <= 1000 ORDER BY embedding <#> query LIMIT 10`: selected the primary-key scan plus sort, not IVF, execution `540.382 ms`.

The mixed-predicate result is intentionally called out: the planner avoided IVF when the btree predicate looked more selective, but actual runtime was poor because the sort still computed vector scores for 1001 rows. That belongs in A6’s broader planner cross-test matrix.

## Artifacts

- `artifacts/simd_bench_cost_kernels.log`
- `artifacts/cost_model_matrix.sql`
- `artifacts/cost_model_matrix.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::cost --no-default-features --features pg18`
- `cargo test --bin simd_bench --no-default-features --features pg18`
- `cargo run --release --bin simd_bench -- --iterations 20000 --log-output review/30076-task28-ivf-cost-model-audit/artifacts/simd_bench_cost_kernels.log`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30076-task28-ivf-cost-model-audit/artifacts/cost_model_matrix.sql --raw --log-output review/30076-task28-ivf-cost-model-audit/artifacts/cost_model_matrix.log`
- `git diff --check`

No DiskANN work is included in this packet.
