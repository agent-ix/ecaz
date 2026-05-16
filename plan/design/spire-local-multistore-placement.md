# SPIRE Local Multi-Store Placement Design

Status: Phase 4 design checkpoint for Task 30
Date: 2026-05-06
Scope: local-node multi-store placement for SPIRE partition objects

This note defines the local multi-store placement contract before the
implementation moves beyond the current single relation-backed local store. It
is subordinate to ADR-049, the Phase 0 partition-object storage design, the
Phase 2 update mechanics plan, and the Phase 3 recursive hierarchy design.

## Goals

Phase 4 keeps SPIRE on one PostgreSQL node while allowing partition-object
bytes to live in a bounded set of local store relations. Each store relation can
be placed in a PostgreSQL tablespace that the operator maps to a physical NVMe
device.

The durable addressing shape remains:

```text
pid -> local_store_id -> object location
```

The root/control index relation remains authoritative for active epoch,
allocator cursors, active store configuration, manifests, and placement
directories. PostgreSQL declarative table partitions are not part of SPIRE
vector partition selection or object placement.

Phase 4 must preserve existing single-store indexes and must not make
multi-NVMe performance claims until benchmark evidence exists on a machine with
multiple physical NVMe devices.

## Configuration Surface

The first user-visible configuration should be relation-local and fixed for an
index build:

```text
local_store_count int             -- default 1, bounded by a small max
local_store_tablespaces text      -- optional comma-separated tablespace names
```

`local_store_count = 1` keeps the current embedded single-store behavior unless
the implementation later adds an explicit opt-in to a dedicated single store
relation. `local_store_count > 1` creates exactly `local_store_count` dedicated
partition-store relations, with store IDs `0..local_store_count - 1`.

The count must be validated at option parse time. A count of zero is invalid,
and the maximum should stay small enough that every scan can open and account
for all local stores without unbounded relation churn. The first implementation
should use a conservative maximum such as 16.

If `local_store_tablespaces` is present, it must name either:

- exactly `local_store_count` tablespaces, one per store ID; or
- no values, meaning every store inherits the index tablespace for functional
  testing only.

Repeated tablespace names are allowed because development and CI often lack
multiple devices, but diagnostics must report the actual tablespace OID/name so
operators do not mistake repeated tablespaces for physical striping.

## Store Relations

For legacy and default single-store indexes, `local_store_id = 0` continues to
be the root/control index relation. The root/control page and object-store pages
already have separate logical roles, and existing placement entries point at
the index relation's relid.

For multi-store indexes, every local store is a dedicated AM-owned store
relation. The root/control relation stores metadata only; partition-object
bytes are appended to the store relations. Store relation names are
implementation details, not durable identity, but should be deterministic for
operators, for example:

```text
ec_spire_store_<index_oid>_<store_id>
```

The durable identity is the relation OID recorded in root/control store
configuration and copied into placement entries. Store relations must have an
internal dependency on the root/control index so drop/cleanup follows normal
PostgreSQL dependency rules.

Each store relation uses the same object-tuple page format as the current
relation-backed object store:

- one tuple for V1 routing or delta object bytes;
- one V2 leaf metadata tuple plus a segment-tuple chain for V2 leaf bytes;
- object tuple TIDs remain relation-local and are meaningful only with the
  placement entry's `store_relid`.

The object-store API should become store-descriptor based:

```text
SpireLocalStoreDescriptor
  local_store_id
  store_relid
  tablespace_oid
  state
```

Readers must open the relation named by the descriptor or placement entry
before interpreting `object_tid`.

## Root/Control Store Set

Root/control metadata needs a versioned active store set. Existing V1
root/control state implies:

```text
local_store_count = 1
store 0 = root/control index relation
tablespace = index tablespace
```

The multi-store implementation should add a root/control V2 state, or an
equivalent root/control-referenced metadata tuple, that records:

```text
local_store_config
  generation
  store_count
  repeated store descriptor:
    local_store_id
    store_relid
    tablespace_oid
    state
```

`generation` changes only when the active store set changes. A published epoch
references one generation, and its placement directory must use only store IDs
from that generation. Existing single-store indexes decode as generation 0 with
one embedded store.

The first Phase 4 implementation should treat store count changes as a rebuild
or offline rewrite boundary. It should not try to migrate active partition
objects between store sets in place.

## Placement Entries

The current placement entry shape is already the right logical surface:

```text
placement_entry
  epoch
  pid
  node_id = 0
  local_store_id
  store_relid
  object_version
  object_tid
  object_bytes
  state
```

For multi-store indexes, `local_store_id` and `store_relid` are no longer
constant. Validation must reject a placement whose store ID is not present in
the active store set, whose relid does not match the active descriptor for that
store ID, or whose object bytes cannot be read from that relation.

Single-store helper constructors can remain as compatibility helpers, but new
write paths should construct placements from the selected store descriptor.

## Hash Placement

Leaf and routing objects are assigned to stores by a stable PID hash:

```text
local_store_id = spire_pid_hash(pid) % local_store_count
```

`spire_pid_hash` must be a fixed algorithm owned by SPIRE, not Rust's default
hasher. A SplitMix64-style finalizer over the little-endian PID is sufficient
and deterministic across platforms.

The configured `local_store_count` is fixed for a built index. Changing the
count changes the modulo home for existing object PIDs, so it requires REINDEX
or a future explicit object rewrite/rebalance path.

Multi-store REINDEX is also an explicit implementation boundary: auxiliary
store relations need a dedicated rebuild lifecycle that creates the new store
set, publishes matching descriptors, and retires the old auxiliary relations.
Internal catalog dependencies alone are not the reindex contract, so Phase 4
rejects multi-store REINDEX explicitly until that lifecycle lands.

The root routing object is still a partition object and follows the same hash
rule when a multi-store index uses dedicated store relations. Root/control
metadata, epoch manifests, object manifests, placement directories, and store
configuration stay in the root/control relation and are not hash-placed.

Delta objects should be colocated with their parent leaf's active store when
the parent placement is available. This keeps insert/delete overlay reads close
to base leaf bytes. If a future delta is not tied to an available parent leaf,
it must fail strict publication rather than silently choosing a different store.

Split and merge replacement leaves use the replacement PID hash. A
PID-preserving rebalance may write a new object version to the same hashed
store. Moving a PID to a different store is a store-set migration and requires
an explicit rewrite plan, not an incidental publish side effect.

## Open and Lock Ordering

Multi-store writes keep the existing publish lock as the serialization point
for epoch publication and allocator cursor advancement.

The write path should use this order:

1. Open the root/control relation and take the publish lock.
2. Load root/control state and active store descriptors.
3. Open all target store relations in ascending `local_store_id`.
4. Write object tuples to their selected stores.
5. Write placement entries and manifests to root/control.
6. Validate the full epoch against the active store set.
7. Advance root/control `active_epoch`.

Readers should open the root/control relation first, load the active snapshot,
then open required store relations in ascending `local_store_id`. This gives
scans, diagnostics, insert, vacuum, and maintenance one lock ordering rule.

Store relation creation or replacement is DDL-like and must happen outside
normal scan paths. It should take an exclusive root/control operation boundary
and publish a new store generation only after every store relation exists.

## Publish Atomicity and Failure Semantics

Partition objects remain immutable once published. A failed write may leave
unreferenced object tuples in a store relation, but it must not advance
root/control and must not publish placement entries that the active manifest
can use.

Strict mode:

- every placement required by the active epoch must be `Available`;
- every placement's store descriptor must be present and readable;
- publication fails if any target store cannot be opened or written;
- scans fail closed on stale or unavailable required placements.

Degraded mode:

- unavailable or skipped placements may be omitted from scan results only when
  the epoch was explicitly published as degraded;
- stale placements are never readable;
- diagnostics must expose which store and PID were skipped.

If one local store is unavailable, the root/control relation must remain
readable so diagnostics can describe the degraded state. New strict publishes
must fail until the store returns or a deliberate rebuild publishes a new store
generation and placement directory.

## Store-Grouped Fetch

Scan routing still selects PIDs from root and internal routing objects. After
selected leaf and delta PIDs are known, the scan path should group placement
lookups by `(node_id, local_store_id)` and fetch object bytes store by store.

The grouping contract is:

```text
selected PIDs
  -> validated placements
  -> store groups
  -> store-local object reads
  -> store-local candidate scoring
  -> global rerank merge
```

Candidate scoring should stay close to the bytes read from each store group.
The first implementation may execute store groups synchronously inside one
backend while preserving this grouping boundary. ADR-057 accepts PostgreSQL
relation prefetch/read-stream as the Phase 10 overlap primitive and keeps
object decoding plus scoring sequential inside that backend. This means
`local_store_count > 1` can make placement explicit before it improves CPU or
store-group throughput. Any claim that the runtime performs or benefits from
parallel multi-NVMe reads must wait for a benchmark packet that compares
one-store and multi-store layouts on real multi-NVMe hardware.
`ec_spire_index_scan_local_store_execution_snapshot(index_oid, query)` exposes
this limitation with `local_store_execution_mode = 'sequential_backend'` and
reports the exact future primitive as
`local_store_parallelism_next_step = 'async_or_parallel_store_group_executor'`.
`ec_spire_index_scan_local_store_read_overlap_harness(index_oid, query)`
provides the repeatable per-store harness for this boundary. It reports route
counts, candidate rows, prefetched object bytes, read-batch count, and
delta-decode count for each touched `(node_id, local_store_id)` so benchmark
packets can distinguish store-grouped sequential reads from future true
overlap.

Delta object decoding is shared across local multi-store scans and remote
candidate endpoints. The selected-leaf candidate collector loads each selected
delta route once with `load_delta_rows_for_routes`, then reuses the loaded rows
for delete suppression and delta-insert candidate scoring. Remote candidate and
tuple-payload endpoints call the same selected-leaf collector before origin-node
heap or tuple payload resolution, so they inherit the same decoded-delta reuse
instead of adding a second delta-object read in the remote handoff path.
The `(node_id, local_store_id)` grouping key above explains why the collector
groups by selected route set before the per-store object read and scoring pass.

## Diagnostics

The authoritative placement diagnostics are root/control store configuration
plus the active placement directory.

`ec_spire_index_placement_snapshot(index_oid)` should grow from the current
single-store aggregate into one row per configured local store, including
zero-object stores. It should report:

- store ID, store relid, tablespace OID/name, and store state;
- active placement count by placement state;
- object count and bytes by object kind;
- assignment count and routing-child count;
- whether the store relation was readable during diagnostics.

`ec_spire_index_scan_placement_snapshot(index_oid, query)` should report one
row per scan-touched store, including:

- routed leaf PID count;
- delta PID count;
- scanned PID count after degraded skips;
- visible candidate row count;
- object bytes read or planned for the query;
- read-batch and delta-decode counters for the query, via
  `ec_spire_index_scan_local_store_read_overlap_harness(index_oid, query)`;
- skipped PID count and placement-state labels.
- local-store execution mode, read-ahead primitive, and the next parallelism
  primitive needed when execution remains sequential inside one backend, via
  `ec_spire_index_scan_local_store_execution_snapshot(index_oid, query)`.

The diagnostics should keep saying "local store" rather than "NVMe" unless
they are reporting actual tablespace identity. Physical device claims belong in
benchmark packets, not in static diagnostics.

## Benchmarks

The required measurement packet for Phase 4 is a local placement benchmark that
compares one-store and multi-store behavior on a host with multiple physical
NVMe devices. The packet must record:

- head SHA and store configuration;
- store relation tablespaces and their physical device mapping;
- corpus, query count, storage format, recursive fanout, nprobe, rerank mode;
- build time, storage bytes, scan latency, recall, and diagnostic store rows;
- whether fetch execution was synchronous store-grouped or actually parallel.

Until that packet exists, the project may claim only that SPIRE has a
multi-store placement design and implementation surface, not that it improves
latency or throughput.

## Deferred

The following work stays outside this design checkpoint:

- code changes for local store reloptions and metadata codecs;
- auxiliary store relation creation/open helpers;
- moving object writes and reads off the root/control relation;
- true parallel execution across store groups;
- online store-count changes or object migration between stores;
- old object tuple reclamation in auxiliary store relations;
- remote node placement and boundary replication.

## Review Checkpoint

The matching review packet is
`review/30509-spire-phase4-local-placement-design/`. It asks for review of the
local multi-store storage contract before metadata or relation-helper code
lands.
