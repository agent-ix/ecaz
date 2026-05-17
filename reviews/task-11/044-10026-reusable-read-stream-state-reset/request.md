# Review Request: Reusable ReadStream State Reset Helpers

Scope:
- `src/am/stream.rs`
- `spec/functional/FR-019-async-io-read-stream.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added pure `reset(...)` helpers to `GraphPrefetchState` and `LinearPrefetchState` in
  `src/am/stream.rs` so the planner-owned ReadStream seam can be reused across graph-batch
  boundaries and rescans without touching `scan.rs` or wiring any PG18 APIs.
- Added unit coverage that verifies both state carriers restart cleanly after partial consumption,
  matching the staged `read_stream_reset()` and `amrescan` lifecycle described in FR-019.
- Tightened FR-019, the test matrix, and Task 11 notes so the documented D1 ReadStream contract now
  explicitly includes reusable state-carrier behavior instead of only one-shot callback state.

Review focus:
- Whether reusable reset helpers are the right last pure FR-019 seam before actual PG18 callback
  binding and runtime scan wiring
- Whether `reset(...)` is the right contract shape for both state carriers, or whether a different
  planner-owned lifecycle helper would better match the eventual PG18 integration lane
- Whether this slice adds meaningful lifecycle fidelity without drifting into runtime behavior or
  more SQL snapshot surface area

Questions to answer:
- Is the reset-based state reuse contract sufficient for later `read_stream_reset()` and `amrescan`
  integration, or is another pure lifecycle helper still obviously missing?
- Should these reset helpers stay on the state carriers directly, or move behind a narrower helper
  API before the runtime lane embeds them?
- Does this leave FR-019 in the right state for now: signatures, callback behavior, and lifecycle
  reuse are modeled in pure code, while PG18 binding and scan/vacuum wiring remain explicitly
  pending?
