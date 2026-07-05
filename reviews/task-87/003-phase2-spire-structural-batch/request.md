# Task 87 Packet 003: SPIRE Structural CandidateBatch Route

## Summary

This packet is the Phase 2 SPIRE structural batching checkpoint from the
Task 87/002 revised plan.

The code adds a shared safe `CandidateBatch` surface under
`src/am/common/candidate_batch.rs` and routes SPIRE's TurboQuant no-QJL
4-bit batch scoring branch through it. The existing generic TurboQuant
and RaBitQ paths stay on their prior inline loops.

## Code

- `src/am/common/candidate_batch.rs`
- `src/am/common/mod.rs`
- `src/am/ec_spire/quantizer/mod.rs`

## Behavior

- For SPIRE TurboQuant with `no_qjl_4bit_lut = Some(_)`,
  `SpirePreparedAssignmentScorer::score_batch_ip` now builds a
  `CandidateBatch` of row-order ids plus borrowed payload chunks and
  flushes it through `score_turboquant_no_qjl_4bit_batch`.
- The scorer still uses the same exact
  `score_ip_from_parts_lut_no_qjl_4bit` function as before, so this is a
  structural route, not a new performance kernel.
- Generic TurboQuant modes still use `score_ip_from_parts`.
- RaBitQ still uses `PreparedEstimator::estimate_ip_scalar_only`.

## Validation

Packet-local logs:

- `artifacts/cargo-test-candidate-batch.log`
- `artifacts/cargo-test-spire-quantizer.log`

Commands:

```text
cargo test --lib am::common::candidate_batch --no-default-features --features pg18
cargo test --lib am::ec_spire::quantizer::tests --no-default-features --features pg18
```

Results:

- CandidateBatch tests: 2 passed, 0 failed.
- SPIRE quantizer tests: 12 passed, 0 failed.

## Not Yet Claimed

This packet does **not** claim final Phase 2 acceptance. The Task 87
real-corpus `ecaz bench suite` evidence is still required before the
SPIRE slice can be accepted as complete. Per Task 87/002, this
structural slice is expected to be byte-equal and non-regressing rather
than a `>= 2x` scoring-share win.

## Review Focus

- Confirm the shared `CandidateBatch` lifetime shape is safe Rust and
  matches the revised Phase 1 contract.
- Confirm only SPIRE's TurboQuant no-QJL 4-bit LUT branch routes through
  `CandidateBatch`.
- Confirm generic TurboQuant and RaBitQ scoring behavior remains
  unchanged.
