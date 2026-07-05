# Task 50 Review Request: HNSW Vacuum Relation Writer

## Summary

This packet handles `src/am/ec_hnsw/vacuum.rs`, one of the remaining top-15 HNSW files.

Code commit:

- `aacded93 Reduce HNSW vacuum relation unsafe blocks`

The change adds a private `VacuumIndexRelation` wrapper and `VacuumPageRewrite` guard for vacuum-owned index relation, block-count, buffer-lock, and GenericXLog rewrite operations. The wrapper is constructed at the PostgreSQL vacuum callback boundary and then carries the live-relation invariant through pass 1, graph repair, pass 2, and finalization.

## Unsafe Count Status

Counts are from `artifacts/block-count-after.log`.

| File | Task 50 start | Follow-up count | Target | Status |
| --- | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/vacuum.rs` | 99 | 68 | <=69 | met |

## Review Notes

- Vacuum pass ordering is unchanged: pass-1 heap-TID pruning, graph repair request collection, stale neighbor unlinking, replacement planning, repair application, fully-dead finalization, and metadata entry-point repair still run in the same order.
- Page rewrite behavior is unchanged: tuple rewrites still happen under exclusive buffer locks and GenericXLog full-image registration.
- The new wrappers centralize the live relation/buffer/WAL invariants that were previously repeated at each call site.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_hnsw/vacuum.rs'` passed with count 68.
- `rustfmt --edition 2021 --check src/am/ec_hnsw/vacuum.rs` passed, with only existing stable-rustfmt warnings about unstable import options.
- `git diff --check` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.

No benchmark result is claimed in this packet. The slice changes vacuum ownership structure but not graph scoring, candidate ordering, tuple encodings, or rewrite ordering.
