# Review Request: Reusable EXPLAIN Counter Struct

Scope:
- `src/am/explain.rs`
- `spec/functional/FR-024-custom-explain.md`
- `spec/tests.md`
- `plan/tasks/11-planner.md`

What changed:
- Added a reusable `TqExplainCounters` struct in `src/am/explain.rs` with one field per staged
  FR-024 counter plus pure record/reset helpers.
- Kept the work entirely in planner-owned code so the graph-search lane can embed this struct into
  `TqScanOpaque` later without this branch editing `scan.rs`.
- Added unit tests for both the per-counter mutation helpers and reset behavior.
- Updated FR-024, the test matrix, and Task 11 notes to record the new struct as the current D1
  seam between planner-owned EXPLAIN scaffolding and future scan-lane wiring.

Review focus:
- Whether `TqExplainCounters` is the right ownership boundary for the scan lane to embed later
- Whether the helper methods expose the right staged contract, or whether a more generic mutation
  API would be better before scan wiring starts
- Whether this slice addresses the reviewer guidance on “real D1 deliverables” more cleanly than
  further SQL-snapshot expansion
- Whether the FR-024 wording now accurately reflects the state of the code: reusable counter struct
  exists, but `TqScanOpaque` storage and EXPLAIN hook wiring are still pending

Questions to answer:
- Is `TqExplainCounters` best kept in `am/explain.rs`, or should it move to shared territory before
  the other agent embeds it into scan state?
- Are any counter helpers or fields missing for the eventual scan-lane embedding work?
- Does the current API make it clear that this is a pure D1 seam, not live EXPLAIN instrumentation?
