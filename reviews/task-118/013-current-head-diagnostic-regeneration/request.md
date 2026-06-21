---
task: 118
packet: reviews/task-118/013-current-head-diagnostic-regeneration
checkpoint_sha: 325241cd6544f004a44968262e22b14be5b3397f
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Current-Head Diagnostic Regeneration Supplement

## Scope

This checkpoint updates Task 118 closeout handoff material after packet 012
changed the frontier diagnostic semantics.

Packet 006's existing 10k frontier rows were generated before commit
`6ff2d1d3d8aa04edced517497d940c65ea3d6bca`, so they should not be reused as
final candidate-containment proof. This packet adds a supplement requiring the
final Intel closeout pass to regenerate 10k `hnsw-frontier` rows on the current
branch head, alongside the still-required 50k/100k Intel suites.

It also adds short notes to the existing packet 010 runbook and packet 011
audit template so reviewers/operators do not miss this current-head
requirement.

## Validation

- Dry-run selected exactly six 10k `hnsw-frontier` steps: source-build and
  compressed-build lanes for TurboQuant, PqFastScan, and RaBitQ.
  - Artifact: `artifacts/suite-dry-run-10k-frontier-current-head.log`
  - Artifact: `artifacts/suite-manifest-dry-run-10k-frontier-current-head.json`

No benchmark was run here. This is an operator correctness checkpoint for the
final evidence collection.

## Remaining Task 118 Closeout Work

On the Intel benchmark host, reinstall the current branch with `pg_test`, run
the 10k frontier regeneration command from this packet, run the 50k/100k suites
from packet 010, then update packet 006 with the final decision table.
