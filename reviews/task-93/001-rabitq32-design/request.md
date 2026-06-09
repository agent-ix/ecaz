# Task 93 Phase 1 Review Request: RaBitQ32 Design

## Summary

This is the design-only checkpoint for Task 93. No source code changes are in
this packet.

The proposed kernel family is `src/quant/rabitq32/` with the ADR-076 module
layout and a 32-candidate block width. The design routes through
`QuantCodec::score_ip_batch`, records whole-block rows under the ISA returned
by `score_block32_<isa>`, and records scalar tails under
`(surface, rabitq, scalar)` via `record_block_scalar_score_for`.

## Design Highlights

- Scalar reference is the existing RaBitQ scorer:
  `score_ip_bits1_batch_from_payloads` / `estimate_ip_bits1_batch` for IVF and
  `estimate_ip_scalar_only` / `RaBitQScorer::score` for other AMs.
- Phase 2 scalar parity is strict `f32::to_bits()` equality against the
  forced-scalar bits=1 RaBitQ reference; production-dispatched
  `estimate_ip_scalar_only` / `estimate_ip_bits1_batch` comparisons use the
  ADR-076 tolerance because normal hosts select NEON/AVX2/FMA scorers.
- SIMD variants use exact popcount diagnostics for the bit stage, then apply
  the existing per-candidate scalar correction fields in candidate order.
- NEON uses `vcntq_u8`; SVE/SVE2 stays vector-length agnostic and must report
  measured vector length in the first real SVE2 evidence packet; AVX2 uses
  nibble-LUT `vpshufb` plus `_mm256_sad_epu8` rather than AVX-512 VPOPCNTDQ.
- Width gating is explicit: batches `<32` scalar-only, whole 32-candidate
  blocks through the kernel, tails scalar.
- DiskANN counter attribution is now resolved for Phase 2: add
  `CandidateBatchScoringSurface::Diskann` before registering DiskANN batch
  scoring, rather than routing through `Unknown`.

## Feedback Response

This revision resolves
`feedback/2026-06-09-01-reviewer.md` by adopting the recommended parity
contract:

- `to_bits()` equality is anchored to the forced-scalar byte-LUT path.
- Production-dispatched per-candidate and batch scorers are checked under the
  ADR-076 f32 tolerance.
- Phase 2 includes a one-shot check of existing production per-candidate vs
  batch scorer agreement on the target host.
- DiskANN gets a real counter surface in Phase 2.

## Artifacts

- `artifacts/rabitq32-design.md`
- `artifacts/manifest.md`

## Validation

No tests or benchmarks were run. This phase is design-only by request.

## Reviewer Ask

Please re-review the scalar parity contract before Phase 2 implementation.
