# Manifest: 455-c1-native-build-query-score-cache

- Head SHA at validation time: `e15624b`
- Packet: `455-c1-native-build-query-score-cache`
- Scope: native BUILD repeated-score reduction
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - This packet changes only the in-memory native BUILD search path.
  - Persisted layout and user-visible SQL surfaces are unchanged.
