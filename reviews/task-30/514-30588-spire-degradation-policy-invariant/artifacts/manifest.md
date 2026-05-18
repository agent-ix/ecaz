# Artifact Manifest

Packet: `30588-spire-degradation-policy-invariant`
Head SHA: `cade462f`

No packet-local measurement logs are attached. Validation was functional
coverage rather than a benchmark or measurement claim.

## Commands

- Command: `cargo check --lib --no-default-features --features pg18`
  - Timestamp: 2026-05-07
  - Result: passed

- Command: `cargo test --lib remote_degradation_policy_contract_matches_fanout_skip_decisions --no-default-features --features pg18`
  - Timestamp: 2026-05-07
  - Result: passed
  - Key lines: 1 passed; 0 failed; 1443 filtered out
