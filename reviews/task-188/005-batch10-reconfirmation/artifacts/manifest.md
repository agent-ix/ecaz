# Task 188 corrected batch-10 confirmation manifest

- Task bucket: `reviews/task-188/005-batch10-reconfirmation/`
- Head SHA for the runner change: `6ef2ae2eb` (`parse_benchmark_seed_variants`
  omitted batch default now inherits 10; paired prediction capture added)
- Measurement extension SHA: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`,
  release profile
- Lane: PG18 local, three-owner physical DistANN, isolated one-index-per-table
  surfaces per scale
- Fixture/query identities: `ec_real_10k` query SHA
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  `ec_real_50k` query SHA
  `95ac799257742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  and `ec_real_100k` query SHA
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Storage/scoring: RaBitQ graph payloads, exact-scored training-landmark head,
  cap 16,384, 32 head seeds, BW4 versus BW8, H100, batch-10
- Suite config: `task188-bw8-batch10-suite.json`
- Command: `ecaz bench suite run --config reviews/task-188/005-batch10-reconfirmation/artifacts/task188-bw8-batch10-suite.json --artifact-dir reviews/task-188/005-batch10-reconfirmation/artifacts/run --manifest-output reviews/task-188/005-batch10-reconfirmation/artifacts/run/suite-manifest.json --results-output reviews/task-188/005-batch10-reconfirmation/artifacts/run/results.jsonl --log-file reviews/task-188/005-batch10-reconfirmation/artifacts/run/suite-run.log`
- Timestamp: 2026-07-26; run completed successfully
- Structured source: `run/suite-manifest.json` and `run/results.jsonl`
- Cited compact source: `task188-bw8-batch10-results.log` and the three
  `run/*/distann-multinode-summary.log` files

Every scale used 200 evaluation queries and 2000 recall trials. Paired output
uses the same query IDs and truth cache, computes per-query top-k recall
wins/losses/ties, and uses a deterministic 10,000-resample paired bootstrap.
All physical arms passed topology with three owners and two verified remote
owners; storage is identical within each scale. The 10k/50k/100k generations
were independent fresh builds, so cross-scale comparisons are descriptive;
the BW4/BW8 comparison is paired within each generation.
