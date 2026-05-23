# Task 50 Packet 101 Count Summary

Code commit: `b1333a5eef6fe016791d2dd2836f07f13a08baf0`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1793 | 1778 | -15 |
| `src/am/common/cost.rs` | 14 | 14 | 0 |
| `src/am/ec_diskann/cost.rs` | 9 | 5 | -4 |
| `src/am/ec_hnsw/shared.rs` | 43 | 42 | -1 |
| `src/am/ec_ivf/cost.rs` | 4 | 0 | -4 |
| `src/am/ec_spire/cost/mod.rs` | 13 | 7 | -6 |
| `src/` unsafe ledger rows | 1793 | 1778 | -15 |

## Deleted Unsafe Call Sites

- AM cost paths no longer read main-fork block count directly at SPIRE, IVF,
  DiskANN, or shared HNSW/common call sites.
- AM cost paths no longer dereference `rd_rel.reltuples` directly at SPIRE,
  IVF, DiskANN, HNSW, or shared common cost call sites.
- `src/am/ec_ivf/cost.rs` now has zero direct unsafe blocks.

## Residual Boundary

The PostgreSQL relation-stat reads are centralized in
`src/am/common/cost.rs::relation_main_fork_block_count` and
`src/am/common/cost.rs::relation_reltuples`, both of which reject null relation
pointers before reading PostgreSQL relation state.
