---
agent: coder1
role: coder
model: gpt-5
date: 2026-05-14
topic: c1-spire-large-typed-payload-text
code_commit: 5935d229
---

# Review Request: Large Typed Payload Text Projection

## Summary

Added a focused PG18 pg_test coverage pin for a 1 MiB text projection through
the typed tuple payload endpoint:

- `test_ec_spire_typed_tuple_payload_large_text_projection_sql` builds a two-row
  table with a `body text` projection column and an `ecvector` SPIRE index.
- The hot row stores `repeat('x', 1048576)`.
- The typed endpoint is queried for `id, body` and asserts exact attnums,
  names, type OIDs, NULL flags, binary formats, `int8send` bytes for `id`, a
  1 MiB `body` byte length, byte-for-byte `textsend(repeat(...))` equality, and
  ready transport/status fields.

This keeps `src/tests/remote_search/tuple_heap.rs` at 1048 lines after the
slice, well below the 2500-line target.

## Scope

Changed:

- `src/tests/remote_search/tuple_heap.rs`

This is partial coverage for 12c.14.e:

- Covered: endpoint-level typed transport preserves a 1 MiB projected `text`
  value without truncation.
- Not covered: CustomScan-level success up to
  `ec_spire.max_remote_payload_bytes_per_row`.
- Not covered: over-cap failure returning
  `SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE`.

## Validation

Passed:

- `cargo fmt --check`
  - Stable rustfmt emitted the repository's existing warnings about nightly-only
    `imports_granularity` and `group_imports`.
- `git diff --check -- src/tests/remote_search/tuple_heap.rs`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_typed_tuple_payload_large_text_projection_sql --no-run`

## Review Focus

Please check whether this endpoint-level assertion is useful enough to keep as
the first 12c.14.e pin before adding the harder CustomScan cap fixture.
