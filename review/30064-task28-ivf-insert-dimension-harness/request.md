# Task 28 IVF Insert Stress Dimension Harness

This packet records commit `656b2dc` (`ecaz: parameterize IVF insert stress
dimensions`).

`ecaz stress ivf-insert` now accepts `--dimensions`, defaults to the existing
4D fixture, includes the dimension count in the summary table, and can generate
larger synthetic vectors for live-insert stress. This lets Task 28 distinguish
small 4D write-path overhead from higher-dimensional centroid assignment,
encoding, and posting payload costs.

## Smoke Result

A short local PG18 smoke against fresh database `task28_ivf_fresh_20260427`
verified a 1536D insert stress surface with admin snapshot metrics:

| metric | value |
| --- | --- |
| `duration_seconds` | 1 |
| `dimensions` | 1536 |
| `total_inserted_rows` | 146 |
| `inserted_rows_per_second` | 146.00 |
| `snapshot_source` | `ec_ivf_index_admin_snapshot` |
| `inserted_since_build` | 146 |
| `changed_row_fraction` | 0.532847 |
| `average_list_live_count` | 17.12 |
| `max_list_live_count` | 62 |
| `list_imbalance_ratio` | 3.620438 |
| `reindex_reason` | `changed_rows` |

This is a harness smoke, not a product benchmark or a throughput comparison.

## Artifacts

- `artifacts/ivf_insert_dim1536_smoke.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz-cli ivf_insert`
- `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task28_ivf_fresh_20260427 stress ivf-insert --table task28_ivf_insert_dim1536_smoke --seed-rows 128 --duration-seconds 1 --concurrency 1 --batch-rows 1 --nlists 16 --nprobe 16 --training-sample-rows 128 --dimensions 1536 --require-admin-snapshot --log-output review/30064-task28-ivf-insert-dimension-harness/artifacts/ivf_insert_dim1536_smoke.log`
- `git diff --check`

## Recommendation

Use this harness for the next live-insert slice before making more conclusions
from the 4D fixture. A useful next measurement is a 10-second c1/c4 run at
`--dimensions 1536`, then re-evaluate whether centroid/model work or posting
payload writes dominate at production-like dimension.
