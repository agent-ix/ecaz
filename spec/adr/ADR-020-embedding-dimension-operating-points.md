---
id: ADR-020
title: "Embedding Dimension Operating Points: 1024 vs 1536 vs 2048"
status: PROPOSED
impact: Affects FR-013, FR-014, FR-017, ADR-007, NFR-001, NFR-002, NFR-003
date: 2026-04-08
---
# ADR-020: Embedding Dimension Operating Points: 1024 vs 1536 vs 2048

## Context

tqvector currently treats dimensionality as an input property of the embedding model, but several
product and performance questions now depend on whether the practical operating point should stay at
`1536`, move down to `1024`, or move up to `2048`.

The earlier ADR-020 draft framed this mostly as competitive positioning. That was the wrong center
of gravity. Competitive comparison is useful background, but the actual decision is about tqvector's
own storage, query-preparation, scoring, and page-layout costs at each dimension.

Three implementation facts matter most:

1. **Internal transform size is `next_power_of_two(dim)`.**
   The SRHT/FWHT path operates on `transform_dim`, not always on the original dimension.

2. **FWHT is not a per-candidate search cost.**
   In tqvector, SRHT/FWHT is paid when encoding a vector and when preparing a raw query.
   Per-candidate search scoring is then an `O(dim)` hot loop over prepared query state and packed
   candidate bytes.

3. **Storage and per-candidate scoring scale with original dimension.**
   The persisted payload stores only the first `dim` MSE coordinates and QJL signs, not the full
   padded transform tail.

This means `1536` and `2048` share the same FWHT transform length (`2048`) but do **not** share
the same candidate-scoring or storage cost.

## Repo-Grounded Facts

The table below uses the current tqvector implementation and layout formulas:

- `transform_dim = next_power_of_two(dim)`
- `payload_len = 4 + ceil(dim * (bits - 1) / 8) + ceil(dim / 8)`
- `code_len = payload_len - 4`
- element tuple payload is `74 + code_len` bytes before PostgreSQL tuple/header alignment
- approximate pure-element tuples/page assume the current `8192` byte page size and ignore neighbor
  tuples, graph topology, and duplicate heap-TID variation

For the current primary configuration (`bits = 4`):

| Original dim | `transform_dim` | FWHT work proxy `n log2 n` | Payload bytes | Code bytes | Approx element tuple storage bytes | Approx pure-element tuples/page | Current AVX2 3-bit hot query bytes (`rotated + sq`) | Scalar/generic LUT bytes |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| `1024` | `1024` | `10,240` | `516` | `512` | `600` | `13` | `8,192` | `32,768` |
| `1536` | `2048` | `22,528` | `772` | `768` | `856` | `9` | `12,288` | `49,152` |
| `2048` | `2048` | `22,528` | `1,028` | `1,024` | `1,112` | `7` | `16,384` | `65,536` |

### Interpretation

#### 1024 dimensions

- Best efficiency point for the current design.
- No padding tax in the SRHT/FWHT path.
- Lowest payload and best page density of the three options.
- Cheapest per-candidate scoring because the hot loop scales with `dim`.
- Strong candidate for throughput-focused work such as tiled FWHT and page-density studies.
- Plausible "very fast mode" if recall remains acceptable after truncation.
- Main risk is quality, not mechanics: the repo has no measured evidence yet that `1024` preserves
  enough semantic signal for the target workloads.

#### 1536 dimensions

- Current reference point and the present benchmark/reporting baseline.
- Pays the padding tax on query preparation because `1536 -> 2048` in the FWHT path.
- Still has materially lower storage and per-candidate scoring cost than `2048`.
- This means `1536` is not simply "the worst of both worlds". It shares `2048`'s query-prep
  transform size, but keeps a `25%` smaller hot scoring loop and payload.

#### 2048 dimensions

- Removes the FWHT padding tax relative to `1536`.
- Does **not** make search latency "the same as 1536" in tqvector's current architecture.
- Query preparation cost is the same as `1536`, but payload bytes and per-candidate scoring are
  both `33%` larger.
- This makes `2048` a quality-oriented comparison point, not an automatic performance win.
- The real hypothesis is narrower: `2048` may buy better recall or semantic headroom while keeping
  the same query-preparation FWHT cost as `1536`.

### Explicit Working Hypotheses

1. **`2048` is the quality candidate.**
   It may deliver better recall than `1536` while sharing the same padded FWHT/query-prep cost.
   The tradeoff is larger payloads and a larger per-candidate hot loop.

2. **`1024` is the speed candidate.**
   It removes the padding tax, shrinks payload bytes substantially, and should lower both
   query-preparation and per-candidate scoring cost. The open question is whether recall stays good
   enough for the target workload after truncation.

3. **`1536` is the comparison baseline.**
   It remains the current reference point because the repo's existing tests, plans, and review
   packets are centered there, not because it has already won the operating-point decision.

### What Not To Assume

1. **Do not model tqvector search as "an FWHT per distance calculation".**
   FWHT is paid once per query preparation and once per encode, not once per candidate score.

2. **Do not treat page-density math as full index-capacity math.**
   The tuple/page numbers above are useful first-order approximations, but real HNSW memory use also
   includes neighbor tuples, page fragmentation, metadata, relcache/buffer effects, and graph
   topology.

3. **Do not infer recall from dimensionality alone.**
   Higher dimension may preserve more signal, but the repo has not yet measured the recall/latency
   tradeoff for `1024` vs `1536` vs `2048`.

4. **Do not project current live insert throughput from future graph-aware insert behavior.**
   The current live insert path appends disconnected nodes. Dimension-sensitive neighbor-selection
   costs matter for bulk build today and for future graph-aware insert work, but not for the
   current narrow live insert callback.

### Supporting Competitive Context

Competitive comparison is still relevant, but only as supporting evidence:

- Against raw `float4` vector bytes alone, current tqvector payload compression remains close to
  `8x` across `1024`, `1536`, and `2048`.
- That competitive framing should not override the repo-grounded distinction between:
  - FWHT/query-prep cost, which follows `transform_dim`
  - candidate scoring and persisted bytes, which follow original `dim`

## Decision

1. **Use `1536` as the current benchmark baseline, not as the settled long-term winner.**
   It is the existing comparison point in the repo and the anchor for evaluating alternatives.

2. **Treat `1024` as the primary efficiency candidate.**
   It is the best candidate for tiled-FWHT work, page-density analysis, and throughput-oriented
   experiments because it removes padding and reduces both hot-loop cost and payload bytes.

3. **Treat `2048` as the primary quality candidate.**
   It is the right comparison point when evaluating whether the additional signal and the removal of
   the `1536 -> 2048` padding tax justify the larger payload and hotter scoring loop.

4. **Do not settle the default operating point without measurement.**
   Choosing `1024`, `1536`, or `2048` requires explicit benchmark evidence for:
   - recall / ranking quality
   - HNSW latency
   - sequential scan throughput
   - build / future graph-aware insert throughput
   - persisted payload bytes
   - total index bytes

## Consequences

### Positive
- ADR-020 now matches the real design question instead of burying it inside competitor analysis.
- The repo has a clear interpretation of `1024`, `1536`, and `2048` that matches the current code.
- Future FWHT optimization work can target `1024` and `2048` intentionally instead of treating all
  dimensions as interchangeable.
- Product discussions can now separate "quality operating point" from "throughput operating point".

### Negative
- The ADR intentionally leaves the final dimension migration decision open until measurement exists.
- `1024` may be operationally attractive but is still recall-unknown.
- `2048` may be quality-attractive but is still costlier in both bytes and per-candidate work.
- The repo now has to benchmark three operating points instead of optimizing blindly around one.

### Neutral
- Competitive comparisons remain useful as supporting context, but they are no longer the ADR's
  primary claim.
- The same reasoning applies to future dimensions beyond these three candidates.

## Required Validation

The following measurements are required before revisiting the default operating point:

1. **FWHT microbenchmarks**
   Report scalar vs SIMD at `1024`, `2048`, and `4096`, with tiled-FWHT results called out
   separately from bootstrap-only FWHT results.

2. **Prepared-query scoring benchmarks**
   Report `score_ip_encoded` and `score_ip_codes_lite` at `1024`, `1536`, and `2048` for the same
   bit-width and host.

3. **Storage accounting**
   Report payload bytes/vector, approximate tuples/page, and actual on-disk index bytes.

4. **End-to-end search quality**
   Report Recall@10 / Recall@100 / NDCG@10 on the same corpus and graph parameters across the three
   operating points.

5. **Build and insert cost**
   Report bulk-build throughput now, and future graph-aware insert throughput once that path exists.

## References

- FR-013: Two-Stage Vector Quantization Pipeline
- FR-014: SIMD Acceleration
- ADR-007: Persist gamma and use raw-query scoring for the high-quality search path
- ADR-017: HNSW over IVF — topology-agnostic indexing for heterogeneous data shapes
- TurboQuant: Online Vector Quantization with Near-optimal Distortion Rate (ICLR 2026)
