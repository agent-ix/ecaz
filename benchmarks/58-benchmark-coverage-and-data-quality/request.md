# Review: Benchmark Coverage and Data Quality

Scope:
- `benches/` (all criterion, iai, dhat files)
- `benches/helpers.rs`
- `tests/recall_integration.rs`
- `tests/proptest_quant.rs`
- `tests/proptest_page.rs`
- `tests/size_of_assertions.rs`
- `src/lib.rs` (bench_api exports)

What was built:
- 8 criterion benchmark suites, 3 iai-callgrind suites, 2 dhat profiling binaries
- 13 proptest properties, 13 size_of assertions, recall integration harness
- 4 fuzz targets, miri tests in source modules

Review focus:
- Functions that exist in the codebase but have no benchmark or test coverage
- Whether synthetic test data is representative of real-world embedding workloads
- Whether recall benchmarks can produce falsely optimistic results

---

## 1. Missing function coverage

### 1a. Scoring variants not benchmarked

`score_ip_from_parts` (prod.rs:164) and `score_ip_encoded_lite` (prod.rs:203) have no
criterion or iai benchmarks. `score_ip_from_parts` is the entry point used when gamma and
code bytes arrive separately (the page-read path during scan). `score_ip_encoded_lite` is
the payload-to-payload variant. Both are hot-path functions.

**Action:** Add criterion benchmarks for both in `benches/criterion/quant_score.rs`,
same parameterization as the existing `score_ip_encoded` and `score_ip_codes_lite` benches.
Add iai-callgrind coverage for `score_ip_from_parts` in `benches/iai/quant_score.rs`.

### 1b. `decode_approximate` not benchmarked

`ProdQuantizer::decode_approximate` (prod.rs:122) reconstructs an approximate vector from
a payload. This is used during HNSW graph traversal for re-ranking. No benchmark exists.

**Action:** Add a criterion benchmark in `benches/criterion/quant_score.rs` (or a new
`quant_decode.rs` if preferred). Test at 1536/4-bit and 3072/4-bit. Report
`Throughput::Elements(1)`.

### 1c. DataPage insert/read operations have zero benchmark coverage

The page codec benchmarks (`benches/criterion/page_codec.rs`) only test
`TqElementTuple::encode`/`decode` and `TqNeighborTuple::encode`/`decode` in isolation.
The actual `DataPage` and `DataPageChain` operations — `insert_element`, `insert_neighbor`,
`read_element`, `read_neighbor`, `insert_raw_tuple`, `update_raw_tuple` — are not benchmarked.

These are the operations the build path executes thousands of times per second. The page
insert path includes offset bookkeeping, capacity checks, and tuple packing that adds
overhead beyond raw encode/decode.

**Action:** Add a `bench_page_insert_read` group to `benches/criterion/page_codec.rs`:
- `DataPage::insert_element` + `read_element` roundtrip at code_len 192, 768
- `DataPage::insert_neighbor` + `read_neighbor` at count 16, 32
- `DataPageChain::insert_element` filling multiple pages (measure chain rollover cost)

These types are already exported in `bench_api`. Add `neighbor_slots` and
`neighbor_tuple_encoded_len` to the bench_api exports if needed.

### 1d. `score_code_inner_product` not tested

`score_code_inner_product` (lib.rs:156) is the `pub(crate)` function backing the SQL `<#>`
operator. It constructs a `ProdQuantizer` internally and calls through to `score_ip_codes_lite`.
While the inner function is benchmarked, the wrapper adds `ProdQuantizer::cached` lookup
overhead that isn't measured.

**Action:** Add a benchmark or at minimum a unit test that exercises `score_code_inner_product`
directly. If it's just a thin wrapper over `cached` + `score_ip_codes_lite`, a unit test
confirming result equivalence is sufficient — no need for a separate criterion bench.

---

## 2. Synthetic data is not representative for recall testing

### 2a. All vectors are uniform random on the hypersphere

Every vector in the entire benchmark suite is generated the same way:
```rust
let mut values: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect();
let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
for value in &mut values { *value /= norm; }
```

This produces vectors uniformly distributed on the unit hypersphere with independent
dimensions. This is the **easiest case** for quantization:
- Maximum entropy — codebook bins are evenly utilized
- No inter-dimension correlation — SRHT rotation has nothing to decorrelate
- No clusters — all vectors are roughly equidistant from each other

Real embedding models (OpenAI, Cohere, sentence-transformers) produce vectors that are:
- **Clustered** — semantically similar documents map to nearby regions
- **Correlated** — dimensions are not independent; PCA on real embeddings shows rapid
  eigenvalue decay
- **Non-uniformly distributed** — angular density varies significantly across the sphere

**Impact:** Recall numbers from uniform data will be systematically higher than production.
This gives false confidence in quantization quality, especially at low bitwidths (2-3 bit)
where codebook resolution matters most.

### 2b. No near-duplicate stress test

Production workloads frequently contain near-duplicate vectors (same document with minor
edits, versioned content, augmented data). These are the hardest case for quantization
because small angular differences can be destroyed by lossy compression. No test currently
evaluates whether ranking is preserved among near-duplicates.

### 2c. No non-unit vectors tested

All test vectors are L2-normalized. If a user passes unnormalized vectors (which is valid —
some embedding models don't normalize), the quantization pipeline may behave differently.
The `encode` path normalizes internally, but the interaction between input scale and
codebook range is untested.

**Action for 2a-2c:** Add realistic data generators to `benches/helpers.rs`:

```rust
/// Generate a corpus of clustered unit vectors (Gaussian mixture model).
/// `n_clusters` cluster centers are placed randomly, then `n / n_clusters`
/// vectors are generated per cluster with `spread` controlling angular dispersion.
pub fn random_clustered_corpus(dim: usize, n: usize, n_clusters: usize, spread: f32, seed: u64) -> Vec<Vec<f32>>

/// Generate pairs of near-duplicate vectors at controlled angular distances.
/// Returns (base_vectors, perturbed_vectors) where each perturbed vector is
/// `angle_radians` away from its corresponding base vector.
pub fn near_duplicate_pairs(dim: usize, n: usize, angle_radians: f32, seed: u64) -> (Vec<Vec<f32>>, Vec<Vec<f32>>)
```

Then update `tests/recall_integration.rs`:
- Add `quantizer_recall_clustered_10k` using `random_clustered_corpus` with 50 clusters,
  spread 0.3. This is the primary recall realism improvement.
- Add `quantizer_recall_near_duplicates` that generates 1000 pairs at angular distance 0.05
  radians and checks whether the quantized ranking preserves the true nearest neighbor
  within each pair.
- Keep the existing uniform tests as a baseline for comparison.

---

## 3. Recall integration test improvements

### 3a. Missing Recall@1

The recall harness computes Recall@10 and Recall@100 but not Recall@1. For many production
use cases (RAG, deduplication), top-1 accuracy is the metric that matters. It's also the
most sensitive to quantization error.

**Action:** Add Recall@1 to the metrics computed in `recall_integration.rs`.

### 3b. Corpus sizes are small

The main recall test uses 10K corpus / 50 queries. At 10K vectors in 1536 dimensions,
the angular separation between vectors is still large — quantization errors rarely flip
rankings. Real deployments are 100K-10M vectors where the candidate pool is much denser.

The `#[ignore]` tests can't go to 1M without being impractical, but 50K would be a
meaningful step up and should complete in under 60 seconds.

**Action:** Increase the primary recall benchmark from 10K to 50K corpus, 100 queries.
Keep the bitwidth and dimension sweep tests at 1K for speed.

---

## 4. Minor gaps

### 4a. proptest doesn't test non-power-of-2 dimensions

The SRHT path pads to the next power of 2 via `transform_dim`. proptest generates dimensions
in `2..512` but doesn't specifically target dimensions like 1536 (pads to 2048) or 3072
(pads to 4096). The pad/unpad boundary is where off-by-one errors hide.

**Action:** Add a proptest case that specifically uses dimensions from
`prop_oneof![Just(1536), Just(768), Just(384), Just(1024)]` for the SRHT roundtrip test.
These are the real-world dimensions that exercise the padding path.

### 4b. No proptest for `decode_approximate` roundtrip

There's a miri test for encode/decode roundtrip at dim=8, but no property test that checks
`decode_approximate(pack_payload(encode(v)))` approximates `v` across a range of dimensions
and bitwidths, with a bounded error tolerance.

**Action:** Add a proptest: for random unit vector `v`, encode → pack → decode_approximate
should produce a vector whose cosine similarity with `v` exceeds a threshold (e.g., 0.8
for 4-bit, 0.6 for 2-bit). This catches regressions in the full encode/decode pipeline.

### 4c. iai-callgrind missing `score_ip_from_parts`

The iai suite benchmarks `score_ip_encoded` and `score_ip_codes_lite` but not
`score_ip_from_parts`, which is the actual call site during scan. If there's overhead
in the parts-based entry point (e.g., extra slice validation), it won't show up in CI
regression tracking.

**Action:** Add `score_ip_from_parts` to `benches/iai/quant_score.rs`.

---

## Summary of required actions

| # | Priority | Action | Files |
|---|----------|--------|-------|
| 1a | High | Bench `score_ip_from_parts` and `score_ip_encoded_lite` | `benches/criterion/quant_score.rs`, `benches/iai/quant_score.rs` |
| 1b | High | Bench `decode_approximate` | `benches/criterion/quant_score.rs` or new file |
| 1c | High | Bench `DataPage` insert/read operations | `benches/criterion/page_codec.rs` |
| 2a | High | Add `random_clustered_corpus` generator | `benches/helpers.rs` |
| 2a | High | Add clustered recall test | `tests/recall_integration.rs` |
| 2b | Medium | Add near-duplicate recall stress test | `benches/helpers.rs`, `tests/recall_integration.rs` |
| 3a | Medium | Add Recall@1 metric | `tests/recall_integration.rs` |
| 3b | Medium | Increase primary corpus to 50K | `tests/recall_integration.rs` |
| 4a | Low | Add real-world dimensions to SRHT proptest | `tests/proptest_quant.rs` |
| 4b | Low | Add `decode_approximate` proptest | `tests/proptest_quant.rs` |
| 4c | Low | Add `score_ip_from_parts` to iai suite | `benches/iai/quant_score.rs` |
| 1d | Low | Test `score_code_inner_product` equivalence | `tests/` or inline unit test |

bench_api export additions needed: `neighbor_slots`, `neighbor_tuple_encoded_len`,
`score_code_inner_product` (change to `pub` or add bench-gated re-export).

## Implementation Handoff

Status after second follow-up slice:
- Already addressed:
  - `1a`: direct benchmark coverage for `score_ip_from_parts` and `score_ip_encoded_lite`
  - `1b`: direct benchmark coverage for `decode_approximate`
  - `1c`: DataPage insert/read element + neighbor benchmarks in `page_codec.rs`
  - `1d`: direct equivalence coverage for `score_code_inner_product`
  - `2a`: `random_clustered_corpus` generator in `benches/helpers.rs` + clustered recall tests
  - `2b`: `near_duplicate_pairs` generator + near-duplicate ranking preservation test
  - `3a`: Recall@1 added to all recall reports
  - `3b`: Primary uniform recall test bumped from 10K to 50K corpus
  - `4a`: added real-world SRHT roundtrip property coverage
  - `4b`: `decode_approximate_bounded_error` proptest with cosine similarity thresholds
  - `4c`: iai coverage for `score_ip_from_parts`
- Still open:
  - `2c`: non-unit vector testing (low priority — encode normalizes internally)

All benchmarks validated (criterion --quick runs pass, all tests green, clippy clean).

Methodology constraints for ongoing work:
- Pre-generate corpora, queries, payloads, and page fixtures outside timed benchmark closures.
- Treat uniform-corpus recall results as optimistic upper bounds; clustered results are more representative.
- Prefer shared data generators in `benches/helpers.rs` over duplicating synthetic-data logic in tests or benches.
