---
id: ADR-007
title: "Persist gamma and use raw-query scoring for the high-quality search path"
status: DECIDED
impact: HIGH for FR-001, FR-005, FR-006, FR-009, FR-013, FR-015
date: 2026-04-04
---
# ADR-007: Persist gamma and use raw-query scoring for the high-quality search path

## Context

Deeper inspection of the TurboQuantDB quantizer implementation uncovered three important facts:

1. The high-quality prepared-query scorer requires a **raw query vector**. It builds:
   - an MSE LUT from the rotated raw query
   - a QJL projection vector `sq` from the raw query

2. The candidate-side QJL correction requires a persisted **residual norm scalar** `gamma`.
   Without `gamma`, the QJL term is underspecified and cannot be implemented faithfully.

3. The symmetric code-to-code fast path is fundamentally lower fidelity. In the reference
   implementation, the pure code-to-code preparation path does not recover the same QJL
   query state as the raw-query path.

The previous specification incorrectly blended these modes. It described:
- HNSW search over `(tqvector, tqvector)` with LUT-quality scoring
- code-to-code scoring with a QJL correction term but no persisted `gamma`

Those statements were not simultaneously implementable.

## Decision

### 1. High-quality search uses a raw query vector

The primary search path SHALL use a raw query embedding (`float4[]`) and a prepared-query
estimator. HNSW operator-class ordering SHALL therefore be defined over:

```sql
<#>(tqvector, float4[])
```

The public code-to-code comparison function remains available for ad-hoc use, but it is not
the highest-fidelity search path.

### 2. Persist `gamma` in every `tqvector`

The `tqvector` binary representation SHALL store a 4-byte `gamma` scalar in addition to the
packed MSE indices and packed QJL bits. This scalar is the residual norm used by the QJL
correction term at score time.

At 1536 dimensions and 4-bit quantization, the payload becomes:

```text
gamma      4 bytes
mse        576 bytes
qjl        192 bytes
total      772 bytes payload
```

### 3. Persist only the first `d` transform coordinates

The quantizer may use an internal transform dimension `n = next_power_of_two(d)`, but the
persisted representation stores only the first `d` MSE coordinates and the first `d` QJL sign
bits. The transform tail `[d, n)` is discarded from storage and treated as zero during
reconstruction.

This is an explicit product decision to preserve the storage target while retaining a defined
estimator contract.

### 4. Code-to-code scoring is MSE-only in v0.1

The symmetric code-to-code estimator SHALL use only the MSE centroid dot product:

```text
score_code_to_code(a, b) = Σ_i centroid[idx_a_i] * centroid[idx_b_i]
```

The QJL correction term is omitted in this path in v0.1.

## Consequences

### Benefits

- The search path is now mathematically specified and implementable.
- The QJL correction has the state it needs (`gamma`).
- High-quality HNSW search uses the better estimator.
- The compressed payload remains close to the original storage target.

### Tradeoffs

- The operator class now targets `(tqvector, float4[])`, not `(tqvector, tqvector)`.
- Ad-hoc code-to-code comparison is lower fidelity than the raw-query path.
- Discarding the transform tail is a deliberate approximation and may reduce fidelity versus
  a fully persisted `n`-coordinate scheme.

## Follow-Up

- If future benchmarks show unacceptable loss from transform-tail truncation, a later ADR
  may introduce a full-`n` storage mode or an alternate packed layout.
- If future benchmarks justify it, a richer compressed-query estimator may be added as a
  separate scoring mode rather than overloading the v0.1 code-to-code contract.

## Evaluation Criteria

The decision in this ADR SHALL be validated with an explicit benchmark program before the
implementation is considered performance-complete.

### Required Variants

At minimum, benchmarking SHALL compare:

1. **Current v0.1 design**
   - persisted payload stores only the first `d` transform coordinates
   - high-quality raw-query scorer
   - MSE-only code-to-code scorer

2. **Tail-retaining reference variant**
   - same estimator family
   - persisted payload stores all `n = next_power_of_two(d)` transform coordinates
   - used only for offline quality comparison, not necessarily productized

3. **MSE-only ablation**
   - same persisted MSE indices
   - `gamma = 0`
   - QJL bits ignored in all scoring paths

### Required Metrics

For each variant, record:

- Recall@10
- Recall@100
- NDCG@10
- Mean absolute score error versus true fp32 inner product
- Spearman rank correlation of candidate scores versus true fp32 ranking
- Top-k set overlap between variant and brute-force fp32 ground truth
- HNSW top-10 latency
- Sequential scan throughput
- Insert latency
- Persisted payload bytes per vector
- Total on-disk index bytes

The benchmark report SHALL present these metrics separately for:
- freshly bulk-built indexes
- indexes after incremental insert drift checkpoints
- the raw-query scorer
- the code-to-code scorer where applicable

### Required Methodology

- Run on at least one 1536-dimension embedding dataset representative of production use.
- Use at least 10,000 indexed vectors for quality experiments and 50,000+ for latency/index-size experiments.
- Use a fixed query set and fixed seeds so runs are comparable across variants.
- Report warm-cache and cold-cache measurements separately when feasible.
- Hold `m`, `ef_construction`, `ef_search`, hardware, compiler flags, and PostgreSQL settings constant across compared variants.
- Use brute-force fp32 inner product over the same raw vectors as the sole ground-truth ranking baseline.
- Report the exact dataset name, row count, dimensionality, query count, random seed, and drift checkpoint fractions.
- Measure drift at a minimum after 0%, 5%, 10%, and 20% of rows have been inserted since the last bulk build or REINDEX.
- Publish the exact SQL or harness configuration used to produce each reported number so the comparison is reproducible.

### Decision Gates

The current truncated-tail design remains the default if it satisfies all of the following:

- It meets the headline HNSW recall targets in NFR-003.
- Its Recall@10 degradation versus the tail-retaining reference variant is no more than 1.5 percentage points at the same `m` and `ef_search`.
- Its NDCG@10 degradation versus the tail-retaining reference variant is no more than 1 percentage point.
- It provides a meaningful storage advantage over the tail-retaining reference variant.
- Its post-insert drift curve remains monotonic and operationally acceptable relative to the freshly bulk-built baseline.

If any of those gates fail, the team SHALL revisit the storage format and consider:
- a full-`n` persisted layout
- an alternate packed tail representation
- a dual-mode storage option
