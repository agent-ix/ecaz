---
id: 30252
title: SPIRE amrescan Query Parsing
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 318768df
---

# Review Request: SPIRE amrescan Query Parsing

## Summary

This checkpoint wires the first live `amrescan` behavior for SPIRE without
loading relation-backed partition objects yet.

- Validates scan shape: no index quals and exactly one ORDER BY key.
- Decodes the ORDER BY datum as a `real[]` query vector.
- Stores the decoded query as `SpireScanQuery` in scan opaque state.
- Fails explicitly at the next unimplemented boundary:
  relation-backed published snapshot/object loading.

## Non-Goals

- No relation-backed SPIRE object persistence or loading.
- No scan plan resolution from loaded root metadata yet.
- No heap exact rerank implementation.

## Review Focus

- Whether the live `amrescan` validation matches the existing IVF scan contract.
- Whether it is acceptable to parse and store the query before failing at the
  persistence-loading boundary.
- Whether the new error boundary is clear enough for the next scan wiring slice.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 163 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
