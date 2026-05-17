# Artifact Manifest

- Head SHA: `7fc2eb9c`
- Packet/topic: `30361-spire-phase1-landing`
- Timestamp: `2026-05-03T17:14:19-07:00`
- Isolation: Rust unit tests only; no shared SQL table surface.
- Measurement claim: none.

## `scan_sanity_status_unit.log`

- Lane: PG18-feature Rust unit test
- Fixture: private `scan_sanity_status` helper states
- Storage format: not applicable
- Rerank mode: helper states cover empty, bounded-leaf, bounded-rerank, and
  full-frontier labels
- Command:
  `cargo test scan_sanity_status_reports_empty_approximate_and_full_scan --no-default-features --features pg18`
- Key result lines:
  - `test am::ec_spire::tests::scan_sanity_status_reports_empty_approximate_and_full_scan ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1129 filtered out; finished in 0.00s`

## `epoch_bundle_residue_unit.log`

- Lane: PG18-feature Rust unit test
- Fixture: synthetic epoch manifest rows
- Storage format: replacement epoch manifest snapshot
- Rerank mode: not applicable
- Command:
  `cargo test epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative --no-default-features --features pg18`
- Key result lines:
  - `test am::ec_spire::tests::epoch_snapshot_bundle_residue_keeps_previous_root_manifest_authoritative ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1129 filtered out; finished in 0.00s`

## `scan_root_cache_unit.log`

- Lane: PG18-feature Rust unit test
- Fixture: scan opaque root-control observations
- Storage format: scan-local root-control cache
- Rerank mode: not applicable
- Command:
  `cargo test scan_opaque_refreshes_root_control_on_every_rescan_observation --no-default-features --features pg18`
- Key result lines:
  - `test am::ec_spire::scan::tests::scan_opaque_refreshes_root_control_on_every_rescan_observation ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1129 filtered out; finished in 0.00s`
