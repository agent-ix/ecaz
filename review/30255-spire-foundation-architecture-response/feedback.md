---
reviewer: opus47
status: open
created: 2026-05-02
checkpoint_commit: 67b2027e
verdict: architecture-gate-cleared-with-followups
scope: holistic second-pass review of the SPIRE foundation after the
  architecture gate (packets 30255–30279), verifying that A1–A9 and S1–S8 from
  review/30219 actually landed in code, and auditing the new live-persistence
  slices (page.rs, SpireRelationObjectStore, ambuild/amrescan wiring)
---

# Review: SPIRE Foundation — second architecture pass

This is a follow-up to `review/30219-spire-foundation-progress-status/feedback.md`.
Coder-1 worked the entire 7-step "Required Slice Order" from the architecture
response, plus all S1–S8 follow-ups, plus opened the live-persistence path
with `page.rs` and `SpireRelationObjectStore`. Tests are green
(185 ec_spire unit + pg_test cases pass on pg18).

The architecture gate is cleared. New flags below are about the live
persistence slices — they are not regressions from the gate, but they reopen
some of the same shape questions in the new buffer-cache code.

## Verified — all gate items landed

I read the diffs and the resulting code, not just commit messages. Each item
below has a concrete code anchor.

| Item | Status | Anchor |
|------|--------|--------|
| **A1** segmented multi-segment leaf objects | ✅ | `storage.rs::leaf_v2_max_segment_rows` (2821), `insert_leaf_object_v2_from_rows` (1721) |
| **A2 (1)** borrow-from-page rows | ✅ | `SpireLeafAssignmentRowRef<'a>` + `SpireVecIdRef<'a>` in `storage.rs` |
| **A2 (2)** column-major layout | ✅ | `SpireLeafObjectColumns<'a>` with parallel `&[u16]/&[u8]/&[ItemPointer]/&[f32]/&[u8]` slices |
| **A2 (3)** batch scoring API | ✅ | `quantizer.rs::SpirePreparedAssignmentScorer::score_batch_ip` (149); used in `scan.rs::append_quantized_v2_column_candidates` (699) |
| **A3** `Validated<Snapshot>` wrapper | ✅ | `meta.rs::SpireValidatedEpochSnapshot` (972); 6 production scan call sites converted, V1 `SpirePublishedEpochSnapshot::new` calls in scan are tests-only |
| **A4** flat routing arrays | ✅ | `SpireRoutingPartitionObject { centroid_ordinals, child_pids, centroids }` parallel arrays + `SpireRoutingChildView<'a>` |
| **A5** bounded heaps | ✅ | `scan.rs::rank_bounded_scored_candidates` and `route_root_object_to_leaf_pids` rewritten to keep-best `BinaryHeap` |
| **A6** PID-indexed lookups | ✅ | `pid_index: HashMap<u64, SpireSnapshotPidLookup<'a>>` built once at scan start; binary_search calls only remain in legacy `get()` (kept for tests) |
| **A7** dedup mode | ✅ | `SpireCandidateDedupeMode { NoReplicaDedupeDisabled, VecIdDedupeEnabled }`; Phase 1 default `NoReplicaDedupeDisabled` skips the HashMap entirely |
| **A8** split validate from encode | ✅ | `validate_for_format_version` / `encode_after_validation` (storage.rs); leaf+routing+delta+V2 codecs no longer encode-as-validate |
| **A9** publish coordinator type-state | ⚠️ | `build.rs` (93–256). State machine exists and consumes `self`, but see F1 below. |
| **S1** epoch back-reference in header | ✅ | `published_epoch_backref: u64` on header; stamped at write in every store path; readers reject `0` or `> placement.epoch` |
| **S2** `SpireObjectReader` trait | ✅ | `storage.rs` defines trait; both `SpireLocalObjectStore` and `SpireRelationObjectStore` implement it; scan/diagnostics consume the trait |
| **S3** drop unused dimension return | ✅ | `encode_assignment_payload -> Result<(f32, Vec<u8>)>` |
| **S4** diagnostics bytes by kind | ✅ | `SpireSnapshotDiagnostics { routing_object_bytes, leaf_object_bytes, delta_object_bytes }` plus aggregate |
| **S5** allocator near-exhaustion | ✅ | `SpireAllocatorExhaustionDiagnostics` derived from root-control cursors without advancing |
| **S6** dedupe visibility predicate | ✅ | `is_visible_primary_assignment_flags(u16)` in storage; scan + storage both call it |
| **S7** epoch + role tie-break | ✅ | `SpireScoredScanCandidate { epoch, assignment_flags, … }`; `scored_candidate_cmp` adds epoch (newer first) → role-rank → existing tie-break |
| **S8** explicit placement-state ctors | ✅ | `local_single_store_available / _stale / _unavailable / _skipped / _with_state` |

Test density also grew: from ~163 to 185 cases on the pg18 lib path, plus
5 pg_test cases now exercise live persistence (`pg_test_ec_spire_relation_leaf_v2_roundtrip`,
`pg_test_ec_spire_relation_object_tuple_roundtrip`, `pg_test_ec_spire_empty_build_scan_no_rows`,
plus AM and opclass registration tests).

## Things that are particularly strong

- **`leaf_v2_max_segment_rows` does honest accounting.** It computes the
  per-row stride (`size_of::<u16>() + vec_id_stride + ITEM_POINTER_BYTES +
  size_of::<f32>() + payload_stride`), then iteratively shrinks until the
  segment fits via `element_or_neighbor_tuple_fits`. No optimistic
  arithmetic.
- **V2 segment chains are written in reverse so each segment's
  `next_segment_locator` is known before write.** Cleaner than a two-pass
  layout. (`insert_leaf_object_v2_from_rows` line 1777 — `(0..segment_count).rev()`.)
- **The provisional-meta + final-meta pattern computes
  `object_bytes_total` from real segment lengths**, not estimates. Placement
  carries the exact total so the reader can bounds-check at meta-decode time.
- **`pid_index` is built once with `with_capacity(...)`**, which is the right
  sizing — no incremental rehashes.
- **The publish coordinator's failed states preserve `(stage, error)`** so a
  later "what failed where" diagnostic doesn't need to re-derive.
- **Live `read_leaf_object_v2` cross-checks `placement.object_bytes ==
  meta.object_bytes_total`.** Catches the truncated/torn-write case at the
  meta tuple before walking segments.

## New flags from the live-persistence slice

These are not in the architecture gate — they're things I noticed reading
`page.rs` and `SpireRelationObjectStore` after they landed (b55dbadd,
486ccdd1, 67b2027e). All are easy to address now while the live path is
small.

### F1. Publish coordinator stages don't carry write evidence

`SpirePublishWritingObjects::objects_written(self) -> SpirePublishWritingPlacements`
and `SpirePublishWritingPlacements::placements_written(self) -> SpirePublishWritingManifest`
are no-op type transitions. They enforce *order* but not *that the writes
happened*. A caller can still write:

```rust
SpirePublishWritingObjects::new(input)
    .objects_written()        // can be called without writing anything
    .placements_written()     // ditto
    .write_manifests()?
    .validate()?
    .publish_active_epoch(locators)?
```

In the in-memory test path that's fine (no writes happen anyway). Once
`SpireRelationObjectStore::insert_leaf_object_v2_from_rows` and
`page::append_object_tuple` start being called from `ambuild`/`aminsert`/
`ambulkdelete`, the coordinator should consume the placements that were
actually written and verify they cover every entry in the
`placement_directory` it was given.

Concrete proposal:

```rust
impl<'a> SpirePublishWritingObjects<'a> {
    pub(super) fn objects_written(
        self,
        written: &[(u64, ItemPointer)],   // (pid, object_tid) returned by store
    ) -> Result<SpirePublishWritingPlacements<'a>, SpirePublishFailed> {
        // every pid in placement_directory must appear in `written`,
        // and the tids must match.
    }
}
```

Today the coordinator is type ceremony. Make the transitions consume
evidence so a future refactor cannot accidentally publish an epoch without
durable objects.

### F2. `page::read_object_tuple` returns `Vec<u8>` (unpins before decode)

`page.rs::read_object_tuple_from_locked_page` does
`std::slice::from_raw_parts(...).to_vec()`, then the caller does
`UnlockReleaseBuffer`. Every read allocates an owned `Vec` per tuple.

This undoes the A2 win at the page-cache boundary: column views now borrow
from owned in-Rust segment bytes instead of borrowing from pinned page
memory. For Phase 1 sanity it's fine; for the eventual hot scan path it is
the per-row allocation we just designed away.

The natural shape that preserves both pin lifetime and the `Vec` API is
`with_pinned_object_tuple<F: FnOnce(&[u8]) -> Result<R>>`:

```rust
pub(super) unsafe fn with_pinned_object_tuple<F, R>(
    index_relation: pg_sys::Relation,
    tid: ItemPointer,
    f: F,
) -> Result<R, String>
where
    F: FnOnce(&[u8]) -> Result<R, String>,
```

Then `read_leaf_object_v2` runs the segment decoder inside the closure
while the buffer is still pinned, and the column view borrows from that
slice for as long as the closure runs. Owned `Vec` stays available as a
secondary path for callers that legitimately need to outlive the pin.

### F3. `append_object_tuple` doesn't use the FSM, only "last block"

```rust
if existing_blocks > FIRST_DATA_BLOCK_NUMBER {
    let last_data_block = existing_blocks - 1;
    if let Some(tid) = try_append_object_tuple_to_block(index_relation, last_data_block, payload)? {
        return Ok(tid);
    }
}
unsafe { append_object_tuple_to_new_block(index_relation, payload) }
```

For one-shot build this is fine. For incremental insert / delta epoch
publication, this strands free space on every earlier block as soon as the
last block fills. The `RecordPageWithFreeSpace` calls inside the helpers do
update the FSM, but the caller never *reads* it via `GetPageWithFreeSpace`.

Either:
- Use `GetPageWithFreeSpace(index_relation, payload_len + line_pointer_overhead)`
  before falling back to "new block", so the FSM actually serves its purpose, or
- Document that this helper is build-only and add a separate
  `append_object_tuple_with_fsm` for the insert path.

### F4. `append_object_tuple_to_new_block` doesn't guard against allocating
block 0 as a data page

If `existing_blocks == 0` (relation has no metadata page yet) and somebody
calls `append_object_tuple` first, `RBM_ZERO_AND_LOCK` with `P_NEW`
allocates block 0 as a regular `PageInit(page, page_size, 0)` data page.
Then a subsequent `initialize_root_control_page` call sees `existing_blocks
> 0`, locks block 0 exclusively, and overwrites its special area —
clobbering the data tuples that were just written.

Today the only callers go through `ambuild`/`ambuildempty` first (which
initialize block 0), so the order is correct. But there's no guard.
Suggest:

```rust
unsafe fn append_object_tuple_to_new_block(...) -> Result<...> {
    let existing_blocks = pg_sys::RelationGetNumberOfBlocksInFork(...);
    if existing_blocks < FIRST_DATA_BLOCK_NUMBER {
        return Err("ec_spire root/control block must be initialized before object tuples".to_owned());
    }
    // ... existing P_NEW path
}
```

### F5. `read_root_control_page` doesn't bounds-check the special area

```rust
let root_control_ptr = pg_sys::PageGetSpecialPointer(page).cast::<u8>();
let root_control_bytes = std::slice::from_raw_parts(
    root_control_ptr,
    SpireRootControlState::encoded_len(),
);
```

`encoded_len()` is constant today, but if the special area's actual size
ever shrinks (older format, partial write, manual page corruption), this
reads off the end. Easy fix:

```rust
let special_size = pg_sys::PageGetSpecialSize(page) as usize;
if special_size < SpireRootControlState::encoded_len() {
    pgrx::error!("ec_spire root/control special area too small: {special_size}");
}
```

### F6. `ec_spire_amrescan` re-reads root control on every rescan

```rust
let root_control = page::read_root_control_page((*scan).indexRelation);
```

That's one buffer pin/unpin per rescan. Under nested-loop with N inner
rescans it is N redundant reads. Cache it on the scan opaque after first
read; invalidate only if `active_epoch` is observed to change between
scans. Phase 1 reads `active_epoch == 0` and short-circuits anyway, so this
is cheap, but it sets a habit. Future epoch loading is much heavier and
needs caching from the start.

### F7. `ec_spire_ambuild` writes root control AFTER the table scan

```rust
let heap_tuples = pg_sys::table_index_build_scan(..., callback, ...);
page::initialize_root_control_page(index_relation, SpireRootControlState::empty());
```

Today the callback `pgrx::error!`s on any tuple, which longjmps and skips
the post-scan `initialize_root_control_page`. CREATE INDEX would also roll
back the relation creation in the same transaction, so the relation never
becomes visible — fine.

But once `ambuild` actually populates a root control with non-empty state,
the order matters: if you write objects/placements first then root control
last, a crash between them leaves a partial relation. The `ambuild`
ordering should match what the publish coordinator (F1) ends up enforcing:
write objects → write placements → write manifests → flip root control's
`active_epoch`. The current single-step `initialize_root_control_page` is
fine for empty-only, but the populated-build code should not just append
more code after the same scan call — it should drive the publish
coordinator end-to-end.

### F8. `SpireRelationObjectStore` accepts mutable I/O through `&self`

```rust
pub(super) unsafe fn insert_routing_object(&self, ...) -> Result<...>
pub(super) unsafe fn insert_leaf_object_v2_from_rows(&self, ...) -> Result<...>
```

Insertion mutates the index relation but borrows `&self` not `&mut self`.
That's a deliberate concurrency choice (Postgres handles buffer locking)
but it sidesteps Rust's exclusivity check at the type level. The
`SpireLocalObjectStore` equivalents are `&mut self` (since they own the
`DataPageChain`). The asymmetry is correct given the underlying contracts,
but worth noting in module docs so future readers don't try to "fix" the
relation store to also be `&mut self` and then break the trait
implementation.

## Minor / nice-to-have

- The two `binary_search_by_key` calls in `meta.rs` (`SpireObjectManifest::get`,
  `SpirePlacementDirectory::get`) are now bypassed by `pid_index` on hot paths.
  They survive only for tests and for the legacy `get()` API. Either delete
  them outright (callers can use the validated wrapper) or comment that
  `pid_index` is the production path.
- `SpireScanQuery` is decoded fresh on every `amrescan`. For nested-loop
  with many iterations of identical query vectors, this is small but
  unnecessary work; consider memoizing under the same opaque the root
  control will live in once F6 lands.
- The `ec_spire_aminsert` callback is still `not_implemented`. That's
  expected for the current slice — flagging it so the next reviewer knows
  it's intentional.
- `column_segments()` returns a `Vec<SpireLeafObjectColumns<'_>>` — fine for
  tests, but for a multi-segment leaf in the scan path it allocates a
  segment-count Vec just to iterate. An `impl Iterator<Item = …>` would
  avoid that allocation. Tiny.

## What's good architectural news

The slice order coder-1 picked (V2 codec → borrowed views → batch scorer →
validated wrapper → flat routing → bounded heaps → dedup mode → publish
coordinator → live persistence) was the right one. By the time
`SpireRelationObjectStore` showed up it could just plug into the
already-redesigned hot path: `read_leaf_object_v2` decodes → segments
expose `column_segments()` → scan calls `score_batch_ip`. None of those
abstractions had to be redesigned to fit the live reader.

The remaining live-persistence concerns (F1–F8) are localized to `page.rs`
and the new `SpireRelationObjectStore` writes. They do not require
architecture changes, just hardening of the new I/O surface.

## Suggested next concrete actions

1. **F1** — make publish coordinator stages consume write evidence before
   live persistence wires up beyond empty-build.
2. **F2** — add a `with_pinned_object_tuple<F>` reader so leaf scans can
   decode while pinned.
3. **F4** — guard `append_object_tuple_to_new_block` against allocating
   block 0.
4. **F5** — bounds-check `PageGetSpecialSize` before reading root control
   bytes.
5. **F3, F6, F7, F8** — nice to have alongside or just before populated
   `ambuild` lands.

Once F1–F5 are addressed, the live-persistence path is ready for
populated `ambuild` + `aminsert` + delta epochs without re-opening the
hot-path layout questions.
