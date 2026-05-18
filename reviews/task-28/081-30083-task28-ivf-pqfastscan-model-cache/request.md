# Task 28 IVF PQ-FastScan Scan Model Cache

## Scope

This packet records commit `2d31046` (`ivf: cache pq fastscan scan model`).

The previous PQ-FastScan IVF scan path loaded the persisted grouped-codebook
model from index pages every time `amrescan` prepared a query. This checkpoint
caches the loaded `IvfPqFastScanModel` on the scan opaque and reuses it across
rescans of the same index scan descriptor. The model is freed in `amendscan`.

This is a local scan-prep quality/performance cleanup for the PQ-FastScan
variant. It does not change the on-disk format, scoring semantics, or default
storage format.

## Validation

Commands run:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_scan_reuses_loaded_model`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_storage_build_scan_insert_vacuum`
- `git diff --check`

All listed checks passed.

## Notes

The new PG18 regression test creates a small PQ-FastScan IVF index, runs two
rescans through one scan descriptor, and verifies the scan opaque keeps the
same loaded grouped-codebook model pointer across both rescans.

No benchmark claim is made in this packet, so there are no raw measurement
logs.
