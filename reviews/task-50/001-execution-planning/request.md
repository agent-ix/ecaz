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
- the benchmark baseline capture plan needed before the first code slice;
- the direct unsafe-block-count tooling contract each packet should use.

## Planning Result

- Slice 1 should implement the cross-AM callback wrapper helper and apply it
  first to IVF, SPIRE, and RaBitQ-adjacent AM callback surfaces.
- Slice 2 should target IVF page tuple visitors and posting-list range
  visitors, because IVF/RaBitQ profiling should start from a safer page
  substrate.
- Slice 3 should introduce a SPIRE `ActiveEpochAnchor` or equivalent typed
  epoch/context wrapper for production read paths.
- Slice 4 should lift cross-AM heap-source scoring behind an owned scorer,
  with IVF and SPIRE first in line.
- HNSW and DiskANN remain important but should mostly consume shared helpers
  after the priority product surfaces prove the pattern.

## Artifacts

- `artifacts/unsafe-block-count-current.log`: direct
  `rg --count-matches 'unsafe\s*\{' src` distribution at packet creation.
- `artifacts/manifest.md`: packet-local artifact metadata.

No tests or benches were run. This packet changes documentation only and
captures planning evidence.
