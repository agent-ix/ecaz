# Artifacts — Task 58.1 / slice 002 (Audit 1)

- head SHA: `d7d019aa7` (Task 58.1/002 commit)
- branch: `task-58-1-floor-recovery`
- task bucket: `reviews/task-58/`
- packet: `005-task-58-1-audit-1/`
- lane / fixture: N/A — structural shape migration only, no
  workload run.
- storage format / rerank mode: N/A
- isolated one-index-per-table vs shared-table: N/A (compile-gate
  only)

## Commands

- count before / after:
  ```
  grep -c "unsafe {" src/am/ec_hnsw/build_parallel.rs
  ```
  before slice (post-Task-58 closeout): **84**
  after slice 002: **84** (metric-neutral — Audit 1 is shape, not count)

- compile-gate:
  ```
  cargo check --no-default-features --features pg18 --lib
  ```
  status: passes (only unrelated pre-existing
  `unused import` warning in `src/am/ec_spire/update.rs:26`).

- clippy on touched file:
  ```
  cargo clippy --no-default-features --features pg18 --lib 2>&1 \
    | grep -c "build_parallel.rs"
  ```
  hits in `build_parallel.rs`: **0**

- anti-pattern call-site grep:
  ```
  grep -nE "(parts|self)\.(header|header_mut|node|node_mut)\(" \
    src/am/ec_hnsw/build_parallel.rs
  ```
  result: **0 matches** — anti-pattern shape removed crate-wide
  for this file.

## Key result lines `request.md` cites

- 4 safe `fn(&self) -> &T` / `fn(&mut self) -> &mut T` accessors on
  `EcHnswConcurrentDsmGraphParts` replaced with `with_*` closure ops.
- 6 call-sites migrated (3 internal, 3 external).
- `node_lock()` left for Audit 3 (LWLock dispatch typing); already
  documented in Task 58.1 plan's Audit 1 table.
- `node_insert_state_cell()` left as-is — already op-shaped via
  typed `PgLockedDsmInsertStateCell`.

## Timestamp

2026-05-25 — Audit 1 API shape committed as `d7d019aa7` on
`task-58-1-floor-recovery`; review packet at `d60161669`;
slice 002.1 doc-parity follow-up committed at `387103152`
addressing reviewer seq 01 HARD BLOCK.

## Slice 002.1 follow-up — doc parity

- 15/15 `unsafe fn` declarations now documented (verified via
  the `grep -cE`/`grep -c` parity pair from the reviewer's seq 01
  feedback).
- Block count unchanged at 84 (doc-only change).
- `cargo check` passes; 0 clippy hits in `build_parallel.rs`.
