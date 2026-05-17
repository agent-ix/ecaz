## Feedback: ADR-030 v2 Grouped Code Generation Seam

Read `train_build_grouped_pq_model`, `derive_grouped_search_code_from_source`, and
`encode_grouped_pq` in `src/am/build.rs`.

### What's right

- Splitting training (`train_build_grouped_pq_model`) from per-row encoding
  (`derive_grouped_search_code_from_source` → `encode_grouped_pq`) means each step is
  independently testable. Training is the expensive-but-rare step; encoding is the
  hot-and-repeated one.
- SRHT rotate happens before per-group k-means, matching what packet 311 measured.
  Good that we're not discovering a train/score shape mismatch at integration time.

### Concerns

1. **Duplicate encoder (again).** `encode_grouped_pq` in `build.rs` and the one in
   `src/bin/approx_score_study.rs` must produce identical packed nibbles. They are
   not guaranteed to — there's no test comparing their output. Before the scorer
   packet lands, add a cross-crate test or (better) move to one shared encoder in a
   module both can import.

2. **Nearest-centroid search.** The inner loop is per-row, per-group linear scan over
   16 centroids. That's fine at `group_size = 16` but grows linearly in `k` if that
   constant ever moves. The `228-encode-nearest-centroid-branchless` packet (earlier
   feedback directory exists) suggests this was already identified. Worth making sure
   that optimization is applied here before real builds — a batched build on a large
   corpus will be bottlenecked on this if left unoptimized.

3. **Training data provenance.** `train_build_grouped_pq_model` uses the source
   column. Packet 319 then builds from the source column too. That's correct but means
   v2 cannot be built over an index-only input (no source column) without the training
   path being rewritten. Worth documenting that invariant in the ADR so nobody tries
   to lift the `build_source_column` requirement without also addressing training.

### Testable claim

The model-training deterministic seed is not visible in the packet or in the constants.
If the training is seeded from a constant, a corpus-scale recall regression test can
be added that asserts spearman stays above a threshold. If it's time-seeded, that test
will flake. Worth checking / making deterministic.
