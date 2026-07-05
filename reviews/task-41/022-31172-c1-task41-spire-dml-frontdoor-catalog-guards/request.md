# Review Request: Task 41 SPIRE DML Frontdoor Catalog Guards

## Summary

This checkpoint wraps the remaining paired index opens inside `src/am/ec_spire/dml_frontdoor/mod.rs`.

The DML frontdoor catalog helpers now use a module-local `AccessShareIndexRelation` guard for catalog index walks and key-column extraction. The guard owns the `AccessShareLock` close, so early `continue`, `?`, and `pgrx::error!` paths no longer rely on manually paired `index_close` calls.

## Safety Delta

- Baseline entries: `4351` -> `4347`.
- `src/am/ec_spire/dml_frontdoor/mod.rs` unsafe-comment baseline entries: `164` -> `160`.
- Direct `index_open`/`index_close` calls in this module are now isolated to the guard implementation.

## Reviewer Focus

- Confirm `dml_frontdoor_catalog_index_and_pk` keeps each opened index guard alive while borrowing `rd_index` / `rd_rel`, and drops before the next loop iteration.
- Confirm `dml_frontdoor_index_key_column_names_from_rel` returns owned column-name strings and does not leak borrowed index metadata past the guard lifetime.
- Confirm the `None` path from `AccessShareIndexRelation::open` preserves the previous null-open behavior.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see `artifacts/manifest.md`.
