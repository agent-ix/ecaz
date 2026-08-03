# Artifact manifest

- head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`
- task bucket: `reviews/task-206`
- packet: `002-100k-sweep`
- lane / fixture: PG18; 3-owner distann local multinode; pre-registered only
- storage format / rerank mode: physical distann; `rabitq` neighbor scoring
- isolated one-index-per-table or shared-table surface: one physical fixture
  per suite step; no run completed
- command: `target/debug/ecaz bench suite audit --config artifacts/task206-100k-sweep.json`
- command: `target/debug/ecaz bench suite run --config artifacts/task206-100k-sweep.json --dry-run`
- timestamp: `2026-08-03T16:05:48-07:00` (packet preparation)
- corpus: `ec_real_100k`, source directory
  `/home/peter/dev/ecaz/data/task106_full_sweep_100k`; corpus/query files are
  external and are not committed

Artifacts:

- `task206-100k-sweep.json`: checked-in SuiteConfig
- `run/suite-manifest.json`: runner-generated dry-run manifest
- `suite-dry-run.md`: cited audit and nine-arm expansion output
- `setup-attempt.md`: packet-local log summary for the stopped setup attempt
- `run/100k/*.log`: runner and per-node PostgreSQL logs from that attempt
