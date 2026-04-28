# Task 28 IVF PQ-FastScan Group Size Reloption

This checkpoint exposes `pq_group_size` as an IVF reloption for
`storage_format = 'pq_fastscan'`.

The decision is covered by the existing PQ/FastScan ADRs:

- ADR-030 defines grouped PQ4 / FastScan as a real grouped-code layout.
- ADR-032 says PqFastScan is a first-class format and that hard-coded
  group parameters should become metadata-driven.
- ADR-036 covers OPQ as a later training-heavy quality lever.
- ADR-038 covers later LSQ/codebook refinement.

This slice does not implement OPQ. It keeps the current SRHT grouped-PQ4
profile, but lets IVF build and persist different grouped-PQ subvector
sizes so the next measurements can compare `pq_group_size` values instead
of treating the original group size as fixed.

## Code

Commit: `3ec6638 ivf: expose pq fastscan group size`

Changes:

- Adds `pq_group_size` to `ec_ivf` reloptions.
- Persists the selected group size in IVF metadata through the existing
  `pq_group_size` field.
- Resolves scan, live insert, and vacuum payload lengths from metadata so
  non-default PQ group sizes remain readable after build.
- Adds unit coverage for group-size override validation.
- Adds a PG18 pgrx test that builds and scans a 32-dim IVF `pq_fastscan`
  index with `pq_group_size = 8`.

## Validation

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::quantizer --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_accepts_group_size_reloption`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_storage_build_scan_insert_vacuum`
- `git diff --check`

## Next Slice

Run a real-data sweep over `pq_group_size = 8, 16, 32` crossed with the
existing `rerank_width` frontier. This should happen before adding heavier
PQ quality work such as OPQ or LSQ.
