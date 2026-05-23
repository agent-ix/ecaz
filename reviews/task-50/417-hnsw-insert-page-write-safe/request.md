# Task 50/417: HNSW insert.rs — `InsertPageWrite` constructors safe-fn lifts

## Why this slice

`InsertPageWrite::{open_tail, open_new, from_locked_buffer}` were the
last three `unsafe fn` constructors on the insert-time page writer in
`src/am/ec_hnsw/insert.rs`. Each retained only the FFI-boundary
unsafe blocks (`LockedBufferGuard::read_main_locked` /
`LockedBufferGuard::read_main` / `wal::GenericXLogTxn::start`) inside
its body; the rest of each function was safe Rust. They were
`unsafe fn` only to push the caller-supplied precondition ("live
index relation, locked buffer belongs to that relation") to callers.

Lifting all three to safe `fn`:

- Removes 6 caller-side `unsafe { ... }` wrappers across insert.rs
  (each `append_*_tuple` / `append_*_tuple_to_new_page` site).
- Internal `unsafe { from_locked_buffer(...) }` inside `open_tail`
  disappears (callee now safe).
- Internal `unsafe { Self::open_tail(...) }` inside `open_new`
  disappears (callee now safe).

Net per-file delta: -8.

## Scope

Three function lifts in `src/am/ec_hnsw/insert.rs`:

1. `InsertPageWrite::from_locked_buffer` → safe `fn`. Retains one
   internal `unsafe { wal::GenericXLogTxn::start(...) }` block.
2. `InsertPageWrite::open_tail` → safe `fn`. Retains one internal
   `unsafe { LockedBufferGuard::read_main_*(...) }` block (the
   if/else over P_NEW vs existing tail). Drops the
   `unsafe { from_locked_buffer(...) }` wrap (now redundant).
3. `InsertPageWrite::open_new` → safe `fn`. Body becomes a single
   safe call to `Self::open_tail(...)`; drops its `unsafe { ... }`
   wrap.

Six caller-side `unsafe { ... }` wraps removed:

- `append_heap_tuple` (line ~1640)
- `append_heap_tuple_to_new_page` (line ~1681)
- `append_turbo_hot_cold_tuple` (line ~1780)
- `append_turbo_hot_cold_tuple_to_new_page` (line ~1834)
- `append_pq_fastscan_tuple` (line ~2010)
- `append_pq_fastscan_tuple_to_new_page` (line ~2067)

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/insert.rs` | 64 | 56 | -8 |
| **HNSW subsystem subtotal** | **482** | **474** | **-8** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 416 | 482 |
| After 417 | 474 |

Net rotation delta: **-75 in HNSW** (-13.7%).

## Soundness rationale

Each retained internal `unsafe { ... }` block has a SAFETY comment
naming the caller-supplied precondition. The pattern matches the
already-safe `read_main_buffer` (slice precedent in `shared.rs`) and
the `parallel_build_shared_workspace_size` / `send_worker_message`
lifts from slice 416.

No anti-pattern B: the constructors return `Self` (an owned
`InsertPageWrite`), not `&'a T`.

## Validation

Artifacts under `reviews/task-50/417-hnsw-insert-page-write-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (117 lines)
- `cargo-check-pg18.log` — clean.

## Performance gate

Insert hot path. No semantic change — same FFI calls with same
arguments. Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

- The 21 other `unsafe fn`s in `insert.rs` (`run_insert_with_adapter`,
  `discover_insert_forward_neighbor_slots`,
  `load_insert_entry_candidate`, `populate_upper_layer_forward_slots`,
  `add_backlinks_to_forward_neighbors`, `plan_backlink_mutations`,
  `apply_backlink_mutations`, `add_backlinks_on_page`,
  `plan_backlink_mutation`, `select_backlink_rewrite_slice`,
  the various `append_*_tuple` / `append_*_tuple_to_new_page` /
  `find_duplicate_*` / `coalesce_duplicate_*` helpers): each holds
  internal unsafe blocks around `graph::*` / `page::*` FFI surfaces
  that themselves need lifts before the wrapping `unsafe fn`s can
  flip. Queued.
