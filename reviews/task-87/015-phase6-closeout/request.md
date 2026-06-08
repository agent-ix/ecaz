# Task 87 Packet 015: Phase 6 Closeout

## Summary

This packet asks for review of Task 87 closeout. It aggregates the completed
Phase 6 matrix evidence from packets 012, 013, and 014, applies the packet 009
Task 91 scope walk-back, and flips the Task 87 status to complete by reference
to this packet.

Current head for this packet:

- `9e4d2ad3642b63eec543d9710ca521d1b8f82787` - `Add Task 87 real100k matrix packet`

## What Changed

- `plan/tasks/87-candidate-batched-scoring-across-ams.md`
  - status changed to `complete (2026-06-08; see reviews/task-87/015-phase6-closeout/)`.
- `artifacts/aggregate-matrix.md`
  - adds the aggregate AM x corpus off/on matrix for recall@10,
    p50/p95/p99 latency, latency deltas, and storage.
- `artifacts/manifest.md`
  - records the closeout artifact metadata and source evidence packets.
- `artifacts/completion-audit.md`
  - maps Task 87's closeout, acceptance, and validation-gate requirements to
    current checked-in evidence.

No code changed in this packet.

## Closeout Evidence

The closeout matrix is built from packet-local benchmark artifacts:

- real10k: `reviews/task-87/012-phase6-suite-prep/`
- real50k: `reviews/task-87/013-phase6-real50k-matrix/`
- real100k: `reviews/task-87/014-phase6-real100k-matrix/`

DiskANN remains represented by the accepted Task 87 Stop Condition packet
`reviews/task-87/005-phase4-diskann-stop-condition/` and the Task 91 handoff
approved in `reviews/task-87/009-scope-walk-back-and-task-91-handoff/`.

## Aggregate Call

- Recall is unchanged across off/on in every measured SPIRE, IVF, and HNSW
  cell.
- Storage is unchanged by construction because off/on flips only a session
  route GUC against the same index surface.
- SPIRE shows consistent end-to-end pipeline gains across all three corpora.
- IVF improves on real10k and real100k, while the real50k RaBitQ cell is
  flat/slightly worse at p50/p95 and slightly faster at p99.
- HNSW preserves recall and has useful real50k p95/p99 improvement, but
  real100k p50 is effectively flat.
- The original universal 2x scoring-share target is not directly proven by
  HNSW/IVF instrumentation and is not claimed. The packet applies the
  reviewer-approved structural-slice carve-outs and Task 91 handoff instead
  of broadening Task 87 into more codec migration.

## Validation

No new test or benchmark command was run for this packet. This is a
documentation/status closeout over already checked-in Phase 6 suite evidence.

The completion audit explicitly notes that packet 015 is still awaiting outside
reviewer response, especially because the original universal scoring-share and
every-cell latency gates are not claimed as fully met.

## Review Focus

- Confirm the aggregate matrix satisfies packet 012's closeout request.
- Confirm the completion audit maps Task 87's requirements to the right
  authoritative evidence.
- Confirm the gate call is honest about structural-slice carve-outs and
  measurement misses.
- Confirm Task 87 can close with DiskANN represented by packet 005 Stop
  Condition plus packet 009 Task 91 handoff.
- Confirm the task status flip to complete should stand.
