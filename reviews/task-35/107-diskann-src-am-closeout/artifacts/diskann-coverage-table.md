# DiskANN Task 35 Coverage Table

| Packet | Surface | Baseline Movement | Notes |
| --- | --- | ---: | --- |
| `062-diskann-cost-safety` | `src/am/ec_diskann/cost.rs` | `2 -> 0` | Cost estimate snapshots and relation option reads. |
| `069-diskann-options-safety` | `src/am/ec_diskann/options.rs` | `6 -> 0` | PostgreSQL reloptions parsing and default handling. |
| `088-diskann-scan-state-safety` | `src/am/ec_diskann/scan_state.rs` | `24 -> 0` | Page traversal, tuple materialization, heap fetch helpers. |
| `090-diskann-insert-safety` | `src/am/ec_diskann/insert.rs` | `50 -> 0` | Insert callback state, vector datum extraction, page writes. |
| `100-diskann-routine-safety` | `src/am/ec_diskann/routine.rs` | `91 -> 0` | AM routine callbacks, vacuum, scan, rewrite, and test helpers. |
| `105-diskann-build-page-datum-safety` | `src/am/ec_diskann/ambuild.rs` build/page/datum layer | `57 -> 27` | Build callbacks, page writes, metadata writes, vector datum decode. |
| `106-diskann-build-simd-safety` | `src/am/ec_diskann/ambuild.rs` SIMD/test-kernel layer | `27 -> 0` | AVX2/FMA and NEON dispatch, vector loads/stores, scalar tails. |

## Current Residual

- `src/am/ec_diskann`: `0` unsafe-comment baseline entries.
- `src/am`: `0` unsafe-comment baseline entries.
- Global baseline: `499` entries across `35` files, all under `src/tests/`.

Task 35 DiskANN production packets cleared `230` baseline entries.
