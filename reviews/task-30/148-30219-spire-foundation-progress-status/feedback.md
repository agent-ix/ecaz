---
reviewer: opus47
status: open
created: 2026-05-02
checkpoint_commit: 318768df
verdict: changes-requested-architecture
scope: holistic foundation review across packets 30162..30252 (ADR-049, Phase 0
  design note, codecs, allocators, build/scan/update helpers)
---

# Review: SPIRE Foundation — first architecture pass

This is a holistic review of what has landed for Task 30 / ADR-049 so far. The
goal is to look at whether the foundation laid out in `src/am/ec_spire/` is
shaped to scale efficiently once it gets wired to live PostgreSQL relations,
not to gate any individual sub-packet. Phase 0 is well-thought-out and the
codec/manifest layering is clean. The concerns below are about **storage
shape, hot-path layout, and validation cost** — things that get harder to
change once persistence lands and once tests can no longer be all-in-memory.

## Things that are working well

- **Layered codecs with isolated round-trip tests.** Header → row → object →
  store → placement → manifest each have validate/encode/decode/test pairs.
  Magic numbers + format_version + reserved-zero discipline is consistent.
- **`SpireVecId` discriminator design.** 0x01 || u64 local + 0x02-reserved
  global is a forward-looking choice that costs nothing now.
- **Cursor-based allocators with copy-on-success commit.** `pid_cursor` /
  `local_vec_id_cursor` operate on `*pid_allocator` clones and only write back
  on success; tests assert non-advancement on failure. This is exactly the
  fail-closed semantics the epoch-publication model needs.
- **`SpirePublishedEpochSnapshot` cross-validates** epoch / object_manifest /
  placement_directory together, including strict-vs-degraded placement-state
  rules. The single source of truth for "is this snapshot consistent" is
  exactly the right shape.
- **Strict-mode is the default** for the local single-store path; degraded
  mode is enforced as opt-in per Phase 0 §"Epoch Layout and Publication".
- **Empty leaves are preserved** by `build_partitioned_single_level_leaf_epoch_draft`.
  Easy to forget; tested explicitly in `partitioned_single_level_draft_preserves_empty_centroid_leaf`.
- **Heap TID is locator-only.** vec_id is the dedupe identity; non-HOT updates
  get fresh local vec_ids. Matches the Phase 0 note.

## Architecture concerns (must address before live persistence wires up)

### A1. A leaf partition object is a single page-bounded tuple

`SpireLocalObjectStore::insert_leaf_object` writes the entire encoded leaf
object as one `DataPageChain::insert_raw_tuple` call. That tuple is rejected
by `element_or_neighbor_tuple_fits` if `aligned_tuple_bytes(payload_len) >
usable_page_bytes(8192)`. Concretely:

- A row with TurboQuant payload at d=128 is roughly `2 (flags) + 1 (vec_len) +
  9 (vec_id) + 6 (heap_tid) + 1 (fmt) + 4 (gamma) + 4 (payload_len) + ~144
  (mse+qjl)` ≈ 170 bytes.
- A leaf header is 46 bytes.
- A leaf with **~45 rows** already saturates an 8 KB page. With encoded
  payload widths typical for production (RaBitQ+rerank or PQ-FastScan),
  the per-leaf cap is in the low-hundreds of vectors at most.

SPIRE's design point is "many small leaves so flat-scan stays fast" — the
paper targets ~thousands of vectors per leaf at the upper end. **Today the
storage primitive cannot represent a single realistic leaf.** This is the
single biggest architectural risk in the foundation. Options to sort out
*before* live persistence:

- Multi-page leaf objects with a chain pointer in the header (analogue of
  `ec_ivf` posting-list pages).
- Page-per-row layout with the leaf header on a meta page and a chained
  `(block_number, offset_number)` index of rows.
- TOAST-style overflow only for the encoded-payload tail.

Whichever shape you pick, it must influence the codec — currently
`SpireLeafPartitionObject::decode` walks one contiguous byte slice. Changing
that shape later means reformatting every persisted object. Better to face it
now while the format is in-memory only.

### A2. Hot-path layout is row-encoded, allocator-heavy, and non-vectorizable

`SpireLeafAssignmentRow::decode` produces an owned `Vec<u8>` per row. The scan
inner loop is:

```
for routed in routed_rows:
  for row in routed.rows:        # already a fully decoded Vec<row>
    if visible_primary(row):
      score(row.assignment)      # FnMut callback, one row at a time
```

Three problems compound:

1. **Per-row allocations.** `decode_prefix` does `input[...].to_vec()` for the
   payload bytes of every row. At 10 K leaves × 200 rows = 2 M `Vec` allocs in
   per-query memory context.
2. **No SIMD-friendly packing.** TurboQuant / RaBitQ payloads sit interleaved
   with `flags` / `vec_id_len` / `heap_tid` / `gamma`. Batched scoring
   (especially PQ-FastScan) needs all payloads in one contiguous block.
3. **`FnMut`-per-row callback.** `rank_routed_leaf_rows_by_ip` invokes the
   scorer once per row. The scorers themselves are perfectly capable of batch
   scoring (`score_ip_from_parts` for prod, `estimate_ip` for rabitq) — the
   leaf-iteration shape forces them into per-row mode.

These are not "optimizations" — they're the shape of the data structure being
written into the on-disk format. Fixing later requires a format bump.

**Concrete proposal — column-major leaf object format (`LeafPartitionObjectV2`):**

Phase 1 leaves are constrained to one payload format per object (the
build-time storage_format), so every row in a leaf has identical
`payload_format` and identical `encoded_payload.len()`. Use that:

```text
LeafPartitionObjectV2 layout (single payload format per leaf):
  header                       (46 bytes, kind=Leaf, + payload_format byte
                                + payload_stride u32 in flags-reserved area)
  vec_ids:    [u8; vec_id_stride * N]   (fixed stride, e.g. 16 bytes padded
                                         from 9 for local IDs; pad to align)
  heap_tids:  [ItemPointer; N]          (6 bytes * N, naturally packed)
  gammas:     [f32; N]                  (4-byte aligned, contiguous)
  flags:      [u16; N]                  (per-row role/tombstone bits)
  payloads:   [u8; payload_stride * N]  (one contiguous block, SIMD-aligned)
```

Decoded as a single zero-copy view:

```rust
pub(super) struct SpireLeafObjectColumns<'a> {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) payload_format: SpireAssignmentPayloadFormat,
    pub(super) payload_stride: usize,
    pub(super) vec_ids:   &'a [u8],   // chunks_exact(vec_id_stride)
    pub(super) heap_tids: &'a [u8],   // chunks_exact(ITEM_POINTER_BYTES)
    pub(super) gammas:    &'a [f32],
    pub(super) flags:     &'a [u16],
    pub(super) payloads:  &'a [u8],   // chunks_exact(payload_stride)
}
```

This trades the current "any payload format per row" flexibility for:

- **Zero per-row allocation on read.** `Columns<'a>` is borrow-from-page.
- **Contiguous payload block.** Hands one slice straight to the quantizer for
  batch scoring; no copy, no per-row indirection.
- **Pre-validated stride.** Header carries `payload_stride`; decoder checks
  `payloads.len() == stride * N` once, then trusts it for every access.
- **Cheaper visibility filter.** `flags: &[u16]` is a tight loop with no
  branches into other fields.

Mixed-format leaves (e.g. a delta containing both insert payloads and
empty-payload deletes) still work via the existing `SpireDeltaPartitionObject`
which is a different kind and can keep the row-encoded shape — deltas are
small and not hot.

**Concrete proposal — batch scorer trait:**

The existing `SpirePreparedAssignmentScorer::score_assignment_ip` takes one
row. Add a batch entry point that the column codec can drive:

```rust
impl SpirePreparedAssignmentScorer {
    pub(super) fn score_batch_ip(
        &self,
        payload_stride: usize,
        payloads: &[u8],
        gammas: &[f32],
        out: &mut [f32],
    ) -> Result<(), String> {
        // TurboQuant/RaBitQ inner loops can dispatch one prepared query
        // against N payload chunks; PQ-FastScan needs this for groupwise
        // table lookups to be worth anything.
    }
}
```

Then `rank_routed_leaf_rows_by_ip` becomes:

```rust
for routed in routed_rows {
    let cols = routed.columns;          // SpireLeafObjectColumns<'_>
    let mut scores = vec![0f32; cols.flags.len()];
    scorer.score_batch_ip(
        cols.payload_stride,
        cols.payloads,
        cols.gammas,
        &mut scores,
    )?;
    // visibility filter + bounded-heap top-K over scores[i] / cols.flags[i] /
    // cols.vec_ids[i] / cols.heap_tids[i]
}
```

This composes with A5's bounded heap and A7's "skip dedup in Phase 1" — the
inner loop becomes a single SIMD-friendly score pass plus a top-K push.

**Migration note.** Keep `SpireLeafPartitionObject` (V1, row-encoded) for
`SpireDeltaPartitionObject` and tests; introduce `LeafPartitionObjectV2` as
the format header version 2 with a different `format_version` discriminator.
The codec layer can decode either by inspecting the header. Delta objects
stay row-encoded since they need mixed-flag rows.

### A3. Snapshot + object validation runs on every read

I count 8+ call sites that re-invoke `SpirePublishedEpochSnapshot::new(...)?`
inside helpers that already received a `&SpirePublishedEpochSnapshot<'_>`
(e.g. `collect_snapshot_leaf_rows` line 199, `collect_snapshot_delta_rows`
line 405, `load_snapshot_root_routing_object` line 568, `collect_snapshot_leaf_rows_for_pid`
line 678, `build_delta_epoch_draft_from_snapshot` line 116). Each call walks
`object_manifest.entries` and binary-searches the placement directory for
each pid. For an N-leaf snapshot this is repeatedly O(N log N) on the hot
path.

Compounding this, `read_leaf_object` decodes the bytes and calls
`validate_header()` which calls `validate_leaf_assignments` → walks every row
+ HashSet-dedupes vec_ids. That's the per-page ingest cost on every scan.

Suggestions:
- A `Validated<SpirePublishedEpochSnapshot>` zero-cost wrapper that internal
  helpers consume; only construction goes through `::new` (which validates).
- Split `validate_leaf_assignments` (write-time, allocates HashSet) from a
  pre-validated read accessor that trusts the encoded bytes.
- Build a `pid → placement_index` map once at scan start instead of binary
  searching per leaf (see A6 below).

### A4. Routing centroid layout will not survive Phase 6

`SpireRoutingChildEntry { centroid_index, child_pid, centroid: Vec<f32> }`
is a vec-of-structs row layout, with each centroid owning its own heap
allocation. `SpireSingleLevelRouteMap::from_centroid_plan` then does
`entry.centroid.clone()` per child. For Phase 1's hundreds-to-thousands of
centroids this is fine; for Phase 6 (graph-over-top-centroids, paper says ~few
million centroids), it is the wrong shape:

- One `Vec<f32>` per centroid = millions of small allocations.
- Iterating `child.centroid` with `inner_product(query_vector, &child.centroid)`
  cannot be SIMD-vectorized across children; you score one centroid at a time.
- No prefetch story — children are scattered in the heap.

If the routing object format gets `centroids: Vec<f32>` (flat,
`dim * child_count` floats) plus `child_pids: Vec<u64>` parallel arrays, the
encoded bytes are the same total size but iteration becomes contiguous and
you can score N children in one SIMD pass. The decoder change is mechanical
now, painful once Phase 6 lands and the object is in real pages.

### A5. Top-`nprobe` selection and dedup use `Vec`+sort+truncate, not bounded heaps

`route_root_object_to_leaf_pids` allocates a Vec of every child's
`(centroid_index, child_pid, ip_score)`, sorts the entire vector, then takes
the top nprobe. `rank_routed_leaf_rows_by_ip` builds an unbounded
`HashMap<SpireVecId, candidate>`, collects to `Vec`, sorts everything, then
truncates to limit. With nprobe=k of N children and limit=L of M rows, this
is O(N log N) and O(M log M) where the IVF-correct cost is O(N log k) and
O(M log L) via bounded heaps.

For Phase 1 this looks small; once N = top-level centroids and M = rows in
all probed leaves, the difference matters. The fix is structural (BinaryHeap
of size k with reverse compare), not a tuning knob.

### A6. `binary_search_by_key` on every placement / manifest lookup

`SpireObjectManifest.get(pid)` and `SpirePlacementDirectory.get(pid)` are both
binary searches on `entries: Vec<...>`. Scan code repeatedly looks up
`object_manifest.get(pid)` and `placement_directory.get(pid)` for the same
pids. For large epochs this is N · log N comparisons per scan when a
once-per-snapshot built `HashMap<u64, &Entry>` (or `(pid, idx)` parallel
array) would be O(1) per lookup.

This intersects A3: a `Validated<Snapshot>` wrapper is the natural place to
cache these lookup tables.

### A7. HashMap dedupe in scan is dead weight in Phase 1

`rank_routed_leaf_rows_by_ip` always builds a `HashMap<SpireVecId, candidate>`
to keep the best-scoring duplicate. In Phase 1 there is exactly one row per
vec_id — boundary replication is Phase 5. So the dedup HashMap is paid on
every scan now even though it can never deduplicate. Either:

- Defer the dedup path entirely until Phase 5, or
- Switch to a "dedup mode" flag the snapshot carries (boundary replication
  on/off), so Phase 1 takes a straight Vec path.

`SpireVecId` hashing is also `Vec<u8>`-based — for the local case it's always
9 bytes (`0x01 || u64`). A specialized `LocalVecId(u64)` newtype with native
hashing would eliminate the byte-slice hash entirely on the local path.

### A8. Encode-as-validate doubles serialization cost

`validate_leaf_assignment` calls `assignment.encode()` to validate, and write
paths then call `encode()` again to actually emit bytes. For million-row
builds this is a 2× serialization tax. Split into a `validate_fields()` that
checks invariants without allocating, and let `encode()` debug-assert the
invariants.

### A9. Publication ordering is documented, not enforced by a type

The Phase 0 design lays out the 6-step publish order (objects → placements →
manifest building → validate → mark published → advance active_epoch). The
codec hands callers a `SpireEncodedPublishBundle` of three byte blobs ready to
write — but nothing in the type system stops a future caller from advancing
`active_epoch` before the bundle is durable. Once the live insert/vacuum path
is written, this is the easiest place to introduce a "scan sees half-published
epoch" bug.

A `SpirePublishCoordinator` with explicit state transitions
(`build → durable → published → active`) and a destructor that fails-closed
on drop would lock this in. Even an in-memory test version would be valuable
now, before any real I/O.

## Smaller things worth fixing while the format is still in-memory

### S1. Object header has no epoch back-reference

The encoded object header carries pid, object_version, parent_pid but not
epoch. Today that's fine (placement carries epoch, immutable objects are
shared across epochs). But consider whether you want a debug-only epoch
back-reference in the header for crash forensics.

### S2. `get_page` is `O(1)` only because `DataPageChain` keeps `pages` contiguous

`get_page(block_number)` does `block_number - FIRST_DATA_BLOCK_NUMBER` and
indexes the Vec. Once persistence drives this from real PostgreSQL buffers,
"give me block N" needs a `ReadBufferExtended` call, not a Vec index. The
reader interface should be abstracted now (`trait SpireObjectReader`) so the
in-memory test path and the live buffer-cache path share a contract — right
now `SpireLocalObjectStore` is the only reader and the codec is hard-wired to
it.

### S3. `encode_assignment_payload` returns `(u16 dim, f32 gamma, Vec<u8>)` but
the dim is always discarded

`encode_assignment_input` ignores the returned dimensions. Either drop the
return value or actually plumb it into a width validator at write time.

### S4. Diagnostics counts bytes-by-kind only via `available_object_bytes` total

`SpireSnapshotDiagnostics.available_object_bytes` is one combined total. For
the eventual operator UI you'll want `root_bytes` / `internal_bytes` /
`leaf_bytes` / `delta_bytes` separately so growth attribution is obvious.

### S5. Allocator near-exhaustion has no diagnostic surface

`SpirePidAllocator` and `SpireLocalVecIdAllocator` error on `u64::MAX` but
don't surface "next_pid is approaching X" anywhere. Phase 1 will probably
never hit it, but a diagnostic field would catch unexpected allocation
patterns (e.g. a runaway insert workload).

### S6. `is_visible_primary_assignment` semantics live in two places

Codec validation requires a role flag; scan filter says "PRIMARY without
{BOUNDARY_REPLICA, TOMBSTONE, DELTA_DELETE, STALE_LOCATOR}". When boundary
replication lands, the correct change is in scan.rs — and storage.rs's
validation will need to relearn the same rule.

### S7. `scored_candidate_cmp` tie-break

Tie-break is heap_tid + pid + row_index. That's fine for determinism today,
but once boundary replicas exist you'll want "primary beats replica when
scores tie" and "newer epoch beats older epoch when split/merge transitions".
Worth pinning this contract down before Phase 5 changes the assumption
silently.

### S8. `SpirePlacementEntry::local_single_store` always returns
`SpirePlacementState::Available`

The constructor sets the state instead of leaving it for the caller. Tests
have to mutate `placement.state = ...` after the fact (see scan.rs tests at
lines 1184, 2173). Consider a builder or two constructors so degraded states
are first-class rather than retro-fitted.

## Suggested next concrete actions, in order

1. **Resolve A1.** Pick the multi-page leaf-object representation now and bake
   it into the codec while everything is still in-memory. This is the only
   foundation decision that gates the rest of Phase 1.
2. **Add `Validated<SpirePublishedEpochSnapshot>` + a pid-indexed lookup
   cache.** Resolves A3 and A6 in one structural change. Keep the existing
   `SpirePublishedEpochSnapshot::new` validator; add a wrapper that internal
   callers consume.
3. **Land `LeafPartitionObjectV2` (column-major, single payload format per
   leaf) + `SpireLeafObjectColumns<'a>` borrow-from-page view + a
   `score_batch_ip` entry on `SpirePreparedAssignmentScorer`.** Resolves all
   three sub-points of A2 in one format change. Keep V1 / row-encoded shape
   for `SpireDeltaPartitionObject` (deltas need mixed flags) and tests.
4. **Replace centroid `Vec<f32>`-per-child with flat parallel arrays in
   routing objects.** Resolves A4 before the format ossifies.
5. **Replace top-K Vec+sort+truncate with bounded BinaryHeap in the routing
   and ranking helpers.** Resolves A5; mechanical change.
6. **Skip the dedup HashMap when boundary replication is off.** Resolves A7
   for Phase 1; keep the HashMap path behind a feature/snapshot flag for
   Phase 5.
7. **Introduce a `SpirePublishCoordinator` state machine.** Even purely
   in-memory now — gives the live persistence path a typed contract for the
   6-step publish sequence (A9).

The codec layering and snapshot-validation discipline are excellent
foundations. The above changes are about making sure the *shape* of those
foundations matches the SPIRE paper's "many small leaves, fast batched
scoring" target before live PostgreSQL pages fix the format.
