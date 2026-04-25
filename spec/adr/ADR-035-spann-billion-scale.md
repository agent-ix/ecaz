---
id: ADR-035
title: "SPANN as Third Access Method for Billion-Scale"
status: DROPPED
impact: Affects FR-014, NFR-001, ADR-032, ADR-034
date: 2026-04-18
---
# ADR-035: SPANN as Third Access Method

## 2026-04-25 Status Update

This ADR is no longer on the active roadmap. tqvector will pursue other
access-method algorithms instead of implementing SPANN. The analysis below
is retained as historical context for IVF-style posting lists,
replication, and billion-scale tradeoffs, but it is not an implementation
plan.

## Context

DiskANN + PqFastScan (ADR-034) extends tqvector's per-server scale ceiling
to ~3–5B vectors. Workloads beyond that — or workloads at 1B+ where p99
latency must be tight — hit a different bottleneck: DiskANN still probes
many graph nodes per query (hundreds), each potentially an SSD read.

SPANN (Chen et al., NeurIPS 2021) attacks this by replacing graph
traversal with **IVF-style clustered posting lists** and adding three
mechanisms:

1. **Balanced hierarchical k-means** — constrains posting-list size so
   per-probe I/O is predictable.
2. **ε-bounded replication** — each vector is stored in 6–12 posting
   lists covering the clusters where it would be a useful candidate for
   a nearby query. Queries probe only 1–4 clusters and still hit
   recall target.
3. **SSD-aligned posting list layout** — each probe is one sequential
   disk read.

Paper reports ~2× lower latency than DiskANN at equivalent recall on
billion-scale corpora.

**No Postgres extension currently ships SPANN.** The closest reference
implementations are Microsoft's SPTAG (research-quality C++) and
FAISS's `IndexIVFPQFastScan` (the routing and scoring parts, without
SPANN's replication/balancing). There is no Rust port and no
Postgres-native prior art.

## Historical Decision

tqvector previously considered **`tqspann`** as a third access method
after ADR-034's DiskANN track. That plan is dropped as of 2026-04-25.

### Per-server ceiling

At 1536d × 8× replication with PqFastScan 48 B codes:

- 1B vectors: ~384 GB on-disk, single large server territory.
- 10B vectors: ~3.8 TB on-disk, single XL server feasible.

PqFastScan is load-bearing here. With SBQ's 192 B codes the same
replication factor would consume 4× more storage (1.5 TB at 1B, 15 TB
at 10B). The small-code property of our scoring kernel is what makes
SPANN economically viable at the corpus scales that justify building
it at all.

### Why SPANN over extending DiskANN

- DiskANN's graph working set grows with corpus size. SPANN's centroid
  router stays tiny regardless of corpus size (only the posting lists
  scale).
- DiskANN's per-query node visits dominate latency at 1B+; SPANN's
  1–4 posting-list reads do not.
- Replication is the defining mechanism; it cannot be retrofitted onto
  a graph ANN without changing the algorithm into something that is
  no longer DiskANN.

### What we adopt from SPANN

- Balanced hierarchical k-means during build.
- RNG-style ε-replication rule for deciding where each vector lives.
- SSD-block-aligned posting list layout.
- Small memory-resident centroid index (BKT, flat, or small HNSW
  depending on centroid count).

### What we compose with

- PqFastScan as the within-posting-list scoring kernel. One LUT
  build per query, amortized across all candidates in all probed
  posting lists.
- RaBitQ binary prefilter (ADR-031) optional within posting lists.
  Likely less valuable than in HNSW due to small nprobe.
- Heap-f32 rerank (GroupedRerankMode::HeapF32) as the natural
  high-recall tail stage.

## Consequences

### Structural work required

SPANN is a bigger lift than DiskANN. Concrete pieces we do not have:

- **Balanced-clustering build pipeline.** Constrained k-means with
  size bounds; split-merge during iteration.
- **Replication decision logic.** RNG-like ε rule at insert time and
  batch-build time.
- **Multi-cluster insert.** Adding one vector may write to 6–12
  posting lists. Requires a new lock-ordering ADR; ADR-026 and the
  forthcoming DiskANN lock-ordering ADR both assume single-structure
  insert.
- **Reverse index for deletion.** Vacuum must find all replicas of a
  deleted tuple. Either scan every posting list (expensive) or
  maintain a `vector_id → cluster_ids` reverse map (bookkeeping
  storage).
- **SSD-aligned posting list writer with WAL integration.** Pages
  must land atomically at block boundaries; recovery must handle
  partial posting list writes.
- **Deduplication at query time.** Cheap hash set; mechanically
  straightforward.

Estimated effort: 3–4× the DiskANN effort. Not 2 years as earlier
hand-waving suggested, but materially larger than ADR-034.

### Deliberately long-horizon

This ADR is proposed, not committed. It stays proposed until:

- ADR-034 (DiskANN) ships and stabilizes.
- A real user workload at 1B+ materializes for tqvector.
- Either of: OPQ rotation (ADR-036) or AQ compression (ADR-037) lands,
  so the scoring kernel is at the recall-per-byte frontier that
  justifies the SPANN investment.

Shipping SPANN before these prerequisites means paying the
implementation cost for a scale band we do not yet serve, on top of
a scoring kernel we already know to upgrade.

### What SPANN unlocks

- Per-server ceilings of ~10–20B vectors — territory no Postgres
  extension currently reaches.
- Low-latency query path at billion scale via 1–4 probes per query.
- A credible "fastest Postgres vector DB at scale" positioning, if
  combined with OPQ/AQ improvements to the scoring kernel.

### What SPANN does not help

- Small-scale workloads (<10M). HNSW+PqFastScan is strictly better
  at those sizes; SPANN's overhead dominates its benefits.
- Workloads where insert throughput is critical. Multi-cluster insert
  is inherently heavier than single-structure insert.
- Workloads with severe storage budget constraints. 6–12× replication,
  even at PqFastScan's byte scale, is a commitment.

## References

- ADR-032: Coexisting Index Formats — TurboQuant and PqFastScan
- ADR-034: DiskANN as Second Access Method (scale tier below SPANN)
- ADR-036: OPQ Rotation (proposed scoring-kernel upgrade)
- ADR-037: AQ/RVQ (proposed scoring-kernel upgrade)
- SPANN (Chen et al., NeurIPS 2021)
- Microsoft SPTAG — reference SPANN implementation (C++, research)
- FAISS `IndexIVFPQFastScan` — closest in-library analog
