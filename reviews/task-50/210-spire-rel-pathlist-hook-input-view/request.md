# Task 50 Packet 210: SPIRE Rel Pathlist Hook Input View

## Summary

This slice consolidates SPIRE `set_rel_pathlist` planner-hook pointer validation behind a borrowed hook-input view.

Updated surface:

- `src/am/ec_spire/custom_scan/planner.rs` now builds `CustomScanRelPathlistInput<'_>` once from the PostgreSQL `root` / `rel` / `rte` hook arguments.
- Vector ORDER BY/LIMIT candidate selection and DML PK SELECT candidate selection are safe methods on that view.
- The previous `unsafe fn custom_scan_candidate_index_oid` and `unsafe fn dml_pk_select_candidate_index_oid` helper contracts were removed.

The remaining unsafe in this planner file is still around PostgreSQL hook entry points, planner-memory allocation, catalog relation probes, path insertion, and plan-node construction.

## Counts

- `src`: `2564 -> 2561` unsafe references.
- `src/am/ec_spire/custom_scan/planner.rs`: `34 -> 31` unsafe references.
- `src/am/ec_spire/custom_scan/planner.rs`: `10 -> 9` unsafe function contracts.

## Validation

Artifacts are under `artifacts/` and indexed in `artifacts/manifest.md`.

- `rustfmt --check src/am/ec_spire/custom_scan/planner.rs`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib custom_scan --no-default-features --features pg18,pg_test --no-run`
- `git diff --check`

All validation passed. The cargo commands still report pre-existing unrelated warnings from `src/am/mod.rs` and Hadamard test helpers.
