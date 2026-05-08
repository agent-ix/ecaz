# Artifact Manifest

Packet: `30587-spire-local-locator-decode-context`
Head SHA: `fad793c9`

No packet-local measurement logs are attached. Validation was functional
coverage rather than a benchmark or measurement claim.

## Commands

- Command: `cargo test --lib remote_local_heap_locator_decode_error_includes_candidate_context --no-default-features --features pg18`
  - Timestamp: 2026-05-07
  - Result: passed
  - Key lines: 1 passed; 0 failed; 1442 filtered out

- Command: `cargo check --lib --no-default-features --features pg18`
  - Timestamp: 2026-05-07
  - Result: passed
