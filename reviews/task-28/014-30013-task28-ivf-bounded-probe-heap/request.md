# Review Request: Task 28 IVF Bounded Probe Heap

Scope: Phase 4 posting-list scan checkpoint. IVF probe candidate
materialization now deduplicates heap TIDs by best score, retains candidates
through a bounded top-k heap, and emits them in deterministic score order.

Task: `plan/tasks/28-ivf-access-method.md` Phase 4

Branch: `task28-ivf`

Head SHA: `d6f21f1f770190e8eeab5c1e68607e9ec907b277`

Owner: coder2

Files:

- `src/am/ec_ivf/scan.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy.
- No test suite was run. The new Rust unit coverage and existing PG checks were
  compiled only.
- No measurement claim is made in this packet.

## Summary

This slice closes the Phase 4 posting-list scan checklist item:

- Replaces full candidate sorting with `CandidateTopK`, a bounded heap that
  retains the best score/TID-ordered candidates and returns deterministic
  output order.
- Deduplicates heap TIDs by keeping the best candidate score before feeding the
  bounded heap.
- Derives the production heap bound from selected list live counts. This keeps
  current exact-in-probed-lists behavior because PostgreSQL does not pass a
  LIMIT to `amrescan`.
- Adds pure Rust coverage for score ordering and heap-TID tie-breaking.
- Updates the task plan to mark posting-list scan complete and move Phase 4
  status to rerank mode next.

## Review Focus

Please review for:

- Whether selected-list live count is the right temporary heap bound until an
  executor LIMIT or scan-local bound exists.
- Whether duplicate heap TIDs should keep the best score, as implemented here,
  or preserve first-seen posting order.
- Whether the score/TID comparison is the right deterministic order for
  repeated scan output.
- Whether the bounded heap should become a reusable helper before rerank mode
  adds second-stage scoring.

## Non-Goals

This packet does not implement rerank mode, storage-format-specific scoring,
live insert, vacuum, planner costing, recall gates, or any measurement claim.
