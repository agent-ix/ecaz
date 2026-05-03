# SPIRE Phase 0 Partition-Object Storage Design

Status: Phase 0 checkpoint for Task 30
Date: 2026-05-01
Scope: local single-store foundation, with local multi-store and remote shape
preserved in metadata

This note resolves the storage decisions that must be settled before writing
SPIRE persistence code. It is subordinate to ADR-049 and updates Task 30's
Phase 0 checklist.

## Decisions

1. Use PostgreSQL-managed relation-backed storage, not raw sidecar files.
2. Make partition objects the durable unit addressed by PID and object version.
3. Publish visibility changes through epoch manifests. Do not mutate published
   partition objects in place.
4. Use index-local `vec_id` values in Phase 1 instead of deriving `vec_id`
   directly from heap TIDs.
5. Expose Phase 1 as an opt-in `ec_spire` access method with explicit SPIRE
   opclasses: `ecvector_spire_ip_ops` and `tqvector_spire_ip_ops`.
6. Before relation-backed persistence, replace the current in-memory
   row-contiguous leaf object with the segmented, column-major V2 shape in
   `plan/design/spire-foundation-architecture-feedback-response.md`.

## Storage Shape

Phase 1 uses one relation-backed local store. The `ec_spire` index relation is
the root/control relation and also hosts `local_store_id = 0` object storage.
The page layout still separates root/control pages from object-store pages so
Phase 4 can move from one local store to bounded auxiliary store relations
without changing logical addressing.

The durable metadata model is:

```text
root/control relation
  active_epoch
  next_pid
  next_local_vec_seq
  local_store_config[]
  epoch_manifest[]
  placement_entry[]

partition-store relation
  partition_object(pid, object_version, kind, level, parent_pid, payload)
```

A placement entry is concrete from Phase 1:

```text
placement_entry
  epoch bigint
  pid bigint
  node_id int              -- 0 for local Phase 1
  local_store_id int       -- 0 for single-store Phase 1
  store_relid oid          -- index relid in Phase 1
  object_version bigint
  object_block int
  object_offset int
  object_bytes int
  state available | stale | unavailable | skipped
```

Future local multi-NVMe placement uses a bounded number of partition-store
relations, each assigned to a configured PostgreSQL tablespace. The placement
shape is still:

```text
pid -> local_store_id -> object location
```

Future remote placement extends the same entry by using non-zero `node_id`:

```text
pid -> node_id -> local_store_id -> object location
```

SPIRE vector partition selection never uses PostgreSQL declarative table
partitions. PostgreSQL chooses the `ec_spire` access path; SPIRE chooses PIDs
from root/hierarchy metadata.

## Partition Objects

Partition objects are immutable once published. Each object has:

```text
PartitionObjectHeaderV1
  format_version u16
  kind root | internal | leaf | delta
  pid u64
  object_version u64
  published_epoch_backref u64 -- diagnostic/forensic back-reference
  level u16
  parent_pid u64          -- 0 means no parent/root
  child_count u32
  assignment_count u32
  flags u32
```

Internal objects store routing metadata and child PIDs. Leaf objects store
logical assignment rows. Delta objects store epoch-published insert/delete
changes for one PID until compaction rewrites a replacement leaf object.

The initial in-memory codec uses a row-contiguous V1 leaf object while Phase 1
helpers are being built. The architecture gate from the first foundation review
requires the live persisted base-leaf format to be `LeafPartitionObjectV2`: one
metadata tuple plus one or more row-segment tuples, with column-major arrays for
flags, fixed-stride `vec_id`s, heap TIDs, gammas, and encoded payload bytes.
The placement entry still addresses one logical `(pid, object_version)` object;
the V2 metadata tuple owns the segment chain. The header
`published_epoch_backref` is stamped when the object is written and is verified
as nonzero and not newer than the placement epoch on reads for diagnostics and
crash forensics; the epoch manifest remains the authoritative compatibility
boundary, and later epochs may reference older immutable objects. Strict-mode
availability requires every segment in that chain to be durable and readable.

V2 base leaves use one assignment payload format and one payload stride per
object. Delete and tombstone deltas may remain row-encoded because they are
small and can carry `payload_format = NONE`; compaction rewrites base plus
deltas into a replacement V2 leaf object.

The logical assignment row is `(vec_id, pid)`. Physically, the leaf object
header carries `pid`, so the row may omit a repeated PID field and the decoder
rehydrates it from the containing object.

```text
LeafAssignmentRowV1
  flags u16
  vec_id_len u8
  vec_id bytea            -- <= 32 bytes, first byte is discriminator
  heap_tid tid            -- local row locator
  payload_format u8
  gamma f32
  encoded_payload_len u32
  encoded_payload bytea
```

`payload_format` values are explicit storage-format tags:

```text
NONE        0  -- delete/tombstone rows without scored payload bytes
TURBOQUANT  1  -- TurboQuant MSE/QJL payload bytes
PQ_FASTSCAN 2  -- grouped-PQ/PQ-FastScan payload bytes
RABITQ      3  -- RaBitQ payload bytes
```

Rows that participate in scoring (`PRIMARY` or `BOUNDARY_REPLICA`) must use a
non-`NONE` known payload format and carry non-empty `encoded_payload` bytes.
Delete delta rows must use `NONE`, zero `gamma`, and an empty payload.

Assignment flags:

```text
PRIMARY             0x0001
BOUNDARY_REPLICA    0x0002
TOMBSTONE           0x0004
DELTA_INSERT        0x0008
DELTA_DELETE        0x0010
STALE_LOCATOR       0x0020
```

Phase 1 writes one primary row per vector. Boundary replicas are deferred, but
they use the same `vec_id` in multiple leaf PIDs with `BOUNDARY_REPLICA` set.

## Identifier Semantics

`pid` is an unsigned 64-bit value scoped to an index OID. `0` is invalid. PIDs
are allocated monotonically by the root/control metadata and are not reused
while any retained epoch can reference them. Rebalancing keeps the PID and
writes a new object version at a new placement. Split/merge that changes a
partition's semantic coverage allocates new child or replacement PIDs instead
of silently changing the meaning of an existing PID for active strict-epoch
queries.

`parent_pid` and child PIDs refer to PIDs, not heap rows or PostgreSQL table
partitions. A root object uses `parent_pid = 0`.

`vec_id` is the dedupe and remote-merge identity. It is unique within an index
OID for live logical vector versions and is encoded in at most 32 bytes,
including the discriminator byte.

Phase 1 local IDs use:

```text
0x01 || local_vec_seq:u64
```

`local_vec_seq` is allocated from `next_local_vec_seq` in root/control metadata.
It is not derived from heap TID, because heap TID reuse and HOT/non-HOT update
behavior should not define dedupe identity. A non-HOT update is delete-old plus
insert-new and receives a new local `vec_id`.

Future global IDs reserve:

```text
0x02 || global_id_bytes
```

The local-to-global transition is an epoch rewrite: build replacement leaf or
delta objects with global `vec_id`s, publish a new epoch manifest, retain old
local-ID epochs until the normal retention horizon passes, and then clean the
local-ID objects. Mixed local/global IDs can coexist only during the retained
epoch window.

## Heap TID, HOT, UPDATE, and Vacuum

The stored heap TID is a local row locator, not vector identity.

For HOT updates where PostgreSQL does not call the index AM, the stored root
line pointer remains valid under PostgreSQL's normal index contract and the
executor follows the HOT chain. SPIRE does not rewrite assignment rows for HOT
movement.

For non-HOT updates, PostgreSQL inserts a new index entry. SPIRE treats that as
delete-old plus insert-new: the new row gets a new local `vec_id`, and the old
row remains a candidate only until heap visibility or vacuum removes it.

For deletes, heap visibility removes the row from query results before SPIRE
physically cleans the assignment. Vacuum then marks matching assignment rows as
`TOMBSTONE` or writes a `DELTA_DELETE` object and publishes a cleanup epoch.
Compaction later rewrites a replacement leaf object without the tombstoned
rows.

SPIRE must never emit an unrelated heap row for a stale locator. If update or
vacuum detects an invalid or stale locator, it either publishes an epoch-safe
repair, tombstones the assignment, or suppresses the candidate with diagnostics.

## Epoch Layout and Publication

Phase 1 uses per-partition object versions referenced by epoch manifests, not
full `(pid, epoch)` object duplication. Unchanged objects can be shared by
successive epochs.

```text
epoch_manifest
  epoch bigint
  state building | published | retired | failed
  consistency_mode strict | degraded
  published_at timestamptz
  retain_until timestamptz
  active_query_count bigint

manifest_entry
  epoch bigint
  pid bigint
  object_version bigint
  placement_entry_locator
```

Publication order:

1. Write replacement or delta partition objects.
2. Write placement entries for every object the epoch needs.
3. Write the epoch manifest in `building` state.
4. Validate that required placements are durable and epoch-compatible.
5. Mark the manifest `published`.
6. Atomically advance root/control `active_epoch`.

If any step fails before the active epoch advances, the manifest remains
`building` or `failed` and the old active epoch remains authoritative. Failed
objects are diagnosable and cleanup-eligible; they are never read through the
active manifest.

Defaults:

```text
min_epoch_retention = 10 minutes
max_retained_epochs = 2 published/retired epochs, plus active
failed_epoch_retention = 60 minutes unless an operator cleans earlier
```

Cleanup may remove retired epoch manifests and unreferenced objects only after
the minimum wall-clock retention has passed and no backend reports that epoch
as active. Failed or abandoned builds may be cleaned after their failed
retention window and after no rewrite job owns them.

Local single-store defaults to strict consistency. Degraded mode is configurable
for local multi-store and remote deployments. Replicated partition objects are
future work and are not part of Phase 1.

## Insert and Delete Lifecycle

Phase 1 chooses replacement epochs with immutable base and delta objects.

Inserts append a `DELTA_INSERT` object for the target PID and publish an epoch
that references the previous base object plus the delta. Deletes are enforced
first by heap visibility; vacuum later writes tombstone/delete deltas and
publishes a cleanup epoch. Compaction squashes base plus deltas into a new leaf
object version.

This deliberately rejects in-place mutable published partition objects as the
source of truth. It keeps strict-mode scans simple: a scan chooses one epoch at
start and reads exactly the objects referenced by that epoch's manifest.

## Reuse of Landed `ec_ivf`

Reuse after extraction or small visibility changes:

- Spherical k-means training and assignment from `src/am/ec_ivf/training.rs`.
  Move the AM-neutral pieces to `src/am/common` with an AM label for error
  strings.
- Quantizer profile selection, encoding, prepared-query creation, and scoring
  from `src/am/ec_ivf/quantizer.rs`. Keep reloption parsing AM-owned, but move
  the neutral payload/profile helpers to a common IVF/partition-scoring module.
- Grouped PQ training in `src/am/common/training.rs`, already shared.
- Candidate top-k, centroid scoring, and heap-f32 rerank patterns from
  `src/am/ec_ivf/scan.rs`, factored before reuse rather than imported through
  `ec_ivf` private modules.
- Admin snapshot and EXPLAIN counter conventions, extended with SPIRE epoch,
  placement, object, and stale-locator diagnostics.

Do not reuse directly:

- `ec_ivf` metadata, centroid directory, and posting-list page format.
- `ec_ivf` list-id-based append and vacuum mutation logic.
- `ec_ivf` heap-TID dedupe key, because SPIRE dedupes by stable `vec_id`.

Existing `ec_ivf` indexes keep the ADR-048 format. SPIRE gets a new
partition-object format under `src/am/ec_spire/`. Any later `ec_ivf` adoption
of partition objects would require a separate ADR and format bump.

## Phase 1 SQL Surface

Phase 1 should expose `ec_spire` as an opt-in access method once the
single-level local path is executable. The documented opclasses are:

```sql
ecvector_spire_ip_ops
tqvector_spire_ip_ops
```

The names are AM-specific even though PostgreSQL namespaces opclasses by access
method. That keeps the experimental SPIRE surface visibly distinct from the
existing HNSW/IVF operator-class names.
