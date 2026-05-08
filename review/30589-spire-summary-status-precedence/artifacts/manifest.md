# Artifact Manifest

Packet: `30589-spire-summary-status-precedence`
Head SHA: `3934d872`

No packet-local measurement logs are attached. Validation was functional
coverage rather than a benchmark or measurement claim.

## Commands

- Command: `cargo check --lib --no-default-features --features pg18`
  - Timestamp: 2026-05-07
  - Result: passed

- Command: `cargo test --lib remote_summary_status_helper_preserves_precedence --no-default-features --features pg18`
  - Timestamp: 2026-05-07
  - Result: passed
  - Key lines: 1 passed; 0 failed; 1444 filtered out
