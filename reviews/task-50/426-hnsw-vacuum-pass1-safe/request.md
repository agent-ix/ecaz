# Task 50/426: HNSW vacuum.rs — `plan_page_pass1` + `heap_tid_is_dead` safe-fn lifts

## Why this slice

Two more cascading lifts from the slice 424/425 progress in
`vacuum.rs`:

- `heap_tid_is_dead(heap_tid, callback, callback_state)` — body
  computes a PG `ItemPointer`, then dispatches a single FFI callback.
  The `unsafe { callback(...) }` block is the only unsafe op.
- `plan_page_pass1(page_ptr, page_size, block_number, storage,
  callback, callback_state)` — body uses the now-safe
  `shared::with_page_line_tuple_bytes` (slice 424) and the
  now-safe `heap_tid_is_dead` (this slice). Zero internal unsafe
  blocks remain.

Lifting both removes two caller-side wraps in `rewrite_page_pass1`
and `repair_metadata_entry_point_after_vacuum`'s share-buffer plan
loop.

## Scope

- `heap_tid_is_dead` lifted to safe `fn`. Internal
  `unsafe { callback(...) }` block retained with SAFETY comment.
- `plan_page_pass1` lifted to safe `fn`. Zero internal unsafe blocks.
- 2 caller-side `unsafe { ... }` wraps stripped: in
  `repair_metadata_entry_point_after_vacuum` (line ~514) and
  `rewrite_page_pass1` (line ~617).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/vacuum.rs` | 39 | 37 | -2 |
| **HNSW subsystem subtotal** | **398** | **396** | **-2** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 425 | 398 |
| After 426 | 396 |

Net rotation delta: **-153 in HNSW** (-27.9%).

## Soundness rationale

`heap_tid_is_dead` body composes only safe operations except for the
single FFI callback call, which retains its SAFETY comment. The PG
bulkdelete callback contract ("supplied by PostgreSQL for this
ambulkdelete invocation; `tid` lives for the callback call") is
unchanged.

`plan_page_pass1` body uses only safe helpers. No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/426-hnsw-vacuum-pass1-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Vacuum hot path (pass-1 plans every page during ambulkdelete). No
semantic change — same callback wiring, same tuple bounds, same
plan accumulation. Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

The remaining 22 `unsafe fn`s in vacuum.rs each have at least one
internal unsafe block tied to a still-unsafe-fn callee (e.g.
`apply_page_pass1_updates`, `plan_page_pass2`, `apply_page_pass2`,
the repair-replacement family, the linear-repair candidate loader).
Each will lift after its inner FFI dependency moves to safe `fn`.
Queued.
