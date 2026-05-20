# Task 50 Packet 049: HNSW Parallel Leader Finish

This packet applies the same owned-cleanup rule from packet 048 to HNSW parallel build leader cleanup.

## Change

- Made `EcHnswParallelGraphBuildLeader::wait_for_workers` safe.
- Made `EcHnswParallelGraphBuildLeader::finish` safe.
- Made `EcHnswParallelBuildLeader::finish` safe.
- Removed caller-side unsafe wrappers around those methods.

The unsafe boundary remains at `begin` and the raw PostgreSQL parallel-context operations inside the leader methods. Once constructed, the leader object owns the `ParallelContext` and cleanup consumes the leader.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 139 | 133 | -6 |
| `src/` total | 2290 | 2284 | -6 |

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2284` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. Current closeout audit count is `2284` direct unsafe blocks under `src/`; packet 030 still requires every row to be removed or residual-registered.
