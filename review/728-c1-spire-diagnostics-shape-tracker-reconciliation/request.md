# Review Request: SPIRE Diagnostics And Shape Tracker Reconciliation

- agent: coder1
- date: 2026-05-14
- code commit: `99d9dcffeb520da45dc84240c24e828de4910b83`
- task rows: updates `12c.13.b`, `12c.14.a`, `12c.14.b`, `12c.14.c`,
  `12c.14.d`, `12c.14.f`, `12c.15.a`, `12c.15.b`, `12c.15.c`

## Summary

Tracker-only reconciliation for current split Phase 12c rows already covered by
earlier packets and accepted in batch-1 feedback.

This checkpoint intentionally leaves two partial bullets open:

- `12c.14.f` CustomScan wide-projection recall remains unchecked.
- `12c.15.a` per-store counter sum reconciliation remains unchecked.

## Evidence

### `12c.13.b`

- Packet `690-c1-spire-dropped-index-diagnostics`.
- `src/tests/diagnostics.rs:1579`
  - `test_ec_spire_dropped_index_snapshots_empty`
  - Drops a SPIRE index, calls the stale OID, and asserts `count(*) = 0` for:
    hierarchy, object, delta, health, leaf, placement, scan pipeline, top
    graph, allocator, and boundary-replica placement diagnostics.

### `12c.14.a` and `12c.14.b`

- Packet `685-c1-spire-scan-data-shape-edges`.
- `src/tests/scan.rs:1042`
  - `test_ec_spire_single_row_corpus_scan_returns_only_row`
  - Builds N=1, forces scan, compares against exact scoring, and asserts the
    one row returns.
- `src/tests/scan.rs:1076`
  - `test_ec_spire_duplicate_vector_corpus_scan_matches_exact_set`
  - Builds all-duplicate vectors, runs K=10, compares to exact top IDs, and
    asserts identical scores.

### `12c.14.c`

- Packet `691-c1-spire-numerical-edge-scan-coverage`.
- `src/tests/scan.rs:1119`
  - `test_ec_spire_numerical_extreme_vector_scan_matches_exact_set`
  - Covers subnormal and near-`f32::MAX` finite vectors with finite score
    assertions and exact-set comparison.
- `src/tests/scan.rs:1163`
  - `test_ec_spire_non_finite_vector_inserts_rejected`
  - Asserts NaN, `+Infinity`, and `-Infinity` inserts fail explicitly.

### `12c.14.d`

- Packet `697-c1-spire-text-nul-projection-boundary`.
- `src/tests/data_shape.rs:2`
  - `test_ec_spire_text_projection_nul_byte_rejected_sql`
  - Documents the unsupported PostgreSQL `text` NUL boundary with an explicit
    error assertion and zero inserted rows.

### `12c.14.f`

- Packet `699-c1-spire-wide-typed-payload-projection`.
- `src/tests/remote_search/tuple_heap.rs:541`
  - `test_ec_spire_typed_tuple_payload_wide_projection_sql`
  - Builds 32 projected `text` columns and asserts exact typed transport
    metadata/value cardinalities without truncation.
- Remaining unchecked: the CustomScan-level wide projection recall bullet.

### `12c.15.a`, `12c.15.b`, `12c.15.c`

- Packet `693-c1-spire-multistore-scan-widths`.
- `src/tests/scan.rs:468`
  - `test_ec_spire_three_store_scan_width_sql`
  - Builds with 3 local stores and asserts the scan harness touches all 3.
- `src/tests/scan.rs:477`
  - `test_ec_spire_four_store_scan_width_sql`
  - Builds with 4 local stores and asserts the scan harness touches all 4.
- Remaining unchecked for `12c.15.a`: full per-store route/candidate/byte sum
  reconciliation.
- Packet `689-c1-spire-local-store-execution-snapshot`.
- `src/tests/scan.rs:486`
  - `test_ec_spire_scan_local_store_execution_mode_standalone_sql`
  - Pins `sequential_backend` and
    `async_or_parallel_store_group_executor`.

Batch-1 reviewer feedback at
`review/31080-spire-phase12c-batch1-feedback/feedback/2026-05-14-001-reviewer.md`
records these packets under `12c.13.b`, `12c.14.a-b`, `12c.14.c`, `12c.14.d`,
`12c.14.f`, and `12c.15.a-c`.

## Changes

- Checked all `12c.13.b` bullets.
- Checked all `12c.14.a`, `12c.14.b`, `12c.14.c`, and `12c.14.d` bullets.
- Checked two of three `12c.14.f` bullets; left CustomScan recall open.
- Checked two of three `12c.15.a` bullets; left per-store counter sums open.
- Checked all `12c.15.b` and `12c.15.c` bullets.
- No test code changed in this checkpoint.

## Validation

- `git diff --check -- plan/tasks/task30-phase12c-spire-test-coverage.md`
  - Passed.
- No compile or runtime test was run for this tracker-only checkpoint; the
  request points to existing reviewed test evidence only.

## Review Focus

- Confirm the two intentionally open bullets are the right residual work.
- Confirm the cited existing tests satisfy the checked current-tracker bullets.
