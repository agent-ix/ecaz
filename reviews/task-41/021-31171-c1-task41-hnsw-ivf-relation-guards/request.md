# Review Request: Task 41 HNSW/IVF Relation Guards

## Summary

This checkpoint closes the remaining raw HNSW/IVF relation helper surface in `src/lib.rs`.

It migrates the remaining production SQL wrappers for IVF page ownership, IVF cost, HNSW cost, and HNSW planner-integration snapshots to `AccessShareIndexRelation`. It also migrates the remaining HNSW test callers to the guard API, then deletes the raw `open_valid_ec_hnsw_index`, raw `open_valid_ec_ivf_index`, and `AccessShareIndexRelation::into_raw` escape hatch.

## Safety Delta

- Baseline entries: `4390` -> `4351`.
- `src/lib.rs` unsafe-comment baseline entries: `185` -> `177`.
- The selected HNSW test files moved from `177` combined entries to a smaller residual, mostly unrelated graph/page unsafe.
- `rg "open_valid_ec_(hnsw|ivf)_index\\(|into_raw\\(" src/lib.rs src/tests` returns no matches.
- The raw relation pointer escape hatch is gone for HNSW/IVF callers.

## Reviewer Focus

- Confirm production `src/lib.rs` IVF/HNSW snapshot wrappers drop the guard immediately after owned AM data is returned.
- Confirm HNSW tests that call graph loaders keep the guard alive across all raw-pointer graph reads and drop before later SPI/query work.
- Confirm deleting `into_raw` is safe now that no raw relation helper callers remain.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see `artifacts/manifest.md`.
