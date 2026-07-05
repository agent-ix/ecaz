# Task 28 IVF PQ-FastScan Profile Checkpoint

## Scope

This packet covers commit `5463234` (`ivf: wire pq fastscan storage profile`).

The checkpoint wires `storage_format = 'pq_fastscan'` through the IVF v1 path:

- accepts the IVF `pq_fastscan` reloption;
- persists grouped PQ codebooks in IVF data pages and records the codebook head/group size in metadata;
- defers ecvector/tqvector posting payload encoding until grouped codebook training is available;
- loads persisted grouped codebooks for scan query preparation;
- re-encodes live inserts against the persisted grouped model;
- adds unit coverage for grouped codebook tuple layout and quantizer dispatch;
- replaces the old unsupported-storage PG test with a PG18 build/scan/insert/vacuum smoke.

## Validation

Commands run on this checkpoint:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::quantizer --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf::page::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_storage_build_scan_insert_vacuum`
- `cargo pgrx test pg18 test_ec_ivf_insert`
- `cargo pgrx test pg18 test_ec_ivf_vacuum`
- `git diff --check`

All listed checks passed.

## Review Focus

Please review:

- whether the persisted IVF PQ codebook tuple chain and metadata fields are sufficient for rebuild-free scans/inserts;
- whether the deferred encoding shape is acceptable for both ecvector and tqvector inputs;
- whether the scan/insert model loading should cache the codebook model in a later slice;
- whether the PQ-FastScan smoke is enough for this checkpoint or should be expanded before the next benchmark slice.
