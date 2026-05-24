# Task 50 Packet 050: GenericXLog Full-Image Registration

This packet continues P3 by replacing raw GenericXLog buffer registration call sites with a guard-based full-image registration API.

## Change

- Added `GenericXLogTxn::register_locked_buffer_full_image(&LockedBufferGuard)`.
- Removed the now-unused raw `GenericXLogTxn::register_buffer` method.
- Updated current WAL write sites in DiskANN, HNSW, IVF, and SPIRE to register full-page images through the locked-buffer helper.

The remaining unsafe WAL boundary is `GenericXLogTxn::start`, because it takes a raw PostgreSQL relation pointer. The registration step now requires a `LockedBufferGuard` and fixes the current ECAZ flag shape to full-page images.

## Counts

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

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2257` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. Current closeout audit count is `2257` direct unsafe blocks under `src/`; packet 030 still requires every row to be removed or residual-registered.
