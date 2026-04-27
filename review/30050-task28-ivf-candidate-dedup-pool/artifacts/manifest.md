# Artifact Manifest

Packet: `30050-task28-ivf-candidate-dedup-pool`

Head SHA: `dc1f36931a47eb383c6da76f8b6a11aae898bb3c`

Timestamp: `2026-04-27T13:10:37-07:00`

This packet introduces no measurement artifacts. It records allocation-pressure
work and focused PG18 validation only.

Validation commands:

- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
  - `6 passed; 0 failed`
- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib ec_ivf --no-default-features --features pg18`
  - `77 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`
