# Artifact Manifest

- head SHA: `c8eb583ca`
- task bucket: `reviews/task-89/`
- packet path: `reviews/task-89/002-cross-am-inventory/`
- timestamp: `2026-06-08T15:20:00Z`
- lane / fixture / storage format / rerank mode: implementation inventory
  only; no benchmark fixture.
- isolated one-index-per-table or shared-table surfaces: not applicable.

## Artifacts

### `cross-am-tqplus-inventory.md`

- command used: manual source inspection using `rg` and focused `sed` reads.
- purpose: maps current IVF, SPIRE, HNSW, and DiskANN TurboQuant support and
  identifies the concrete files needed for TQ+ work.

## Key Result Lines

- DiskANN currently has no baseline `turboquant` storage format.
- IVF and SPIRE already have top-level storage-format reloption patterns but
  no TurboQuant profile reloption.
- HNSW uses the shared `quant::Family` storage-format reloption and needs
  profile metadata/version discipline.
