# Task 64 Packet 003 Artifact Manifest

- head SHA: `8c8577dae8eda13741031f91ab5fbb65f41dfca9`
- task bucket: `reviews/task-64/003-hnsw-codec-existing-format-smoke`
- timestamp: `2026-05-26T17:19:09-07:00`
- lane: HNSW codec adapter existing-format runtime validation
- fixture: two one-index-per-table 4-dimensional `ecvector` HNSW indexes
- storage formats: `turboquant`, `pq_fastscan`
- rerank mode: source-backed cold rerank payloads via `build_source_column = 'source'`
- isolation: one index per table, matched rows and reloptions

## Artifacts

### `ecaz-dev-sql-pg18-hnsw-existing-formats-setup.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-64/003-hnsw-codec-existing-format-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-existing-formats-setup.log --sql "<create TurboQuant/PqFastScan HNSW indexes, force index scans, insert live rows>"`
- result: pass
- key result lines:
  - `CREATE INDEX`
  - `DO`
  - `task64_hnsw_existing_formats_setup_insert_passed |                  16384 |          16384`

### `ecaz-dev-sql-pg18-hnsw-existing-formats-delete.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-64/003-hnsw-codec-existing-format-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-existing-formats-delete.log --sql "<delete id 1 from both tables and count remaining rows>"`
- result: pass
- key result lines:
  - `DELETE 1`
  - `task64_hnsw_existing_formats_delete_passed |                         6 |                 6`

### `ecaz-dev-sql-pg18-hnsw-existing-formats-vacuum-turboquant.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-64/003-hnsw-codec-existing-format-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-existing-formats-vacuum-turboquant.log --sql "VACUUM task64_hnsw_turboquant;"`
- result: pass
- key result line: `VACUUM`

### `ecaz-dev-sql-pg18-hnsw-existing-formats-vacuum-pq.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-64/003-hnsw-codec-existing-format-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-existing-formats-vacuum-pq.log --sql "VACUUM task64_hnsw_pq;"`
- result: pass
- key result line: `VACUUM`

### `ecaz-dev-sql-pg18-hnsw-existing-formats-post-vacuum.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-64/003-hnsw-codec-existing-format-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-existing-formats-post-vacuum.log --sql "<force post-vacuum indexed scans and assert deleted rows are absent>"`
- result: pass
- key result lines:
  - `DO`
  - `task64_hnsw_existing_formats_post_vacuum_passed |                  16384 |          16384`
