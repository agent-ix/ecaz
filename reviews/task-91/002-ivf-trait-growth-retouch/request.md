# Task 91 Packet 002: IVF Trait-Growth Retouch

## Summary

This packet asks for review of Task 91 Phase 2. It retouches the IVF
`QuantCodec` adapter so IVF is the reference implementation for the grown
Task 91 trait contract before SPIRE, HNSW, and DiskANN migrate.

Code checkpoint under review:

- `4366618ad80de1f1a0a2fad65d9cb41d57050103` - `Retouch IVF QuantCodec model binding`

## Changes

- Added `IvfQuantCodec<'a>` as the model-bound IVF `QuantCodec` adapter.
  - Unbound TurboQuant and RaBitQ still work through `IvfQuantizer`'s existing
    trait impl.
  - Grouped-PQ/PqFastScan uses `IvfQuantizer::quant_codec_with_pq_model(...)`
    to bind persisted model bytes before `encode_source` or `prepare_ip_query`.
  - The common trait does not grow a model-specific method; model binding stays
    inside the concrete adapter as approved in Packet 001.
- Delegated the existing `impl QuantCodec for IvfQuantizer` through the unbound
  adapter to preserve old call sites.
- Added grouped-PQ candidate metadata validation so mismatched
  `CandidateMeta::GroupedPq { group_count }` fails before scoring.
- Locked scalar-tail counter attribution to `isa=Scalar` in the Task 91/92
  contracts, addressing Packet 001 feedback F1 and Task 92 feedback F2.
- Added per-shape `f32::to_bits()` parity tests required by Packet 001 feedback
  F2:
  - `TurboQuant`
  - `TurboQuantNoQjl4BitLut`
  - `RaBitQ`
  - `PqFastScan`

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `artifacts/git-diff-check.log`
  - `git diff --check`
  - result: passed with no output
- `artifacts/cargo-test-ivf-quantizer.log`
  - `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
  - result: `23 passed; 0 failed`

## Review Focus

- Confirm model binding belongs in `IvfQuantCodec<'a>` rather than a new
  model-aware method on the shared `QuantCodec` trait.
- Confirm the existing unbound `IvfQuantizer` trait impl remains compatible.
- Confirm grouped-PQ candidate metadata validation is the right early-failure
  behavior.
- Confirm the four prepared-query shapes have sufficient bit-exact parity
  coverage for Phase 2.
