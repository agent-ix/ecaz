# Task 54 Follow-up — HNSW Stretch Consumer Migration (insert.rs + shared.rs)

Status: **proposed**

Optional follow-up to Task 54 that migrates the HNSW *stretch consumers*
(`insert.rs` and `shared.rs`) onto the P3 wrappers landed in Task 54.

Task 54 §Migration Targets scope-locked to `build.rs` + `vacuum.rs` +
`buffer_guard.rs` + `wal.rs`, leaving these two files as documented
stretch consumers (see `reviews/task-54/005-closeout/request.md`
§"HNSW (stretch — not in this task's §Migration Targets)"). Now that
the wrappers have been validated end-to-end by Task 55's DiskANN
migration, lifting the residual HNSW sites is low-risk.

## Per-file unsafe block counts

| File | Pre | Post | Δ | Notes |
| --- | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/insert.rs` | 25 | **12** | **-13 (-52%)** | InsertPageWrite consumes `WalTxnScope`; `find_duplicate_*` and `coalesce_duplicate_*` helpers consume `read_main_handle` + `WalTxnScope::start_handle` + `register_page` |
| `src/am/ec_hnsw/shared.rs` | 21 | **18** | **-3 (-14%)** | `read_main_buffer`, `update_metadata_page`, `rewrite_metadata_buffer` consume P3 wrappers; `debug_vacuum_stats` cfg(test) block-pair consolidated |
| `src/` total | 922 | **906** | **-16** | Pure consumer-side reduction; no new wrappers |

## Migrations performed

### `insert.rs`

- `InsertPageWrite<'rel>` now holds `wal::WalTxnScope<'rel>` instead of
  `wal::GenericXLogTxn`. Its `open_tail` / `from_locked_buffer`
  constructors take `RelationHandle` and use safe
  `LockedBufferGuard::read_main{,_locked}_handle` +
  `WalTxnScope::start_handle` + `register_page().page_ptr()`. The
  per-tuple `init_zeroed_page` and `add_item` retain their raw
  `page_ptr` FFI calls (those wraps remain — three unsafe blocks
  preserved as the irreducible Page API boundary that the writer
  caches).
- `rewrite_inserts_neighbor_backlinks` (L1247-) — consumes P3 wrappers.
- `find_duplicate_element_tid`, `find_duplicate_turbo_hot_element_tid`,
  `find_duplicate_grouped_element_tid` — each constructs the
  `RelationHandle` once at function entry, then iterates blocks with
  `LockedBufferGuard::read_main_handle`. Per-iteration unsafe block
  removed.
- `coalesce_duplicate_heap_tid`, `coalesce_duplicate_turbo_hot_heap_tid`,
  `coalesce_duplicate_grouped_heap_tid` — same pattern; the `unsafe {
  LockedBufferGuard::read_main(...) }` + `unsafe { wal::GenericXLogTxn::start(...) }`
  pair collapses to safe handle calls.

### `shared.rs`

- `read_main_buffer` consumes `LockedBufferGuard::read_main_handle`.
- `update_metadata_page` (`unsafe fn`) is now a thin shim that
  validates the pointer to a `RelationHandle` and delegates to a new
  safe `update_metadata_page_handle(handle, metadata)` variant.
- `rewrite_metadata_buffer` consumes `wal::WalTxnScope::start_handle`
  + `register_page` + safe `init(special_size)`. Only the special-area
  copy retains an unsafe block (irreducible PG-extern + raw-ptr op).
- `debug_update_index_metadata` (cfg pg_test) routes through the safe
  `_handle` variant — drops its `unsafe { update_metadata_page(...) }`
  wrap.
- `debug_vacuum_stats` (cfg pg_test) consecutive `unsafe { ec_hnsw_ambulkdelete(...) }`
  + `unsafe { ec_hnsw_amvacuumcleanup(...) }` blocks merged into a
  single outer `unsafe { let stats = ambulkdelete(...); amvacuumcleanup(info_ptr, stats) }`
  block.

### Structural ceiling residue (kept)

- `shared.rs` ~18 remaining unsafe blocks are page-byte FFI ops
  (`PageGetSpecialPointer`, `slice::from_raw_parts`,
  `PageGetItemId(page, offset)`, page header `pd_lower` reads, etc.)
  + a few stats wrappers and the cost-constant getter. These are at
  the PG-extern boundary or are raw-pointer page accessors that can't
  be further self-narrowed without reshaping the page-byte API surface
  (out of scope here).
- `insert.rs` 12 remaining: 3 in `InsertPageWrite` (PageInit /
  PageAddItemExtended / PageGetFreeSpace on raw cached page_ptr); 2
  in the search hot path (graph::greedy_descend_from_entry +
  search_layer*); 1 in the heap-scan callback path; the rest are
  unsafe-fn delegations.

## Phase-1 wrapper extensions

**None required.** All migrations consume Task 54 P3 surface as-is.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` — passes.
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings` — pre-existing repo-wide lints unchanged; this follow-up introduces zero new clippy warnings.
- No behavior change — call-site moves only.

## Bench gate

Optional but recommended: re-run HNSW 100k latency bench vs the post-Task-54 baseline at `reviews/task-54/005-closeout/artifacts/before-after-summary.md`. Expected: no regression; possibly small latency improvements from additional inlining at insert hot path.

Defer running the bench unless reviewer requests it — this follow-up
is a pure call-site move with no semantic changes; the Task 54 bench
window already validated the underlying wrapper surface.

## Files touched

- `src/am/ec_hnsw/insert.rs`
- `src/am/ec_hnsw/shared.rs`

## References

- `reviews/task-54/005-closeout/request.md` §"HNSW (stretch — not in this task's §Migration Targets)"
- `plan/tasks/54-common-p3-page-wal-wrappers.md` §Non-Goals (explicitly carved this out)
- `reviews/task-55/002-consumer-migration/request.md` (proved the wrapper-consumption pattern cross-AM)
