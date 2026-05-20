---
task: 50
packet: reviews/task-50/027-closeout-summary
head_sha: 5e325212d65615f6d84a6af67bf9ec1250253ff9
code_commit: n/a
generated_at: 2026-05-20T07:44:17-07:00
lane: task-50-closeout-summary
fixture: n/a
storage_format: n/a
rerank_mode: n/a
surface: all original top-15 unsafe-density files
index_surface: one-index-per-table for cited benchmark baseline where applicable
shared_table_surface: no
---

# Artifact Manifest

## Artifacts

### `final-top15-unsafe-count.log`

- Command: `make unsafe-block-count PATHS='src/am/ec_hnsw/scan_debug.rs src/am/ec_hnsw/scan.rs src/am/ec_hnsw/build_parallel.rs src/am/ec_spire/dml_frontdoor/mod.rs src/am/ec_ivf/page.rs src/am/ec_hnsw/insert.rs src/am/ec_ivf/scan.rs src/am/ec_hnsw/vacuum.rs src/am/ec_diskann/routine.rs src/am/ec_hnsw/source.rs src/am/ec_hnsw/shared.rs src/am/ec_spire/coordinator/hierarchy_snapshots.rs src/am/common/parallel.rs src/quant/hadamard.rs src/am/ec_spire/coordinator/snapshots.rs'`
- Timestamp: 2026-05-20 07:44:17 -07:00
- Exit code: 0
- Result: all original top-15 files meet their Task 50 targets.

### `final-top40-unsafe-count.log`

- Command: `rg --files src -g '*.rs' | xargs bash scripts/unsafe_block_count.sh | head -40`
- Timestamp: 2026-05-20 07:44:17 -07:00
- Exit code: 0
- Result: final top-40 distribution captured for follow-on lane planning.

### `git-status-short.log`

- Command: `git status --short`
- Timestamp: 2026-05-20 07:44:17 -07:00
- Exit code: 0
- Result: only untracked `callgrind.out.1807567` and this closeout packet were present at capture time.

## Benchmark Baseline Citation

- Baseline packet: `benchmarks/task-50-local-baseline/manifest.md`
- Baseline HEAD: `cc06046177ce63f969da51150d66a83260efe4d7`
- Baseline host: `DESKTOP-BMB4AFO` WSL2, Intel i9-10900K, AVX2/FMA, no AVX512.
