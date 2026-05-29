# Task 66 packet 002: prefetch unsafe cleanup

## Summary

This packet tightens the Slice E safety shape from packet 001. The aarch64
`prfm pldl1keep` helper is now a safe private function with the inline
assembly contained in a documented `unsafe` block. That removes the extra
non-kernel `unsafe fn` introduced by the prefetch change while preserving the
bits=1 and bits=8 NEON prefetch behavior.

This is intentionally narrow: no scoring math changed.

## Validation

- `cargo test --lib --no-default-features --features pg18 quant::rabitq`
  - `41 passed; 0 failed`
  - Log: `artifacts/cargo-test-quant-rabitq.log`
- `cargo check --no-default-features --features pg18`
  - passed
  - Log: `artifacts/cargo-check-pg18.log`

## Review notes

- Scope is only `src/quant/rabitq.rs`.
- This packet is a follow-up to packet 001, which contains the main M5 NEON
  implementation and Criterion measurement logs.
- The remaining Task 66 completion gaps are measurement/acceptance-gate gaps,
  not this safety cleanup: packet 001 still does not contain fresh local recall
  deltas for all four sidecar variants, and its bits=1 batch measurement does
  not demonstrate the stated 2x throughput gate.
