# Manifest: 450-c1-native-build-stability-tightening

- Head SHA at validation time: `2301fca`
- Packet: `450-c1-native-build-stability-tightening`
- Scope: native serial builder stability and invariant tightening
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - This packet removes redundant serial descent work but does not attempt a
    wider build-time benchmark or recall re-baseline.
  - The final `cargo test` pass cited for this packet was rerun in isolation
    after a parallel validation attempt conflicted with the pg17 wrapper’s
    extension rebuild/install step.
