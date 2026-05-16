# Review Request: SPIRE Typed Tuple Tracker Reconciliation

- agent: coder1
- date: 2026-05-14
- code commit: `d055dca360b32fcf9683c99a93225bda87734118`
- task rows: closes `12c.12.a`, `12c.12.b`, `12c.12.c`

## Summary

Tracker-only reconciliation for current split Phase 12c typed tuple rows.

Packet `680-c1-spire-typed-tuple-edge-coverage` added the missing typed tuple
edge coverage, and batch-1 reviewer feedback accepted `12c.12.a-c` via that
packet. This checkpoint updates only
`plan/tasks/task30-phase12c-spire-test-coverage.md`.

## Evidence

### `12c.12.a`

- `src/tests/remote_search/tuple_heap.rs:92`
  - `test_ec_spire_typed_tuple_payload_scalar_parity_sql`
  - Calls `ec_spire_remote_search_tuple_payload_typed` with
    `ARRAY[]::text[]`.
  - Asserts `payload_column_count = 0`.
  - Asserts `payload_names`, type OIDs, typmods, collations, nulls, values, and
    formats are all empty arrays.
  - Asserts `tuple_transport = 'pg_binary_attr_v1'` and ready statuses.

### `12c.12.b`

- `src/tests/remote_search/tuple_heap.rs:458`
  - `test_ec_spire_typed_tuple_payload_composite_only_sql`
  - Creates `ec_spire_typed_pair_only` without a domain wrapper.
  - Asserts typed metadata arrays for `id` and the composite column.
  - Asserts the composite value bytes equal `record_send(...)`.

### `12c.12.c`

- `src/tests/remote_search/tuple_heap.rs:280`
  - `test_ec_spire_typed_tuple_payload_null_array_wire_bytes_sql`
  - Captures `NULL::text[]` and empty `text[]` rows through typed transport.
  - Asserts the NULL array row has `payload_nulls = ARRAY[false, true]`.
  - Asserts the NULL array value bytes are `''::bytea` and not
    `array_send(ARRAY[]::text[])`.
  - Asserts the non-NULL empty array row has `payload_nulls =
    ARRAY[false, false]` and value bytes equal `array_send(ARRAY[]::text[])`,
    not `''::bytea`.

Batch-1 reviewer feedback at
`review/31080-spire-phase12c-batch1-feedback/feedback/2026-05-14-001-reviewer.md`
records `12c.12 Typed Tuple Transport | 12.a-c via 680`.

## Changes

- Checked the two `12c.12.a` bullets.
- Checked the two `12c.12.b` bullets.
- Checked the three `12c.12.c` bullets.
- No test code changed in this checkpoint.

## Validation

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- No compile or runtime test was run for this tracker-only checkpoint; the
  request points to existing reviewed test evidence only.

## Review Focus

- Confirm the cited tuple transport tests satisfy the current split tracker
  rows.
- Confirm no additional tracker text is needed for these typed tuple rows.
