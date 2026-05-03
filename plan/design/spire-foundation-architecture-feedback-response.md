# SPIRE Foundation Architecture Feedback Response

Status: pre-persistence architecture gate for Task 30
Date: 2026-05-02
Review source: `review/30219-spire-foundation-progress-status/feedback.md`

This note records the response to the first holistic SPIRE foundation review.
The review accepts the Phase 0 storage direction, but correctly identifies
storage-format and hot-path choices that must be fixed while the SPIRE object
format is still in-memory only.

Live PostgreSQL relation-backed persistence is blocked until the hardening
items below are either implemented or explicitly replaced by an accepted design
update.

## Gate Decisions

1. Replace single-contiguous leaf objects with a segmented, column-major
   `LeafPartitionObjectV2` before live persistence.
2. Add borrowed leaf-object read views and batch scoring entry points before
   scan callbacks consume persisted partition objects.
3. Construct one validated epoch snapshot with PID-indexed object and placement
   lookups at scan/publication boundaries; internal hot-path helpers must not
   revalidate a snapshot repeatedly.
4. Replace routing-object child entries with flat centroid and child-PID arrays
   before relation-backed routing objects are durable.
5. Use bounded heaps for top-`nprobe` routing and candidate top-k selection.
6. Keep `vec_id` dedupe off the Phase 1 hot path while boundary replicas are
   disabled; re-enable through an explicit scan/snapshot mode when replicas or
   remote merge land.
7. Introduce a typed publish coordinator before live object and epoch I/O so
   publication order is enforced by code, not only by comments.

## Leaf Partition Object V2

The logical storage unit remains:

```text
(pid, object_version) -> partition object
```

The physical representation changes from one page-bounded tuple to a logical
object made of one metadata tuple plus one or more row-segment tuples. The
placement entry for a PID points at the metadata tuple; the metadata tuple owns
the segment chain. In strict mode, the placement is available only when the
metadata tuple and every referenced segment tuple are durable and readable.

```text
LeafPartitionObjectV2 meta tuple
  partition_object_header
    format_version = 2
    kind = leaf
    pid
    object_version
    level
    parent_pid
    assignment_count
    flags
    published_epoch_backref
  payload_format
  payload_stride
  vec_id_kind
  vec_id_stride
  segment_count
  first_segment_locator
  object_bytes_total

LeafPartitionObjectV2 segment tuple
  pid
  object_version
  segment_no
  row_base
  row_count
  next_segment_locator
  flags:     u16[row_count]
  vec_ids:   u8[vec_id_stride * row_count]
  heap_tids: u8[ITEM_POINTER_BYTES * row_count]
  gammas:    f32[row_count]
  payloads:  u8[payload_stride * row_count]
```

Each segment is sized so its tuple fits one PostgreSQL data page. A leaf with
thousands of assignments therefore spans many row segments while retaining one
logical PID and one object version. This removes the current low-hundreds row
ceiling without changing the Phase 0 placement shape.

Phase 1 leaf objects are constrained to one assignment payload format per
object. That matches the build-time storage format and enables fixed
`payload_stride` validation. Delete/tombstone deltas can remain row-encoded
because they are small and may need `payload_format = NONE`; compaction rewrites
base plus deltas back into a V2 base leaf object.

Phase 1 local IDs use a fixed-width local row representation:

```text
vec_id_kind = local_u64
vec_id_stride = 16
vec_id bytes = 0x01 || local_vec_seq:u64 || zero padding
```

The existing variable-width `SpireVecId` remains the logical API and remote
future-proofing boundary. A future local-to-global rewrite publishes replacement
objects with `vec_id_kind = global_bytes` and a wider fixed stride, then retains
old local-ID epochs until the retention horizon passes. Mixed local/global IDs
inside the same V2 base leaf are not allowed; mixed identity windows occur
across retained epochs or in small delta objects.

## Borrowed Reads and Batch Scoring

The V2 decoder should return a borrowed column view instead of owned rows:

```rust
pub(super) struct SpireLeafObjectColumns<'a> {
    pub(super) header: SpirePartitionObjectHeader,
    pub(super) payload_format: SpireAssignmentPayloadFormat,
    pub(super) payload_stride: usize,
    pub(super) vec_id_kind: SpireVecIdKind,
    pub(super) vec_id_stride: usize,
    pub(super) flags: &'a [u16],
    pub(super) vec_ids: &'a [u8],
    pub(super) heap_tids: &'a [u8],
    pub(super) gammas: &'a [f32],
    pub(super) payloads: &'a [u8],
}
```

The row-encoded V1 helper can stay for deltas and compatibility tests, but scan
collection should move to borrowed accessors:

```rust
pub(super) struct SpireLeafAssignmentRowRef<'a> {
    pub(super) flags: u16,
    pub(super) vec_id: SpireVecIdRef<'a>,
    pub(super) heap_tid: ItemPointerData,
    pub(super) gamma: f32,
    pub(super) encoded_payload: &'a [u8],
}
```

Scoring should add a batch entry point that accepts the column payload block and
gamma array. TurboQuant and RaBitQ can initially loop over chunks behind this
entry point; PQ-FastScan needs the same shape before grouped table lookup is
worth wiring.

```rust
score_batch_ip(payload_stride, payloads, gammas, out_scores)
```

The visibility predicate for primary, replica, and tombstone flags should live
in one helper shared by V1 row references and V2 columns.

Implementation checkpoint: `SpirePreparedAssignmentScorer::score_batch_ip`
now validates payload stride/counts and scores TurboQuant/RaBitQ payload chunks
into caller-provided output storage. The scan path still uses row-at-a-time
scoring until V2 column views are wired into leaf reads.

Implementation checkpoint: decoded `LeafPartitionObjectV2` segments now expose
borrowed column views over flags, fixed-stride vec_id bytes, heap TIDs, gammas,
and payload chunks, plus bounds-checked row accessors for compatibility code.
The view currently borrows from the decoded in-memory object; relation-backed
buffer readers can later construct the same view directly over page bytes.

## Snapshot Lookup and Validation

`SpirePublishedEpochSnapshot::new` remains the construction-time validation
boundary. Hot-path helpers should consume a wrapper that proves the snapshot was
validated once and carries PID-indexed lookup caches:

```rust
pub(super) struct SpireValidatedEpochSnapshot<'a> {
    snapshot: SpirePublishedEpochSnapshot<'a>,
    pid_index: HashMap<SpirePid, SpireResolvedObjectPlacement>,
}
```

The cache is built once at scan start, publication validation, or update-draft
construction. It resolves each PID to its manifest entry plus placement entry,
including strict/degraded placement-state eligibility. Leaf reads through this
wrapper should validate object header, format version, segment chain, and slice
lengths. Expensive row-level uniqueness checks belong on write/publish paths,
not every scan read.

Implementation checkpoint: scan and diagnostics helpers use the validated
snapshot wrapper, build and delta publication helpers validate through the same
wrapper, and `build_delta_epoch_draft_from_snapshot` keeps the wrapper through
base PID lookup, assignment-ID collection, and visible-row validation.

## Routing Object V2

Root and internal routing objects must move from one allocated centroid vector
per child to flat arrays:

```text
RoutingObjectV2
  dimension
  child_count
  child_pids:        u64[child_count]
  centroid_ordinals: u32[child_count]
  centroids:         f32[child_count * dimension]
```

This keeps child PID lookup and centroid scoring contiguous. The route map can
borrow slices from the decoded object and score centroid rows with chunked or
SIMD-friendly loops.

Implementation checkpoint: `SpireRoutingPartitionObject` now stores
`child_pids`, `centroid_ordinals`, and a flat `centroids` block. Constructors
still accept `SpireRoutingChildEntry` for build compatibility, while scan and
diagnostics consume borrowed child views from the flat arrays.

## Top-K Selection

Top-`nprobe` routing and candidate ranking should use bounded heaps:

```text
route_root_object_to_leaf_pids: O(children * log nprobe)
rank_leaf_candidates:          O(rows * log limit)
```

Implementation checkpoint: the row-encoded routed scan helper now keeps bounded
max-heaps whose head is the worst retained entry, then sorts only the retained
entries before returning them. Route selection ranks higher centroid
inner-product first, then lower centroid index, then lower child PID. Candidate
selection ranks lower ORDER BY score first (`score = -inner_product`), then
lower heap TID block/offset, PID, row position, and `vec_id` bytes. When
boundary replicas or remote merge are enabled, the explicit dedupe step applies
before final limit selection.

## Dedup Mode

Phase 1 writes one primary assignment row per live vector and has no boundary
replicas. The scan plan should carry an explicit dedupe mode:

```text
NoReplicaDedupeDisabled
VecIdDedupeEnabled
```

Implementation checkpoint: `SpireSingleLevelScanPlan` now carries this mode and
the resolver defaults Phase 1 local scans to `NoReplicaDedupeDisabled`.
`rank_routed_leaf_rows_by_ip` only allocates the per-candidate
`HashMap<SpireVecId, ...>` when the caller passes `VecIdDedupeEnabled`;
otherwise candidates flow directly into the bounded top-k heap. `VecIdDedupeEnabled`
is still available for boundary replicas, retained mixed-ID epochs, or remote
candidate merge.

## Encode and Validate Split

Codec methods should stop using encode-as-validation on hot paths. The shape
should be:

```rust
validate_fields() -> Result<(), String>
encoded_len() -> Result<usize, String>
encode_into(dst) -> Result<(), String>
decode_borrowed(bytes) -> Result<View<'_>, String>
```

Write paths call `validate_fields()` and uniqueness checks before publication.
Read paths validate only structural invariants that protect memory safety and
object compatibility.

## Publish Coordinator

Before relation-backed persistence, publication should move through a typed
coordinator:

```text
WritingObjects
  -> WritingPlacements
  -> WritingManifest
  -> Validating
  -> PublishingActiveEpoch
  -> Published | Failed
```

Each transition consumes the previous state and exposes only the operations
valid for the next state. The active epoch can advance only from
`PublishingActiveEpoch` after object, placement, and manifest validation
succeeds. Failures before that transition record failed/building state and leave
the prior active epoch authoritative.

## Smaller Follow-Ups

- Add `published_epoch_backref` to object headers for diagnostics and forensic
  cleanup.
- Introduce a `SpireObjectReader` trait so the current in-memory object store
  and the future buffer-cache reader share the same read contract.
- Stop returning discarded dimensions from assignment payload helpers unless
  the caller validates them.
- Split diagnostics bytes by routing, leaf-base, delta, and future graph kinds.
- Add allocator near-exhaustion diagnostics for PID and local vec ID cursors.
- Keep primary-assignment visibility semantics in one helper.
- Extend tie-break documentation when replicas and newer replacement epochs
  participate in the same result merge.
- Add placement builders/constructors that make available/stale/unavailable
  state explicit at construction time.

## Required Slice Order

1. Implement V2 segmented columnar leaf codecs and borrowed views.
2. Switch scan collectors to borrowed V2 reads and batch scorer entry points.
3. Add validated snapshot wrappers and PID-indexed lookup caches.
4. Convert root routing objects to the flat V2 layout.
5. Replace sort/truncate with bounded heaps.
6. Add explicit dedupe mode and skip dedupe in the Phase 1 primary-only path.
7. Add the publish coordinator before live object/placement/manifest writes.

Only after these slices should relation-backed snapshot loading or partition
object persistence be wired into `ambuild`, `aminsert`, `ambulkdelete`,
`amvacuumcleanup`, or live `amrescan`.
