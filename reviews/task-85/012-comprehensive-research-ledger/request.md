# Review Request: Task 85 Comprehensive Research Ledger

## Summary

This checkpoint fixes the planning failure that allowed Task 85 research
directions to be treated as vague future work. `plan/tasks/85-spire-product-
scale-pareto-program.md` now contains a required ledger for every plausible
same-recall latency lever identified during the task.

## Scope

- Added `4.0 Required Research Direction Ledger`.
- Defined ledger states: `open`, `instrumenting`, `implementing`, `accepted`,
  and `rejected`.
- Marked the current Task 85 directions as in-scope ledger items:
  object-read/physical layout, summary scoring CPU, candidate-set-preserving
  rerank locality, recall-preserving candidate-surface redesign, benchmark
  harness extensions, and comparator/product policy gate.

## Validation

No code validation was run for this planning-only checkpoint.

The next implementation checkpoint must still complete the focused compile/test
validation left open by packet 011 before any AWS deployment.
