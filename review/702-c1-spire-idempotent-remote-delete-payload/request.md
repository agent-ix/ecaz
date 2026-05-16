---
agent: coder1
role: coder
model: gpt-5
date: 2026-05-14
topic: c1-spire-idempotent-remote-delete-payload
code_commit: 9603ff82
---

# Review Request: Idempotent Remote Delete Payload Shape

## Summary

Added a focused PG18 pg_test for the remote delete primitive's idempotent
response shape:

- New file: `src/tests/dml_frontdoor_delete.rs`.
- Included from `src/tests/mod.rs` so DELETE-specific DML coverage does not
  further grow the already-large `src/tests/dml_frontdoor.rs`.
- `test_ec_spire_remote_delete_tuple_payload_idempotent_shape_sql` creates a
  one-row SPIRE table, calls `ec_spire_remote_delete_tuple_payload` twice for
  the same bigint PK, and asserts:
  - first call returns `deleted_count=1|ready`;
  - second call returns `deleted_count=0|ready`;
  - the heap row remains absent after the re-delete.

This tightens the 12c.9.e idempotent delete response-shape coverage at the
remote primitive boundary. The existing coordinator test already covers missing
placement and stale local placement no-op rows; this slice pins the direct
remote endpoint shape separately.

## Scope

Changed:

- `src/tests/dml_frontdoor_delete.rs`
- `src/tests/mod.rs`

File-size note:

- `src/tests/dml_frontdoor_delete.rs`: 47 lines.
- `src/tests/dml_frontdoor.rs`: left unchanged at 2570 lines.

## Validation

Passed:

- `cargo fmt --check`
  - Stable rustfmt emitted the repository's existing warnings about nightly-only
    `imports_granularity` and `group_imports`.
- `git diff --check -- src/tests/mod.rs src/tests/dml_frontdoor_delete.rs`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_remote_delete_tuple_payload_idempotent_shape_sql --no-run`

## Review Focus

Please check whether this should be treated as a 12c.9.e tightening slice, and
whether future DELETE coverage should continue landing in the new split file.
