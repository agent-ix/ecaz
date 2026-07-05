# Task 50 Packet 106 Count Summary

- code commit: `e76a31fca63b22fd10876fc50170658ed4fbacc7`
- scope: SPIRE and DiskANN planner cost boundary cleanup
- before baseline: packet 105 after-state, `1756` direct unsafe blocks across `126` files under `src/`
- after state: `1750` direct unsafe blocks across `126` files under `src/`
- delta: `-6` direct unsafe blocks

## Touched File Movement

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_diskann/cost.rs` | 3 | 2 | -1 |
| `src/am/ec_spire/cost/mod.rs` | 7 | 2 | -5 |
| `src/` total direct unsafe blocks | 1756 | 1750 | -6 |

## Notes

- DiskANN planner callback bodies now use the shared `pg_am_callback!`
  boundary instead of a local `pgrx_extern_c_guard` unsafe block.
- SPIRE cost snapshot callers now share safe local helpers for active snapshot
  diagnostics and hierarchy snapshots; those helpers own the remaining raw
  snapshot calls.
