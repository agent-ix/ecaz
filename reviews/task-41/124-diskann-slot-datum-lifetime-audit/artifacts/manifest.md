# Manifest: Task 41 Invariant #2 DiskANN slot Datum lifetime audit

- head SHA: `902d924bed5b3f107b8033ed38f7daa9d497dc89`
- task bucket and packet path:
  `reviews/task-41/124-diskann-slot-datum-lifetime-audit/`
- lane / fixture / storage format / rerank mode: source audit; no SQL fixture,
  storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:08:03Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; no
  benchmark or SQL execution.

## Artifacts

### diskann-slot-callers.log

- command used:
  `rg -n "required_slot_datum\\(|fetch_heap_row_version\\(|with_heap_source_vector\\(|with_ecvector_datum_slice\\(|ExecClearTuple\\(" src/am/ec_diskann -g '*.rs'`
- key result lines:
  - `src/am/ec_diskann/routine.rs:1937`: only production
    `required_slot_datum` caller.
  - `src/am/ec_diskann/routine.rs:1938`: returned Datum is consumed by
    `ambuild::with_ecvector_datum_slice(datum, f)`.
  - `src/am/ec_diskann/routine.rs:1939`: slot is cleared after closure-scoped
    consumption.

### scan-state-slot-excerpt.log

- command used:
  `sed -n '259,300p' src/am/ec_diskann/scan_state.rs`
- key result lines:
  - `fetch_heap_row_version` clears the slot before fetching a new tuple.
  - `required_slot_datum` materializes slot attributes, rejects NULL, and
    returns the raw Datum.

### routine-rerank-slot-excerpt.log

- command used:
  `sed -n '1896,1941p' src/am/ec_diskann/routine.rs`
- key result lines:
  - `with_heap_source_vector` fetches the tuple, reads the Datum, consumes the
    source vector through a closure, then clears the slot.
  - `exact_heap_rerank_distance` returns only the computed `f32`.

### git-status.log

- command used:
  `git status --short --branch`
- key result lines:
  - branch was `task41-invariant2-lifetimes`.
  - only the new audit packet was untracked.
