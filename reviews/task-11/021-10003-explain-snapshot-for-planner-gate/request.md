# Review Request: Explain Snapshot For Planner Gate

Scope:
- `src/am/mod.rs`
- `src/am/shared.rs`
- `src/lib.rs`
- `spec/functional/FR-006-sql-operators.md`
- `spec/functional/FR-009-hnsw-scan.md`
- `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md`
- `spec/tests.md`
- `plan/tasks/archive/05-build-and-scan.md`

What changed:
- Added a read-only SQL/explain scaffolding surface, `tqhnsw_index_explain_snapshot(regclass)`,
  that reports whether planner-visible scans are enabled, whether ordered scan is ready, the
  current planner-gate reason, effective `ef_search`, tuning source, and live-node count.
- Kept the planner gate hard-disabled and hard-coded `ordered_scan_ready = false`, so this surface
  stays descriptive rather than implying that PostgreSQL EXPLAIN can already choose a `tqhnsw`
  index scan.
- Added pg coverage for both the happy path and non-`tqhnsw` index rejection.
- Follow-up: the shared validation helper now includes the caller function name, so
  `tqhnsw_index_explain_snapshot(...)` reports its own rejection error text instead of naming the
  admin snapshot surface.
- Updated ADR-011, FR-006, FR-009, the test matrix, and Task 05 tracking so explain-oriented
  scaffolding is recorded explicitly without overstating planner readiness.

Review focus:
- Whether a separate explain snapshot function is the right staging seam before real planner/EXPLAIN
  integration exists
- Whether exposing `ordered_scan_ready = false` plus an explicit reason string is the right current
  contract for planner/admin tooling
- Whether the surface stays descriptive enough to respect ADR-011 instead of becoming a de facto
  planner-enablement path
- Whether the spec/task updates clearly separate this scaffolding from actual EXPLAIN output or
  planner-visible index scans

Questions to answer:
- Is `tqhnsw_index_explain_snapshot(regclass)` the right near-term boundary for explain-facing
  scaffolding, or should this data stay folded into the admin snapshot until PostgreSQL EXPLAIN
  hooks exist?
- Is the `ordered_scan_ready` plus `planner_gate_reason` shape explicit enough for future tooling
  without creating a misleading stability promise too early?
- Is sharing one caller-parameterized validation helper across the admin and explain snapshot
  surfaces the right long-lived shape for future planner/admin SQL helpers?
