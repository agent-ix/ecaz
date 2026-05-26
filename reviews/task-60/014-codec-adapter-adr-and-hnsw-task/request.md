# Review Request: Codec Adapter ADR and HNSW Companion Task

## Scope

This checkpoint records the design direction discussed after the DiskANN RaBitQ
integration work.

Changes under review:

- Adds `spec/adr/ADR-072-index-local-quantized-codec-adapters.md`.
- Updates `spec/adr/index.md`.
- Adds companion Task 64:
  `plan/tasks/64-hnsw-quantized-codec-adapters.md`.
- Updates Task 63 so HNSW RaBitQ references Task 64 as the companion codec
  adapter extraction task.
- Updates `plan/tasks/README.md` with Task 64.

## Design Summary

ADR-072 keeps the boundary explicit:

- shared quantizer families own quantization math and scoring semantics;
- each AM owns an index-local codec adapter for metadata, tuple/list layout,
  sidecars, traversal binding, and compatibility rules;
- HNSW should first extract a HNSW-local adapter seam before Task 63 adds
  RaBitQ, instead of forcing a cross-AM trait too early.

## Validation

Docs-only checkpoint. No code tests were run.

Artifact: `reviews/task-60/014-codec-adapter-adr-and-hnsw-task/artifacts/manifest.md`

## Remaining Task 60 Gate

Task 60 still requires the external benchmark-host 100k/1M DiskANN RaBitQ run
and recorded shipping decision before it can be marked complete.
