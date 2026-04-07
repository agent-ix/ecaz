---
id: ADR-018
title: "HNSW Graph Quality with TurboQuant-Compressed Distances"
status: DECIDED
impact: Affects FR-008, FR-009, FR-015, NFR-003
date: 2026-04-06
---
# ADR-018: HNSW Graph Quality with TurboQuant-Compressed Distances

## Context

HNSW graph construction uses distance calculations to select neighbor edges. When vectors are quantized, these distances are approximate — introducing noise into the graph topology. The question is whether TurboQuant-compressed distances produce a high-quality HNSW graph, and what mitigation options exist.

### Three Build Patterns in Production Systems

| Pattern | Graph Built With | Search With | Who Uses It | Tradeoff |
|---|---|---|---|---|
| **Build raw, store compressed** | Full-precision distances | Compressed distances | tqvector (via `build_source_column`), DiskANN | Best graph quality, requires raw vectors at build time |
| **Build compressed, search compressed** | Quantized distances | Quantized distances | tqvector (default), Weaviate HNSW+PQ (pre-v1.21) | Simpler, lower memory, noisier graph topology |
| **Build compressed, rescore top-k** | Quantized distances | Over-fetch compressed → rescore with raw | Elasticsearch BBQ, Weaviate v1.21+, DiskANN search | Good quality from rescoring, requires raw vectors on disk |

### Measured Impact of Quantized Graph Construction

**Weaviate HNSW+PQ (SIFT1M benchmark)**:
- Uncompressed HNSW: 0.99974 recall at 1,772 us
- HNSW+PQ (compressed build): 0.99658 recall at 1,937 us
- Recall drop: **0.03%** — negligible for most applications
- PQ reduces memory 47-88% depending on compression level

**Key insight**: The graph topology is surprisingly robust to distance noise. HNSW's redundant connectivity (m bidirectional edges per node, multiple layers) means that even with noisy distance estimates, the graph remains navigable. A few "wrong" edges don't break navigability — they just add a few extra hops.

### TurboQuant vs PQ Distance Quality

TurboQuant has a fundamental advantage over PQ for graph construction:

| Property | TurboQuant | Product Quantization |
|---|---|---|
| Distance estimator | Unbiased (QJL correction) | Biased (codebook quantization error) |
| Training required | No (data-oblivious) | Yes (k-means on subspaces) |
| Recall at 4-bit, 1536-dim | ~97% (NFR-003 target) | Often <90% at equivalent compression |
| Per-dimension information loss | Controlled by Lloyd-Max optimality | Dependent on subspace assignment |
| Works on any distribution | Yes (SRHT rotation normalizes) | Degrades on non-clustered data |

Since TurboQuant's distance estimates are higher-fidelity than PQ, and Weaviate sees only 0.03% recall loss with PQ distances, TurboQuant graph construction should produce near-optimal graph quality.

### Weaviate's Rotational Quantization — Same Core Idea as TurboQuant Stage 1

Weaviate's 8-bit RQ (introduced in v1.32) uses the same mathematical foundation as TurboQuant's MSE stage:

| Component | Weaviate 8-bit RQ | TurboQuant |
|---|---|---|
| Rotation | Walsh-Hadamard (3 rounds, blocked) | SRHT (Subsampled Randomized Hadamard) |
| Quantization | 8-bit scalar per dimension | b-bit Lloyd-Max per dimension |
| Residual correction | None | QJL 1-bit random projection |
| Recall (1M vectors) | 98-99% | ~97% at 4-bit (more compressed) |
| Compression | 4x (fp32 → int8) | ~7.7x (fp32 → 4-bit + QJL) |
| Training | None | None |

Weaviate's key findings on rotational quantization:
- Random rotation **smoothens entries**: after rotation, each coordinate follows a uniform distribution regardless of input
- **Increased magnitude**: L1 norm concentrates around sqrt(D), providing more signal to quantize
- **Distributed similarity**: correlation patterns spread across all dimensions
- Centering vectors around cluster centers can add 1-2 bits equivalent improvement, but isn't necessary

These properties apply equally to TurboQuant's SRHT rotation.

### Data Distribution Effects on HNSW + Quantization

A 2024 study on HNSW recall factors (Dolatshah et al., arXiv:2405.17813) found:

| Factor | Effect on Recall | Relevance to tqvector |
|---|---|---|
| Intrinsic dimensionality | Higher intrinsic dim → ~50% recall drop | Embedding models (1536-dim) have moderate intrinsic dim (~50-100) |
| Insertion order | Up to 12pp recall difference | Bulk build inserts in heap-scan order (random) — could optimize |
| High-LID-first insertion | +2.6-6.2pp recall improvement | Low-cost experiment: sort by estimated LID before build |
| Clustered data | Lower intrinsic dim → better HNSW | Agent memories are naturally clustered per-agent |
| Data-oblivious quantization | Rotation normalizes distribution | TurboQuant's SRHT makes all distributions equally quantization-friendly |

### The `build_source_column` Escape Hatch

tqvector already supports building from raw fp32 vectors via the `build_source_column` reloption (FR-008-AC-5). This provides Pattern 1 (best graph quality) when needed:

```sql
CREATE INDEX ON memories USING tqhnsw (embedding)
    WITH (build_source_column = 'raw_embedding');
```

The raw column is only needed at build time — it can be dropped afterward if storage is a concern.

## Decision

**Default to building the HNSW graph on TurboQuant-compressed distances (Pattern 2).** This is simpler, requires no raw vector storage, and TurboQuant's distance quality is sufficient for near-optimal graph construction.

**Retain `build_source_column` as an option** for workloads where maximum recall is critical and raw vectors are available.

**Add insertion-order optimization to the benchmark suite** (BC-041) to quantify the LID-ordering effect on tqvector specifically.

## Consequences

### Positive
- Default build path requires only tqvector data — no raw vectors needed
- TurboQuant's unbiased estimator produces higher-quality graphs than PQ
- Existing benchmarks (BC-005 through BC-007, BC-017 through BC-021) validate graph quality empirically
- `build_source_column` provides an escape hatch without architectural changes

### Negative
- Compressed-distance graph is theoretically suboptimal vs raw-distance graph
- No rescoring/reranking mechanism (Pattern 3) — would require raw vectors on disk

### Neutral
- The recall difference between Pattern 1 and Pattern 2 is expected to be <1% based on Weaviate's measurements with lower-fidelity PQ

## References

### HNSW + Quantization Measurements
- [Weaviate: HNSW+PQ — Exploring ANN Algorithms](https://weaviate.io/blog/ann-algorithms-hnsw-pq) — recall impact of building HNSW on PQ distances (0.99974 → 0.99658 on SIFT1M)
- [Weaviate: PQ Rescoring](https://weaviate.io/blog/pq-rescoring) — over-fetch + rescore strategy, v1.21 recall recovery
- [Elastic: Measuring Recall of Quantized Vector Search](https://www.elastic.co/search-labs/blog/recall-vector-search-quantization) — BBQ quantization recall analysis, >99% match to exact search

### Rotational Quantization
- [Weaviate: 8-bit Rotational Quantization](https://weaviate.io/blog/8-bit-rotational-quantization) — Walsh-Hadamard rotation + scalar quantization, 98-99% recall at 4x compression
- [Weaviate RQ Documentation](https://docs.weaviate.io/weaviate/configuration/compression/rq-compression) — configuration and deployment guide

### Data Distribution and HNSW Quality
- [The Impacts of Data, Ordering, and Intrinsic Dimensionality on Recall in HNSW (arXiv:2405.17813)](https://arxiv.org/html/2405.17813v1) — insertion order shifts recall by up to 12pp, high-LID-first improves recall
- [Qdrant: HNSW Indexing Fundamentals](https://qdrant.tech/course/essentials/day-2/what-is-hnsw/) — graph construction, layer assignment, search mechanics
- [OpenSearch: Practical Guide to HNSW Hyperparameters](https://opensearch.org/blog/a-practical-guide-to-selecting-hnsw-hyperparameters/) — ef_construction, m, ef_search tuning

### TurboQuant vs Competing Quantization
- [TurboQuant paper (ICLR 2026, arXiv:2504.19874)](https://arxiv.org/abs/2504.19874) — outperforms PQ on recall at every bit-width tested
- [Google Research: TurboQuant blog](https://research.google/blog/turboquant-redefining-ai-efficiency-with-extreme-compression/) — data-oblivious, near-optimal distortion rate
- [Qdrant issue #8524: TurboQuant support request](https://github.com/qdrant/qdrant/issues/8524) — community interest, comparison to scalar/binary quantization
- [RaBitQ paper (SIGMOD 2024)](https://dl.acm.org/doi/pdf/10.1145/3654970) — randomized binary quantization with formal guarantees, closest competitor

### DiskANN Build Strategy
- [DiskANN (NeurIPS 2019)](https://suhasjs.github.io/files/diskann_neurips19.pdf) — builds Vamana graph with full precision, stores PQ on disk, rescores from SSD
- [Milvus: DiskANN Explained](https://milvus.io/blog/2021-09-24-diskann.md) — compressed in-memory + exact on-disk search strategy
