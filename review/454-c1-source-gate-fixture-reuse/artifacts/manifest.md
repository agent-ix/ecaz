# Manifest: 454-c1-source-gate-fixture-reuse

- Head SHA at validation time: `bc93123`
- Packet: `454-c1-source-gate-fixture-reuse`
- Scope: source-build recall gate harness hardening
- Validation commands:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- Validation date: `2026-04-19`
- Measurement status: no new measurement claim in this packet
- Notes:
  - This packet does not alter production BUILD behavior.
  - It makes the ignored source-build parity lane reusable and cheaper to rerun.
