# Artifact Manifest

- head SHA measured: `51293e7531bc1bc29393bff22ed75c909c12e474`
- task bucket: `reviews/task-77/`
- packet path: `reviews/task-77/002-phase1-no-slice-closeout/`
- benchmark packet: `benchmarks/task77-intel-local-candidate-cost-attribution/`
- lane: `intel-local`
- fixture: `ec_real_100k`
- storage format: `turboquant`
- rerank mode: `rerank_width=25`
- isolated/shared surface: isolated Task 77 prefix, one SPIRE index per table
- timestamp: `2026-05-31T23:04:30Z`

## Artifacts

- `request.md`: closeout decision and validation summary.
- `artifacts/clippy-pg18.log`: PG18 clippy validation for the closeout branch.
- `benchmarks/task77-intel-local-candidate-cost-attribution/manifest.md`:
  benchmark source of truth.
- `benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-manifest.json`:
  completed suite manifest.
- `benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/funnel-attribution-summary.json`:
  aggregated per-query attribution summary.

## Key Cited Result Lines

- Suite status: `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- tg64/nprobe64: `10,420,357` candidates, p50 score `69.163 ms`, score share `82.9%`.
- tg96/nprobe96: `15,506,227` candidates, p50 score `102.464 ms`, score share `82.1%`.
- tg128/nprobe128: `20,000,000` candidates, p50 score `132.107 ms`, score share `83.2%`.
- Fixed-candidate microbench: `77,531` candidates, `avx2+fma`, bits4 batch
  `10,774.52 ns/score`.
