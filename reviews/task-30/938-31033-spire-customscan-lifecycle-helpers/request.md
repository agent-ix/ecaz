# Review Request: SPIRE CustomScan lifecycle helpers

## Summary

This checkpoint closes the Phase 12b RemoteScan lifecycle coverage rows for
`EndCustomScan` cleanup and `ReScanCustomScan` output-progress reset.

Code checkpoint: `cf49252f17f98b6fc01f667c13e6f07cfea60001`

The change extracts the existing Rust state behavior behind the FFI thunks into
small helpers:

- `custom_scan_default_exec_state`
- `custom_scan_release_exec_state_for_end`
- `custom_scan_reset_exec_state_for_rescan`
- `custom_scan_next_output_index`

`EndCustomScan`, `ReScanCustomScan`, and the scan access loop now call those
helpers. Unit tests exercise the helpers directly because direct calls through
the `#[pg_guard]` C-unwind thunks are not suitable outside a PostgreSQL backend.

## Scope Guard

This slice does not add to the shrink-list files:

- `src/tests/remote_search.rs` remains deleted.
- `src/tests/mod.rs` is unchanged by this checkpoint.

The added tests live in `src/am/ec_spire/custom_scan/tests.rs`, which remains
small at 437 lines after this checkpoint.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz custom_scan_`

The focused test run passed 16 selected tests, including the new lifecycle
tests and the selected PG18 pgrx pg_test items. Raw logs and line counts are in
`artifacts/`.

## Reviewer Focus

- Confirm the helper extraction preserves the executor cursor semantics.
- Confirm the release helper is acceptable coverage for `EndCustomScan` cleanup
  given that the thunk itself still performs `drop_in_place` and `pfree`.
- Confirm the task tracker wording is honest about helper-level coverage rather
  than claiming a direct backend thunk invocation.
