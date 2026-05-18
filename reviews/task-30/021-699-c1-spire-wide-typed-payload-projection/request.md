---
agent: coder1
role: coder
model: gpt-5
date: 2026-05-14
topic: c1-spire-wide-typed-payload-projection
code_commit: 96cf8d43
---

# Review Request: Wide Typed Payload Projection

## Summary

Added a focused PG18 pg_test coverage pin for the typed tuple payload endpoint's
wide projection path:

- `test_ec_spire_typed_tuple_payload_wide_projection_sql` builds a table with
  32 projected `text` columns plus an `ecvector` index.
- The test requests all 32 payload columns through
  `ec_spire_remote_search_tuple_payload_typed`.
- It asserts the exact payload width, names, metadata array cardinalities,
  binary values, null bitmap, formats, transport status, and ready status.

This covers the typed-transport width subcase of task 12c.14.f while keeping the
existing tuple transport test file under the target file-size ceiling
(`src/tests/remote_search/tuple_heap.rs`: 985 lines after this slice).

## Scope

Changed:

- `src/tests/remote_search/tuple_heap.rs`

Not covered by this slice:

- The remaining 12c.14.f CustomScan-level wide projection and recall@k fixture.
- Large string payload coverage from 12c.14.e.

## Validation

Passed:

- `cargo fmt --check`
  - Stable rustfmt emitted the repository's existing warnings about nightly-only
    `imports_granularity` and `group_imports`.
- `git diff --check -- src/tests/remote_search/tuple_heap.rs`
- `cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_typed_tuple_payload_wide_projection_sql --no-run`

## Review Focus

Please check whether the endpoint-level wide projection assertion is tight
enough for this subcase, especially the generated SQL expectations for payload
names and `textsend(...)` bytea values.
