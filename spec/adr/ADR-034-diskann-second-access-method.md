---
id: ADR-034
title: "DiskANN as Second Access Method for Medium-Large Scale"
status: PROPOSED
impact: Affects FR-014, NFR-001, ADR-026, ADR-027, ADR-032
date: 2026-04-18
---
# ADR-034: DiskANN as Second Access Method

## Context

The HNSW + PqFastScan stack from ADR-032 is expected to serve corpora up to
~500M vectors per server on commodity hardware. Beyond that point HNSW's
structural limits bind:

- The graph must be memory-resident for good latency; per-node footprint
  (code + neighbors ≈ 176 B at 1536d with PqFastScan) limits what fits in RAM.
- Random-access graph traversal is cache-hostile regardless of node size.
- ADR-026 lock ordering for live insert gets harder to honor cleanly as the
  graph grows.

Two Postgres extensions already occupy the scale-past-HNSW lane:
**pgvectorscale** (DiskANN + SBQ) and **VectorChord** (DiskANN + RaBitQ).
Both use Microsoft's DiskANN / Vamana graph structure, designed for disk
residency and single-layer topology.

Our PqFastScan scoring kernel is **structure-agnostic** — it scores a batch of
PQ-coded vectors without caring whether they come from an HNSW hop, a Vamana
walk, or an IVF posting list. Work invested in PqFastScan (tasks 15, 16)
therefore carries forward to any outer structure we adopt next.

## Decision

tqvector will add **`tqdiskann`** as a second index access method after
tasks 15 and 16 land. The scoring kernel is PqFastScan (identical to
`tqhnsw`'s PqFastScan path). The graph layer is Vamana, following the
pgvectorscale design as reference.

### Per-node footprint

At 1536d, Vamana node footprint with PqFastScan codes:

- 48 B search code (4-bit grouped PQ)
- ~128 B neighbor list at M=32
- ≈ 176 B per node total

This is ~2× smaller than pgvectorscale's SBQ-based nodes (~320 B) and
~1.8× smaller than VectorChord's RaBitQ nodes.

### Scale ceiling

Per-server estimate on a well-provisioned box (128 GB RAM, multi-TB NVMe):

- `tqhnsw` + PqFastScan: up to ~500M vectors.
- `tqdiskann` + PqFastScan: ~3–5B vectors.

This tier extends tqvector into the scale band currently owned by
pgvectorscale and VectorChord, with a strictly stronger scoring kernel
(4-bit grouped PQ vs 1-bit SBQ/RaBitQ).

### Access method layout

- New AM: `tqdiskann` (handler + opclass parallel to `tqhnsw`).
- New wire tag: `INDEX_FORMAT_V3_DISKANN` (or similar) for page-layout
  versioning. TurboQuant and PqFastScan continue to live under
  `tqhnsw`; `tqdiskann` is a separate format space.
- Shared: quantizer training pipeline, SRHT rotation, grouped PQ
  codebooks, binary prefilter sidecar infrastructure.

### What changes relative to HNSW

- Single-layer graph (no upper layers).
- α-pruning during Vamana construction rather than HNSW's
  neighbor-selection heuristic.
- Neighbor lists designed to read as one SSD I/O per node visit.
- Cache/page-cache residency becomes the dominant latency driver rather
  than RAM capacity.

## Consequences

### Structural work required

- New build pipeline implementing Vamana α-pruning.
- New insert pipeline; ADR-026's HNSW lock-ordering rules don't apply
  directly. A new "ADR: Vamana insert lock ordering" will be required.
- New vacuum pipeline; ADR-027's HNSW repair logic likewise needs a
  Vamana analogue.
- New cost model entries in `am/cost.rs` for the Vamana access pattern.

### Non-goals for this ADR

- Flipping default access method. `tqhnsw` remains default; users opt
  into `tqdiskann` when their corpus warrants it.
- OPQ rotation (ADR-036) or AQ compression (ADR-037). DiskANN lands
  first with the PqFastScan encoding as it stands after task 15.
- SPANN-style IVF routing (ADR-035). Explicitly deferred; DiskANN and
  SPANN are complementary scale bands, not competing designs.

### Sequencing

- Blocked on task 15 (PqFastScan first-class) and task 16 (TurboQuant
  iteration). Both stabilize the scoring kernel before a second AM
  adopts it.
- Informed by pgvectorscale's public design; accelerates our learning
  curve but we should not assume a direct port is safe — their insert
  concurrency story has known rough edges we would need to address
  independently.

### What this does not unlock

- Billion-vector-plus scales where SPANN's replication wins dominate
  (ADR-035). DiskANN tops out around 3–5B per server in our analysis.
- Workloads where latency SLA cannot tolerate any SSD read; those
  remain HNSW+PqFastScan territory.

## References

- ADR-026: Live insert backlink lock ordering (HNSW)
- ADR-027: Vacuum graph repair lock ordering (HNSW)
- ADR-030: FastScan Grouped Subvector Scoring
- ADR-032: Coexisting Index Formats — TurboQuant and PqFastScan
- ADR-035: SPANN as Third Access Method for Billion-Scale (proposed peer)
- pgvectorscale (Timescale), public Vamana implementation for Postgres
- VectorChord, public DiskANN implementation for Postgres
- DiskANN (Subramanya et al., NeurIPS 2019)
- FreshDiskANN (Singh et al., 2021) for streaming-insert variant
