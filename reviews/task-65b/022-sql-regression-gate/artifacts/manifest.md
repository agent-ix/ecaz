# Task 65b packet 022 artifact manifest

- Task bucket: `reviews/task-65b/022-sql-regression-gate`
- Head SHA under validation: `05427d5cef90bcb36137174b9b0d36dbcfe98e69`
- Timestamp: `Sat Jun  6 14:48:31 UTC 2026`
- Lane: local PG18 / pgrx home `/Users/peter/.pgrx`
- Purpose: SQL-level DiskANN insert/scan/build/vacuum regression gate for
  Task 65b closeout.

## Unprivileged pgrx attempt

- Artifact: `cargo-pgrx-test-pg18-ec-diskann.log`
- Command:
  `script -q reviews/task-65b/022-sql-regression-gate/artifacts/cargo-pgrx-test-pg18-ec-diskann.log cargo pgrx test pg18 ec_diskann`
- Result: failed because `cargo-pgrx` could not install the test extension:
  `Operation not permitted (os error 1)` while writing
  `/opt/homebrew/share/postgresql@18/extension/ecaz.control`.
- Interpretation: environment/sandbox failure. The subsequent pgrx test mutex
  failures were downstream of this extension-install failure.

## Escalated pgrx gate

- Artifact: `cargo-pgrx-test-pg18-ec-diskann-escalated.log`
- Command:
  `script -q reviews/task-65b/022-sql-regression-gate/artifacts/cargo-pgrx-test-pg18-ec-diskann-escalated.log cargo pgrx test pg18 ec_diskann`
- Result:
  `199 passed; 0 failed; 0 ignored; 0 measured; 1777 filtered out; finished in 47.55s`.
- Notes:
  - The command installed the PG18 test extension successfully after escalation.
  - Subsequent filtered binaries reported zero-test successful summaries.
  - The run covers `pg_test_ec_diskann_*` SQL routine tests plus the DiskANN
    Rust unit filter.

## Release extension restore

- Artifact: `install-release-after-pgrx-test.log`
- Command:
  `./target/debug/ecaz --log-file reviews/task-65b/022-sql-regression-gate/artifacts/install-release-after-pgrx-test.log dev install ecaz-pg-test --pg 18`
- Result: backend artifact assertion passed; installed backend
  `/opt/homebrew/lib/postgresql@18/ecaz.dylib`; sha256
  `b206d0568414b689d5546103fa19d07ec533023f4b6c69b2e88a0af95452d097`.
