# Review Request: SPIRE TurboQuant LUT Scoring

## Summary

This checkpoint applies the useful query-LUT finding to a real TQ-backed index surface.

IVF already routes no-QJL 4-bit TurboQuant through `ProdQuantizer::score_ip_from_parts_lut_no_qjl_4bit`, and HNSW exposes `full_lut` / `tiled_lut` scan modes. SPIRE assignment scoring was still using the generic `PreparedQuery` path, which leaves the no-QJL 4-bit prepared LUT empty and scores by direct codebook multiply.

The change:

- Adds an optional `PreparedLutNoQjl4BitQuery` to `SpirePreparedAssignmentScorer::TurboQuant`.
- Prepares it only when `ProdQuantizer::exact_score_mode() == ExactScoreMode::MseNoQjl4Bit`.
- Uses the LUT scorer for single payload scoring, zero-gamma max chunk scoring, and batch scoring.
- Leaves QJL-active and non-4-bit TQ lanes on the generic scorer.
- Does not change SPIRE storage format, payload bytes, page layout, or query semantics.

## Evidence

Artifact manifest: `reviews/task-86/005-spire-tq-lut/artifacts/manifest.md`

Focused validation:

```text
cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_spire::quantizer::tests::turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path -- --nocapture
```

Result:

```text
test am::ec_spire::quantizer::tests::turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1978 filtered out; finished in 0.03s
```

## Interpretation

This is the lowest-risk index-facing Task 86 improvement found so far. It does not require adopting TQ+ storage, and it aligns SPIRE with the no-decompression packed-code query-LUT scoring path already available in the quantizer and IVF.

The next benchmark should use `ecaz bench suite` on a SPIRE TurboQuant lane to verify recall parity and measure whether assignment scoring latency improves in the real scan path.

## Review Focus

- Whether all SPIRE TurboQuant scoring entry points now consistently take the LUT path when eligible.
- Whether the optional prepared query state is acceptable for SPIRE scorer memory and lifecycle.
- Whether a SPIRE suite lane should be the first index-level benchmark before attempting a TQ+ storage/profile extension.
