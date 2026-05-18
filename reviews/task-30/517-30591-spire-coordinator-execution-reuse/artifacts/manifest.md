# Artifact Manifest

Packet: `30591-spire-coordinator-execution-reuse`
Head SHA: `b387b00a`

No packet-local measurement logs are attached. Validation was functional PG18
coverage rather than a benchmark or measurement claim.

## Commands

- Command: `cargo check --lib --no-default-features --features pg18`
  - Timestamp: 2026-05-07
  - Result: passed

- Command: `cargo pgrx test pg18 final_summary`
  - Timestamp: 2026-05-07
  - Result: passed
  - Key lines: 1 passed; 0 failed; 1444 filtered out
