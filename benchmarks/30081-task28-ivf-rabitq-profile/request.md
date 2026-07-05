# Task 28 IVF RaBitQ Quantizer Profile Checkpoint

## Summary

This packet records the A8 RaBitQ sub-slice at head
`e803579b6edbc8f47a1e3a9f470eec83c88ffac1`.

`ec_ivf` now accepts `storage_format = 'rabitq'` and routes build,
scan, insert duplicate checks, and vacuum payload decoding through the
persisted storage format's quantizer payload length. The `auto` default
remains TurboQuant.

This is not full A8 closure. `storage_format = 'pq_fastscan'` remains
rejected until IVF has a real grouped-codebook persistence path.

## Scope

- Added `IvfQuantizerProfile::RaBitQ`.
- Added `IvfPreparedQuery::RaBitQ`.
- Routed RaBitQ encode/query-prep/scoring through the existing
  `crate::quant::rabitq::RaBitQQuantizer`.
- Changed IVF build, insert, and vacuum helpers to derive posting payload
  length from `metadata.storage_format` instead of assuming TurboQuant.
- Added a PG18 smoke covering RaBitQ IVF build, scan, live insert, and
  vacuum removal.

## Validation

Commands run:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::quantizer --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_rabitq_storage_build_scan_insert_vacuum`
- `cargo pgrx test pg18 test_ec_ivf_insert`
- `cargo pgrx test pg18 test_ec_ivf_vacuum`
- `git diff --check`

Results:

- IVF quantizer unit tests: 6 passed.
- IVF unit tests: 44 passed.
- PG18 RaBitQ storage smoke: 1 passed.
- PG18 IVF insert tests: 6 passed.
- PG18 IVF vacuum tests: 4 passed.
- `git diff --check`: clean.

## Follow-Up

Continue A8 with PQ-FastScan. Unlike RaBitQ, PQ-FastScan needs IVF to
persist and reload grouped PQ codebooks; accepting the reloption before that
would only create a fake dispatch arm.
