---
id: ADR-045
title: "SymphonyQG: Quantized-Graph Access Method with No-Rerank Query Path"
status: PROPOSED
impact: Affects ADR-018, ADR-022, ADR-030, ADR-031, ADR-032, ADR-033, ADR-034, ADR-036, ADR-041
date: 2026-04-19
---
# ADR-045: SymphonyQG Quantized-Graph Access Method

## Context

SymphonyQG (Gou, Gao, Xu — SIGMOD 2025, "Towards Symphonious
Integration of Quantization and Graph for Approximate Nearest
Neighbor Search") is a quantized-graph ANN method that co-designs
three things that tqvector currently treats as separate layers:

1. **A graph index** (HNSW/Vamana-style) whose neighbor-list layout
   is aligned to the SIMD batch size used by the scoring kernel.
2. **A quantizer** (RaBitQ) used as the *primary* distance, not a
   prefilter — the graph traversal never reads fp32 vectors.
3. **A quantization-aware edge-selection rule** at build time that
   prunes against the same quantized distance the scan will use.

Reported results: 1.5–4.5× QPS over competitive baselines at 95%
recall, 8× faster build than NGT-QG. The authors are the RaBitQ /
FastScan lineage, so the paper composes cleanly with the techniques
already proposed in ADR-030 (FastScan) and ADR-031 (RaBitQ).

tqvector's current state leaves this on the table:

- ADR-031 treats RaBitQ as a **prefilter**, with exact rerank as
  the final stage. SymphonyQG eliminates rerank.
- ADR-030 aligns *flat scan* to FastScan batch width. SymphonyQG
  applies the same alignment to *per-node adjacency lists* during
  graph traversal.
- ADR-018 notes that HNSW edge selection against quantized
  distances leaves recall on the table. SymphonyQG's
  quantization-aware pruning is the principled fix.
- ADR-032 and ADR-033 already establish the "coexisting formats
  with a shared graph lifecycle" pattern that a third AM variant
  would slot into.

## Decision

Adopt SymphonyQG as a **third access-method variant** alongside
`ec_hnsw` (ADR-032) and DiskANN (ADR-034), named `symphony` and
housed under `src/am/symphony/` per ADR-041. Land in three stages,
each independently shippable.

### Stage 1 — RaBitQ as a first-class quantizer

Implement RaBitQ as a standalone quantizer module alongside
`prod.rs` and `grouped_pq.rs`. Includes:

- Random rotation (or reuse existing SRHT / future OPQ rotation).
- 1-bit-per-dim encoding with scalar normalization factor.
- Unbiased distance estimator with usable error bound API.
- SIMD-accelerated Hamming / signed-popcount scoring.

Validation gate: on the 50k and 1M real seams, RaBitQ recall@10
within **1pp of exact** at the bit budget required to match PQ4
storage. This is the only research risk that can kill the effort;
ship Stage 1 alone and publish a recall study before committing
to Stages 2–3.

This stage supersedes ADR-031: RaBitQ stops being a prefilter of
fp32 and becomes a standalone distance.

### Stage 2 — Quantized-graph build, rerank still on

New AM variant `symphony` reusing the `ec_hnsw` page, build, and
insert skeletons, with two structural changes:

1. **Out-degree padding.** Each node's neighbor list is padded to
   a multiple of the FastScan SIMD batch size by selecting
   additional real edges (not dummies). Storage grows modestly;
   traversal issues only full-width kernels with no tail path.

2. **Quantization-aware edge selection.** The RNG / α-pruning
   rule evaluates candidate edges using the RaBitQ distance, not
   fp32. The built graph is self-consistent with the scoring path.

Query path in Stage 2 still reranks with exact fp32 as a safety
net. This isolates graph-layout risk from quantizer-accuracy risk.

### Stage 3 — No-rerank query path

Flip off rerank. Top-k returned directly from RaBitQ estimates,
using the error bound to size the candidate pool conservatively.
Gated by recall@10 holding at the Stage 1 baseline on full
benches.

### Relationship to other ADRs

- **ADR-031 (RaBitQ prefilter):** superseded. RaBitQ graduates
  from prefilter to primary distance. The prefilter design stays
  available as a fallback for `ec_hnsw`-format indexes that do
  not migrate.
- **ADR-030 (FastScan):** the per-node out-degree padding is a
  graph-scoped extension of the same batching principle. No
  conflict; the two layouts can share kernels.
- **ADR-018 (HNSW quantized graph quality):** the
  quantization-aware pruning rule is the principled response to
  the quality gap described there. `ec_hnsw` may adopt the same
  rule independently.
- **ADR-022 (drop scoring LUT for direct multiply):** converges
  on the same philosophy — stop translating between kernel
  representations. SymphonyQG is the graph-level embodiment.
- **ADR-032, ADR-033:** `symphony` slots in as a third coexisting
  format with the shared lifecycle already designed.
- **ADR-034 (DiskANN):** orthogonal. DiskANN targets scale;
  SymphonyQG targets latency-per-recall at moderate scale.
  DiskANN could eventually adopt SymphonyQG-style pruning against
  RaBitQ for its in-memory tier.
- **ADR-036 (OPQ):** compatible. OPQ's learned rotation can
  replace the random rotation in RaBitQ's front-end.
- **ADR-041:** `am/symphony/` is the third top-level AM module.

## Consequences

### Build-time cost

Per-node out-degree padding and RaBitQ-aware pruning together
inflate build time relative to `ec_hnsw` by an estimated 1.3–2×,
dominated by evaluating RaBitQ distances during neighbor
selection (cheaper than fp32, but evaluated more often because
the pruning rule is stricter). This is offset by the paper's
reported **8× build speedup vs NGT-QG** once the kernel is SIMD-tuned.

Large-corpus builds are a candidate for GPU acceleration — see
ADR-046 for the offline-trainer push model. CAGRA (cuVS) can
produce an initial k-NN graph that the CPU pipeline then refines
with out-degree padding and RaBitQ-aware pruning. GPU is
optional; the CPU path remains authoritative.

### Runtime cost

Scoring kernel reduces to XOR + POPCNT at ~8 ns/candidate
(ADR-031 measurement), issued in full-width SIMD batches because
adjacency is padded. Rerank elimination (Stage 3) removes the
~14 μs/candidate fp32 tail entirely. Expected end-to-end: 2–4×
QPS over current `ec_hnsw` at equal recall, consistent with the
paper's reported range.

### Storage

- RaBitQ codes: ~`D/8` bytes per vector (192 B at 1536d), vs
  768 B for PQ4 and 6144 B for raw f32.
- Adjacency padding: typically 5–15% increase in edges stored,
  depending on degree distribution.
- Net: Stage-2 `symphony` indexes are **smaller** than `ec_hnsw`
  despite the padding, because the code shrinks more than the
  graph grows.

### Wire format

New `INDEX_FORMAT_V5_SYMPHONY` (or equivalent reloption under
ADR-032's coexistence scheme). Not auto-migratable from
`ec_hnsw` — REINDEX only, consistent with ADR-030 and ADR-032.

### Query-path simplicity

Stage 3 collapses the three-stage pipeline sketched in ADR-031
(RaBitQ → FastScan → exact) into a single stage. The composition
in ADR-031's "if both succeed" section becomes moot once
SymphonyQG lands: the graph *is* the filter.

## Alternatives considered

### Keep ADR-031 as prefilter, never graduate

Preserves `ec_hnsw`'s rerank path as the safety net. Leaves the
graph-layout win (out-degree padding) on the table. Reasonable if
Stage 1's recall study fails.

### Adopt only out-degree padding in `ec_hnsw`

A minimal subset: keep PQ4 + rerank, but pad the neighbor lists
to FastScan batch width. Captures the SIMD-saturation half of
the paper without the quantizer swap. Worth considering as a
fallback if the RaBitQ accuracy gate fails, though the gain is
modest (~1.2–1.5× QPS estimated) without the rerank elimination.

### Full SymphonyQG as a replacement for `ec_hnsw`

Rejected. ADR-032's coexistence posture is load-bearing; forcing
a migration is both unnecessary and incompatible with the
no-auto-upgrade discipline.

### Wait for a later paper

Rejected. SymphonyQG is the direct successor to the techniques
already accepted or proposed (RaBitQ, FastScan, HNSW). Deferring
it leaves tqvector with a three-stage pipeline that SymphonyQG's
authors have already demonstrated is unnecessary.

## References

- Gou, Gao, Xu, "SymphonyQG: Towards Symphonious Integration of
  Quantization and Graph for Approximate Nearest Neighbor
  Search" (SIGMOD 2025) — [arXiv:2411.12229](https://arxiv.org/abs/2411.12229)
- NTU Vector DB Group — [SymphonyQG announcement](https://vectordb-ntu.github.io/news/symqg/)
- ADR-018: HNSW quantized graph quality
- ADR-022: Drop scoring LUT for direct multiply
- ADR-030: FastScan grouped subvector scoring
- ADR-031: RaBitQ binary prefilter (superseded in scope by this ADR)
- ADR-032: Coexisting index formats
- ADR-033: Shared graph lifecycle format adapters
- ADR-034: DiskANN as second access method
- ADR-036: OPQ rotation successor to SRHT
- ADR-041: Module structure for multi-AM multi-quantizer growth
- ADR-046: GPU-accelerated offline build trainer
