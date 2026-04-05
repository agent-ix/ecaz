# Review Request: `amrescan` Defensive Cases

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `amrescan` already rejects malformed scan setup for the current planner-disabled scan path.
- The next narrow slice is expected to add focused coverage for defensive cases that the latest review called out:
  - NULL ORDER BY queries
  - empty `real[]` queries
  - unsupported index quals
  - unsupported multiple ORDER BY keys

Review focus:
- Whether the added helper/test surface exercises the SQL-visible error paths without widening executor behavior
- Whether the error boundaries stay planner-safe and deterministic
- Whether any of these defensive checks should be tightened in code rather than only covered by tests

Questions to answer:
- Do the new tests cover the highest-value `amrescan` misuses first?
- Is any error path still effectively untestable with the current helper surface?
- Is there a smaller scan-safety slice that should have gone first?

Status at `0abf7d9`:
- Addressed by adding helper coverage and regression tests for all four requested defensive `amrescan` cases.
- No code-path widening was needed beyond test-only helper entry points.
