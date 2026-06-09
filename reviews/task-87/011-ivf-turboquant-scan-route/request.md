# Task 87 Packet 011: IVF TurboQuant Scan Route Reachability

## Summary

This packet asks for review of a narrow IVF reachability fix needed before
Task 87 Phase 6 off/on measurement.

Code checkpoint under review:

- `e0942a33d1a75ebc41094eee329820e9b6472742` - `Enable IVF TurboQuant CandidateBatch scan route`

Packet 004 added the IVF TurboQuant no-QJL 4-bit batch scoring helper, but
the scan-level scratch SoA gate was still RaBitQ-only. That made the
TurboQuant scan route unreachable even when `ec_ivf.scratch_soa_batch_decode`
was enabled.

## Changes

- Factored the scratch SoA gate into a format/bit helper.
- Admitted `StorageFormat::TurboQuant` with `quant_bits == 4`.
- Preserved the existing RaBitQ 1-bit and 8-bit scratch SoA behavior.
- Kept `StorageFormat::Auto` and `StorageFormat::PqFastScan` rejected.
- Added a focused unit test covering accepted and rejected combinations.

## Scope Note

This is not additional Task 87 cross-AM `QuantCodec` migration work. It only
makes the already-implemented IVF TurboQuant no-QJL 4-bit batch route
reachable through the existing per-AM scan path.

The downstream scoring dispatch still declines unsupported prepared-query
states, including non-no-QJL TurboQuant queries, so this gate does not broaden
the scoring semantics beyond the intended no-QJL 4-bit lane.

## Validation

See `artifacts/manifest.md` for packet-local log metadata.

- `artifacts/cargo-test-ivf-scan.log`
  - `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
  - result: `24 passed; 0 failed`
- `artifacts/cargo-test-ivf-quantizer.log`
  - `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
  - result: `17 passed; 0 failed`

## Review Focus

- Confirm the scan gate admits only the intended TurboQuant 4-bit scratch SoA
  surface plus the pre-existing RaBitQ lanes.
- Confirm `ec_ivf.scratch_soa_batch_decode` remains the session-level switch.
- Confirm this remains within the Task 87 walk-back guard from packet 009 and
  does not add HNSW/DiskANN `QuantCodec` work.
