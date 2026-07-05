# Task 81 Review Request: Block Summary Diagnostics and Format Contract

## Summary

This slice records ADR-074 as the accepted SPIRE leaf block-summary format plan and adds diagnostics needed before the next Task 81 benchmark run.

Code changes:

- Keep V2 as the row-payload-only fallback and document V3/V4 summarized leaves in ADR-074.
- Add leaf block diagnostics for available, selected, and skipped summary blocks.
- Split leaf summary scoring time from leaf row scoring time while preserving the existing aggregate candidate score timer.
- Expose per-leaf summary bytes, row bytes, block counts, and split score timings through `ec_spire_index_scan_leaf_candidate_snapshot`.
- Add a storage-meta helper for deriving row-segment bytes from explicit V3/V4 metadata.

## Validation

Packet artifacts are under `reviews/task-81/001-block-summary-diagnostics/artifacts/`.

- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18` passed.
- `cargo-test-leaf-partition-v.log`: leaf V2/V3/V4 storage filter passed; includes V3/V4 block-summary round trips and coverage rejection.
- `cargo-test-global-block-selection.log`: global RaBitQ summary-radius block selection filter passed.
- `cargo-test-scan-diagnostics.log`: scan placement diagnostics filter passed.

`cargo fmt --check` is not cited as passing because it currently reports an unrelated pre-existing format diff in `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`.

## Review Focus

- Confirm the diagnostics semantics are acceptable for Task 81 benchmark evidence: summary bytes are V3/V4 summary-chain bytes; row bytes are row segment bytes derived from total object bytes minus summary and meta bytes.
- Confirm keeping the large placement snapshot unchanged and exposing the new detailed counters on the per-leaf snapshot is the right SQL surface.
- Check that full-leaf fallback accounting as `selected_blocks == available_blocks` is the desired diagnostic convention when pruning is disabled or ineffective.
