# Manifest: 453-c1-native-build-helper-coverage

- Head SHA at validation time: `920ef8e`
- Packet: `453-c1-native-build-helper-coverage`
- Scope: native BUILD helper regression coverage
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - This packet only adds deterministic helper tests; it does not change
    runtime behavior.
  - The final `cargo test` result cited here comes from a clean standalone
    rerun after the pg17 wrapper invalidated an earlier parallel run.
