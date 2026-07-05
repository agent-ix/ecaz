# Task 87 Packet 004: IVF Structural CandidateBatch Route

## Summary

This packet asks for review of the IVF structural slice for Task 87. It routes the `ec_ivf` TurboQuant no-QJL 4-bit posting scratch path through the shared `CandidateBatch` abstraction introduced in packet 003, while preserving the existing scalar scoring semantics.

Code checkpoint under review:

- `a382d756f23f6c20e12dfb03eb34608223ffa3fa` - `Route IVF TurboQuant scoring through CandidateBatch`

## Changes

- Added `IvfQuantizer::score_turboquant_no_qjl_4bit_batch_from_payloads`, which:
  - only accepts `IvfPreparedQuery::TurboQuantNoQjl4BitLut`;
  - builds a borrowed `CandidateBatch` from the posting SoA payload/gamma arrays;
  - delegates scoring to `score_turboquant_no_qjl_4bit_batch`;
  - declines non-no-QJL compatible profiles and preserves the existing prepared-query mismatch error.
- Updated `process_scratch_soa_postings` to try the TurboQuant no-QJL batch helper before the existing RaBitQ batch helper, then record candidates using the unchanged score-to-distance sign conversion.
- Added a unit test proving the IVF no-QJL batch helper returns the same scores as the existing scalar `score_ip_from_parts` path.

This is a structural packet. It does not claim the final Task 87 real-corpus acceptance gate or the real batch-kernel performance target; those remain for the later validation/kernel packets described in packet 002.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `artifacts/cargo-test-ivf-quantizer.log`
  - `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
  - result: `14 passed; 0 failed`
- `artifacts/cargo-test-ivf-scan.log`
  - `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
  - result: `23 passed; 0 failed`

## Review Focus

- Confirm the IVF no-QJL route is correctly scoped to `TurboQuantNoQjl4BitLut`.
- Confirm using `CandidateMeta::Gamma` is acceptable for this structural pass even though the no-QJL helper ignores gamma today.
- Confirm the scan-loop branch ordering is appropriate before the existing RaBitQ batch path.
