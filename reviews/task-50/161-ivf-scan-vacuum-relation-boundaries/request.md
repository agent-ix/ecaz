# Review Request: IVF Scan/Vacuum Relation Boundaries

## Summary

This checkpoint continues the packet 159/160 cleanup by propagating raw-relation contracts through the remaining IVF scan, vacuum, admin, and PQFastScan helper layers.

Code commit: `f9fe7f68907559f0af8ac25812ac424c688b0735`

## Scope

- Marked relation-carrying helper functions unsafe:
  - `directory_drift_summary`
  - `load_pq_fastscan_model`
  - `store_scan_prepared_query`
  - `pq_fastscan_model_for_scan`
  - `build_selected_probe_plan`
  - `load_directory_entries`
  - `bulkdelete_list_postings`
  - `finish_vacuum_stats`
- Removed redundant inner unsafe blocks now covered by those helper-level contracts.
- Kept the live relation invariant rooted in PostgreSQL-facing unsafe callback/debug entry points.

## Completion Audit Note

This advances Wave 2 / IVF relation-boundary cleanup. It does not close Task 50; the comprehensive plan still requires ledger/residual registry coverage and many remaining unsafe blocks across SPIRE, HNSW, DiskANN, tests, storage, quant, hardening, and vendor disposition.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
