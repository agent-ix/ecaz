# Review Request: DiskANN RaBitQ SQL Format Coverage

- task: `plan/tasks/60-ec-diskann-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- topic: `diskann-rabitq-sql-format-test`
- code checkpoint: `aec09c409b7d804919ec849f415eb89ad9e46dab`

## What Changed

- Added a focused `pg_test` in `src/am/ec_diskann/routine.rs` that builds
  `ec_diskann` indexes with both `storage_format = 'pq_fastscan'` and
  `storage_format = 'rabitq'`.
- The test asserts the metadata search-code discriminator and format-specific
  payload flags/codebook state.
- The test forces ordered SQL scans through each index and verifies the nearest
  row is returned first.
- Added a packet-local SQL smoke fixture,
  `artifacts/diskann-format-smoke.sql`, so backend validation can be rerun via
  `ecaz dev sql --file` instead of a large inline SQL command.

## Review Focus

- Is the metadata assertion enough to catch accidental fallback from RaBitQ to
  grouped-PQ payloads?
- Does the SQL smoke cover the build/scan path without overfitting to the tiny
  fixture?
- Are the retained diagnostic logs useful, or should future packets keep only
  the final file-based smoke?

## Validation

- `cargo check --no-default-features --features 'pg18 pg_test'`
  - passed; existing unrelated HNSW `unused_unsafe` warnings remain.
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
  - passed with sandbox approval for local pgrx install.
- `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --file reviews/task-60/004-diskann-rabitq-sql-format-test/artifacts/diskann-format-smoke.sql --log-output reviews/task-60/004-diskann-rabitq-sql-format-test/artifacts/ecaz-dev-sql-pg18-diskann-format-smoke-file.log`
  - passed; final status row is `task60_diskann_format_smoke_passed`.

Full 100k/1M recall and storage acceptance remains in the benchmark packet and
still needs the benchmark host run.
