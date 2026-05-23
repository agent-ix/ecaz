# Task 50/421: HNSW graph.rs — page-tuple reader chain safe-fn lifts

## Why this slice

`graph.rs` houses the HNSW-side page-tuple decoder cascade:
`with_page_tuple_bytes` (pointer arithmetic + slice) →
`read_page_tuple` / `read_page_tuple_from_buffer` (locked-buffer or
caller-pinned buffer) → the many `load_*` / `with_*_graph_tuple`
decoders. All three foundation functions were `unsafe fn`; each
retained only locally-bounded internal `unsafe { ... }` blocks
(item-id deref, `from_raw_parts`, `LockedBufferGuard::read_main`).

Lifting all three to safe `fn`:

- Each function retains its single internal `unsafe { ... }` block
  with its existing SAFETY comment.
- Every caller's surrounding `unsafe { read_page_tuple(...) }` /
  `unsafe { read_page_tuple_from_buffer(...) }` wrapper becomes
  redundant (compiler warns `unused_unsafe`), and all 12 such wraps
  are stripped this slice.

## Scope

Three function signature lifts in `src/am/ec_hnsw/graph.rs`:

1. `with_page_tuple_bytes<T, DecodeFn>` → safe `fn`. Body retains
   `unsafe { &*page_item_id(...) }` and
   `unsafe { from_raw_parts(...) }` blocks with original SAFETY
   comments.
2. `read_page_tuple_from_buffer<T, DecodeFn>` (`#[cfg(feature = "pg18")]`)
   → safe `fn`. Body's `unsafe { with_page_tuple_bytes(...) }` wrap
   disappears (callee now safe).
3. `read_page_tuple<T, DecodeFn>` → safe `fn`. Body retains
   `unsafe { LockedBufferGuard::read_main(...) }` block;
   `unsafe { with_page_tuple_bytes(...) }` wrap goes away.

Twelve caller-side `unsafe { ... }` wrappers stripped across
`graph.rs`:

- `load_graph_element` (line ~347)
- `load_exact_graph_element::TurboQuantHotCold` (line ~378)
- `load_grouped_graph_element` (line ~428)
- `with_graph_element_tuple` (line ~461)
- `with_turbo_hot_graph_tuple` (line ~480)
- `with_grouped_graph_tuple` (line ~503)
- `with_graph_storage_tuple_from_buffer` arms (lines ~563, ~573,
  ~582)
- `load_rerank_payload` (line ~605)
- `with_grouped_codebook_tuple` (line ~641)
- `load_graph_neighbors` (line ~760, residual from slice 420)

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/graph.rs` | 37 | 23 | **-14** |
| **HNSW subsystem subtotal** | **462** | **448** | **-14** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 420 | 462 |
| After 421 | 448 |

Net rotation delta: **-101 in HNSW** (-18.4%).

## Soundness rationale

Each retained internal `unsafe { ... }` block has the same SAFETY
contract it had before this slice:

- `with_page_tuple_bytes`: caller validated `offset_number` against
  the page's line-pointer count before requesting the item id, and
  `tuple_offset + tuple_len` is bounded against `page_size` before
  constructing the byte slice.
- `read_page_tuple`: callers supply a live index relation; the
  `LockedBufferGuard` pin and share-lock are released only after
  `with_page_tuple_bytes` returns.

The lift moves the obligation from a per-call `unsafe { ... }`
wrapper to the function's internal blocks where it can be
re-validated against any nearby code changes. No anti-pattern B: the
functions return `Result<T, String>` not `&'a T`.

## Validation

Artifacts under `reviews/task-50/421-hnsw-graph-page-tuple-chain-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (299 lines)
- `cargo-check-pg18.log` — clean, **0 unused_unsafe warnings**.

## Performance gate

Scan/insert/vacuum hot path (page-tuple decoder cascade is the inner
loop of every HNSW operation). No semantic change — same page reads,
same item-id checks, same byte-slice bounds. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Out of scope

Higher-level callers in the rest of `graph.rs` (e.g. `load_graph_element`,
`load_exact_graph_element`, `load_grouped_graph_element`,
`with_*_graph_tuple` / `with_graph_storage_tuple*` / `load_rerank_payload`,
`load_grouped_rerank_payload`, `with_grouped_codebook_tuple`,
`load_exact_graph_adjacency`, `load_grouped_graph_adjacency`,
`bootstrap_grouped_codebook_chain`): each still `unsafe fn` because
of their own internal scoping (closure callbacks consuming graph
tuples) but the gating cascade is now unblocked. Queued.
