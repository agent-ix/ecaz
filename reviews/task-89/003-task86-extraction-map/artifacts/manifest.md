# Artifact Manifest

- head SHA: `0bdc8a025`
- task bucket: `reviews/task-89/`
- packet path: `reviews/task-89/003-task86-extraction-map/`
- timestamp: `2026-06-08T15:45:00Z`
- lane / fixture / storage format / rerank mode: implementation extraction
  map only; no benchmark fixture.
- isolated one-index-per-table or shared-table surfaces: not applicable.

## Artifacts

### `task86-tqplus-extraction-map.md`

- command used: `git show` inspection of preserved Task 86 commits.
- purpose: separates reusable shared TQ+ math from rejected Task 86
  `turboquant_tqplus` storage-format wiring.

## Key Result Lines

- Reuse shared `src/quant/prod.rs` TQ+ math.
- Do not re-land `StorageFormat::TurboQuantTqPlus = 4` as the production
  write path.
- First post-ADR code slice should be shared math plus unit coverage only.
