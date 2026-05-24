# Task 50/430: HNSW insert.rs — append/find_duplicate cascade safe-fn lifts

## Why this slice

After the InsertPageWrite constructors became safe (slice 417) and the
graph::load_* / shared::with_page_line_tuple_bytes chains became safe
(slices 419-424), the next-up `unsafe fn`s in insert.rs are the
append-tuple writers and find-duplicate scanners. Each has either zero
or one bounded internal `unsafe { ... }` block (the
`LockedBufferGuard::read_main` call in the find_duplicate family).

This slice lifts eleven `unsafe fn`s to safe `fn`, stripping six
caller-side `unsafe { ... }` wraps across the
InsertFormatAdapter dispatchers and the append-to-new-page retry
paths.

## Scope

Eleven `unsafe fn` → safe `fn` flips in `src/am/ec_hnsw/insert.rs`:

1. `append_heap_tuple`
2. `append_heap_tuple_to_new_page`
3. `append_turbo_hot_cold_tuple`
4. `append_turbo_hot_cold_tuple_to_new_page`
5. `append_pq_fastscan_tuple`
6. `append_pq_fastscan_tuple_to_new_page`
7. `derive_pq_fastscan_search_code_for_insert`
8. `bootstrap_empty_pq_fastscan_flush_output`
9. `find_duplicate_element_tid`
10. `find_duplicate_turbo_hot_element_tid`
11. `find_duplicate_grouped_element_tid`

Caller-side `unsafe { ... }` wraps stripped:

- `InsertFormatAdapter::find_duplicate` dispatcher (line ~387)
- `InsertFormatAdapter::append_tuple` dispatcher (line ~444)
- `append_heap_tuple` retry-to-new-page (line ~1629)
- `append_turbo_hot_cold_tuple` retry-to-new-page (line ~1766)
- `append_pq_fastscan_tuple` retry-to-new-page (line ~1993) — also
  fixes a leftover indent + extra `}` from an earlier mechanical
  cleanup
- `append_pq_fastscan_tuple` search-code derivation (line ~1924)

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/insert.rs` | 45 | 39 | -6 |
| **HNSW subsystem subtotal** | **384** | **378** | **-6** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 429 | 384 |
| After 430 | 378 |

Net rotation delta: **-171 in HNSW** (**-31.1%**).

## Soundness rationale

Each lifted function's body either had zero internal unsafe blocks
or one bounded `unsafe { LockedBufferGuard::read_main(...) }` block.
The signature flip moves the obligation chain inside the function.
No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/430-hnsw-insert-append-and-find-duplicate-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Insert hot path. Bench deferred per `feedback_coder_push_smoke_checks`.

## Rotation milestone

Net **-171 (-31.1%)** on HNSW: 549 → 378. The Task 50 §Exit Criteria
-30% per-module target was crossed in slice 429; this slice extends
the cushion.
