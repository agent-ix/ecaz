# Review Request: A4 Score Function Divergence Between Build and Search

## Summary

The graph is built using one distance function but searched using a different one. If these produce incompatible orderings, the graph edges guide search in the wrong direction — explaining the 1.7% Recall@10 on a correctly-constructed graph.

## The divergence

**Build-time distance** (`build.rs:508`, `BuildCodeDistance::eval`):
```rust
fn eval(&self, va: &[u8], vb: &[u8]) -> f32 {
    self.score_offset - score_code_inner_product(self.dimensions, self.bits, self.seed, va, vb)
}
```
- Symmetric code-to-code scoring
- `score_offset` derived from `max_abs_centroid^2 * dimensions` — a global constant per index
- Used by hnsw-rs to build the graph (assign neighbors, select connections)

**Search-time scoring** (`scan.rs:1324-1335`, `score_scan_element_result`):
```rust
fn score_scan_element_result(opaque: &TqScanOpaque, gamma: f32, code_bytes: &[u8]) -> f32 {
    -quantizer.score_ip_from_parts(prepared_query, gamma, code_bytes)
}
```
- Asymmetric query-to-code scoring
- Uses `gamma` (a per-element f32 stored in each element tuple)
- Uses `prepared_query` (lookup tables precomputed from the raw float query)
- Negated for min-heap ordering

## Why this matters for A4

For HNSW search to work, the search scoring function must produce an ordering that is **compatible** with the ordering used to build the graph. The graph edges connect nodes that are close under the build distance. If the search distance ranks those same nodes differently, greedy descent follows the wrong edges and beam search expands the wrong candidates.

Specifically:
1. Build uses `score_offset - code_ip(a, b)` — a code-to-code function
2. Search uses `-quantizer.score_ip_from_parts(query, gamma, code)` — a query-to-code function with per-element gamma

If `gamma` varies significantly across elements, it shifts their relative scores in ways the graph edges don't account for. The graph was built ignoring gamma, but search uses gamma to rank candidates. This could cause the traversal to consistently prefer high-gamma nodes over actually-close nodes.

## Suggested investigation

1. **Check gamma variance**: Query the recall gate corpus for the distribution of `gamma` values. If gamma is near-constant, this isn't the bug. If it varies by >10%, it's a strong suspect.

2. **Score correlation test**: For a fixed query, compute both `score_offset - code_ip(query_code, element_code)` and `-score_ip_from_parts(prepared_query, gamma, code)` for all elements. Rank by each and compute Spearman correlation. If correlation is high (>0.95), the orderings are compatible and this isn't the root cause.

3. **Gamma-free search test**: Temporarily replace `gamma` with a constant (e.g., 0.0 or the mean gamma) in `score_scan_element_result` and re-run the recall gate. If recall jumps, gamma is the culprit.

## Files to read

- `src/am/build.rs:482-512` — `BuildCodeDistance` implementation
- `src/am/scan.rs:1324-1335` — `score_scan_element_result`
- `src/quant/prod.rs` — `ProdQuantizer::score_ip_from_parts` and `prepare_ip_query`
- `src/am/build.rs:596-662` — `build_hnsw_graph` (how hnsw-rs is invoked)

## Review focus

- Whether the build and search distance functions produce compatible orderings
- Whether `gamma` variance could explain the ~17x-over-random but still catastrophic recall
- Whether the `score_offset` in build and the negation in search are algebraically consistent
