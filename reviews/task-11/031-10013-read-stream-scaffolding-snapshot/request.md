# Review Request: ReadStream Scaffolding Snapshot

Scope:
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/am/stream.rs`
- `src/lib.rs`
- `spec/functional/FR-019-async-io-read-stream.md`
- `spec/functional/FR-027-pgrx-pg18-upgrade.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a pure planner-owned `src/am/stream.rs` seam that reports the intended graph and linear
  ReadStream modes plus their access patterns, while keeping callback, scan, and vacuum readiness
  explicitly false until PG18 support exists.
- Added a read-only SQL/admin surface, `tqhnsw_read_stream_snapshot()`, so productization and
  cross-agent review can inspect that staged boundary without implying any live runtime wiring on
  PG17.
- Extended `tqhnsw_planner_integration_snapshot(regclass)` with
  `pg18_read_stream_surface_ready`, derived from the three explicit ReadStream readiness bits.
- Added Rust and pg coverage for the new snapshot and updated FR-019 / FR-027 / the test matrix /
  Task 11 notes so this remains descriptive planner scaffolding rather than execution behavior.

Review focus:
- Whether a dedicated `tqhnsw_read_stream_snapshot()` surface is the right D1 seam, or whether the
  intended ReadStream boundary should stay only inside the broader planner-integration snapshot
- Whether surfacing graph-vs-linear modes plus random-vs-sequential access patterns is the right
  amount of detail for productization without overspecifying runtime internals too early
- Whether the three explicit readiness bits (`callback_surface`, `scan_wiring`,
  `vacuum_wiring`) are the right long-lived boundary for the future PG18/runtime integration lane
- Whether `pg18_read_stream_surface_ready` is useful cross-lane context in the consolidated
  planner integration snapshot

Questions to answer:
- Should ReadStream staging stay as its own snapshot surface, or be folded back into
  `tqhnsw_planner_integration_snapshot(regclass)` once the other agent starts wiring runtime work?
- Are the current mode names and access-pattern strings durable enough to expose now, or should
  they stay more abstract until real ReadStream binding exists?
- Does this slice make FR-019's current-vs-target boundary clearer without accidentally implying
  that any PG17 scan or vacuum path already uses `read_stream_next_buffer()`?
