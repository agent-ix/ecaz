# 30352 SPIRE Heterogeneous Concurrency

## Request

Review the PG18 coverage added for SPIRE concurrent insert, SQL VACUUM, and
scan activity against the same active epoch stream.

## Scope

- Added `test_pg18_ec_spire_concurrent_insert_vacuum_scan`.
- Added `spawn_psql_commands` so SQL `VACUUM` runs as its own `psql -c`
  command while still sharing one external session setup.
- Updated Task 30 status to record the heterogeneous-concurrency coverage.

## Behavior Covered

The test builds a one-list `ec_spire` index, creates one post-build insert
delta, deletes the original heap row, then releases three external sessions
through the same advisory barrier:

- an insert worker publishing another post-build delta
- a SQL `VACUUM` worker competing for the publish path
- a scan worker forcing index-routed reads while writers advance epochs

After all workers complete, the test asserts:

- the heap has the two live rows
- root control reaches the expected fourth active epoch
- PID and local vec-id allocators advance once for the concurrent insert
- leaf plus delta assignment accounting remains coherent for either
  insert-before-vacuum or vacuum-before-insert ordering
- the deleted row is absent from an index-routed query
- both live rows are returned by an index-routed query

## Validation

- `cargo fmt`
- `cargo test --lib test_pg18_ec_spire_concurrent_insert_vacuum_scan --no-default-features --features pg18 -- --nocapture`
- `git diff --check`

