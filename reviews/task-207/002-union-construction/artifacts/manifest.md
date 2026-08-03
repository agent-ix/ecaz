# Artifact manifest

- head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`
- task bucket: `reviews/task-207`
- packet: `002-union-construction`
- lane / fixture: PG18; 3-owner distann local multinode; control/candidate A/B
- storage format / rerank mode: physical distann; `rabitq` neighbor scoring
- isolated one-index-per-table or shared-table surface: one fixture per A/B
  step; no run completed
- command: `target/debug/ecaz bench suite audit --config artifacts/task207-100k-union-ab.json`
- command: `target/debug/ecaz bench suite run --config artifacts/task207-100k-union-ab.json --dry-run`
- timestamp: `2026-08-03T16:06:00-07:00` (packet preparation)
- corpus: `ec_real_100k`, source directory
  `/home/peter/dev/ecaz/data/task106_full_sweep_100k`; corpus/query files are
  external and are not committed

Artifacts:

- `task207-100k-union-ab.json`: checked-in SuiteConfig
- `task207-10k-union-ab.json`: checked-in diagnostic SuiteConfig
- `run/suite-manifest.json`: runner-generated 100k dry-run manifest
- `run-10k/suite-manifest.json`: runner-generated 10k dry-run manifest
- `suite-dry-run.md`: cited audit and expansion output
