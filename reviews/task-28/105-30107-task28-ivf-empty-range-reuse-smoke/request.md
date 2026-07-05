# Task 28 IVF Empty-Range Reuse and Quantizer Alias

## Scope

This packet records the `955add3b` checkpoint:

- Adds `quantizer = 'turboquant' | 'pq_fastscan' | 'rabitq' | 'auto'` as an `ec_ivf` reloption alias for the existing `storage_format` spelling.
- Rejects conflicting `storage_format` and `quantizer` values.
- Removes dead `live_head_block` / `live_tail_block` tracking from the vacuum per-posting loop.
- Preserves an emptied list's old posting block range so later inserts can reuse the range.
- Documents that v1 range reuse can walk the whole list range under churn.

## Validation

Focused tests passed on PG18:

- `cargo pgrx test pg18 test_ec_ivf_quantizer_reloption_alias_accepts_pq_fastscan`
- `cargo pgrx test pg18 test_ec_ivf_quantizer_reloption_alias_accepts_rabitq`
- `cargo pgrx test pg18 test_ec_ivf_quantizer_reloption_conflicts_with_storage_format`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_storage_build_scan_insert_vacuum`
- `cargo pgrx test pg18 test_ec_ivf_rabitq_storage_build_scan_insert_vacuum`
- `cargo pgrx test pg18 test_ec_ivf_pq_fastscan_accepts_group_size_reloption`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_repairs_empty_list_directory_refs`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_compacts_deleted_posting_space_for_reuse`
- `cargo pgrx test pg18 test_ec_ivf_admin_snapshot`

Focused non-PG tests passed:

- `cargo test -p ecaz --lib pq_fastscan_dispatch_scores_grouped_code_with_persisted_model`
- `cargo test -p ecaz --lib rabitq_dispatch_matches_direct_quantizer_score`
- `cargo test -p ecaz-cli profiles`

`git diff --check` passed.

## Smoke Result

The same-distribution replacement smoke was rerun at `955add3b`. It matches packet 30106:

| surface | after build | after delete vacuum | after refill |
| --- | ---: | ---: | ---: |
| nlists=8 | 448 kB | 448 kB | 448 kB |
| nlists=32 | 448 kB | 448 kB | 464 kB |
| nlists=64 | 448 kB | 448 kB | 536 kB |

Interpretation: preserving empty-list ranges fixes the explicit empty-list refill path covered by PG tests, but this particular 5k replacement fixture still has no additional index-size convergence beyond packet 30106. A3 remains partially open for nlists=32/64 sustained churn.

## Artifacts

- `artifacts/ivf_empty_range_reuse_smoke.sql`
- `artifacts/ivf_empty_range_reuse_smoke.log`
- `artifacts/manifest.md`
