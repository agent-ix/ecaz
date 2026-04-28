# Artifact Manifest

Head SHA: `3ec6638`

Packet: `review/30086-task28-ivf-pqfastscan-group-size`

Timestamp: 2026-04-28 00:40 America/Los_Angeles

This is a code checkpoint packet, not a benchmark packet. No measurement
claims are made here.

Validation commands:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::quantizer --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_accepts_group_size_reloption`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_storage_build_scan_insert_vacuum`
- `git diff --check`
