# Review Request: Plan And Spec Backfill

Scope:
- `plan/plan.md`
- `plan/tasks/01-quantizer-core.md`
- `plan/tasks/02-datum-and-io.md`
- `plan/tasks/03-sql-surface.md`
- `plan/tasks/04-page-layout-and-wal.md`
- `plan/tasks/archive/05-build-and-scan.md`
- `plan/tasks/archive/06-vacuum-and-insert.md`
- `plan/tasks/archive/07-simd-and-benchmarks.md`
- `plan/tasks/archive/08-safety-and-ci.md`
- `spec/functional/FR-007-hnsw-page-layout.md`
- `spec/functional/FR-009-hnsw-scan.md`
- `spec/functional/FR-016-hnsw-insert.md`
- `spec/adr/ADR-011-planner-cost-override-until-ordered-scan.md`

What changed:
- Backfilled the implementation plan and per-task files so completed and in-progress work are reflected accurately.
- Updated FR-007 to document inline duplicate heap-TID capacity and current `(gamma, code_bytes)` duplicate semantics.
- Updated FR-009 to document the bootstrap linear scan stage and the temporary planner cost gate.
- Updated FR-016 to state that `build_source_column` remains a bulk-build-only path in v0.1.
- Added ADR-011 for the deliberate planner cost override while ordered scan semantics are incomplete.

Review focus:
- Whether the plan/task status now reflects the actual implementation state cleanly
- Whether the spec backports capture current staged behavior without overcommitting future design
- Whether ADR-011 is the right documented boundary for planner suppression

Questions to answer:
- Do the plan and task statuses now represent the project’s true execution state well enough to use for ongoing tracking?
- Do FR-007, FR-009, and FR-016 correctly separate current staged behavior from final target behavior?
- Is ADR-011 the right place to document the planner gate, or should any of that rationale live directly in FR-009 as normative text instead?
