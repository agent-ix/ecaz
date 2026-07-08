# Review request — Task 165 M3 slice 2: FR-083 delta-buffer insert (D5)

**Branch:** `task-165-ec-distann-m3`. Second M3 DML slice (after tombstone delete).

## What landed

`aminsert` now spools inserts into a bounded exact-scan **delta buffer** (ADR-085
D5) instead of erroring:
- `DistannDeltaTuple` (tag `0x0A`: next_tid + vec_id + heap_tid + vector),
  chained from `metadata.delta_buffer_head`, prepended per insert.
- `append_delta_tuple` extends the relation with a fresh WAL-logged page per
  entry; `read_delta_chain` walks it from the live relation.
- `delta_insert` (local identity mode): dimension-checked, bounded (cap 4096 —
  REINDEX to drain), idempotent per vec_id. `source_identity='include'` and
  empty-index inserts are **rejected with clear messages** (delta-only scan +
  multi-node write endpoint are later slices) — never a silent drop.
- **Scan merge**: after the FR-081 orchestration, the delta buffer is
  exact-scanned and merged into the ranked hits, so inserted rows are visible
  same-statement at their true rank; a vec_id already among graph hits keeps the
  closer (UPDATE = old tombstoned + new delta).

## Evidence (`artifacts/test-evidence.log`)

`test_ec_distann_delta_insert_visible_same_statement` (FR-083-AC-3),
`test_ec_distann_insert_into_empty_index_is_rejected`,
`distann_delta_tuple_round_trips`; delete tombstone still green. **89 pg_tests
pass, 0 failed**; clippy clean.

## Remaining M3 slices

Write endpoint (`ec_distann_apply_record_writes`); FR-082 full lifecycle
(3-worker build/publish/retire); fault-drill matrix (TC-042); 50k multinode
distinct_recall bench. Also tracked: delta-only scan path (empty-index inserts),
include-mode delta routing, and delta drain at epoch build (FR-083-AC-2).

## Ask

Review the delta tuple/buffer, the append/WAL path, and the scan-merge
correctness. Not closing the request.
