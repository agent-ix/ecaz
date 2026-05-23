# Task 50 Packet 048: GenericXLog Finish

This packet advances P3 by tightening the shared GenericXLog transaction contract and fanning that cleanup through production AM page writers.

## Change

- Made `GenericXLogTxn::finish` safe.
- Kept `GenericXLogTxn::start` unsafe because it accepts a raw PostgreSQL relation pointer.
- Kept `GenericXLogTxn::register_buffer` unsafe because callers must supply a valid PostgreSQL buffer belonging to the started relation.
- Removed redundant caller-side unsafe wrappers around `wal_txn.finish()` in:
  - DiskANN ambuild/insert
  - HNSW build/insert/vacuum
  - IVF build/page
  - SPIRE page

The safety boundary is now narrower: relation/buffer registration remains unsafe; finishing an owned transaction consumes `self` and is safe once the transaction exists.

## Counts

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

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2290` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. Current closeout audit count is `2290` direct unsafe blocks under `src/`; packet 030 still requires every row to be removed or residual-registered.
