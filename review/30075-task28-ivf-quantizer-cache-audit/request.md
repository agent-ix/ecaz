# Task 28 IVF Quantizer Cache Audit

## Scope

This packet records the A5 follow-up for Task 28: audit that `ProdQuantizer::cached` survives across IVF scans on the same index key.

The code checkpoint is `4e232f5210a4c3e862ad79860cc7f02b5be79f25` (`ivf: audit quantizer cache reuse`).

## What Changed

- Added a test-only `ProdQuantizer::cached_ptr(dim, bits, seed)` helper so regression tests can observe cache identity without exposing internals in production builds.
- Added a PG test/debug IVF helper that reads IVF metadata and reports the cached quantizer pointer for the exact key used by current IVF TurboQuant dispatch: `(dimensions, DEFAULT_QUANT_BITS, DEFAULT_QUANT_SEED)`.
- Added `test_ec_ivf_rescan_reuses_cached_prod_quantizer`, which runs two IVF rescan preparations against the same index and asserts the same cached `ProdQuantizer` entry remains in use.

This is a correctness/regression audit packet, not a timing measurement packet.

## Validation

Validated on PG18-focused paths:

- `cargo fmt --check`
- `cargo test --lib cached_quantizer_reuses_instances --no-default-features --features pg18`
- `cargo test --lib cached_with_presence_reports_whether_entry_already_existed --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_rescan_reuses_cached_prod_quantizer`
- `git diff --check`

## Result

A5 is complete for the current IVF quantizer path: the regression test proves consecutive IVF rescans on one index reuse the same cached `ProdQuantizer` allocation for the dispatch key currently used by IVF.

No DiskANN work is included in this packet.
