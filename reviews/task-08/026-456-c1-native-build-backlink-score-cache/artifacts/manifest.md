# Manifest: 456-c1-native-build-backlink-score-cache

- Head SHA at validation time: `fcfffd0`
- Packet: `456-c1-native-build-backlink-score-cache`
- Scope: native BUILD backlink rescoring reduction
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - This packet only reduces repeated in-memory scoring during native BUILD.
  - Persisted layout and user-visible SQL surfaces are unchanged.
