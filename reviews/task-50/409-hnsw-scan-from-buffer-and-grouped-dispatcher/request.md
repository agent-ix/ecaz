# Task 50/409: HNSW scan.rs — buffer variant + grouped dispatcher lifts

## Why this slice

Two more conversions in the scan.rs `score_grouped_*` / `cached_graph_*`
family, both safe after slice 406-408's leaf lifts:

- `cached_graph_element_from_buffer` (`#[cfg(feature = "pg18")]`): the
  pg18-only buffered variant of `cached_graph_element`. After slice
  408, `cached_graph_element` is safe `fn`; the buffered variant
  has the identical body shape (one internal
  `unsafe { graph::with_graph_storage_tuple_from_buffer(...) }` block)
  and was the lone remaining `unsafe fn` cache-loader. One caller in
  `cached_graph_element_with_prefetch` drops its `unsafe { ... }` wrapper.
- `score_grouped_candidate_context` (the grouped-traversal scoring
  dispatcher): after slices 406-407 every callee (`exact`, `binary`,
  `approx`) is now safe and takes references. The dispatcher's body
  no longer contains any internal `unsafe { ... }` blocks — converting
  the signature to safe `fn` taking `&mut TqScanOpaque` removes three
  `unsafe { ... }` caller wrappers across scan.rs.

## Scope

- `cached_graph_element_from_buffer` lifted to safe
  `fn(&mut TqScanOpaque, &PinnedBufferLockGuard<'_>, page::ItemPointer)`.
  Internal `scan_opaque_mut(opaque)` removed; parameter is the borrow.
  Single caller `cached_graph_element_with_prefetch` passes
  `scan_opaque_mut(opaque)`.
- `score_grouped_candidate_context` lifted to safe
  `fn(pg_sys::Relation, &mut TqScanOpaque, GroupedScoreContext, u8)`.
  Internal `scan_opaque_ref(opaque)` removed; the predicates
  `grouped_exact_traversal_full_candidate_scoring_for_layer` and
  `grouped_binary_traversal_score_enabled` take `&TqScanOpaque` and
  are now called directly on the `&mut TqScanOpaque` parameter (Rust
  reborrows as `&` for the call duration). Three callers each drop
  their `unsafe { ... }` wrappers and pass `scan_opaque_mut(opaque)`.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 116 | 112 | -4 |
| **HNSW subsystem subtotal** | **505** | **501** | **-4** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 408 | 505 |
| After 409 | 501 |

Net rotation delta: **-48 in HNSW** (-8.7%).

## Soundness rationale

- `cached_graph_element_from_buffer` body keeps one internal
  `unsafe { ... }` block around the `with_graph_storage_tuple_from_buffer`
  FFI helper, same shape as the non-buffer variant from slice 408.
- `score_grouped_candidate_context` body has **zero** internal
  `unsafe { ... }` blocks after slices 406-407. The lift is pure
  signature change — no obligation moves.
- Borrow-check passes because the dispatcher only uses immutable reads
  of `opaque` (`&mut T` reborrows as `&T`) and then performs at most
  one `&mut` call (each branch returns immediately, so borrows don't
  overlap).

No anti-pattern B; new signatures take references not raw pointers.

## Validation

Artifacts under `reviews/task-50/409-hnsw-scan-from-buffer-and-grouped-dispatcher/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean.

## Performance gate

Scan hot path. No semantic change: dispatcher selects the same scorer
each call; buffer variant reads the same cached entries; record_* calls
fire identically. Bench deferred per `feedback_coder_push_smoke_checks`.

## Out of scope

- `exact_score_cached_graph_element` and `score_cached_graph_element_dispatch`
  — both still `unsafe fn`, body has `exact_score_cached_graph_element`
  (still unsafe) inside. Next slice candidate.
- `cached_graph_element_and_score` and related higher-level scoring
  dispatch — chained on the above.
- `prefetch_*`, `cached_graph_neighbors`, `cached_graph_adjacency` —
  most still `unsafe fn`; lifts queued.
