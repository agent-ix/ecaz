# Artifact Manifest

- head SHA: `3aef930a1`
- task bucket: `reviews/task-89/`
- packet path: `reviews/task-89/004-validation-matrix/`
- timestamp: `2026-06-08T16:10:00Z`
- lane / fixture / storage format / rerank mode: validation planning only;
  future `storage_format=turboquant`, `turboquant_profile=standard|tqplus`.
- isolated one-index-per-table or shared-table surfaces: matrix requires
  one-index-per-table for final measurements.

## Artifacts

### `task89-validation-matrix.md`

- command used: manual source inspection of existing suite configs and task
  requirements.
- purpose: records required all-AM DBPedia, cross-corpus, and streaming-insert
  drift matrices.

### `task89-all-am-real10k-template.json`

- command used: manual suite-template authoring from existing suite schema.
- purpose: post-port real10k template showing standard-vs-TQ+ load cells for
  IVF, SPIRE, HNSW, and DiskANN.

## Key Result Lines

- DBPedia matrix must cover IVF, SPIRE, HNSW, and DiskANN at 10k/50k/100k.
- Cross-corpus matrix must run all four AMs on at least one non-DBPedia corpus.
- Drift matrix must compare post-insert TQ+ against full-rebuild TQ+ baselines
  at 10%, 25%, and 50% inserted rows.
