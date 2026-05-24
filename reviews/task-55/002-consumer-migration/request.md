# Task 55 Packet 002 — DiskANN Consumer Migration to Phase-1 Wrappers

Status: **proposed**

Migrates DiskANN's HNSW-shaped page/WAL/buffer/scan consumer chain
onto the Phase-1 wrappers landed by Tasks 53 (P6 datum) and 54 (P3
page/WAL/buffer). Single packet covering all three §Migration-Target
files (`routine.rs`, `ambuild.rs`, `insert.rs`) plus the small-file
self-narrows.

## Per-file unsafe block counts

| File | Pre | Post | Δ | §Target | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_diskann/routine.rs` | 27 | **14** | **-13 (-48%)** | ≤ 16 | **met (+2 margin)** |
| `src/am/ec_diskann/ambuild.rs` | 19 | **11** | **-8 (-42%)** | ≤ 11 | **met (0 margin)** |
| `src/am/ec_diskann/insert.rs` | 8 | **5** | **-3 (-38%)** | ≤ 5 | **met (0 margin)** |
| `src/am/ec_diskann/scan_state.rs` | 5 | 3 | -2 | residual | self-narrow |
| `src/am/ec_diskann/diagnostics.rs` | 1 | 0 | -1 | residual | self-narrow |
| `src/am/ec_diskann/cost.rs` | 4 | 4 | 0 | residual | no migration this slice |
| `src/am/ec_diskann/options.rs` | 1 | 1 | 0 | residual | no migration this slice |
| **DiskANN subsystem total** | **65** | **38** | **-27 (-42%)** | ≤ 40 | **met (+2 margin)** |
| `src/` total | 949 | 922 | -27 | — | net category absorption |

All three hard §Migration-Targets met; subsystem total `≤ 40` §Exit
gate met with +2 margin.

## Migrations performed

### `routine.rs`

- `apply_tuple_rewrites(rel)` → split into safe `apply_tuple_rewrites_handle(handle, rewrites)` body plus a thin `unsafe fn apply_tuple_rewrites` shim. Safe body consumes `LockedBufferGuard::read_main_handle` + `wal::WalTxnScope::start_handle` + `register_page` (P3 wrappers from Task 54). Drops the per-loop `unsafe { LockedBufferGuard::read_main(...) }` and `unsafe { wal::GenericXLogTxn::start(...) }` wraps.
- `write_raw_tuple_bytes` (cfg pg_test) similarly migrated to handle inputs and P3 wrappers.
- `count_live_node_tuples` and `plan_diskann_backlink_repair` callers of `materialize_chain_from_index` graduated to the new safe `_handle` variant in `scan_state.rs`.
- Test helpers (`index_metadata`, `index_materialized_chain`, `test_ec_diskann_session_list_size_override_changes_scan_width`) consume the safe `_handle` variant — drops 4 test-side `unsafe { ... }` wraps.
- Consolidated the L1101 `unsafe { apply_tuple_rewrites(...) }` to a direct safe call against `index_relation_handle` already held locally.

### `ambuild.rs`

- `write_data_pages` → safe `fn(handle: RelationHandle, chain: &DataPageChain)`. Consumes P3 `LockedBufferGuard::read_main_locked_handle` + `WalTxnScope::start_handle` + `RegisteredBufferPage::{init, add_item}`; the open-coded `if offset == InvalidOffsetNumber { error!() }` is now `page.add_item(tuple).unwrap_or_else(...)`.
- `initialize_metadata_page(rel, ...)` → split into safe `initialize_metadata_page_handle(handle, ...)` body + `unsafe fn` shim.
- `overwrite_metadata_page(rel, ...)` → same split.
- `write_metadata_to_buffer` consumes the P3 wrappers; only the special-area copy (`pg_sys::PageGetSpecialPointer` + `ptr::copy_nonoverlapping`) retains its unsafe block (irreducible PG-extern + raw-ptr op).
- `with_ecvector_datum_slice` / `ecvector_datum_to_vec` (unsafe fn) consolidate inner `unsafe { ... }` wraps that were redundant inside the `unsafe fn` body, keeping SAFETY documentation at the function header.

### `insert.rs`

- `DiskannInsertRelation::read_main` / `read_main_locked` consume the safe `_handle` variants; the per-call `unsafe { ... }` wrap is gone (now wrapper-internal).
- `bind_duplicate_heap_tid` consumes the safe `materialize_chain_from_index_handle` directly via `index_relation.handle()`.

### `scan_state.rs`

- New safe `materialize_chain_from_index_handle(handle: RelationHandle)` is the canonical body; `unsafe fn materialize_chain_from_index(rel)` becomes a thin validating shim. Consumes `LockedBufferGuard::read_main_handle` for both the metadata page (block 0) and each data block.

### `diagnostics.rs`

- `summarize_diskann_graph` consumes `scan_state::materialize_chain_from_index_handle` directly — drops its only `unsafe { ... }` wrap.

## Phase-1 wrapper extensions

None required. All migration patterns matched Task 53/54 precedents
directly; no new wrapper surface needed.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` — passes.
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings` — pre-existing repo-wide lints unchanged; Task 55 introduces zero new clippy warnings (verified by spot-check on touched files).
- No behavior change: all migrations are call-site moves; PG semantics (PageInit / PageAddItemExtended / GenericXLog) are unchanged.

## Files touched

- `src/am/ec_diskann/routine.rs`
- `src/am/ec_diskann/ambuild.rs`
- `src/am/ec_diskann/insert.rs`
- `src/am/ec_diskann/scan_state.rs`
- `src/am/ec_diskann/diagnostics.rs`

## References

- `plan/tasks/55-diskann-unsafe-burndown.md` §Migration Targets, §Exit Criteria.
- `reviews/task-55/001-execution-plan/request.md` — slice plan.
- `reviews/task-54/005-closeout/request.md` — P3 wrapper surface and DiskANN handoff list cited as targets.
- `reviews/task-53/004-closeout/request.md` — P6 datum wrapper surface.
