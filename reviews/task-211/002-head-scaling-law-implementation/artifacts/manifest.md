# Task 211 implementation artifacts

- Implementation commits: `4fe5d5c53`, `9c8f2aafb`
- Task bucket: `reviews/task-211/`
- Packet: `002-head-scaling-law-implementation`
- Code/evidence head: `0a526ac1eb840a975ac00130201058b187f4057d`
- Validation lane: PG18 library and ecaz-cli compile
- Commands:
  - `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check --lib --features pg18`
  - `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check --lib --features 'pg18 distann-head-attribution-benchmark'`
  - `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --lib --features pg18 head_scaling_attestation_is_deterministic_and_digest_bound`
- Timestamp: 2026-08-01 (America/Los_Angeles)
- Storage surface: isolated one-index-per-table physical multinode arms; all
  external run directories were removed after capture.
- Result: both compiles passed; the focused attestation test passed (`1 passed`).
- Follow-up: `9c8f2aafb` adds counter capture/reset to recall/latency output and
  labels crown-induced seed-set changes in provenance.
- Suite config: `artifacts/task211-head-law-suite.json`; CLI dry-run expanded
  six control/law steps across 10k, 50k, and 100k and showed the three law
  flags on the law arms.
- Benchmark command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config artifacts/task211-head-law-suite.json --artifact-dir artifacts/bench-run-v2`
- Benchmark head: `0a526ac1eb840a975ac00130201058b187f4057d` for 100k; 10k/50k
  results were run at `d4c39c8218055195eed559249116251bf0315f73`.
- Corpus provenance: `ec_real_10k` query SHA
  `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  `ec_real_50k` query SHA
  `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  `ec_real_100k` query SHA
  `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Key results: control/law physical recall and mean latency were
  `0.9940/39.00` vs `0.9940/37.90` at 10k,
  `0.9595/51.80` vs `0.9595/51.00` at 50k, and
  `0.9145/53.00` vs `0.9145/52.00` at 100k; storage ratios were
  `1.235867`, `1.332667`, and `1.351147`.
- Structured artifacts: `bench-run-v2/results.jsonl` and
  `bench-run-v2/suite-manifest.json`.
