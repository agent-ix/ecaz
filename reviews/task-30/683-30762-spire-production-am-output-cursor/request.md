# 30762 - SPIRE Production AM Output Cursor

## Summary

This packet reviews commit `3460dd27e2d8d5520610fde5741380599f6ea2c0`
(`Cursor SPIRE production AM outputs`).

The slice moves the SPIRE index AM cursor onto the production heap-resolution
result stream for AM-deliverable rows. `amrescan` now asks the production stream
for the current scan output set, converts only coordinator-local heap rows into
`SpireScanOutput` entries, and leaves `amgettuple` as a cursor over AM-shaped
outputs. A stream with any blocker, including `remote_row_materialization`,
fails before setting `xs_heaptid`.

This still does not implement remote row materialization. Remote-origin rows
remain blocked from PostgreSQL index AM tuple delivery until a same-indexed-heap
shadow/proxy row exists.

## Key Files

- `src/am/ec_spire/scan/callbacks.rs`
- `src/am/ec_spire/scan/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/scan/tests/runtime_state.rs`
- `plan/design/spire-production-coordinator-executor.md`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `git diff --check -- <changed code/docs>`
- `cargo check --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo test production_scan_result_stream_am_outputs --no-default-features --features pg18`
  compiled, then hit the known direct-test pgrx loader failure:
  `undefined symbol: SPI_finish`.

No PostgreSQL server or distributed fixture was started for this packet.

## Review Focus

- Is `SpireScanOutputCursor` the right scan-opaque contract for final
  `amgettuple` delivery?
- Does `amrescan` now depend on the production stream without accidentally
  allowing remote-origin heap coordinates into `xs_heaptid`?
- Is the AM top-k default through the production stream's scan-plan candidate
  limit appropriate?
- Are remote materialization blockers surfaced early enough for the remaining
  implementation slice?
