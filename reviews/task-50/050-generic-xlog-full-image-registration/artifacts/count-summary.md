# Count Summary

- head SHA: `cdfe24f198bfdf8a6b075956221df3450b04de4a`
- previous SHA: `6498d831`
- task bucket: `reviews/task-50/050-generic-xlog-full-image-registration/`
- timestamp: `2026-05-20`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_diskann/ambuild.rs` | 40 | 38 | -2 |
| `src/am/ec_diskann/insert.rs` | 45 | 40 | -5 |
| `src/am/ec_diskann/routine.rs` | 56 | 56 | 0 |
| `src/am/ec_hnsw/build.rs` | 31 | 30 | -1 |
| `src/am/ec_hnsw/insert.rs` | 78 | 73 | -5 |
| `src/am/ec_hnsw/shared.rs` | 44 | 44 | 0 |
| `src/am/ec_hnsw/vacuum.rs` | 65 | 65 | 0 |
| `src/am/ec_ivf/build.rs` | 22 | 21 | -1 |
| `src/am/ec_ivf/page.rs` | 64 | 56 | -8 |
| `src/am/ec_spire/page.rs` | 53 | 48 | -5 |
| `src/storage/wal.rs` | 4 | 4 | 0 |
| `src/` total | 2284 | 2257 | -27 |

## Disposition

- Added `GenericXLogTxn::register_locked_buffer_full_image`, which accepts a `LockedBufferGuard` and fixes the current ECAZ GenericXLog flag shape to full-page images.
- Removed the unused raw `register_buffer` method after all current call sites moved to the guard-based API.
- Left `GenericXLogTxn::start` unsafe because it still accepts a raw PostgreSQL relation pointer.
