---
id: ADR-048
title: "IVF as Optional Access Method"
status: PROPOSED
impact: Affects ADR-017, ADR-032, ADR-035, ADR-041, FR-008, FR-009, FR-010, FR-016, FR-020, NFR-001, NFR-002, NFR-003
date: 2026-04-25
---
# ADR-048: IVF as Optional Access Method

## Context

ADR-017 chose HNSW as the initial and default index structure because
tqvector had to serve heterogeneous, evolving embedding sets. That
decision still holds for the default access method. The project now has
enough AM structure, quantizer seams, and benchmark harnessing to add IVF
as an opt-in sibling without weakening the HNSW path.

IVF is worth building for three concrete reasons:

- live insert appends to one posting list instead of mutating graph edges
- selected posting lists are sequential-read friendly
- a native IVF baseline lets us measure VectorChord-style IVF tradeoffs
  against `ec_hnsw`, Symphony, and future access methods
- the posting-list shape composes naturally with the quantizers already
  available in-tree: TurboQuant, PqFastScan, and RaBitQ

This ADR activates task 28. It does not make IVF the default and does not
adopt multi-list replication or balanced hierarchical routing.

## Decision

Add a new PostgreSQL index access method named **`ec_ivf`**. It is a
plain IVFFlat-style AM: train centroids during `CREATE INDEX`, assign
each indexed vector to exactly one posting list, scan the nearest
`nprobe` posting lists for an ORDER BY query, and emit ordered results
through the normal index-scan callback lifecycle.

### SQL Contract

- Access method: `ec_ivf`.
- Operator classes: reuse `tqvector_ip_ops` and `ecvector_ip_ops` for
  `USING ec_ivf` if PostgreSQL's AM-scoped operator-class namespace
  accepts the same names. If bootstrap verification finds a catalog
  conflict, use `tqvector_ivf_ip_ops` and `ecvector_ivf_ip_ops`.
- Session GUC: `ec_ivf.nprobe`.
- Reloptions:
  - `nlists`: centroid count. `0` means auto.
  - `nprobe`: default posting-list probe count. `0` means auto.
  - `storage_format`: `turboquant | pq_fastscan | rabitq | auto`.
  - `training_sample_rows`: maximum sampled rows for centroid training.
    `0` means auto.
  - `seed`: deterministic training seed.
  - `rerank`: `off | heap_f32 | source_column | auto`, depending on
    which source/rerank policy is available for the indexed type.

The session GUC overrides the reloption when set to a positive value,
matching the existing `ec_hnsw.ef_search` control-surface pattern.

### Metric and Training Contract

The first IVF implementation targets the existing inner-product operator
surface. The centroid router is independent from the posting-list
quantizer profile. Centroid training uses **spherical k-means**:

1. decode or source-read the build vector as f32
2. normalize it for centroid training and assignment
3. store normalized centroids
4. route queries by inner product against normalized centroids
5. score candidates with the selected posting-list quantizer profile

This keeps the router direction-based while preserving candidate scoring
over the original vector payload. If real-corpus recall shows norm
variance is the primary failure mode, a later ADR may adopt a MIPS
transform or a learned router. That is not part of v1.

Posting-list candidate scoring supports the quantizer families already in
the repository:

- **TurboQuant:** compatible baseline and current default semantics.
- **PqFastScan:** expected hot path for IVF because list scans are dense,
  sequential, and batch-scoreable.
- **RaBitQ:** compact score/pre-filter profile with optional rerank,
  useful for VectorChord-style comparisons and high-ingest storage
  experiments.

Each profile has its own recall and latency gate. A fast profile that
requires rerank to hold recall must keep rerank explicit in metadata,
EXPLAIN, and measurement packets.

### Storage Contract

`ec_ivf` owns its own format space under `src/am/ec_ivf/` per ADR-041.
Metadata stores:

- dimensions and quantizer/source shape
- selected posting-list `storage_format`
- `nlists`
- default `nprobe`
- training seed and training version
- centroid storage location
- posting-list directory head/tail refs
- per-list live/dead counts
- insert-since-build and list-imbalance drift counters

Posting-list pages store candidate payloads in the selected quantizer
format plus heap TIDs. The page codec should reuse shared storage/WAL
primitives where they are already AM-neutral, but IVF should not force
new cross-AM abstractions before the first baseline proves the shape.

### Build, Insert, Scan, and Vacuum

- **Build:** train centroids, assign each row to one list, and write
  posting lists sequentially using the selected quantizer profile.
- **Insert:** assign the new row to the nearest centroid and append to
  that list. Centroids do not move online.
- **Scan:** score all centroids, select `nprobe` lists, sequentially read
  selected lists, score candidates through the selected quantizer profile,
  deduplicate heap TIDs, and emit ordered results.
- **Vacuum:** remove dead heap TIDs from lists, repair directory counts,
  and update drift stats. Vacuum does not retrain centroids.

## Acceptance Gates

### Functional Gate

- Empty, singleton, duplicate-heavy, and multi-page list indexes build and
  scan without panics.
- `nprobe = nlists` is a full-probe mode and must match exact ordered
  candidate scoring for indexed rows when the selected profile/rerank mode
  claims exact final scoring.
- TurboQuant, PqFastScan, and RaBitQ profile metadata must reject
  mismatched dimensions, seeds, codebook shape, and unsupported rerank
  source layouts with explicit errors.
- Rescan, exhaustion, backward-scan rejection, score-slot emission, and
  duplicate heap-TID draining follow the same visible contract as
  `ec_hnsw`.

### ANN Gate

Before planner-preferred use, publish real `10K` and `50K` recall@10
sweeps over `nlists` and `nprobe`, comparing:

- exact scan
- `ec_hnsw` at current accepted operating points
- `ec_ivf` full-probe mode
- `ec_ivf` ANN probe settings for each enabled quantizer profile

No default or cost-model promotion is allowed from synthetic-only data.

### Performance and Storage Gate

Any claim that IVF improves latency, storage, or write amplification must
store packet-local raw logs and compare against `ec_hnsw` on the same
head SHA. At minimum:

- warm and cold p50/p95/p99 at equal recall
- `pg_relation_size`
- build WAL
- live-insert WAL
- vacuum runtime and post-vacuum recall sanity
- quantizer profile and rerank mode used for the run

## Consequences

### Positive

- Adds a low-write-amplification AM without perturbing HNSW.
- Gives the project a native posting-list storage shape that future
  partitioned or posting-list algorithms can learn from.
- Makes cold sequential-read behavior measurable instead of theoretical.

### Negative

- Adds centroid staleness and REINDEX pressure that HNSW does not have.
- Requires careful inner-product routing validation because simple IVF
  clustering is not naturally MIPS-optimal.
- Adds another planner/cost model surface.

### Neutral

- HNSW remains the default access method.
- Multi-list replication, balanced hierarchical k-means, and SSD-aligned
  multi-list layout are not part of this task.
- Online centroid retraining is deferred. The first operational answer to
  drift is drift observability plus REINDEX.

## Relationship to Other ADRs

- **ADR-017:** amended. HNSW remains the default, but IVF is no longer only
  a deferred option.
- **ADR-032:** compatible. IVF is a separate AM and explicitly consumes
  the same TurboQuant and PqFastScan quantizer families where the page
  layout supports them.
- **ADR-045:** compatible. RaBitQ is available as an IVF posting-list
  profile or prefilter/rerank profile; Symphony remains a separate graph
  AM.
- **ADR-035:** dropped from the active roadmap. IVF v1 assigns one list per
  vector and has no replication.
- **ADR-041:** follows the multi-AM module layout under `src/am/ec_ivf/`.

## Non-Goals

- Replacing `ec_hnsw` as default.
- Multi-list replication.
- Balanced hierarchical k-means.
- Online centroid retraining.
- L2/cosine operator support.
- Cross-AM posting-list abstractions before the first IVF baseline lands.
