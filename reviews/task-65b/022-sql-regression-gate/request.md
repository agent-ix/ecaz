---
task: 65b
packet: 022-sql-regression-gate
role: coder
date: 2026-06-06
head: 05427d5cef90bcb36137174b9b0d36dbcfe98e69
status: review-requested
---

# Task 65b SQL Regression Gate

This packet adds the SQL-level PG18 DiskANN regression gate that was missing
from the closeout audit packet.

## Result

The authoritative rerun passed:

- Command: `script -q reviews/task-65b/022-sql-regression-gate/artifacts/cargo-pgrx-test-pg18-ec-diskann-escalated.log cargo pgrx test pg18 ec_diskann`
- Result: `199 passed; 0 failed; 0 ignored; 0 measured; 1777 filtered out`

This covers the Task 65b acceptance requirement that DiskANN insert and scan
tests not regress. The passing filter includes the `pg_test_ec_diskann_*`
routine tests for SQL build/scan, duplicate insert handling, planner behavior,
prefilter/session-list-size behavior, and vacuum repair behavior, along with
the pure Rust scan/insert/build/reader/vacuum tests.

## Sandbox note

The first unprivileged attempt is retained as an artifact because it explains
why the rerun needed escalation:

- Artifact: `artifacts/cargo-pgrx-test-pg18-ec-diskann.log`
- Failure cause: `cargo-pgrx` could not copy `ecaz.control` into
  `/opt/homebrew/share/postgresql@18/extension`; the pgrx mutex failures were
  downstream of that install failure, not code assertions.

After the passing pgrx test installed a test/dev extension, I restored the
release PG18 extension:

- Command: `./target/debug/ecaz --log-file reviews/task-65b/022-sql-regression-gate/artifacts/install-release-after-pgrx-test.log dev install ecaz-pg-test --pg 18`
- Result: backend artifact assertion passed, installed backend
  `/opt/homebrew/lib/postgresql@18/ecaz.dylib`, sha256
  `b206d0568414b689d5546103fa19d07ec533023f4b6c69b2e88a0af95452d097`.

## Review Ask

Please treat this as a narrow supplement to packet 021, closing the SQL-level
DiskANN regression evidence gap for Task 65b closeout.
