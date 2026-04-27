# Task 28 IVF Insert Stress Admin Snapshot Guard

This packet records commit `4303b4c` (`ecaz: require IVF admin snapshot for
insert stress`).

The `ecaz stress ivf-insert` harness now accepts
`--require-admin-snapshot`. When set, the command fails instead of silently
falling back to relation stats if `ec_ivf_index_admin_snapshot(oid)` is not
installed in the target database. This keeps future insert packets from
accidentally omitting drift/list metrics.

## Smoke Result

A 1-second PG18 smoke against the fresh local database
`task28_ivf_fresh_20260427` passed with the new flag:

| metric | value |
| --- | --- |
| `snapshot_source` | `ec_ivf_index_admin_snapshot` |
| `total_inserted_rows` | 397 |
| `inserted_rows_per_second` | 397.00 |
| `inserted_since_build` | 397 |
| `changed_row_fraction` | 0.284180 |
| `reindex_reason` | `changed_rows` |

This smoke only verifies the harness guard and admin snapshot path; it is not a
throughput comparison.

## Artifacts

- `artifacts/ivf_insert_require_admin_smoke.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz-cli ivf_insert`
- `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_require_admin_smoke --seed-rows 1000 --duration-seconds 1 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 1000 --require-admin-snapshot --log-output review/30061-task28-ivf-insert-admin-snapshot-required/artifacts/ivf_insert_require_admin_smoke.log`
- `git diff --check`

## Recommendation

Use `--require-admin-snapshot` on future Task 28 insert stress packets whenever
drift/list metrics are part of the cited result. If it fails on a long-lived
scratch database, create a fresh local PG18 database with the current `ecaz`
extension SQL instead of accepting fallback fields.
