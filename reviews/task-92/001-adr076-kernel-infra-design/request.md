# Task 92 Packet 001: ADR-076 Kernel Infrastructure Design

## Summary

This packet asks for review of the Task 92 Phase 1 design gate. It drafts
ADR-076 and walks the seven in-scope quant families through the proposed
kernel skeleton. It does not change Rust runtime code.

Design checkpoint under review:

- `5cdcf38a529ddec50665a4ea44b806f03383897f` - Task 87 merge baseline

## Changes

- Added proposed ADR:
  - `spec/adr/ADR-076-universal-block-kernel-pattern.md`
- Added skeleton fit audit:
  - `artifacts/skeleton-fit-audit.md`
- Added counter and suite contract:
  - `artifacts/counter-and-suite-contract.md`
- Added packet manifest:
  - `artifacts/manifest.md`

## Decisions

- Universal block width: 32 candidates.
- Kernel dispatch entry point: `QuantCodec::score_ip_batch`, matching the Task
  91 Phase 1 recommendation.
- Module layout:
  - `src/quant/<kernel>/mod.rs`
  - `src/quant/<kernel>/scalar.rs`
  - `src/quant/<kernel>/neon.rs`
  - `src/quant/<kernel>/sve.rs`
  - `src/quant/<kernel>/avx2.rs`
- Runtime ISA enum:
  - `Scalar`
  - `Neon`
  - `Sve`
  - `Sve2`
  - `Avx2`
- ARM target: AWS Graviton 4 (Neoverse V2, SVE2 at 128-bit vector length).
  SVE/SVE2 kernels must be vector-length agnostic. Graviton 4 packets use the
  `Sve2` dispatch branch when available and report the measured runtime vector
  length verbatim, for example `sve2-128`.
- Correctness:
  - scalar reference is strict `to_bits()` where the current scorer is
    deterministic;
  - SIMD tolerance is <= 4 ULP or `1e-6` relative, with recall@k preservation
    as the bench-level gate.

## Validation

See `artifacts/manifest.md` for artifact metadata.

No tests were run. This is the design-only Phase 1 packet required before Task
92 implementation starts.

## Review Focus

- Confirm ADR-076 locks the right common kernel contract for Tasks 93-98.
- Confirm the Graviton 4/SVE2 wording is correct and avoids assuming a fixed
  vector width without measurement.
- Confirm the seven-family skeleton audit covers the intended rollout surface.
- Confirm Task 92 should wait for Task 91 Phase 2 grouped-PQ model binding
  before implementation.
