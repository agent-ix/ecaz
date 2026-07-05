# Task 50 Packet 100 Count Summary

Code commit: `0e94c73dbdf818290793740206a8d22f3a242959`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1801 | 1793 | -8 |
| `src/am/common/cost.rs` | 14 | 14 | 0 |
| `src/am/ec_diskann/cost.rs` | 11 | 9 | -2 |
| `src/am/ec_hnsw/shared.rs` | 44 | 43 | -1 |
| `src/am/ec_ivf/cost.rs` | 6 | 4 | -2 |
| `src/am/ec_spire/cost/mod.rs` | 15 | 13 | -2 |
| `src/am/ec_spire/custom_scan/cost_helpers.rs` | 3 | 2 | -1 |
| `src/am/ec_spire/custom_scan/mod.rs` | 2 | 2 | 0 |
| `src/` unsafe ledger rows | 1801 | 1793 | -8 |

## Deleted Unsafe Call Sites

- Planner cost GUC callers no longer wrap
  `current_planner_cost_constants` in unsafe across AM common, HNSW, IVF,
  SPIRE, and DiskANN cost paths.
- SPIRE custom scan cost estimation no longer reads `pg_sys::cpu_tuple_cost`
  directly at its call site.

## Residual Boundary

Backend-local planner cost global reads are centralized in
`src/am/common/cost.rs::current_planner_cost_constants` and
`src/am/common/cost.rs::current_cpu_tuple_cost`.
