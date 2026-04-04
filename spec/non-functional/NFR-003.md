---
id: NFR-003
title: Recall Quality
type: non-functional-requirement
status: APPROVED
traces:
  - StR-001
---
# NFR-003: Recall Quality

## Requirement

### Recall@10 Targets (50K × 1536, 4-bit)

| Configuration | Minimum Recall@10 |
|---|---|
| m=8, ef_search=128 | ≥ 89% |
| m=8, ef_search=200 | ≥ 93% |
| m=16, ef_search=200 | ≥ 97% |
| Sequential scan over `tqvector` codes (no HNSW) | Measured separately; approximate |

### Ground Truth

Recall is measured against brute-force exact inner product over raw fp32 vectors. Ground truth computed outside Postgres using numpy or equivalent.

### Unbiased Estimation

The extension SHALL implement the query-to-code and code-to-code estimators exactly as declared in FR-013 and FR-015. The implementation SHALL NOT introduce additional bias beyond those declared formulas (for example by substituting a different correction term, changing normalization constants, rounding intermediate values differently across code paths, or silently omitting required state).

### Incremental Insert Drift

The headline recall targets above apply to freshly bulk-built indexes. Recall after incremental inserts SHALL be benchmarked separately and reported as a function of the fraction of nodes inserted since the last bulk build or REINDEX.

## Measurement

Recall benchmarks SHALL be run against the DBpedia OpenAI embeddings dataset (or equivalent) and reported in `BENCHMARKS.md`.

### Required Methodology

- Use brute-force exact inner product over raw fp32 vectors as ground truth.
- Use the same query set for all compared estimator and storage variants.
- Report results at minimum for 1536-dimensional vectors and 4-bit quantization.
- Report freshly bulk-built results separately from post-insert-drift results.
- Hold `m`, `ef_construction`, `ef_search`, hardware, compiler flags, and PostgreSQL settings constant across compared variants.
- Measure post-insert drift at a minimum after 0%, 5%, 10%, and 20% of rows have been inserted since the last bulk build or REINDEX.
- Publish dataset name, row count, dimensionality, query count, random seed, and checkpoint definitions with every benchmark report.

### Required Comparisons

- Compare the raw-query prepared scorer against the symmetric code-to-code scorer.
- Compare the current truncated-tail storage layout against a tail-retaining offline reference variant.
- Compare full MSE+QJL scoring against an MSE-only ablation.

### Reference Variant Definition

For this requirement, the tail-retaining offline reference variant is defined as an evaluation-only build that keeps the same quantizer, codebook generation, and scoring formulas as the current design, but persists and scores the full transform-domain tail instead of truncating coordinates `[original_dim, transform_dim)`. It is not a required product mode; it exists only as the normative comparison baseline for quality-loss measurement.

### Required Metrics

In addition to Recall@10, each benchmark report SHALL include:
- Recall@100
- NDCG@10
- mean absolute score error versus true fp32 inner product
- Spearman rank correlation versus the true fp32 ranking
- top-k set overlap versus ground truth

### Decision Gates

The current truncated-tail design remains acceptable if:
- the headline recall targets above are met
- Recall@10 degradation versus the tail-retaining reference variant is no more than 1.5 percentage points
- NDCG@10 degradation versus the tail-retaining reference variant is no more than 1 percentage point
- the post-insert drift curve remains monotonic and reported at the required checkpoints

If those gates fail, the storage/scoring design SHALL be revisited.
