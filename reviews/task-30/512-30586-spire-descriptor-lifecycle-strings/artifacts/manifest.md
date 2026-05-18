# Artifact Manifest

Packet: `30586-spire-descriptor-lifecycle-strings`
Head SHA: `2dbeb51d`

No packet-local measurement logs are attached. Validation was functional PG18
coverage rather than a benchmark or measurement claim.

## Commands

- Command: `cargo check --lib --no-default-features --features pg18`
  - Timestamp: 2026-05-07
  - Result: passed

- Command: `cargo pgrx test pg18 remote_node_descriptor`
  - Timestamp: 2026-05-07
  - Result: passed
  - Key lines: 3 passed; 0 failed; 1439 filtered out
