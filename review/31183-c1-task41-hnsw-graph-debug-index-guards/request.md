# Review Request: Task 41 HNSW Graph Debug Index Guards

## Summary

Task 41 slice for HNSW graph-inspection debug helpers.

The code commit `4ce96d2102ca3e68b7387c3b4be5e57b3b1ea714` replaces manual `index_open` / `index_close` lifetimes with `IndexRelationGuard::access_share` in three no-scan graph debug helpers:

- `debug_all_top_level_heap_tids`
- `debug_top_level_reachable_heap_tids`
- `debug_layer0_reachable_live_element_tids`

The same commit also processes 31180 feedback by adding a module-level note to `src/storage/relation_guard.rs` that documents the split between low-level relation ownership and AM-validation helpers.

## Baseline Delta

- unsafe baseline entries: `4265 -> 4256`
- `src/am/ec_hnsw/scan_debug.rs`: `442 -> 433`

See `artifacts/manifest.md` and `artifacts/validation.md`.

## Validation

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passed with the existing PG18 C-header warnings and the existing unused re-export warning in `src/am/mod.rs`.

## Review Focus

- Confirm each guard binding remains alive for all raw `pg_sys::Relation` uses in the helper.
- Confirm early returns now rely on guard drop without changing result behavior.
- Confirm the relation-guard module doc accurately describes the resource-vs-validation split.
