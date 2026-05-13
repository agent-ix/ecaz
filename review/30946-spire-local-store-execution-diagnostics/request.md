# Review Request: SPIRE Local Store Execution Diagnostics

Code checkpoint: `cb4a6fd8` (`Expose SPIRE local-store execution mode`)

## Scope

- Advances Phase 12.8 by exposing the current local multi-store execution
  limitation in a SQL diagnostic surface.
- Adds `ec_spire_index_scan_local_store_execution_snapshot(index_oid, query)`,
  a narrow companion to the existing scan-placement snapshot. It reports one
  row per scan-touched local store group with:
  - `local_store_execution_mode = 'sequential_backend'`;
  - `local_store_read_ahead_primitive = 'pg18_read_stream'` on PG18;
  - `local_store_parallelism_next_step =
    'async_or_parallel_store_group_executor'`;
  - route, prefetch, and scanned-PID counts from the existing scan-placement
    diagnostic collector.
- Documents the distinction between PG18 ReadStream read-ahead and true
  concurrent store-group execution in both operator diagnostics and the local
  multi-store design note.
- Marks the Phase 12.8 sequential-execution diagnostic row complete.

## Validation

- `git diff --check cb4a6fd8^ cb4a6fd8`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_ec_spire_scan_placement_snapshot_sql`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and key result lines.

## Review Focus

- Confirm the new narrow diagnostic snapshot is preferable to widening
  `ec_spire_index_scan_placement_snapshot`, which is already near pgrx's
  table-row type limits.
- Confirm the stable labels are precise: `pg18_read_stream` is read-ahead
  inside one backend, and `async_or_parallel_store_group_executor` is only the
  future primitive needed for real local-store overlap.
