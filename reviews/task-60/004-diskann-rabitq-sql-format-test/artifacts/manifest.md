# Artifact Manifest

- head SHA: `aec09c409b7d804919ec849f415eb89ad9e46dab`
- task bucket: `reviews/task-60/`
- packet path: `reviews/task-60/004-diskann-rabitq-sql-format-test/`
- timestamp: `2026-05-26`

| artifact | command | result |
| --- | --- | --- |
| `cargo-check-pg18-pg-test.log` | `cargo check --no-default-features --features 'pg18 pg_test'` | passed; existing unrelated HNSW `unused_unsafe` warnings remain |
| `diskann-format-smoke.sql` | packet-local SQL fixture for the PG18 smoke | builds `pq_fastscan` and `rabitq` `ec_diskann` indexes and checks ordered index scan results |
| `ecaz-dev-sql-pg18-diskann-format-smoke-file.log` | `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --raw --file reviews/task-60/004-diskann-rabitq-sql-format-test/artifacts/diskann-format-smoke.sql --log-output reviews/task-60/004-diskann-rabitq-sql-format-test/artifacts/ecaz-dev-sql-pg18-diskann-format-smoke-file.log` | passed; final row `task60_diskann_format_smoke_passed`, `pq_index_bytes=16384`, `rabitq_index_bytes=16384` |
| `cargo-pgrx-install-pg18-test-rerun3.log` | `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features` | passed after sandbox approval; installed the current build into local PG18 |

Retained diagnostic attempts:

- `cargo-pgrx-test-pg18-diskann-format-surface.log`: direct `cargo pgrx test pg18 test_ec_diskann_storage_formats_build_and_scan_sql_surface` compiled, then failed before backend execution with the local `CacheRegisterRelcacheCallback` symbol lookup issue.
- `cargo-pgrx-install-pg18-test.log`: first install attempt failed on sandbox read-only access to `/home/peter/.pgrx`.
- `ecaz-dev-sql-pg18-diskann-format-smoke.log`: first SQL attempt failed on sandbox socket access.
- `ecaz-dev-sql-pg18-diskann-format-smoke-rerun.log`: connected, then failed because the draft fixture used invalid `build_list_size=8`.
- `ecaz-dev-sql-pg18-diskann-format-smoke-rerun2.log`: connected and built both indexes, then failed because the draft fixture used `list_size=8` below the default `rerank_budget=64`.
- `ecaz-dev-sql-pg18-diskann-format-smoke-rerun3.log`: passed with inline SQL before the fixture was moved to `diskann-format-smoke.sql`.
