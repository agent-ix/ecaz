# Review Request: Task 27 Slice 2 — Symphony Page-Layout Delta

Scope: documentation only. Freezes the Phase-0 page-codec delta from
`src/am/ec_hnsw/page.rs` to the future `src/am/symphony/page.rs`
enough to start the Stage-2 scaffold without rediscovering the tuple
shape mid-implementation.

Task: `plan/tasks/27-symphony-access-method.md` Phase 0
("Page-layout delta").

Branch: `task27-symphony-stage2-phase0-oracle` (slice 2 builds on
`7dc640b`).

Primary in-tree baseline:
- `src/am/ec_hnsw/page.rs`

External design inputs:
- `review/20015-task25-task27-handoff-contract-v2/request.md`
- `spec/adr/ADR-045-symphonyqg-quantized-graph-access-method.md`
- SymphonyQG paper §3.1.1, §3.1.2, §3.2

## ec_hnsw baseline

`ec_hnsw` currently stores graph topology as:

- metadata page with index format, `m`, `ef_construction`, `dimensions`,
  `bits`, `seed`, entry point, and codec-kind flags
- element tuple:
  - level / deleted / heap tids
  - scalar-search payload (`gamma`, `code`, optional binary sidecar)
  - `neighbortid` pointer
- neighbor tuple:
  - `tag`
  - `count: u16`
  - `tids: Vec<ItemPointer>`

For the baseline neighbor tuple, the payload is topology only. Search
codes live with the element tuple, not with the adjacency.

Symphony inverts that last assumption: the visiting vertex is the
center, so the search code belongs to the **edge as seen from that
center**, not to the neighbor globally.

## Frozen Symphony delta

### 1. New format constant

Stage 2 gets a dedicated on-disk format:

`INDEX_FORMAT_V5_SYMPHONY`

This is not byte-compatible with `ec_hnsw` V1/V2/V3. REINDEX only.

### 2. Metadata page stays recognizably ec_hnsw-shaped

Reuse the existing metadata-page framework and keep the current core
fields:

- `m`
- `ef_construction`
- `entry_point`
- `dimensions`
- `seed`
- `max_level`
- `inserted_since_rebuild`

Add the minimum Symphony-specific fields:

- `format_version = INDEX_FORMAT_V5_SYMPHONY`
- `padding_factor` (test seam allows `1`; production values are batch
  aligned)
- `rabitq_bits` (frozen to `1` for Symphony, but carried explicitly so
  the format is self-describing)

No new grouped/turboquant codec flags are needed for the Stage-2 path.

### 3. Element tuple keeps the graph-header role

The element tuple still owns:

- per-vertex level / deleted state
- heap-tid list
- `neighbortid` pointer

The key change is semantic: Stage-2 traversal no longer treats the
element tuple's search payload as the hot scoring bytes for neighbors.
Those move into the adjacency tuple because the same neighbor has a
different centered code under different visited vertices.

This slice intentionally does **not** freeze a new per-element center
payload beyond what Stage 2 already needs for the existing source /
rerank seam. The Phase-0 storage decision that matters for correctness
and SIMD batching is the adjacency encoding below.

### 4. Neighbor tuple becomes topology + centered-code slab

The Symphony neighbor tuple keeps the existing header:

```text
[tag: u8][count: u16]
```

and replaces the payload with two aligned slabs keyed by the same
`count`:

```text
[neighbor_tid[count]]
[centered_code[count]]
```

where:

- `neighbor_tid[i]` is the `ItemPointer` for edge `i`
- `centered_code[i]` is the task-25 centered RaBitQ code of that
  neighbor relative to the owning element

The centered code byte width is frozen by task 25:

```text
centered_code_len = ceil(dim / 8) + 12
                   = packed sign bits
                   + ||v - c|| : f32
                   + o_dot      : f32
                   + <x_bar, c_tilde> : f32
```

At `D = 1536`, this is `192 + 12 = 204` bytes per edge.

### 5. `count` changes meaning

This is the one semantic delta from `ec_hnsw` that Phase 0 freezes.

For Symphony:

- `count` means the **physical stored out-degree**
- every stored edge is a real edge, never a dummy
- for production Stage 2, `count % padding_factor == 0`
- for the Phase-0 oracle seam only, `padding_factor = 1` is allowed

No separate "logical vs padded" count is stored, because the padding
edges are part of the real graph after refinement.

### 6. Why the payload is slabbed instead of interleaved

The hot loop wants dense code bytes:

- scalar reference today
- signed-POPCNT / batched FastScan-style kernel later

Keeping `neighbor_tid[]` and `centered_code[]` in separate contiguous
slabs lets scan code:

1. score a full batch from the code slab with no per-edge stride
2. then gather the corresponding `ItemPointer`s from the tid slab

An interleaved `[(tid, code)] * count` layout would work functionally
but bakes a wider stride into the future SIMD path for no gain.

## What does NOT change in this slice

- cross-AM storage primitives in `crate::storage::page`
- page chaining / `ItemPointer` encoding
- one element tuple pointing at one neighbor tuple per layer
- exact rerank remaining on in Stage 2
- task-25 code layout for centered RaBitQ itself

## Consequences for the Stage-2 implementation

This delta gives the implementation a narrow first target:

1. write / read the new neighbor tuple codec
2. batch-decode centered codes from the adjacency slab
3. prove the `padding = 1` oracle from packet `20017`

Only after that should the branch change:

- pruning distance from fp32 to centered-RaBitQ
- neighbor count from unpadded to padded

That ordering keeps layout bugs separate from graph-quality changes.

## Open items deliberately left unfrozen

- whether Stage 2 stores any additional per-element center payload
  beyond the current source/rerank seam
- the final reloptions surface
- Stage 3 no-rerank metadata
- the exact SIMD kernel shape over the centered-code slab

Those depend on scan-path implementation details. The adjacency tuple
delta above does not.

## Closing

The Phase-0 page decision is:

- keep the `ec_hnsw` metadata / element / neighbor tuple hierarchy
- move the hot search bytes from element payload to adjacency payload
- make the neighbor tuple carry `count` topology entries and `count`
  centered residual codes in parallel slabs
- treat `count` as the real stored out-degree, batch-aligned in
  production and `1` only for the oracle seam

That is the smallest page-layout change consistent with Symphony's
per-center residual encoding and future batched scan path.
