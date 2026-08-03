# Artifact manifest

- head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`
- task bucket: `reviews/task-206`
- packet: `003-full-scale-decision`
- lane / fixture: PG18; 3-owner physical distann; 10k diagnostic
- storage format / rerank mode: physical distann; `rabitq` neighbor scoring
- isolated one-index-per-table or shared-table surface: one diagnostic fixture
- command: `target/debug/ecaz bench suite audit --config artifacts/task206-10k-diagnostic.json`
- timestamp: `2026-08-03T16:33:00-07:00` (diagnostic result capture)
- corpus: `ec_real_10k`, source directory
  `/home/peter/dev/ecaz/data/task106_intel_dbpedia_staged`; corpus/query files
  are external and are not committed

Artifacts:

- `task206-10k-diagnostic.json`: checked-in SuiteConfig
- `task206-50k-diagnostic.json`: checked-in 50k SuiteConfig
- `run/suite-manifest.json`: runner-generated dry-run manifest
- `run-50k/suite-manifest.json`: runner-generated 50k dry-run manifest
- `suite-dry-run.md`: audit and expansion result
- `run/results.jsonl`: normalized suite result rows cited by the request
- `run/10k/distann-multinode-summary.log`: packet-local runner summary with
  topology, recall, latency, and storage lines
- `run/10k/physical-bw32-h8-{recall,latency}.log`: child benchmark outputs
- `run/10k/*-predictions.json`: recall prediction artifacts
