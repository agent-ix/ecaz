# Manifest: 451-c1-native-build-heuristic-tests

- Head SHA at validation time: `8a1ca68`
- Packet: `451-c1-native-build-heuristic-tests`
- Scope: native BUILD helper regression coverage
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - This packet adds heuristic regression coverage only; it does not alter
    runtime behavior or claim new recall measurements.
  - The final `cargo test` result for this packet comes from a clean standalone
    rerun after the pg17 wrapper invalidated an earlier parallel run.
