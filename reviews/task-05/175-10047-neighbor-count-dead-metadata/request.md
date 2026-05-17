# Review Request: Neighbor Tuple Count Field — Dead Metadata

Scope:
- `src/am/page.rs` — `TqNeighborTuple`
- `src/am/build.rs` — `flush_build_state`
- `src/am/insert.rs` — `append_heap_tuple`
- `src/am/graph.rs` — `load_graph_neighbors`
- `spec/functional/FR-007-hnsw-page-layout.md`

## Problem

`TqNeighborTuple.count` has inconsistent semantics across write and read paths, and is never
meaningfully consumed.

**Write paths:**
- Build (build.rs:559): `count = neighbor_refs.len()` — total allocated SLOTS (including INVALID)
- Insert (insert.rs:80): `count = 0` — empty neighbor

**Read path (graph.rs:61-68):**
```rust
let count = neighbor.count as usize;
if count > neighbor.tids.len() {
    pgrx::error!("...");
}
```
The `count` is checked for sanity but never used for actual filtering. All neighbor filtering
happens in `valid_neighbor_tids_for_layer` (graph.rs:277-294) which uses layer slot bounds and
skips `ItemPointer::INVALID` — completely ignoring `count`.

**Spec (FR-007):**
> count — "Total number of active neighbors across all layers"

Neither write path matches this definition. Build writes total slots (active + empty). Insert
writes 0.

## Suggested Fix

Either:

**Option A — Remove the field.** Since it's never consumed, drop `count` from the encoding.
Saves 2 bytes per neighbor tuple. This is a page-layout break (acceptable pre-v1).

**Option B — Fix the semantics.** Make `count` actually track active (non-INVALID) neighbors.
Update build to count only `Some(...)` entries. Update insert to write 0 (correct for empty).
Add a read-path assertion that `count` matches the actual non-INVALID tid count. This gives
vacuum a reliable "active neighbor count" without re-scanning the tid array.

Please review which option is better given the upcoming vacuum work (FR-010). If vacuum needs to
know "how many live neighbors does this node have?" then Option B has value. Otherwise Option A
is simpler.
