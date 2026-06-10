# Manifest: Task 93 Phase 1 RaBitQ32 Design

- head SHA: `2102cd61b566cebde861598eb85f0e24729a6099`
- task bucket: `reviews/task-93`
- packet path: `reviews/task-93/001-rabitq32-design`
- timestamp: `2026-06-09T16:13:12Z`; revised `2026-06-09T19:50:00Z`
- lane: Task 93 / RaBitQ popcount block-kernel design
- fixture: none
- storage format: RaBitQ
- rerank mode: not applicable
- isolated/shared surface: not applicable; design packet only

## Artifacts

### `artifacts/rabitq32-design.md`

- command used: manual source/spec review; no test or benchmark command run
- key cited lines:
  - ADR-076 requires `QuantCodec::score_ip_batch`, 32-candidate width gating,
    backend-returned ISA attribution, and scalar tails under `Isa::Scalar`.
  - Existing RaBitQ scalar reference is
    `PreparedEstimator::estimate_ip_scalar_only` /
    `PreparedEstimator::estimate_ip_bits1_batch` /
    `RaBitQScorer::score`.
  - Design keeps per-candidate RaBitQ scalar metadata
    (`candidate_norm`, `candidate_o_dot`, `candidate_x_norm`) in the finish
    step after the block bit/popcount stage.
  - Revision after reviewer feedback anchors strict `f32::to_bits()` parity to
    the forced-scalar bits=1 byte-LUT path and uses ADR-076 tolerance against
    production-dispatched NEON/AVX2/FMA scorers.
  - Revision moves `CandidateBatchScoringSurface::Diskann` into Phase 2.

### `feedback/2026-06-09-01-reviewer.md`

- command used: reviewer source audit; no test or benchmark command run
- key cited lines:
  - architecture approved;
  - original strict `to_bits()` parity against production-dispatched RaBitQ
    scorers requested changes;
  - recommended contract adopted in this packet revision.

## Validation

No tests or benchmarks were run. Phase 1 is design-only by instruction.
