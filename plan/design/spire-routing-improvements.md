# SPIRE Routing — Improvement Landscape

This doc catalogs known and proposed improvements to SPIRE's routing
layer. It is not a commitment to implement any of them — it exists so
that recurring "could routing be better?" conversations have a
reference point and so future ADRs can cite a baseline.

Three layers, ordered by maturity / risk:

1. **Catch-up to the SPIRE paper** — the implementation is staged; not
   every paper element has landed yet.
2. **Static improvements past the paper** — well-established techniques
   from adjacent literature that the SPIRE paper does not adopt.
3. **Learned routing** — research-grade improvements that introduce
   training pipelines and concept-drift exposure.

## 1. Paper catch-up (already on the roadmap)

| Gap | Today | Paper baseline | Roadmap |
|---|---|---|---|
| Recursion depth | flat IVF (Phase 1) | 4 levels @ 2B, 6 @ 1T | Phase 3 (`spire-recursive-hierarchy.md`) |
| Internal-level `nprobe` | hardcoded `nprobe=1` (TODO at `src/am/ec_spire/scan/types.rs`) | per-level tunable | small follow-up; needs a durable per-level metadata field |
| Boundary replication | single primary assignment | 6–12 replicas (Top-N rule) | Phase 5 (`spire-boundary-replication.md`) |
| Top-level routing | linear scan over flat centroids | HNSW or DiskANN over top centroids | Phase 3 (cited in `ADR-049:39-40`) |
| Centroid persistence | rebuilt from diagnostics | persisted | TODO at `src/am/ec_spire/build/types.rs` |

These items are scoped, defensible, and primarily about feature
velocity vs staging risk. Recall-at-fixed-latency improves
incrementally as each lands. None requires research.

## 2. Static improvements past the paper (cheap, no learning)

These are proven techniques from the literature that the SPIRE paper
does not adopt. Each could land as a self-contained ADR with a
measurable A/B against the current router.

### 2.1 IMI (Inverted Multi-Index)

Babenko & Lempitsky 2012. Product-quantize the centroid table itself
so an `nlists = k` index becomes effectively `√k × √k` leaves at
`√k` search cost.

- Effect: ~10× more leaves at the same routing time → finer
  partitioning, higher recall at fixed `nprobe`.
- Risk: low. Pure data-layout change in the centroid storage path;
  no impact on assignment, posting list, or scan kernels.
- Open question: interaction with SPIRE's recursive hierarchy.
  IMI-on-centroids is well-defined for flat IVF; recursive variant
  is not in published literature and would need experimental work.

### 2.2 Multi-probe centroid scoring

Adapted from multi-probe LSH (Lv et al. 2007). Perturb the query
slightly and combine the perturbed probes' centroid rankings.

- Effect: ~2× effective `nprobe` for ~1.3× cost. Most useful when
  query embeddings are noisy or when `nprobe` is small.
- Risk: low. No storage changes. Implementation is a small change in
  the centroid-scoring loop.
- Caveat: gains are workload-dependent; may be near-zero on already
  well-aligned embeddings.

### 2.3 Adaptive `nprobe` per query

Microsoft SPTAG and others. The score distribution after the first
probe predicts whether the query needs more probes. Easy queries
(most of them) terminate early; hard queries get more budget.

- Effect: ~30–60% QPS improvement on mixed workloads at the same
  recall floor.
- Risk: low–medium. No training. Needs a per-query budget controller
  and explainable termination criteria for predictable p99.
- Closest to "free win" in this list.

### 2.4 Anisotropic centroid scoring (ScaNN-style)

Guo et al. 2020. k-means treats all embedding components equally, but
similarity search cares more about score-relevant components. ScaNN's
anisotropic VQ trains centroids that minimize loss specifically on
the score function rather than reconstruction error.

- Effect: **the single biggest known win past vanilla IVF/SPIRE** —
  1.5–2× recall at same QPS on dense embeddings (ScaNN paper, Google
  blog measurements).
- Risk: medium. Changes the centroid training objective; affects
  build pipeline and would need careful interaction analysis with
  RaBitQ scoring (which already has its own anisotropic-flavor
  rotation step).
- This is the standalone improvement most worth a real ADR + bench.

## 3. Learned routing (research-grade)

Higher upside, real maintenance cost (retraining as data shifts), and
a class of failure modes the rest of ecaz does not have. Out of scope
for current planning; documented for completeness.

### 3.1 NN-routing classifier

Small MLP from query embedding to top-k partition IDs directly.
Skips centroid scoring entirely. Shipping in Pinecone (proprietary)
and ScaNN+.

- Effect: routing cost becomes constant in `nlists`; the centroid
  scoring loop disappears.
- Cost: model training, retraining schedule, drift detection,
  versioning across SPIRE epochs.

### 3.2 Query difficulty estimator

Predict the right `nprobe` from query embedding shape (norm,
dispersion, etc.) before any probing happens.

- Effect: subsumes adaptive `nprobe` (§2.3) with a model. Small
  marginal win over rule-based adaptive `nprobe` if §2.3 is already
  in place.

### 3.3 Routing reranker

Small MLP that re-ranks top-`2×nprobe` candidates from coarse
routing. Catches most of the recall lost to coarse pruning.

- Effect: works well when centroids drift from data distribution;
  becomes the answer when retraining centroids is expensive.

## Recommended sequencing (if any of this is ever pursued)

1. **Layer 1 catch-up** as planned per existing phase docs.
2. **§2.3 adaptive `nprobe`** — cheapest standalone win, no training,
   small implementation.
3. **§2.4 anisotropic centroid scoring** — biggest static win past
   the paper. Land as an ADR with a measured 10m/100m bench
   comparing recall-at-QPS against vanilla SPIRE.
4. **§2.1 IMI** — only worth doing if §2.4 doesn't already saturate
   the recall headroom we care about.
5. **Layer 3** — only if a specific workload hits a wall the static
   layers can't clear.

## Notes on "is the paper optimal?"

The SPIRE paper is solid 2022-vintage state of the art. Specific
known weaknesses:

- Vanilla k-means centroids; no anisotropic objective.
- Fixed `nprobe` per query; no adaptive budget.
- No multi-probe perturbation.
- Boundary replication is the paper's main novelty for recall, but
  it costs storage linearly in replica count.

ScaNN (Google) and Pinecone-proprietary indexes have publicly
demonstrated 1.5–2× recall at fixed QPS over IVF-family routers via
combinations of §2.3 + §2.4. So yes — the paper is improvable; the
question is whether ecaz's product roadmap values that improvement
enough to spend the ADR + bench cycles to land it.

## References

- SPIRE paper: cited in `spec/adr/ADR-049-spire-on-single-level-ivf-foundation.md:336`
  (no URL in repo — unresolved citation gap).
- IMI: Babenko & Lempitsky, "The Inverted Multi-Index", CVPR 2012.
- Multi-probe LSH: Lv, Josephson, Wang, Charikar, Li, "Multi-probe LSH",
  VLDB 2007.
- Adaptive nprobe / SPTAG: Microsoft SPTAG, GitHub microsoft/SPTAG.
- ScaNN / anisotropic VQ: Guo, Sun, Lindgren, Geng, Simcha, Chern,
  Kumar, "Accelerating Large-Scale Inference with Anisotropic
  Vector Quantization", ICML 2020.
- Learned routing surveys: numerous; not load-bearing here.
