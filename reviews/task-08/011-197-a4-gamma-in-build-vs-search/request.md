# Review Request: A4 Gamma Term Absent From Build Distance

## Summary

Each element stores a `gamma: f32` that participates in search scoring via `score_ip_from_parts`. But the build-time distance function (`BuildCodeDistance::eval`) does not use gamma at all. If gamma is load-bearing for correct scoring, the graph was built with a distance function that disagrees with the one used to traverse it.

## Where gamma appears

**Element storage** (`page.rs`, `TqElementTuple`):
- Every element tuple stores `gamma: f32`
- Set during build from the quantizer output

**Search scoring** (`scan.rs:1334`):
```rust
-quantizer.score_ip_from_parts(prepared_query, gamma, code_bytes)
```
- `gamma` is passed directly to the quantizer's score function
- It shifts the final score by a per-element amount

**Build scoring** (`build.rs:508-512`):
```rust
impl Distance<u8> for BuildCodeDistance {
    fn eval(&self, va: &[u8], vb: &[u8]) -> f32 {
        self.score_offset - score_code_inner_product(self.dimensions, self.bits, self.seed, va, vb)
    }
}
```
- No gamma parameter. Only uses raw code bytes.
- `score_offset` is a **global** constant (max centroid norm), not per-element.

## What gamma represents

`gamma` appears to be a per-vector quantization residual or norm correction term from the product quantizer. In asymmetric distance estimation, the full inner product `<q, x>` is approximated as `gamma(x) + <q_lut, code(x)>` where `gamma` captures the component that the code alone cannot represent (e.g., the contribution of the norm or the residual from subspace rounding).

If this is correct, then:
- Build's `score_code_inner_product(code_a, code_b)` is computing `<code_a, code_b>` without any gamma correction — this is the code-to-code symmetric IP
- Search's `score_ip_from_parts(query, gamma, code)` is computing `gamma + <query_lut, code>` — the full asymmetric estimate

The gamma term would make two elements with identical codes but different original vectors score differently in search but identically in build. The graph connects them as if they're equal, but search prefers one over the other.

## Impact on recall

If gamma varies across elements (which it should, since different vectors have different norms and quantization residuals), then:
- Graph neighbors were selected without gamma → graph optimizes for code-only IP
- Search uses gamma → search follows a different gradient
- Greedy descent follows edges optimized for the wrong function
- Beam search expands candidates ranked by a different criterion than the graph was built for

This is consistent with 1.7% recall: the graph is structurally valid but optimized for a different distance, so traversal systematically walks away from the true nearest neighbors.

## Suggested investigation

1. **Gamma distribution**: Compute `min, max, mean, stddev` of gamma across the 10K recall corpus. If stddev is negligible relative to the code IP range, gamma is inert and this isn't the bug.

2. **Gamma-ablated recall test**: In `score_scan_element_result`, temporarily set gamma to 0.0 (or the corpus mean). Re-run the recall gate. If recall improves dramatically, gamma is the root cause.

3. **Build-with-gamma test**: Modify `BuildCodeDistance::eval` to incorporate element gamma terms (would require storing gamma alongside codes during build). Re-build and check recall.

4. **Trace the ProdQuantizer API**: Read `score_ip_from_parts` in `src/quant/prod.rs` to confirm whether gamma is additive, multiplicative, or something else.

## Files to read

- `src/quant/prod.rs` — `score_ip_from_parts`, `prepare_ip_query` — how gamma enters the score
- `src/am/build.rs:482-512` — `BuildCodeDistance` — confirms gamma is absent
- `src/am/scan.rs:1324-1335` — `score_scan_element_result` — confirms gamma is present
- `src/lib.rs` — `unpack` and `pack` — how gamma is set when creating tqvector values

## Review focus

- Confirm whether gamma is a load-bearing scoring term or merely a passthrough constant
- If load-bearing: quantify the expected rank-order distortion from omitting gamma in build
- Assess whether the fix belongs in the build distance function or in the search scoring function
