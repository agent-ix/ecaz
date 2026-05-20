# Task 50 Packet 105 Count Summary

- code commit: `e50d9e59f9fba3f2428cfa27bf59ccb4613cdf95`
- scope: HNSW planner cost boundary cleanup
- before baseline: packet 104 after-state, `1763` direct unsafe blocks across `126` files under `src/`
- after state: `1756` direct unsafe blocks across `126` files under `src/`
- delta: `-7` direct unsafe blocks

## Touched File Movement

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/common/cost.rs` | 13 | 6 | -7 |
| `src/` total direct unsafe blocks | 1763 | 1756 | -7 |

## Notes

- HNSW planner callback bodies now use the shared `pg_am_callback!` boundary
  instead of hand-written local `pgrx_extern_c_guard` unsafe blocks.
- Planner cost GUC reads are grouped into one residual unsafe block.
- Planner tree-height reads now use a safe helper with a null `IndexOptInfo`
  guard; the raw `tree_height` field read remains owned by the helper.
