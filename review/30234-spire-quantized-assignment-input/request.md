---
id: 30234
title: SPIRE Quantized Assignment Input
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 08e2efc9
---

# Review Request: SPIRE Quantized Assignment Input

## Summary

This checkpoint adds a production helper for turning a heap row locator plus a
source vector into a SPIRE leaf assignment input with encoded scoring payload
bytes.

- Adds `encode_assignment_input(...)`.
- Reuses the existing SPIRE assignment payload encoder for TurboQuant/RaBitQ
  payload bytes.
- Sets the assignment input's payload format tag, gamma, heap TID, and encoded
  payload in one place.
- Rejects invalid heap TIDs before assignment-row allocation.
- Updates routed scan tests to use the production helper instead of local test
  assembly for quantized assignment inputs.

## Non-Goals

- No AM build or insert callback wiring.
- No relation-backed persistence.
- No PQ-FastScan input helper support beyond the existing explicit deferral.

## Review Focus

- Whether `quantizer.rs` is the right owner for this helper, or whether it
  should move into `assign.rs`/`build.rs` when build callback wiring starts.
- Whether invalid heap TID validation should live here as well as in the row
  allocator, or remain allocator-only.
- Whether the helper should preserve the encoded vector dimension for later
  diagnostics, even though `LeafAssignmentRowV1` does not currently store it.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 147 passed, 0 failed
- `cargo fmt`
- `cargo fmt --check`
- `git diff --check`
- `git diff --cached --check`

`cargo fmt` and `cargo fmt --check` emitted the existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`.
