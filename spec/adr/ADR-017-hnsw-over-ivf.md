---
id: ADR-017
title: "HNSW over IVF: Topology-Agnostic Indexing for Heterogeneous Data Shapes"
status: PROPOSED
impact: Affects FR-008, FR-009, FR-016, FR-021, FR-022, StR-003
date: 2026-04-06
---
# ADR-017: HNSW over IVF — Topology-Agnostic Indexing for Heterogeneous Data Shapes

## Context

tqvector serves multiple use cases with fundamentally different data shapes:

1. **Agent memories** — per-agent embedding collections, semantically clustered within each agent's topic space. Partitioned by `HASH(agent_id)` into 16 partitions, each containing ~6,250 agents' memories.
2. **Knowledge graphs** — entity embeddings spanning many semantic domains with no clean cluster boundaries. Multi-modal, cross-domain, and evolving over time.

The two dominant ANN index structures are:

**HNSW (Hierarchical Navigable Small Worlds)**:
- Builds a navigable multi-layer graph with bidirectional edges
- Topology-agnostic — graph connectivity adapts to local data distribution
- O(m log n) insert with edge updates to m neighbors
- WAL write amplification: ~9 page writes per insert (bidirectional edges)
- All neighbors must be in memory (or buffer cache) for traversal

**IVF (Inverted File Index)**:
- Partitions data into k clusters via k-means training
- Assigns each vector to its nearest centroid, stores in posting list
- O(1) insert — append to posting list
- Minimal WAL: 1 posting list page write per insert
- Disk-friendly: sequential reads within clusters

### IVF's Semantic Assumption

IVF's core assumption is that the data has a stable cluster structure capturable by k-means centroids. This is a **semantic constraint**, not just a performance tradeoff:

- Centroids go stale as data distribution shifts (new agents, topic drift, new knowledge domains)
- Knowledge graph embeddings have no natural cluster boundaries — entities bridge domains
- Cross-cluster queries (common in knowledge graphs) require high `nprobe`, negating IVF's efficiency
- Retraining centroids (REINDEX) is expensive and creates downtime

### HNSW WAL Amplification at Scale

VectorChord's analysis quantifies the HNSW write amplification problem:
- A 2KB vector insert generates ~20KB of WAL (10x amplification)
- At tqvector's 783 bytes/vector, absolute WAL bytes are lower but the ratio is similar
- pgvector users report insert rates dropping from 300+/sec to 3/sec at millions of rows

However, tqvector's partitioned architecture mitigates this:
- 16 hash partitions → each partition index is 1/16th of total
- At 625K vectors/partition (light load), HNSW inserts are fast
- At 6.25M vectors/partition (typical), insert rate is low enough per-partition that WAL volume is manageable
- Bulk loads use the drop-index → COPY → CREATE INDEX pattern (FR-021 parallel build)

### Competitive Landscape

| Extension | Index | Quantization | Index Structure Choice |
|---|---|---|---|
| pgvector | HNSW, IVFFlat | halfvec, binary | Both available, HNSW recommended |
| pgvectorscale | StreamingDiskANN | SBQ | DiskANN for disk-resident, large-scale |
| VectorChord | IVF+RaBitQ, DiskANN+RaBitQ | RaBitQ | IVF for disk-friendly writes |
| tqvector | HNSW | TurboQuant | HNSW for topology-agnostic multi-use |

## Decision

**Use HNSW as the initial and default index structure.** IVF or DiskANN may be added in the future if specific workloads require their characteristics (e.g., extreme scale, disk-resident indexes, write-heavy workloads).

TurboQuant is orthogonal to the index structure — it is a compression/scoring layer. IVF could be added as a separate access method (`ivf_tqhnsw`) without changing the quantization layer. The current use cases (agent memories, knowledge graphs) are well-served by HNSW.

### 2026-04-25 Amendment

ADR-048 activates IVF as an optional sibling access method named `ec_ivf`.
This does not change the default: HNSW remains the primary access method
for heterogeneous, evolving data. The IVF lane exists to measure
write-amplification, sequential-read, and posting-list tradeoffs behind a
separate SQL surface, with TurboQuant, PqFastScan, and RaBitQ available as
posting-list quantizer profiles.

### WAL Amplification Mitigations

For deployments where WAL volume becomes a concern:
1. `wal_compression = zstd` (PG15+) — compresses full-page WAL images
2. Increase `checkpoint_timeout` — reduces full-page write frequency
3. More hash partitions (32/64) — keeps per-partition indexes smaller
4. Bulk load pattern: drop index → COPY → CREATE INDEX USING ec_hnsw

## Consequences

### Positive
- Single index implementation to maintain, test, and optimize
- Works correctly on agent memories (clustered), knowledge graphs (multi-modal), and future use cases without index-specific tuning
- No centroid training phase — zero-config indexing
- HNSW supports efficient incremental inserts (knowledge graph evolution)

### Negative
- WAL write amplification at scale (~10x) increases replication bandwidth
- HNSW requires graph + vectors in buffer cache for good performance (PG18 ReadStream mitigates cold-cache)
- Insert throughput degrades at very large per-partition sizes (>10M vectors)
- Cannot leverage IVF's disk-sequential access patterns for cold storage tiers

### Neutral
- TurboQuant's data-oblivious compression pairs equally well with HNSW or IVF — the decision is reversible at the index layer
- IVF or DiskANN may be revisited if workloads emerge that require: extreme scale (>100M vectors per index), disk-resident indexing, or write-heavy append patterns exceeding WAL mitigation capacity

## References

### HNSW vs IVF Analysis
- [VectorChord: Why HNSW is Not the Answer](https://blog.vectorchord.ai/why-hnsw-is-not-the-answer) — WAL write amplification analysis, disk-access patterns, IVF advantages for writes
- [Milvus: How to Choose Between IVF and HNSW](https://milvus.io/blog/understanding-ivf-vector-index-how-It-works-and-when-to-choose-it-over-hnsw.md) — decision framework, memory vs disk tradeoffs
- [KodeSage: HNSW vs IVFFLAT vs IVF_RaBitQ](https://kodesage.ai/blog/vector-indexes-hnsw-vs-ivfflat-vs-ivf-rabitq) — recall/latency benchmarks across index types
- [MyScale: HNSW vs IVF Explained](https://www.myscale.com/blog/hnsw-vs-ivf-explained-powerful-comparison/) — insert throughput, filtered search, update handling

### pgvector HNSW Scale Issues
- [pgvector issue #588: Insert performance with HNSW index](https://github.com/pgvector/pgvector/issues/588) — 10-20x insert slowdown with index present
- [pgvector issue #877: Slow inserts with HNSW](https://github.com/pgvector/pgvector/issues/877) — 3 rows/sec at millions of rows
- [pgvector issue #810: Insert/update very slow](https://github.com/pgvector/pgvector/issues/810) — insert degradation at scale
- [pgvector issue #822: Index creation stuck at tens of millions](https://github.com/pgvector/pgvector/issues/822) — build time at scale

### DiskANN Alternative
- [DiskANN: Fast Accurate Billion-point Nearest Neighbor Search on a Single Node (NeurIPS 2019)](https://suhasjs.github.io/files/diskann_neurips19.pdf) — Vamana graph + PQ, SSD-optimized
- [Tiger Data: HNSW vs DiskANN](https://www.tigerdata.com/learn/hnsw-vs-diskann) — architecture comparison, when to choose which
- [Azure: DiskANN Vector Index in PostgreSQL](https://techcommunity.microsoft.com/blog/adforpostgresql/introducing-diskann-vector-index-in-azure-database-for-postgresql/4261192) — Microsoft's PostgreSQL integration
