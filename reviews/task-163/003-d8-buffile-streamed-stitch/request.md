# Review request — Task 163 D8 BufFile streamed stitch

**Status:** review requested; Task 179 prerequisite, not Task 163 closeout.

**Branch:** `task-179-ec-distann-physical-shards`
**Code checkpoint:** `079a235f9433d52796107fdbc926c0ee5274940f`

## Outcome

ADR-085 D8 / FR-077-CON-4 no longer retains `Vec<ShardGraph>` across the stitch.
Each completed, node-sorted shard is encoded to its own resource-owner-scoped
PostgreSQL `BufFile`. The stitch opens one sequential cursor per shard, keeps
only each cursor's next `(node, neighbor_count)` header in memory, and reads an
adjacency payload only when that cursor participates in the current minimum
node group.

## What changed

- Per-shard Vamana builds remain parallel in bounded batches. Each completed
  batch is immediately spilled and dropped; no shard graph survives into the
  stitch.
- Each shard uses a separate sequential `BufFile`. This avoids the random
  seek/buffer reload per node that a single shared spool would impose.
- A `BinaryHeap<Reverse<(node, shard)>>` performs the k-way cursor merge.
- Multi-membership groups union, sort, de-duplicate, and globally
  `robust_prune`; single-membership groups preserve adjacency order exactly.
- The merge now rejects missing node coverage, out-of-range nodes/neighbors,
  over-degree entries, unsorted shard streams, and unconsumed cursor tails.
- The build NOTICE replaces the old all-resident count with:
  `shard_output_spill_bytes`, `stitch_peak_cursor_bytes`,
  `stitch_peak_group_bytes`, and `stitch_peak_retained_bytes`.

## D8 memory accounting

`stitch_peak_cursor_bytes` includes all per-shard PostgreSQL `BufFile` block
buffers, cursor structs, and merge-heap capacity. Cursor lookahead contains no
neighbor payload. `stitch_peak_group_bytes` includes the current passthrough or
union capacity, one `Candidate` per unique neighbor, and the R-sized prune
result. `stitch_peak_retained_bytes` records the maximum cursor-plus-active-
group total.

As specified, this incremental stitch bound excludes the already-resident
source vectors and the required output graph. The unit-test transport is an
in-memory stand-in so pure tests do not require a backend; the focused PG18
tests build and reindex through the installed extension and therefore execute
the real `BufFile` implementation.

## Validation

- Focused pure shard suite: 10 passed, including the new
  `tc038_d8_spill_and_cursor_bound` plus degree, uniqueness, reachability,
  alpha-prune, determinism, idempotence, and repair regressions.
- Strict PG18 library clippy: clean with `-D warnings`.
- Focused live PG18 tests: sharded self-recall and deterministic reindex both
  pass through the real extension build path.
- No benchmark was rerun: this checkpoint changes stitch storage/memory and
  preserves graph output; the existing Task 163 recall evidence remains the
  quality result. Task 179/172 retain their later performance gates.

## Reviewer focus

Please verify:

1. No complete shard output remains reachable when `stitch_shard_spool` starts.
2. Cursor lookahead is header-only and each adjacency is read exactly once,
   sequentially, for its active node group.
3. The accounting includes every per-shard `BufFile` buffer and all active
   union/prune capacities without claiming source/output graph memory.
4. Single-membership order and multi-membership `BTreeSet`-equivalent ordering
   preserve the pre-change deterministic graph.
5. The runtime/test `cfg` split cannot cause the extension path to select the
   in-memory test transport.

Outside-review acceptance of this packet closes only the D8 prerequisite. Task
163 remains subject to its full acceptance criteria, and Task 179 must not claim
streamed handoff until this review lands.
