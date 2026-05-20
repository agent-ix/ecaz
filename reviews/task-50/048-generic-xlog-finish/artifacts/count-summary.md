# Count Summary

- head SHA: `616e97547b0e48a001563a89e3283152d53af984`
- previous SHA: `fdcfb9e7`
- task bucket: `reviews/task-50/048-generic-xlog-finish/`
- timestamp: `2026-05-20`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_diskann/ambuild.rs` | 42 | 40 | -2 |
| `src/am/ec_diskann/insert.rs` | 50 | 45 | -5 |
| `src/am/ec_hnsw/build.rs` | 32 | 31 | -1 |
| `src/am/ec_hnsw/insert.rs` | 86 | 78 | -8 |
| `src/am/ec_hnsw/vacuum.rs` | 66 | 65 | -1 |
| `src/am/ec_ivf/build.rs` | 23 | 22 | -1 |
| `src/am/ec_ivf/page.rs` | 72 | 64 | -8 |
| `src/am/ec_spire/page.rs` | 58 | 53 | -5 |
| `src/storage/wal.rs` | 4 | 4 | 0 |
| `src/` total | 2321 | 2290 | -31 |

## Disposition

- Made `GenericXLogTxn::finish` safe because the transaction owns the PostgreSQL `GenericXLogState` and consumes `self`, preventing double finish.
- Left `GenericXLogTxn::start` and `register_buffer` unsafe because they still take raw PostgreSQL relation/buffer state from callback context.
- Removed caller-side unsafe wrappers from WAL finish calls across IVF, SPIRE, HNSW, and DiskANN.
