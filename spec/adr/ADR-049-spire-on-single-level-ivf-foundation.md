---
id: ADR-049
title: "Build SPIRE on a Partition-Object IVF Foundation"
status: PROPOSED
impact: Affects ADR-048, future SPIRE planning, distributed storage planning; supersedes the dropped SPANN direction
date: 2026-05-01
deciders:
  - TBD
phase0_design: plan/design/spire-phase0-partition-object-storage.md
phase1_architecture_gate: plan/design/spire-foundation-architecture-feedback-response.md
---
# ADR-049: Build SPIRE on a Partition-Object IVF Foundation

## Status

Proposed

## Context

We need a billion-scale ANN index inside the Postgres extension and we are
building SPIRE to provide it. This ADR is about how we build SPIRE:
specifically the staging of work and the storage shape we need to choose now
to keep SPIRE's full structure reachable without committing to its full
distributed complexity up front.

Relevant structural facts about SPIRE:

- SPIRE is IVF-shaped at every level. Leaf partitions are clustered posting
  lists, queries fan out to the top-`m` or top-`nprobe` clusters at each level,
  and updates use cluster-level split-and-merge using the LIRE/SPFresh pattern.
- SPIRE adds two things on top of plain IVF: recursive multi-level hierarchy
  and boundary replication. Centroids of one level become inputs to the level
  above; the paper uses 4 levels at 2B vectors and 6 levels at 1T. Boundary
  vectors near cluster boundaries are replicated across multiple partitions,
  preserving accuracy across the hierarchy.
- The top level is a proximity graph, HNSW or DiskANN over the top-level
  centroids, not a flat IVF scan. Once recursion compresses the dataset down to
  a few million top centroids, that set fits on one machine and a graph index
  gives log-scaling for top-level lookup.
- Lower levels are stored as independent partition objects addressed by PIDs.
  The SPIRE paper places those objects by hashing PID across storage nodes. A
  single-node implementation should preserve the same object/placement model so
  it can first stripe objects across local NVMe devices and later extend the
  placement map across physical machines.
- Inside each leaf partition, SPIRE does not mandate a structure. The paper
  effectively flat-scans, relying on the balanced-granularity finding that
  keeps leaf partitions small.

The components shared between plain IVF and SPIRE are substantial: k-means
centroid training, PQ codebook training and encoding, vector-to-centroid
assignment, posting-list storage, candidate scoring and rerank, and cluster
split-and-merge for updates. SPIRE's contributions sit above this layer
(recursion, top-level graph) and adjacent to it (boundary replication in the
assignment step). The Postgres implementation must also separate algorithmic
partitioning from physical placement:

- A **SPIRE partition** is an index-internal cluster object addressed by PID. It
  is not a PostgreSQL table partition.
- A **partition store** is a bounded physical container for many SPIRE partition
  objects. On one instance, stores can be placed in different tablespaces backed
  by separate NVMe devices. In a distributed deployment, the same PID placement
  map can route to remote nodes.

## Decision

### 1. Build single-level IVF first; layer SPIRE on top as a second phase

Every foundational component listed above is shared unchanged. Building IVF
first means we ship a working, debuggable index before stacking hierarchy on
top of it. If a recursive SPIRE is broken at level N, we cannot easily tell
whether the bug is in the leaf-level primitive or in the recursion logic;
building IVF first gives us a known-good inner loop.

### 2. Store vector membership as logical `(vec_id, pid)` rows inside partition objects

Store vector membership as assignment/posting rows inside SPIRE partition
objects, not as a `partition_id` column on the vectors table.

This is one of the design choices we lock in now, before any SPIRE persistence
code is written. The reason: standard IVF assigns each vector to exactly one
cluster, but SPIRE's boundary replication assigns boundary vectors to multiple
nearby clusters. With a column on the vectors table, adding boundary
replication later requires a schema migration and makes index-private state
user-visible. With logical `(vec_id, pid)` rows, initial IVF writes one row per
vector and SPIRE's boundary replication writes multiple rows per vector. The
logical schema does not change.

The row is logical, not necessarily a user-visible heap table. The first local
implementation should persist rows in AM-owned partition objects. Diagnostics
should expose read-only SQL views/functions over those objects rather than
allowing direct user DML against index internals.

Leaf assignment/posting rows must carry enough identity to support the local
and distributed path:

- `vec_id`: stable vector identity used for deduplication and remote result
  merge.
- local heap TID or row locator: required for local PostgreSQL result emission.
- `pid`: owning SPIRE partition object ID.
- encoded payload and scoring metadata.
- flags such as primary assignment, boundary replica, tombstone, or delta row.

For the first local implementation, `vec_id` is not derived from or mirrored
from the heap TID. Phase 0 chooses an index-local monotonically allocated ID
encoded as `0x01 || local_vec_seq:u64`. The heap TID remains a local row
locator only. `vec_id` must be unique within an index OID for live logical
vector versions, encoded in no more than 32 bytes, and reserve a discriminator
byte so a local ID can coexist with or be rewritten into a future global ID
through an epoch transition.

The first architecture review adds a pre-persistence gate: persisted base leaf
objects must use a segmented, column-major `LeafPartitionObjectV2` rather than
one row-contiguous tuple. The logical row remains `(vec_id, pid)`, but the
physical base-leaf payload is stored as fixed-stride `vec_id`, heap-TID, gamma,
flag, and encoded-payload columns split across page-sized row segments. Small
delta objects can remain row-encoded until compaction rewrites them into a V2
base leaf.

### 3. Use partition objects and a placement map, not one monolithic relation forever

SPIRE persistence is organized around PostgreSQL-managed relation-backed
partition objects:

```text
(pid, object_version) -> partition object bytes
```

Internal partition objects store routing metadata and child PIDs. Leaf partition
objects store vector assignment/posting rows. Root/control metadata stores the
top graph, hierarchy metadata, active epoch, PID allocation state, local
`vec_id` allocation state, and PID placement map.

The single-node implementation may start with one partition store, but the
format must model physical placement explicitly:

```text
pid -> local_store_id -> object location
```

Phase 1 uses the `ec_spire` index relation as the root/control relation and the
single `local_store_id = 0` object store. Local multi-NVMe operation extends
that shape to a bounded number of partition-store relations, each placed in a
tablespace backed by a different NVMe device:

```text
store_id = hash(pid) % local_store_count
```

The later distributed shape extends the same map:

```text
pid -> node_id -> local_store_id -> object location
```

We will not create one PostgreSQL relation per SPIRE partition; that would push
SPIRE's partition count into `pg_class` and make catalog overhead the dominant
storage problem. We also will not use PostgreSQL declarative table partitions
for SPIRE partition selection. PostgreSQL's planner chooses whether to use the
SPIRE access path; SPIRE itself chooses PIDs from the query vector and hierarchy.

The placement map addresses one logical partition object by PID and object
version. A large V2 leaf object may physically span multiple object-store
tuples; the placement entry points at the V2 metadata tuple, and that metadata
tuple owns the segment chain. In strict mode, the object is readable only when
the metadata tuple and all referenced segment tuples are available.

### 4. Version partition objects with published epochs

SPIRE must be able to serve a query against a consistent set of root metadata,
hierarchy metadata, placement metadata, and partition objects. Local PostgreSQL
MVCC handles local heap visibility, but it does not coordinate remote machines
or independently rewritten partition objects.

Each query should choose an active SPIRE epoch at start. Reads then target
object versions compatible with that epoch:

```text
active_epoch = 42
manifest[42] maps pid -> object_version -> placement
```

Phase 0 chooses immutable per-partition object versions referenced by an epoch
manifest, not full `(pid, epoch)` object duplication. Writers prepare
replacement or delta objects, then atomically publish a new root/control epoch
after all required objects are present. Old epochs remain readable until the
retention window passes and in-flight queries finish. The initial defaults are a
10 minute minimum retention window, current plus two published/retired epochs,
and cleanup only when no backend reports the old epoch.

### 5. Keep SPIRE modular inside one Postgres extension

SPIRE will be a single Postgres extension with cleanly modular internal
structure.

We will not build pluggable abstractions for hypothetical alternative index
strategies. Instead, we will factor SPIRE's internal components, including
codebook training, partition-object storage, PID placement, epoch publication,
libpq pipeline pool, background worker infrastructure, CustomScan integration
glue, and progress/checkpoint helpers, as modules with clean boundaries. This
is code hygiene that keeps SPIRE maintainable and testable; it is not
architecture in service of swappable strategies.

### 6. Build the SPIRE layer as additions, not replacements

When we add SPIRE on top of working IVF:

- Recursion is a build-coordinator concern: run IVF on the input vectors, take
  the resulting centroids, run IVF on those, and repeat to depth. This is
  orchestration around the existing IVF primitive, not a rewrite of it.
- The top-level graph is a separate, smaller code path: stock HNSW or DiskANN
  over the top-level centroids. It is additional code, not replacement code.
- Boundary replication modifies the assignment step only: for each vector,
  "find nearest centroid" becomes "find nearest centroids and write a membership
  row into each selected leaf PID." The object row shape from Decision 2 absorbs
  the change.
- Multi-level query routing is new: fan out at each level, descend into
  children, and repeat to leaves. It is implemented above the IVF query
  primitive, not inside it.
- Remote execution is a later coordinator concern: route selected PIDs to
  storage nodes, ask each node to score local partition objects near data, and
  merge compact candidate results. The first version should use PostgreSQL's
  existing wire protocol through libpq/pipeline mode before considering a custom
  network protocol.

Phase 1 should expose the single-level local foundation as an opt-in
`ec_spire` access method once executable. The planned operator classes are
`ecvector_spire_ip_ops` and `tqvector_spire_ip_ops`.

## Consequences

### Positive

- We ship a working IVF-based system before SPIRE complexity lands. This lowers
  risk, speeds validation, and puts real query traffic on the inner loop before
  we trust it as SPIRE's foundation.
- IVF gives us debuggable ground truth. A SPIRE bug at any level can be
  isolated by testing the same code as a flat IVF at level 1.
- The partition-object decision costs little now and avoids rewriting the
  storage model when local multi-NVMe or remote-node placement arrives.
- Epoch publication gives a concrete consistency model for partition rewrites,
  split/merge, and later remote serving.

### Negative

- Two-phase delivery means billion-scale capability arrives later than a
  SPIRE-first path would deliver it. We are betting the IVF phase is short
  enough to justify this.
- Some assumptions baked into single-level IVF will need to be revisited when
  SPIRE arrives. The most likely one, partition assignment cardinality, is
  mitigated by Decision 2. Others may surface during SPIRE bring-up; we accept
  this risk.
- The `(vec_id, pid)` row layout is slightly more expensive than a column on the
  vectors table. This is acceptable given the migration cost it avoids.
- Partition-object placement adds root/control metadata and object lifecycle
  work before a fully distributed implementation exists.
- Epoch/version management adds complexity to update, split, merge, and cleanup
  paths. The first executable path should start with offline-built immutable
  epochs, then add epoch-published insert/delete delta objects before broader
  split/merge mechanics.

## Alternatives Considered

### Build SPIRE end-to-end from the start

Rejected. Recursion and hierarchy management before a validated single-level
index means debugging multiple layers of complexity simultaneously. The risk of
having no working index for an extended period outweighs the modest duplicated
effort of staging.

### Build a pluggable index-strategy abstraction inside the extension

Rejected. We have one index strategy to ship. Designing an abstraction in
advance of a second concrete user produces an interface shaped by guesses
rather than requirements, and burdens SPIRE's implementation with conformance to
a contract no one is enforcing. Modular internal code gives us maintainability
without speculative ceremony.

### Use a `partition_id` column on the vectors table

Rejected. It is cheaper now, but makes boundary replication a schema migration
later. The future cost of the migration is larger than the present cost of the
logical assignment row.

### Store all SPIRE objects in one PostgreSQL index relation forever

Rejected as the target architecture. One relation is acceptable as a local
prototype or single-store configuration, but it hides the placement unit and
does not preserve a direct path to striping partition objects across local NVMe
devices or routing them across machines.

### Use PostgreSQL declarative table partitions for SPIRE partitions

Rejected. PostgreSQL table partition pruning is driven by SQL predicates and
constraints. SPIRE partition selection is driven by query-vector routing through
learned centroids and a top-level graph, so it belongs inside the SPIRE
index/coordinator.

## Implementation Phases

### Phase 1: Single-level partition-object IVF

- pre-persistence hardening from the architecture gate: V2 segmented leaf
  objects, borrowed leaf reads, validated snapshot PID caches, flat routing
  centroid arrays, bounded heaps, explicit dedupe mode, and a publish
  coordinator
- k-means centroid training using a mini-batch sample, parallelizable
- PQ codebook training and encoding
- vector-to-centroid assignment, one partition per vector initially
- partition-object storage with logical `(vec_id, pid)` assignment rows
- candidate scoring and rerank path
- cluster split-and-merge for updates using the LIRE/SPFresh pattern
- one local partition store first, then local multi-store placement by `hash(pid)`

### Phase 2: SPIRE Layer

- recursive build coordinator: IVF-on-centroids, repeated to target depth
- top-level proximity graph: HNSW or DiskANN over top centroids
- boundary replication in the assignment step
- multi-level query routing: fan out per level, descend to leaves
- hierarchy metadata and level-aware update propagation
- epoch publication and old-epoch retention for in-flight queries

### Phase 3: Multi-Store and Distributed Placement

- local multi-NVMe partition stores through tablespace-backed store relations
- PID placement diagnostics and rebalancing policy
- coordinator-to-remote-node search over libpq/pipeline mode
- remote-node partition search functions returning compact candidates
- distributed epoch manifest and stale-node handling

## References

- SPIRE paper
- SPFresh / LIRE: split-and-merge update mechanics, applicable across both
  phases
- SPANN: small-flat-clusters-as-posting-lists pattern, reference for leaf-level
  layout
- IP-DiskANN (Xu, Manohar et al., February 2025): in-place updates for graph
  indexes; useful background on the broader ANN-update problem space
