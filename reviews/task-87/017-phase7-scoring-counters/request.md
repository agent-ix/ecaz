# Task 87 Phase 7 Scoring Counters

## Scope

This packet is a Phase 7 instrumentation slice for the reopened 32-block kernel work. It responds to the packet 015 seq 02 requirement for direct scoring-share counters before final closeout.

Code checkpoint:

- `76df28d44e64a8d951d923700654991240193c4d` - `Add Task 87 scoring counters`

What changed:

- Added shared `CandidateBatch` scoring counters for `spire`, `ivf`, `hnsw`, and `unknown`.
- Counters track batch scorer flushes, candidates, elapsed nanos, LUT32 flushes, and LUT32 candidates.
- Routed SPIRE, IVF, and HNSW TurboQuant no-QJL 4-bit batch calls through explicit per-AM attribution.
- Added SQL diagnostics:
  - `ec_task87_candidate_batch_scoring_reset()`
  - `ec_task87_candidate_batch_scoring_snapshot()`
- Added focused unit coverage for counter attribution and reset behavior on a 39-candidate block-plus-tail batch.

## Validation

Packet-local logs:

- `artifacts/cargo-test-candidate-batch.log`
  - `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
  - Result: 4 passed; 0 failed.
- `artifacts/cargo-test-quant-lut32.log`
  - `cargo test --lib quant::lut32 --no-default-features --features pg18`
  - Result: 2 passed; 0 failed.

## Review Notes

This is not the final Phase 7 closeout. It makes the scoring-share measurement surface available for the required real-corpus reruns and HNSW batch-width decision. The final closeout packet still needs the superseding aggregate matrix with these counters included.
