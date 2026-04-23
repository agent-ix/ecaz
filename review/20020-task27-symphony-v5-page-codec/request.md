# Review Request: Task 27 Slice 4 — Symphony V5 Page Codec

Scope: implement the first real Symphony-owned on-disk codec in
`src/am/symphony/page.rs`. This slice turns the Phase-0 page-layout
delta into code without landing build, scan, insert, or vacuum
behavior yet.

Task: `plan/tasks/27-symphony-access-method.md` Phase 1
("Wire format" + first "Page layout" slice).

Branch: `task27-symphony-stage2-phase0-oracle` (slice 4 builds on
`f844953`).

Files in scope:
- `src/am/symphony/page.rs`

Validation:
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo test pg_test_operator_class_is_registered --features "pg18 pg_test" --no-default-features -- --test-threads=1`
- `cargo pgrx test pg18`
  - current result is **not green**: the full pg18 lane still collapses
    after an early pg-test hard abort, then the pgrx test mutex fans the
    rest out into secondary failures.
  - two narrow pg18 registration smokes do pass individually:
    - `cargo test pg_test_access_method_is_registered -- --nocapture`
    - `cargo test pg_test_operator_class_is_registered --features "pg18 pg_test" --no-default-features -- --test-threads=1`
  - this packet records the page-codec slice honestly rather than
    pretending the full pg18 suite is green. The hard-abort source still
    needs separate isolation.

## What landed

### 1. Dedicated Symphony metadata page

`MetadataPage` now owns a V5-specific metadata payload carrying:

- `m`
- `ef_construction`
- `entry_point`
- `dimensions`
- `rabitq_bits`
- `max_level`
- `seed`
- `inserted_since_rebuild`
- `format_version`
- `padding_factor`

The decoder rejects:

- non-`INDEX_FORMAT_V5_SYMPHONY` pages
- `rabitq_bits != 1`
- `padding_factor == 0`

That keeps the Stage-2 metadata seam narrow and self-describing.

### 2. Minimal Symphony element tuple

`SymphonyElementTuple` / `SymphonyElementTupleRef` preserve the graph
header role only:

- level
- deleted bit
- inline heap tids
- `neighbortid`

There is intentionally no hot search payload on the element tuple in this
slice. The centered score bytes move to the adjacency tuple, which is the
load-bearing page decision for Symphony.

### 3. Slabbed centered-adjacency tuple

`SymphonyNeighborTuple` / `SymphonyNeighborTupleRef` implement the frozen
Phase-0 layout:

```text
[tag][count]
[neighbor_tid[count]]
[centered_code[count]]
```

with:

- `count` = exact physical stored out-degree
- no dummy edges
- all centered codes required to share one fixed width
- borrowed accessors that expose TIDs and centered codes as separate
  slabs for the future batched scan path

This is the first point where the code matches the packet-20018 layout
decision rather than just describing it.

### 4. Page helpers for later build/scan slices

The module now also owns:

- `centered_code_len(dimensions)` using the task-25 scalar tail
- `neighbor_tuple_encoded_len(count, centered_code_len)`
- `max_padded_degree_that_fits(...)`
- `default_max_padded_degree(dimensions)`
- `DataPage` / `DataPageChain` helpers for Symphony element and neighbor
  tuple insert/read/update

Those helpers stay Symphony-prefixed to avoid colliding with existing
`ec_hnsw` inherent methods.

### 5. Unit coverage

Added page-local tests for:

- centered code length math
- metadata encode/decode and page-special round-trip
- invalid metadata rejection
- element tuple round-trip
- slabbed neighbor tuple round-trip
- borrowed slab access
- page insert/read helpers
- default padded-degree sanity

## What this slice intentionally does NOT do

- no page use from build/insert yet
- no centered-code persistence during index construction
- no scan path over borrowed neighbor slabs
- no reloptions wiring into metadata
- no quantization-aware pruning
- no out-degree padding behavior

This slice is strictly the codec and helper surface those later changes
will consume.

## Review focus

Please focus on:

1. Whether the V5 metadata payload is the right minimum frozen set for
   Stage 2.
2. Whether the slabbed neighbor tuple accurately matches the packet-20018
   page decision.
3. Whether the borrowed tuple refs and `DataPage` helpers are a good seam
   for the upcoming scan/build work without leaking `ec_hnsw` structure
   back in.

## Closing

Task 27 now has a real Symphony-owned V5 page module: metadata page,
element header, centered adjacency slab, and page-chain helpers. The next
slice can use this codec directly instead of rediscovering the tuple shape
during build or scan implementation.
