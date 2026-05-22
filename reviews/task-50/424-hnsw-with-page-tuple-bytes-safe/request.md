# Task 50/424: HNSW shared.rs — `with_page_line_tuple_bytes` + `with_writable_page_tuple_bytes` safe-fn lifts

## Why this slice

`shared::with_page_line_tuple_bytes` (immutable page tuple visitor)
and `shared::with_writable_page_tuple_bytes` (writable variant) were
the last two heavily-called `unsafe fn` helpers in HNSW's
`shared.rs` page-decoding surface. Each retains exactly one internal
`unsafe { &*shared::page_item_id(...) }` + one
`unsafe { from_raw_parts(...) }` block with the bounds-validation
contract already in place. Lifting both to safe `fn` strips a
substantial chain of caller-side `unsafe { ... }` wraps from
shared.rs, scan.rs, insert.rs, and vacuum.rs.

## Scope

Two function lifts in `src/am/ec_hnsw/shared.rs`:

1. `with_page_line_tuple_bytes<R, F>(page_ptr, page_size,
   block_number, offset, context, visit)` → safe `fn`. Body retains
   the `unsafe { &*page_item_id(...) }` and `unsafe { from_raw_parts(...) }`
   blocks.
2. `with_writable_page_tuple_bytes<R, F>(page_ptr, page_size,
   tuple_tid, tuple_kind, visit)` → safe `fn`. Same shape.

Caller-side `unsafe { ... }` wraps stripped across HNSW:

- `shared.rs`: 2 (count_element_tuples, highest_level_live_entry_candidate)
- `scan.rs`: 1 (linear scan element decoder, ~line 4905)
- `insert.rs`: 7 (backlink encode + repair-replacement encoders, lines
  ~1322, ~2136, ~2211, ~2291, ~2374, ~2451, ~2531)
- `vacuum.rs`: 9 (pass-1 plan / apply / pass-2 plan / apply / repair
  collect / linear-repair candidate / apply repair plans, lines ~663,
  ~787, ~925, ~1428, ~1633, ~1731, etc.)

19 caller wraps removed total. Where the original `unsafe { ... }`
block held the call expression directly, the close-`}` was joined
with the closing `);` of the call to preserve the trailing
semicolon.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/shared.rs` | 29 | 27 | -2 |
| `src/am/ec_hnsw/scan.rs` | 89 | 88 | -1 |
| `src/am/ec_hnsw/insert.rs` | 52 | 45 | -7 |
| `src/am/ec_hnsw/vacuum.rs` | 50 | 41 | -9 |
| **HNSW subsystem subtotal** | **426** | **407** | **-19** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 423 | 426 |
| After 424 | 407 |

Net rotation delta: **-142 in HNSW** (-25.9%).

## Soundness rationale

Each retained internal `unsafe { ... }` block has the same SAFETY
contract it had before this slice (offset bounded by line-pointer
count, tuple_offset + tuple_len bounded by page_size). The lift
moves the obligation inside the function body where it's
re-validated against any nearby code change.

No anti-pattern B: both functions return `Result<Option<R>, String>`
/ `R`, not `&'a T`.

## Validation

Artifacts under `reviews/task-50/424-hnsw-with-page-tuple-bytes-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (466 lines)
- `cargo-check-pg18.log` — clean, **0 unused_unsafe warnings**.

## Performance gate

Scan/insert/vacuum hot path (page-tuple visitor is the inner loop
for every page-level mutation in HNSW). No semantic change — same
item-id checks, same byte-slice bounds, same callback shape. Bench
deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

**Past the -25% threshold** on HNSW: 549 → 407, net -142 (-25.9%).
Closing in on the Task 50 §Exit Criteria's -30% per-module target.

## Out of scope

- `shared.rs::with_locked_metadata_page` — still `unsafe fn` because
  its callback is given `&mut MetadataPage` and the rewrite path
  calls `pg_sys::PageInit` + `PageAddItem` directly. Queued.
- `shared.rs::initialize_metadata_page`, `update_metadata_page`,
  `count_element_tuples`, `highest_level_live_entry_candidate`,
  `index_admin_snapshot`, `index_cost_snapshot`, `index_explain_snapshot`,
  `planner_integration_snapshot`, `read_data_page`,
  `decode_heap_tid`, `page_item_id`, `ec_hnsw_noop_vacuum_stats`:
  each remaining `unsafe fn` either retains a single internal unsafe
  block tied to a still-`unsafe fn` callee (e.g. `count_element_tuples`
  uses `count_live_elements_on_buffer`, which is the next-up lift)
  or carries the FFI surface obligation directly.
