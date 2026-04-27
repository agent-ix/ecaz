# Artifact Manifest

Packet: `30049-task28-ivf-quantizer-dispatch`

Head SHA: `0e9202d35cdb80d7b96e0f13225da0cd872bfcd6`

Timestamp: `2026-04-27T13:07:23-07:00`

This packet introduces no measurement artifacts. It records substrate wiring
and focused PG18 validation only.

Validation commands:

- `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib am::ec_ivf::scan::tests --no-default-features --features pg18`
  - `6 passed; 0 failed`
- `cargo test --lib ec_ivf --no-default-features --features pg18`
  - `77 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`
