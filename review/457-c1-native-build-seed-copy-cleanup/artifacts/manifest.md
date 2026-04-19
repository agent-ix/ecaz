# Manifest: 457-c1-native-build-seed-copy-cleanup

- Head SHA at validation time: `a344ad9`
- Packet: `457-c1-native-build-seed-copy-cleanup`
- Scope: native BUILD upper-layer seed copy removal
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - This packet only removes an unnecessary in-memory clone.
  - Persisted layout and user-visible SQL surfaces are unchanged.
