# Task 59 Packet 001 Artifact Manifest

- task bucket: `reviews/task-59/001-relation-reader-parity/`
- head SHA: `1aae58aba833c027f8c3adc18d0484276d5376a7`
- date: 2026-05-24
- scope: Task 55 packet 005 H2 relation-reader parity gate

## Artifacts

### `cargo-check-pg18-pg-test.log`

- command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,pg_test" reviews/task-59/001-relation-reader-parity/artifacts/cargo-check-pg18-pg-test.log`
- result: passed
- key line: `Finished dev profile`

### `cargo-pgrx-test-relation-reader-parity.log`

- command: `script -q -c "cargo pgrx test pg18 test_ec_diskann_relation_reader_parity" /tmp/ecaz-relation-reader-pgrx.log`
- result: blocked before test dispatch by local pgrx loader
- key line: `undefined symbol: BufferBlocks`

### `cargo-pgrx-test-existing-diskann-sql-ordered.log`

- command: `script -q -c "cargo pgrx test pg18 test_ec_diskann_sql_ordered_index_scan_executes" /tmp/ecaz-existing-diskann-pgrx.log`
- result: blocked before test dispatch by the same local pgrx loader issue
- key line: `undefined symbol: BufferBlocks`

## Coverage Notes

The new `test_ec_diskann_relation_reader_parity` fixture covers:

- live PostgreSQL index creation,
- `RelationGraphReader::read_node` parity with `PersistedGraphReader`,
- `RelationGraphReader::first_live_tid` parity,
- scan-shell result parity between relation-backed and chain-backed readers.

The test is compile-checked under `pg18,pg_test`, but H2 should remain marked
open until the pgrx harness executes the test body successfully.
