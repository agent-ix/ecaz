# Review Request: C1 ADR-030 V2 Storage-Aware Exact Graph Search

## Context

Packet 380 renamed the runtime format surface to:

- `TurboQuant`
- `PqFastScan`

But insert and vacuum still had one important scalar assumption buried in their
graph traversal paths:

- forward search, backlink scoring, and repair search all loaded exact graph
  payloads as scalar `TqElementTuple`s
- that meant the shared lifecycle could not actually reuse one "exact neighbor /
  exact entry / exact candidate" seam across both storage formats

For `PqFastScan`, exact graph payload lives across:

- grouped hot tuples for topology and search payload
- cold rerank tuples for exact `gamma + code`

So the next architectural step was to make exact graph reads and exact graph
search depend on `GraphStorageDescriptor`, not just `code_len`.

## Problem

The old exact-read/search shape had three concrete costs:

1. insert forward traversal and backlink planning still assumed scalar exact
   element reads
2. vacuum repair search still assumed scalar exact entry/candidate loads
3. the new format-adapter architecture from packet 375 still stopped short of
   the exact graph-read layer, which is the layer real `PqFastScan` insert /
   vacuum parity needs

Without fixing that seam first, real `PqFastScan` append/repair work would have
to keep threading ad hoc scalar-vs-grouped special cases through insert and
vacuum.

## Planned Slice

One architectural checkpoint:

1. add storage-aware exact graph loaders in `src/am/graph.rs`
2. add storage-aware exact graph-search helpers in `src/am/graph.rs`
3. switch insert forward traversal and backlink scoring to those helpers
4. switch vacuum repair search to those helpers
5. leave actual `PqFastScan` insert/vacuum success paths out of scope

This packet is about shared exact graph read/search architecture, not payload
append/finalize parity.

## Implementation

Updated:

- `src/am/graph.rs`
- `src/am/insert.rs`
- `src/am/vacuum.rs`

### 1. `src/am/graph.rs` now exposes exact loaders by storage format

Added:

- `load_exact_graph_element(...)`
- `load_exact_graph_adjacency(...)`

Behavior:

- `TurboQuant` still decodes the scalar element tuple directly
- `PqFastScan` now decodes:
  - the grouped hot tuple for topology fields
  - the cold rerank tuple for exact `gamma` and exact rerank code

That produces one shared `GraphElement` view for exact-scoring callers.

### 2. `src/am/graph.rs` now exposes storage-aware exact search helpers

Added:

- `load_layer0_successor_candidates_with_storage(...)`
- `greedy_descend_from_entry_with_storage(...)`
- `search_layer0_result_candidates_with_storage(...)`
- `search_layer_result_candidates_with_storage(...)`

These mirror the existing scalar helpers, but source exact candidates through
`GraphStorageDescriptor` instead of raw `code_len`.

### 3. Insert now uses the storage-aware exact traversal seam

`src/am/insert.rs` now threads `InsertFormatAdapter::graph_storage()` through:

- entry-candidate load
- upper-layer search
- layer-0 search
- backlink planning
- backlink rewrite candidate scoring

The result is that insert-side exact graph reads no longer assume scalar tuple
layout even though `PqFastScan` append is still intentionally unsupported.

### 4. Vacuum repair search now uses the same storage-aware exact seam

`src/am/vacuum.rs` now threads `VacuumFormatAdapter::graph_storage()` through:

- source-element load for repair planning
- entry-candidate load
- greedy descent
- upper/lower layer repair search
- exact scoring of already-linked candidates

The linear scan top-up helper remains TurboQuant-only on purpose. It still
decodes scalar element tuples directly, and this packet leaves top-level
`PqFastScan` vacuum unsupported.

## Measurements

No new benchmark or recall measurements in this slice. This is shared runtime
architecture only.

## Validation

Passed:

- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`, and
  `errstart`

## Outcome

This checkpoint moves the shared adapter architecture down to the exact graph
read/search layer:

1. insert exact traversal is storage-aware
2. insert backlink scoring reads exact payload through storage adapters
3. vacuum repair search is storage-aware
4. one shared exact `GraphElement` view now exists for both formats

What it still does **not** do:

- `PqFastScan` live insert append
- `PqFastScan` duplicate coalescing
- `PqFastScan` vacuum cleanup/finalize
- `PqFastScan` linear top-up candidate scan

## Next Slice

The next practical slices are:

1. replace the `PqFastScan` insert unsupported branch with a real grouped hot +
   cold append path
2. do the same for vacuum's format-specific cleanup/finalize path once grouped
   tuple rewrite/update rules are explicit
