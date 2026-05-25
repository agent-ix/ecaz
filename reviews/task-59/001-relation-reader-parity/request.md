# Task 59 Packet 001 - Relation Reader Parity Gate

Status: **proposed**

Addresses Task 55 packet 005 reviewer hard ask H2 as the first Task 59
checkpoint before spending more AWS time.

## Scope

- Adds a focused `#[pg_test]` fixture:
  `test_ec_diskann_relation_reader_parity`.
- Updates Task 59 to require the final AWS Graviton suite across 10k, 50k,
  100k, and 1M after optimizations and Graviton profile/config choices are
  settled.
- Carries forward reviewer gates H1 and H2 directly into the task file.

## Test Coverage Added

The new test builds a real PostgreSQL `ec_diskann` index and then:

- materializes the same index into a `DataPageChain`,
- reads every live node through both `PersistedGraphReader` and
  `RelationGraphReader`,
- asserts node tuples are field-equal,
- asserts `first_live_tid()` matches,
- runs the pure scan shell with both readers and asserts identical result sets.

This directly covers the default relation-backed reader path introduced by
`cbf037334ce0a9f499507d206049574b8278282e`.

## Validation

Passed:

- `cargo fmt --all`
- `cargo check --all-targets --no-default-features --features pg18,pg_test`

Blocked locally:

- `cargo pgrx test pg18 test_ec_diskann_relation_reader_parity`

The focused pgrx command compiles, but the local harness exits before test
dispatch with:

```text
undefined symbol: BufferBlocks
```

To confirm this was not caused by the new test body, I reran an existing
DiskANN pg_test:

- `cargo pgrx test pg18 test_ec_diskann_sql_ordered_index_scan_executes`

It fails with the same `BufferBlocks` loader error before dispatch. The parity
test is therefore present and compile-checked, but H2 is not fully closed until
the local/CI pgrx harness runs it inside PostgreSQL.

## Artifacts

- `artifacts/cargo-check-pg18-pg-test.log`
- `artifacts/cargo-pgrx-test-relation-reader-parity.log`
- `artifacts/cargo-pgrx-test-existing-diskann-sql-ordered.log`
- `artifacts/manifest.md`

## AWS State

No AWS teardown or direct SSM was used for this checkpoint. The `10k` profile
was checked through `ecaz cloud status` and remains running.
