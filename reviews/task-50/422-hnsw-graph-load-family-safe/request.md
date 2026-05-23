# Task 50/422: HNSW graph.rs — load_* family safe-fn cascade

## Why this slice

Slice 421 lifted the page-tuple reader chain
(`with_page_tuple_bytes` / `read_page_tuple_from_buffer` /
`read_page_tuple`) to safe `fn`. That removed the last internal
`unsafe { ... }` blocks from the higher-level `load_*` /
`load_*_adjacency` helpers, so they can now flip safe in turn.

This slice lifts the entire load-family cascade:

- `load_graph_element` (scalar `TqElementTuple` decoder)
- `load_exact_graph_element` (storage-discriminated TurboQuant /
  TurboQuantHotCold / PqFastScan dispatcher)
- `load_grouped_graph_element` (grouped hot tuple decoder)
- `load_rerank_payload` (cold rerank payload reader)
- `load_grouped_rerank_payload` (grouped rerank forwarder)
- `load_exact_graph_adjacency` (element + neighbors)
- `load_grouped_graph_adjacency` (grouped element + neighbors)

Each body now uses only safe operations after slice 421.

## Scope

- Seven `pub(crate)` functions in `src/am/ec_hnsw/graph.rs` flipped
  from `unsafe fn` to safe `fn`. No body change required beyond
  stripping the residual `unsafe { ... }` wraps around lifted
  callees (`load_graph_element`, `load_grouped_graph_element`,
  `load_rerank_payload`, `load_grouped_rerank_payload`,
  `load_exact_graph_element`, `load_exact_graph_adjacency`).
- Caller-side `unsafe { ... }` wraps stripped wherever the compiler
  reported `unused_unsafe`:
  - graph.rs internal call sites in `load_exact_graph_element`
    (TurboQuant/PqFastScan arms), `load_grouped_rerank_payload`,
    `load_exact_graph_adjacency`, `load_grouped_graph_adjacency`,
    `bootstrap_grouped_codebook_chain` (cascade), and the
    layer0/layer-N successor scoring helpers (~lines 803, 1235, 1247).
  - insert.rs at `run_insert_with_adapter` entry-point repair (line
    ~827) and `load_insert_entry_candidate` (line ~1074).
  - scan.rs at `score_cached_graph_element_from_storage` (line
    ~2508) and the cross-page grouped rerank payload reload (line
    ~2628).
  - vacuum.rs at `plan_repair_replacement` (line ~1104) and the
    cross-page linear-repair rerank reload (line ~1562).

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/graph.rs` | 23 | 13 | -10 |
| `src/am/ec_hnsw/scan.rs` | 93 | 91 | -2 |
| `src/am/ec_hnsw/insert.rs` | 54 | 52 | -2 |
| `src/am/ec_hnsw/vacuum.rs` | 52 | 50 | -2 |
| **HNSW subsystem subtotal** | **448** | **432** | **-16** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 421 | 448 |
| After 422 | 432 |

Net rotation delta: **-117 in HNSW** (-21.3%).

## Soundness rationale

After slice 421, every `load_*` function in this family had zero
internal `unsafe { ... }` blocks of its own — they only composed
already-safe `read_page_tuple` / `load_*` calls. The `unsafe fn`
declarations were legacy; flipping to safe `fn` is signature-only
and adds no new soundness obligation.

No anti-pattern B: every function returns an owned tuple type, not
`&'a T`.

## Validation

Artifacts under `reviews/task-50/422-hnsw-graph-load-family-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (275 lines)
- `cargo-check-pg18.log` — clean, **0 unused_unsafe warnings**.

## Performance gate

Scan/insert/vacuum hot path. No semantic change — same tuple reads,
same decoders, same error paths. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Rotation milestone

**Past the -21% threshold** on HNSW: 549 → 432, net -117 (-21.3%).
The graph.rs FFI surface that gated every higher-level read is now
mostly safe-fn; remaining graph.rs unsafe surface (read_page_tuple's
LockedBufferGuard call + the with_*_graph_tuple closure entries +
load_layer0_*_with_storage closure entries) totals 13 blocks.

## Out of scope

- `with_grouped_codebook_tuple`, `with_graph_element_tuple`,
  `with_turbo_hot_graph_tuple`, `with_grouped_graph_tuple`,
  `with_graph_storage_tuple`, `with_graph_storage_tuple_from_buffer`,
  `load_layer0_successor_candidates_with_storage`,
  `load_successor_candidates_for_layer_with_storage`,
  `bootstrap_grouped_codebook_chain`, `search_layer0_result_candidates_with_storage`:
  each still `unsafe fn` because they take FnMut/FnOnce closures
  that are themselves unsafe-fn callees. Queued.
