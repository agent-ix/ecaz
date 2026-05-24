# Task 50/420: HNSW graph.rs — `load_grouped_codebook_model` + `load_graph_neighbors` safe-fn lifts

## Why this slice

Two more leaf `unsafe fn` loaders in `graph.rs` whose bodies each
contain exactly one internal `unsafe { ... }` block around a typed
page-tuple reader (`with_grouped_codebook_tuple` /
`read_page_tuple`). The functions take `pg_sys::Relation` and a
target TID and either return a decoded value or panic — same shape
as the `from_index_relation` lift in packet 419.

Lifting both to safe `fn` removes 6 caller-side `unsafe { ... }`
wrappers across HNSW (vacuum.rs, scan.rs, graph.rs, insert.rs).

## Scope

- `graph::load_grouped_codebook_model(rel, &MetadataPage)` lifted
  from `pub(crate) unsafe fn` to `pub(crate) fn`. Internal
  `unsafe { with_grouped_codebook_tuple(...) }` block retained.
  2 caller wraps removed (`insert.rs:1898`, `scan.rs:2180`).
- `graph::load_graph_neighbors(rel, neighbor_tid)` lifted to safe
  `fn`. Internal `unsafe { read_page_tuple(...) }` block retained.
  4 caller wraps removed: `graph.rs:794` (in
  `load_exact_graph_adjacency`), `graph.rs:809` (in
  `load_grouped_graph_adjacency`), `vacuum.rs:1003`, `scan.rs:3041`.

## Unsafe block counts

| File | Before | After | Δ |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/scan.rs` | 95 | 93 | -2 |
| `src/am/ec_hnsw/insert.rs` | 55 | 54 | -1 |
| `src/am/ec_hnsw/vacuum.rs` | 53 | 52 | -1 |
| `src/am/ec_hnsw/graph.rs` | 39 | 37 | -2 |
| **HNSW subsystem subtotal** | **468** | **462** | **-6** |

Cumulative rotation delta:

| Stage | HNSW total |
| --- | ---: |
| Pre-399 | 549 |
| After 419 | 468 |
| After 420 | 462 |

Net rotation delta: **-87 in HNSW** (-15.8%).

## Soundness rationale

Each lifted function retains exactly one internal `unsafe { ... }`
block with the original SAFETY comment naming the caller-supplied
precondition (live index relation, valid TID). The pattern matches
the `from_index_relation` lift in slice 419 and the
`read_metadata_page` lift in slice 399.

No anti-pattern B: neither function returns `&'a T`.

## Validation

Artifacts under `reviews/task-50/420-hnsw-graph-loaders-safe/artifacts/`:

- `manifest.md`
- `per-file-after.log`
- `diff.patch` (88 lines)
- `cargo-check-pg18.log` — clean.

## Performance gate

Cold/medium path (codebook load on PqFastScan setup; neighbor load on
every traversal step). No semantic change. Bench deferred per
`feedback_coder_push_smoke_checks`.

## Out of scope

Remaining `unsafe fn` loaders in graph.rs (`load_graph_element`,
`load_exact_graph_element`, `load_grouped_graph_element`,
`with_*_graph_tuple` closure entries, `load_rerank_payload`,
`load_grouped_rerank_payload`, `load_exact_graph_adjacency`,
`load_grouped_graph_adjacency`, `with_grouped_codebook_tuple`,
`read_page_tuple`, `read_page_tuple_from_buffer`,
`with_page_tuple_bytes`): each retains one or more internal unsafe
blocks; lifting requires the underlying `read_page_tuple` /
`with_page_tuple_bytes` chain to lift first. Queued.
