# Artifact Manifest

- head SHA: `6ad6a3509d88db3ecc5f20a1a8e448d4a7e14822`
- task bucket: `reviews/task-89/`
- packet path: `reviews/task-89/001-format-design-adr/`
- timestamp: `2026-06-08T14:58:56Z`
- lane / fixture / storage format / rerank mode: documentation-only Phase 1
  ADR; no benchmark fixture; TQ+ design surface only.
- isolated one-index-per-table or shared-table surfaces: not applicable.

## Artifacts

### `spec/adr/ADR-076-turboquant-tqplus-format-and-validation.md`

- command used: manual source inspection and ADR authoring.
- purpose: chooses TQ+ as a `turboquant_profile` under the existing
  TurboQuant storage family, defines compatibility rules, calibration-storage
  policy, per-AM measurement gates, and streaming-insert drift thresholds.

### `spec/adr/index.md`

- command used: manual ADR index update.
- purpose: registers ADR-076 in the canonical ADR index.

### `reviews/task-89/001-format-design-adr/request.md`

- command used: manual review-packet authoring.
- purpose: requests reviewer approval for the Task 89 Phase 1 ADR gate.

## Key Result Lines

- ADR-076 decision: `TQ+ is a TurboQuant calibration profile, not a new
  top-level quantizer family.`
- Preferred DDL: `storage_format = 'turboquant'`,
  `turboquant_profile = 'tqplus'`.
- Drift gate: recall@10 delta `<= 0.5` percentage points at 25% inserted rows
  and `<= 1.0` percentage point at 50% inserted rows.
