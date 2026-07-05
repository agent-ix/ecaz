# Task 41 HNSW concurrent DSM lock guard

## Summary

This packet requests review for Task 41 invariant #3 follow-up work in the
HNSW concurrent DSM graph builder.

Code commit: `ad8f6f0e45740a13da34b4cad55ebab8f146a328`

Changes:

- Added `EcHnswConcurrentDsmLockGuard`, a local RAII wrapper that drops through
  the injected concurrent-DSM lock release function.
- Added guard-producing `EcHnswConcurrentDsmLockOps::shared` and
  `EcHnswConcurrentDsmLockOps::exclusive` methods, preserving the existing
  PostgreSQL/test lock injection shape.
- Replaced manual `locks.release(...)` calls in the node insert, node complete,
  successor load, and backlink update paths with scoped guards.
- Updated `scripts/unsafe_comment_baseline.txt`; baseline entries moved from
  `3698` to `3691`, and `src/am/ec_hnsw/build_parallel.rs` moved from `211`
  to `204`.

## Safety Effect

The HNSW concurrent DSM graph assembly path no longer relies on paired manual
release calls at each `continue`, `return`, or `pgrx::error!` edge in the
touched lock scopes. The lock remains held for the same data-access region, but
release now happens through `Drop`, including early exits that pgrx converts to
PostgreSQL ERROR.

The raw PostgreSQL lock functions remain centralized in the local
`concurrent_dsm_lwlock_*` adapter functions. Call sites now acquire locks via
`locks.shared(...)` or `locks.exclusive(...)`.

## Review Focus

- Confirm the source-node shared lock in
  `load_concurrent_dsm_successor_candidates_into` still drops before scoring
  neighbors outside the lock.
- Confirm `add_concurrent_dsm_backlinks` releases the target lock on every
  `continue`, after slot mutation, and on ERROR paths.
- Confirm test lock injection remains intact via `EcHnswConcurrentDsmLockOps`.

## Validation

See `artifacts/manifest.md` for the command log and key result lines.
