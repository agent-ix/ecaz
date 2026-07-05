# Task 28 IVF rerank width session GUC

## Scope

This packet covers commit `d6a90fb` (`ivf: add session rerank width override`).

The change makes the IVF heap-rerank frontier width configurable through a session/query-time GUC:

- `ec_ivf.rerank_width = -1`: use the index `rerank_width` reloption.
- `ec_ivf.rerank_width = 0`: rerank the full probed frontier.
- `ec_ivf.rerank_width > 0`: bound the heap-rerank frontier to that many approximate candidates.

This mirrors the existing `ec_ivf.nprobe` override pattern and lets recall/latency sweeps vary both `nprobe` and rerank frontier width without rebuilding indexes or repeatedly mutating index reloptions.

## Code paths

- `src/am/ec_ivf/options.rs`: registers `ec_ivf.rerank_width` and resolves relation/session/effective width.
- `src/am/ec_ivf/scan.rs`: uses the resolved effective width for pre-rerank candidate limiting and heap-f32 rerank truncation.
- `src/am/ec_ivf/admin.rs` and `src/lib.rs`: expose relation/session/effective rerank width in `ec_ivf_index_admin_snapshot(...)`.
- `src/lib.rs`: extends PG18 tests for scan behavior and admin snapshot reporting.

## Validation

Focused PG18 tests run:

- `cargo pgrx test pg18 test_ec_ivf_heap_f32_rerank_width_bounds_exact_frontier`
- `cargo pgrx test pg18 test_ec_ivf_admin_snapshot`
- `git diff --check`

All passed.

## Notes for review

The override is intentionally scan-time only. Physical IVF build knobs remain index/build choices: `nlists`, `training_sample_rows`, `storage_format`, and `pq_group_size` still require rebuilding/reindexing to materially change the built index.
