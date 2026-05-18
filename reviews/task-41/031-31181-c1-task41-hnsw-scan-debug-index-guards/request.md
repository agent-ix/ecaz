# Review Request: Task 41 HNSW Scan Debug Index Guards

## Summary

Task 41 slice for HNSW pg_test scan-debug helpers.

The code commit `62d8a4324dc5550e3e0eaa79bd0c924668dd1a06` replaces the first `src/am/ec_hnsw/scan_debug.rs` cluster of manual `index_open` / `index_close` calls with the shared `IndexRelationGuard`.

This is intentionally narrow:

- covers `debug_begin_end_scan` through `debug_gettuple_after_rescan_result`
- preserves the existing `ec_hnsw_ambeginscan`, `ec_hnsw_amendscan`, and `IndexScanEnd` behavior
- leaves heap-backed scan, active snapshot, tuple slot, and later scan-debug helper lifetimes for follow-up slices

## Baseline Delta

- unsafe baseline entries: `4301 -> 4284`
- `src/am/ec_hnsw/scan_debug.rs`: `478 -> 461`

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

- Confirm this guard use is valid for pg_test helpers that intentionally raise PostgreSQL errors after `ambeginscan`; the intended behavior is that pgrx unwinds Rust frames and drops `IndexRelationGuard`.
- Confirm no scan lifecycle semantics changed; this slice only transfers relation close ownership into the shared guard.
- Identify the best next guard boundary for this file: heap-backed scan state, scan descriptor guard, snapshot guard, or later repeated debug helper groups.
