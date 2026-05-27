# Task 63 Packet 007 Artifact Manifest

- head SHA: `8441806a59957a456cb11429860ad54a17133f51`
- task bucket: `reviews/task-63/007-hnsw-rabitq-pg18-lifecycle-smoke`
- timestamp: `2026-05-26T17:12:36-07:00`
- lane: HNSW RaBitQ lifecycle validation
- fixture: `task63_hnsw_rabitq`, 4-dimensional `ecvector` values with a matching `source real[]` build source column
- storage format: `rabitq`
- rerank mode: source-backed cold rerank payloads
- isolation: one-index-per-table SQL smoke surface

## Artifacts

### `cargo-test-hnsw-rabitq-lifecycle-no-run.log`

- command: `cargo test -q --lib test_ech_rabitq_build_scan_insert_vacuum_round_trip --no-run`
- result: pass
- key result: focused lifecycle pg_test compiled; warning output is existing unrelated unused/unsafe test-helper noise.

### `ecaz-dev-sql-pg18-hnsw-rabitq-lifecycle-setup.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-63/007-hnsw-rabitq-pg18-lifecycle-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-rabitq-lifecycle-setup.log --sql "<setup, build, indexed scan, live insert smoke>"`
- result: pass
- key result lines:
  - `CREATE INDEX`
  - `DO`
  - `task63_hnsw_rabitq_setup_insert_passed |       16384`

### `ecaz-dev-sql-pg18-hnsw-rabitq-delete.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-63/007-hnsw-rabitq-pg18-lifecycle-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-rabitq-delete.log --sql "<delete id 1 and count remaining rows>"`
- result: pass
- key result lines:
  - `DELETE 1`
  - `task63_hnsw_rabitq_delete_passed |              8`

### `ecaz-dev-sql-pg18-hnsw-rabitq-vacuum.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-63/007-hnsw-rabitq-pg18-lifecycle-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-rabitq-vacuum.log --sql "VACUUM task63_hnsw_rabitq;"`
- result: pass
- key result line: `VACUUM`

### `ecaz-dev-sql-pg18-hnsw-rabitq-post-vacuum.log`

- command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --log-output reviews/task-63/007-hnsw-rabitq-pg18-lifecycle-smoke/artifacts/ecaz-dev-sql-pg18-hnsw-rabitq-post-vacuum.log --sql "<force indexed post-vacuum scan and assert deleted row is absent>"`
- result: pass
- key result lines:
  - `DO`
  - `task63_hnsw_rabitq_post_vacuum_passed |       16384`

### `ecaz-dev-sql-pg18-hnsw-rabitq-lifecycle.log`

- command: exploratory combined lifecycle smoke using one multi-statement `psql -c` call.
- result: failed before final check because PostgreSQL rejects `VACUUM` inside the implicit transaction block for multi-statement `psql -c`.
- disposition: superseded by the split setup/delete/vacuum/post-vacuum logs above.
