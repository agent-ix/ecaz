# Task 50/423: HNSW graph.rs — with_*_graph_tuple closure entries safe-fn lifts

## Why this slice

After slice 422, the `with_*_graph_tuple` and
`with_graph_storage_tuple*` closure-entry functions in graph.rs had
zero internal `unsafe { ... }` blocks (they wrap already-safe
`read_page_tuple` / `read_page_tuple_from_buffer` calls). The
`unsafe fn` declarations were legacy.

Lifting all six functions removes the residual caller-side wraps in
`graph.rs` (cookbook chain loader at line ~621), `scan.rs`
(`cached_graph_element` and `cached_graph_element_from_buffer` at
lines ~2430 / ~2479), and frees the remaining `bootstrap_*` cascade.

## Scope

Six `unsafe fn` → safe `fn` flips in `src/am/ec_hnsw/graph.rs`:

1. `with_graph_element_tuple<R, F>` (TqElementTupleRef closure)
2. `with_turbo_hot_graph_tuple<R, F>` (TqTurboHotTupleRef closure)
3. `with_grouped_graph_tuple<R, F>` (TqGroupedHotTupleRef closure)
4. `with_graph_storage_tuple<R, F>` (storage-discriminated dispatcher;
   match arms strip their per-arm `unsafe { ... }` wraps too)
5. `with_graph_storage_tuple_from_buffer<R, F>` (pg18-only buffer
   variant of the dispatcher)
6. `with_grouped_codebook_tuple<R, F>` (codebook tuple closure)

Caller-side wraps stripped:

- `graph.rs:621` (inside `load_grouped_codebook_model`)
- `scan.rs:2430` (inside `cached_graph_element`)
- `scan.rs:2479` (inside `cached_graph_element_from_buffer`)

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/graph.rs` | 13 | 9 | -4 |
| `src/am/ec_hnsw/scan.rs` | 91 | 89 | -2 |
| **HNSW subsystem subtotal** | **432** | **426** | **-6** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 422 | 432 |
| After 423 | 426 |

Net rotation delta: **-123 in HNSW** (-22.4%).

## Soundness rationale

Every body composes only safe operations after slices 421/422
(`read_page_tuple` / `read_page_tuple_from_buffer` are already
safe). Lifting is signature-only. No anti-pattern B.

## Validation

Artifacts under `reviews/task-50/423-hnsw-graph-with-tuple-closures-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch`
- `cargo-check-pg18.log` — clean, **0 unused_unsafe warnings**.

## Performance gate

Scan/insert hot path. No semantic change. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Out of scope

Remaining graph.rs `unsafe fn`s (9 blocks total):
- `load_layer0_successor_candidates_with_storage`
- `greedy_descend_from_entry_with_storage`
- `search_layer0_result_candidates_with_storage`
- `search_layer_result_candidates_with_storage`
- `load_layer0_refill_successors_with_storage`
- `expand_layer0_visible_seeds_with_storage`
- `load_successor_candidates_for_layer_with_storage`

Each takes FnMut/ScoreFn closures and threads `(rel, storage)` into
the closure callees. The bodies now hold their own unsafe wraps
around closure-internal calls only; lifting depends on each closure
caller chain being analyzed individually. Queued.
