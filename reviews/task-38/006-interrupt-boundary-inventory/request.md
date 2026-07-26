# Review Request: Exact Interrupt-Boundary Inventory

## Summary

This checkpoint closes the Task 38 accounting requirement to inventory current
interrupt sites and file missing/ambiguous long-running surfaces as follow-up
work.

`docs/hardening.md` now maps each explicit five-AM backend interrupt boundary
to its owning source path and loop:

- HNSW and IVF parallel-build leader waits;
- DiskANN graph traversal and vacuum passes;
- SPIRE's bounded remote-dispatch cancellation/statement-timeout poll;
- DistANN physical shard generation/stitch/repair; and
- DistANN remote transport boundaries placed outside the thread-local
  `RefCell` borrow.

The documentation explicitly does not claim that every long-running loop
already polls. New Task 200 owns the exhaustive classification of the known
unpolled or ambiguous HNSW, IVF, DiskANN, SPIRE, and DistANN surfaces and
requires each to be documented as bounded/outer-polled, safely remediated, or
split into a narrower structural follow-up.

## Validation

See `artifacts/manifest.md` and `artifacts/explicit-interrupt-sites.log`.

- exact source search captured every current named interrupt macro/helper call
  under the five AM trees plus SPIRE's dynamic PostgreSQL flag poll;
- `git diff --check` passed;
- no runtime behavior changed, so no Cargo or PG test was run; and
- no Intel/Linux-only lane was invoked.

## 2026-07-26 Outside-Review Response

The response closes both findings in
`feedback/2026-07-26-01-reviewer.md`:

- the raw search and documentation now include DistANN's inner 5 ms
  `InterruptPending`/`QueryCancelPending`/`ProcDiePending` poll, bounded remote
  cancel, pooled-connection clearing, and outer raising boundary; and
- trailing blank lines were removed and
  `git diff --check 476407ed9` passed for the complete checkpoint range plus
  this response.

## Reviewer Focus

- Does the inventory accurately describe every current explicit poll?
- Is Task 200 sufficiently concrete to satisfy “missing sites are filed as
  follow-ups” without claiming remediation?
- Are the longjmp-safety constraints explicit enough to prevent unsafe poll
  placement?
