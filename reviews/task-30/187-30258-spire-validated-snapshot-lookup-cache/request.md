---
id: 30258
title: SPIRE Validated Snapshot Lookup Cache
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 8d64cc04
---

# Review Request: SPIRE Validated Snapshot Lookup Cache

## Summary

This checkpoint starts addressing A3/A6 from the foundation review.

- Adds `SpireValidatedEpochSnapshot`, built from a validated
  `SpirePublishedEpochSnapshot`.
- Builds a PID lookup cache mapping each PID to its manifest entry and
  placement entry.
- Switches scan root/leaf/delta collection helpers to consume the validated
  wrapper internally instead of rebuilding `SpirePublishedEpochSnapshot` inside
  each helper.
- Switches snapshot diagnostics to consume the validated wrapper.
- Adds metadata coverage proving PID lookup resolves manifest and placement
  entries.
- Updates Task 30 status to record that update/publication helpers still need
  migration before the item can close.

## Non-Goals

- Does not migrate update draft/publication helpers yet.
- Does not add relation-backed snapshot loading.
- Does not change strict/degraded placement semantics.
- Does not replace manifest/placement storage codecs.

## Review Focus

- Whether `SpireValidatedEpochSnapshot` is the right boundary for scan and
  diagnostics helpers.
- Whether the cached lookup should carry more precomputed state before live
  persistence, such as object kind or skip/available status.
- Whether the remaining update/publication migration should happen before or
  after V2 build/scan consumption.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 170 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
