---
id: 30246
title: SPIRE Scan Opaque Lifecycle
agent: coder1
status: open
created: 2026-05-02
checkpoint_commit: 118e3239
---

# Review Request: SPIRE Scan Opaque Lifecycle

## Summary

This checkpoint adds SPIRE scan callback state without loading any
relation-backed partition objects yet.

- Adds `SpireScanOpaque` with query vector, resolved scan plan, and candidate
  cursor state.
- Wires `ambeginscan` to allocate the opaque state in PostgreSQL memory.
- Wires `amgettuple` to drain a populated cursor into heap TID plus ORDER BY
  score output.
- Wires `amendscan` to drop the Rust-owned opaque state.
- Leaves `amrescan` relation-backed snapshot loading and persistence reads
  explicitly unwired.

## Non-Goals

- No relation-backed SPIRE object persistence or loading.
- No heap-row exact rerank implementation.
- No query parsing or scan-plan resolution in `amrescan` yet.

## Review Focus

- Whether the opaque state owns the right minimal scan-lifecycle fields.
- Whether the `PgBox` allocation/drop pattern is appropriate for a state struct
  containing Rust-owned `Vec` values.
- Whether `amgettuple`'s cursor-drain behavior matches the expected future
  callback contract.

## Validation

- `cargo test --lib ec_spire --no-default-features --features pg18`
  - 160 passed; 0 failed
- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    nightly-only `imports_granularity` and `group_imports`.
- `cargo fmt --check`
  - Completed with the same rustfmt warnings.
- `git diff --check`
- `git diff --cached --check`
