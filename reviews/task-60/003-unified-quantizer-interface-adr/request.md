# Review Request: Unified Quantizer Interface ADR

- task: `plan/tasks/60-ec-diskann-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- topic: `unified-quantizer-interface-adr`

## What Changed

- Added `spec/adr/ADR-071-unified-quantizer-interface.md`.
- Updated `spec/adr/index.md`.

## Review Focus

- Does the ADR make the shared quantizer interface direction clear without
  making Task 60 depend on a broad extraction?
- Are the extraction triggers concrete enough for the follow-on HNSW then IVF
  RaBitQ work?
- Does the boundary between quantizer family semantics and AM-owned payload
  layout match the existing architecture?

## Validation

Static documentation change only. No tests run.
