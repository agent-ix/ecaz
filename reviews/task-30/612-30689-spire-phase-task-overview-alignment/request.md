# Review Request: SPIRE Phase Task Overview Alignment

## Summary

Task 30's overview now matches the split phase task files and keeps the
requested file naming convention visible:

- `plan/tasks/task30-phase9-spire-graph-architecture.md`
- `plan/tasks/task30-phase10-spire-execution-performance.md`

Docs checkpoint: `b4e822f4` (`Align SPIRE phase task overview`)

## Scope

- Updates `plan/tasks/30-spire-ivf-foundation.md` so the Phase 10 overview
  reflects the detailed Phase 10 task file status.
- Marks Phase 10.1 through Phase 10.6 complete in the overview, with pointers
  to ADR-056, ADR-057, ADR-058, ADR-059, and the packet-local rerank/adaptive
  benchmark evidence.
- Leaves Phase 10.7 performance harness open, matching
  `plan/tasks/task30-phase10-spire-execution-performance.md`.
- Preserves the existing `task30-phase9-...` and `task30-phase10-...` file
  names; no phase task file rename is needed.

## Validation

- `git diff --cached --check`

## Notes

Planning/docs-only checkpoint. No code, SQL, benchmark, or test behavior
changed.
