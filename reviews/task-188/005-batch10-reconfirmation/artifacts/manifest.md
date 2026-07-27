# Task 188 corrected batch-10 confirmation manifest

- Task bucket: `reviews/task-188/005-batch10-reconfirmation/`
- Head SHA for the runner changes: `6ef2ae2eb` (`parse_benchmark_seed_variants`
  omitted batch default now inherits 10; paired prediction capture added) and
  `c810b6e5e` (normalize `physical_benchmark_paired_recall` into `results.jsonl`)
- Measurement extension SHA: `c1c43a9bf66c25b390535ba47e52e0e251a5d6e7`,
  release profile
- Lane: PG18 local, three-owner physical DistANN, isolated one-index-per-table
  surfaces per scale
- Fixture/query identities: `ec_real_10k` query SHA
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  `ec_real_50k` query SHA
  `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  and `ec_real_100k` query SHA
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Storage/scoring: RaBitQ graph payloads, exact-scored training-landmark head,
  cap 16,384, 32 head seeds, BW4 versus BW8, H100, batch-10
- Suite config: `task188-bw8-batch10-suite.json`
- Command: `ecaz bench suite run --config reviews/task-188/005-batch10-reconfirmation/artifacts/task188-bw8-batch10-suite.json --artifact-dir reviews/task-188/005-batch10-reconfirmation/artifacts/run --manifest-output reviews/task-188/005-batch10-reconfirmation/artifacts/run/suite-manifest.json --results-output reviews/task-188/005-batch10-reconfirmation/artifacts/run/results.jsonl --log-file reviews/task-188/005-batch10-reconfirmation/artifacts/run/suite-run.log`
- Timestamp: 2026-07-26; run completed successfully
- Structured source: `run/suite-manifest.json` and `run/results.jsonl`
- Re-normalization command: `ecaz bench suite report --manifest
  reviews/task-188/005-batch10-reconfirmation/artifacts/run/suite-manifest.json
  --results-output
  reviews/task-188/005-batch10-reconfirmation/artifacts/run/results.jsonl`
- Normalized-results SHA-256: `a1e0f57d9f18cfdd5d7ac1c6ed15dc70b947e655838093e939a770dc587c474e`
  (contains three `physical_benchmark_paired_recall` rows)
- Cited compact source: `task188-bw8-batch10-results.log` and the three
  `run/*/distann-multinode-summary.log` files

Every scale used 200 evaluation queries and 2000 recall trials. Paired output
uses the same query IDs and truth cache, computes per-query top-k recall
wins/losses/ties, and uses a deterministic 10,000-resample paired bootstrap.
All physical arms passed topology with three owners and two verified remote
owners; storage is identical within each scale. The 10k/50k/100k generations
were independent fresh builds, so cross-scale comparisons are descriptive;
the BW4/BW8 comparison is paired within each generation.

The accepted batch-10 matrix did not enable stage counters. The mechanism
explanation is therefore qualified by the instrumented eager-0 attribution
run: BW4 averaged 9.72 traversal hop rounds and 25.86 remote candidates per
scan, while BW8 averaged 5.58 rounds and 29.56 remote candidates. That supports
the explanation that fewer dispatch rounds can outweigh wider per-round work
under batch-10, but those counters are not presented as batch-10 measurements.
A fresh batch-10 stage-counter diagnostic was attempted in packet 006 and
failed during the 100k build with `ENOSPC`; its failure is recorded there.
