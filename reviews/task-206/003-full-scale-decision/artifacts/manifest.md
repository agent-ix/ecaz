# Artifact manifest

- head SHA: `74d752ddb`
- task bucket: `reviews/task-206`
- packet: `003-full-scale-decision`
- lane / fixture: PG18; 3-owner physical distann; 10k diagnostic
- storage format / rerank mode: physical distann; `rabitq` neighbor scoring
- isolated one-index-per-table or shared-table surface: one diagnostic fixture
- command: `target/debug/ecaz bench suite audit --config artifacts/task206-10k-diagnostic.json`
- timestamp: `2026-08-03T16:30:00-07:00` (packet preparation)
- corpus: `ec_real_10k`, source directory
  `/home/peter/dev/ecaz/data/task106_intel_dbpedia_staged`; corpus/query files
  are external and are not committed

Artifacts:

- `task206-10k-diagnostic.json`: checked-in SuiteConfig
- `run/suite-manifest.json`: runner-generated dry-run manifest
- `suite-dry-run.md`: audit and expansion result
- `run/`: runner output; populated only if the diagnostic completes
