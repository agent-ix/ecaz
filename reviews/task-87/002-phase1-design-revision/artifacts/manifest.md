# Task 87 Packet 002 Artifact Manifest

- head SHA: `edfb914ca`
- task bucket: `reviews/task-87`
- packet path: `reviews/task-87/002-phase1-design-revision`
- lane: Phase 1 design revision
- fixture: source review and reviewer feedback response
- storage format: not applicable
- rerank mode: not applicable
- command used: source inspection with `rg`, `sed`, and `nl`
- timestamp: 2026-06-08
- surface mode: no benchmark run; design-only packet

## Artifacts

- `design-revision.md` — responses to Task 87/001 reviewer blockers B1
  through B4, plus the RaBitQ metadata note.

## Key Result Lines Cited By Request

- HNSW Task 87 batching is scoped to `TurboQuantExactScoreMode::FullLut`
  only.
- DiskANN grouped-PQ/RaBitQ will not be treated as satisfying a
  TurboQuant no-QJL 4-bit gate.
- Follow-up Task 90 is opened for DiskANN TurboQuant search-code
  enablement or Stop Condition evidence.
- HNSW will use owned code scratch as the backing for borrowed
  `CandidateBatch` entries.
- SPIRE Phase 2 is structural; the `>= 2x` scoring-share target is moved
  to the first real batch-kernel packet.
