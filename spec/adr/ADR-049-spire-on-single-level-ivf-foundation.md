---
id: ADR-049
title: "Build SPIRE on Top of a Single-Level IVF Foundation"
status: PROPOSED
impact: Affects ADR-035, ADR-048, future SPIRE planning
date: 2026-05-01
deciders:
  - TBD
---
# ADR-049: Build SPIRE on Top of a Single-Level IVF Foundation

## Status

Proposed

## Context

We need a billion-scale ANN index inside the Postgres extension and we are
building SPIRE to provide it. This ADR is about how we build SPIRE:
specifically the staging of work and the one schema choice we need to make now
to keep SPIRE's full structure reachable without committing to its full
complexity up front.

Relevant structural facts about SPIRE:

- SPIRE is IVF-shaped at every level. Leaf partitions are clustered posting
  lists, queries fan out to the top-`nprobe` clusters at each level, and
  updates use cluster-level split-and-merge using the LIRE/SPFresh pattern.
- SPIRE adds two things on top of plain IVF: recursive multi-level hierarchy
  and boundary replication. Centroids of one level become inputs to the level
  above; the paper uses 4 levels at 2B vectors and 6 levels at 1T. Boundary
  vectors near cluster boundaries are replicated across multiple partitions,
  preserving accuracy across the hierarchy.
- The top level is a proximity graph, HNSW or DiskANN over the top-level
  centroids, not a flat IVF scan. Once recursion compresses the dataset down to
  a few million top centroids, that set fits on one machine and a graph index
  gives log-scaling for top-level lookup.
- Inside each leaf partition, SPIRE does not mandate a structure. The paper
  effectively flat-scans, relying on the balanced-granularity finding that
  keeps leaf partitions small.

The components shared between plain IVF and SPIRE are substantial: k-means
centroid training, PQ codebook training and encoding, vector-to-centroid
assignment, posting-list storage, candidate scoring and rerank, and cluster
split-and-merge for updates. SPIRE's contributions sit above this layer
(recursion, top-level graph) and adjacent to it (boundary replication in the
assignment step).

## Decision

### 1. Build single-level IVF first; layer SPIRE on top as a second phase

Every foundational component listed above is shared unchanged. Building IVF
first means we ship a working, debuggable index before stacking hierarchy on
top of it. If a recursive SPIRE is broken at level N, we cannot easily tell
whether the bug is in the leaf-level primitive or in the recursion logic;
building IVF first gives us a known-good inner loop.

### 2. Store partition assignments as `(vec_id, partition_id)` rows

Store partition assignments in a separate table, not as a `partition_id` column
on the vectors table.

This is the one design choice we lock in now, before any SPIRE code is written.
The reason: standard IVF assigns each vector to exactly one partition, but
SPIRE's boundary replication assigns boundary vectors to multiple partitions.
With a column on the vectors table, adding boundary replication later requires a
schema migration. With a separate `(vec_id, partition_id)` table, initial IVF
writes one row per vector and SPIRE's boundary replication writes multiple rows
per vector. The schema does not change.

We accept the slightly higher per-row overhead of the join-table layout in
exchange for the flexibility.

### 3. Keep SPIRE modular inside one Postgres extension

SPIRE will be a single Postgres extension with cleanly modular internal
structure.

We will not build pluggable abstractions for hypothetical alternative index
strategies. Instead, we will factor SPIRE's internal components, including
codebook training, libpq pipeline pool, background worker infrastructure,
CustomScan integration glue, and progress/checkpoint helpers, as modules with
clean boundaries. This is code hygiene that keeps SPIRE maintainable and
testable; it is not architecture in service of swappable strategies.

### 4. Build the SPIRE layer as additions, not replacements

When we add SPIRE on top of working IVF:

- Recursion is a build-coordinator concern: run IVF on the input vectors, take
  the resulting centroids, run IVF on those, and repeat to depth. This is
  orchestration around the existing IVF primitive, not a rewrite of it.
- The top-level graph is a separate, smaller code path: stock HNSW or DiskANN
  over the top-level centroids. It is additional code, not replacement code.
- Boundary replication modifies the assignment step only: for each vector,
  "find nearest centroid" becomes "find nearest centroids and write a row for
  each boundary partition." The schema from Decision 2 absorbs the change.
- Multi-level query routing is new: fan out at each level, descend into
  children, and repeat to leaves. It is implemented above the IVF query
  primitive, not inside it.

## Consequences

### Positive

- We ship a working IVF-based system before SPIRE complexity lands. This lowers
  risk, speeds validation, and puts real query traffic on the inner loop before
  we trust it as SPIRE's foundation.
- IVF gives us debuggable ground truth. A SPIRE bug at any level can be
  isolated by testing the same code as a flat IVF at level 1.
- The schema decision costs little now and avoids a migration later.

### Negative

- Two-phase delivery means billion-scale capability arrives later than a
  SPIRE-first path would deliver it. We are betting the IVF phase is short
  enough to justify this.
- Some assumptions baked into single-level IVF will need to be revisited when
  SPIRE arrives. The most likely one, partition assignment cardinality, is
  mitigated by Decision 2. Others may surface during SPIRE bring-up; we accept
  this risk.
- The `(vec_id, partition_id)` join-table layout is slightly more expensive
  than a column on the vectors table. This is acceptable given the migration
  cost it avoids.

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
join table.

## Implementation Phases

### Phase 1: Single-level IVF

- k-means centroid training using a mini-batch sample, parallelizable
- PQ codebook training and encoding
- vector-to-centroid assignment, one partition per vector initially
- posting-list storage with `(vec_id, partition_id)` schema
- candidate scoring and rerank path
- cluster split-and-merge for updates using the LIRE/SPFresh pattern

### Phase 2: SPIRE Layer

- recursive build coordinator: IVF-on-centroids, repeated to target depth
- top-level proximity graph: HNSW or DiskANN over top centroids
- boundary replication in the assignment step
- multi-level query routing: fan out per level, descend to leaves
- hierarchy metadata and level-aware update propagation

## References

- SPIRE paper
- SPFresh / LIRE: split-and-merge update mechanics, applicable across both
  phases
- SPANN: small-flat-clusters-as-posting-lists pattern, reference for leaf-level
  layout
- IP-DiskANN (Xu, Manohar et al., February 2025): in-place updates for graph
  indexes; useful background on the broader ANN-update problem space
