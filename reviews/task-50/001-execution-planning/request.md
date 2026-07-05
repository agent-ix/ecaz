# Task 50 Packet 001: Execution Planning Bridge

## Code Under Review

- Commit: planning packet only; no production code changes.
- Packet type: Task 50 execution plan / planning bridge.
- Task: `plan/tasks/50-unsafe-structural-reduction.md`.

## Scope

This packet turns the Task 35 closeout material into an execution-ready
Task 50 plan. It is intentionally doc-only because Task 50 implementation is
waiting for adjacent Task 39 / Task 47 test work to land.

The plan records:

- consolidated and ranked structural-reduction candidates from Task 35
  closeouts 083, 104, 107, 121, and 122;
- a first-slices sequence biased toward the current product priorities:
  RaBitQ, SPIRE, and IVF;
- the benchmark baseline policy, including existing AWS captures and the
  SPIRE-specific baseline gap;
- the direct unsafe-block-count tooling contract each packet should use;
- the top-15 coverage map and risk register needed to make execution
  bisectable.

Task 50 starts from approximately the same direct `unsafe { ... }` block
density that Task 35 documented. Task 35 drove the unsafe-comment baseline to
zero; it did not try to reduce the number of unsafe blocks except in the
test-sweep macro prototypes. The new asset for Task 50 is that every remaining
block now has an auditable invariant that can be lifted into structure.

## Planning Result

- Slice 1 is split into 1a / 1b / 1c: helper plus one IVF user, then IVF
  rollout, then SPIRE rollout.
- Slice 2 should target IVF page tuple visitors and posting-list range
  visitors, because IVF/RaBitQ profiling should start from a safer page
  substrate.
- Slice 3 is split into 3a / 3b / 3c: anchor type plus smallest user,
  coordinator snapshot rollout, then SPIRE production read-efficiency path.
- Slice 4 should lift cross-AM heap-source scoring behind an owned scorer,
  with IVF and SPIRE first in line.
- HNSW and DiskANN remain part of the top-15 exit criterion; the coverage map
  names later direct/shared-helper slices for those files rather than treating
  them as out of scope.

## Artifacts

- `artifacts/unsafe-block-count-current.log`: direct
  `rg --count-matches 'unsafe\s*\{' src` distribution at packet creation.
- `artifacts/manifest.md`: packet-local artifact metadata.

## Revision Notes

This packet was revised after reviewer feedback
`feedback/2026-05-19-01-reviewer.md` to add:

- `top-15-coverage-map.md`
- `risk-register.md`
- `local-bench-plan.md`
- split Slice 1 and Slice 3 sequencing
- existing AWS baseline mapping and local-vs-cloud policy
- local baseline generation requirements for the full corpus profile spread
  across priority IVF/RaBitQ, SPIRE/RaBitQ, HNSW, and DiskANN rows
- `rg` fallback requirements for unsafe-block counting

No tests or benches were run. This packet changes documentation only and
captures planning evidence.
