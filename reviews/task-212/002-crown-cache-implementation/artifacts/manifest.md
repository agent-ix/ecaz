# Task 212 implementation artifacts

- Implementation commits: `4fe5d5c53`, `9c8f2aafb`
- Task bucket: `reviews/task-212/`
- Packet: `002-crown-cache-implementation`
- Code/evidence head: `0a526ac1eb840a975ac00130201058b187f4057d`
- Validation lane: PG18 library and ecaz-cli compile
- Commands:
  - `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check --lib --features pg18`
  - `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check --lib --features 'pg18 distann-head-attribution-benchmark'`
  - `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --lib --features pg18 crown_cache`
- Timestamp: 2026-08-01 (America/Los_Angeles)
- Storage surface: isolated one-index-per-table physical multinode arms; all
  external run directories were removed after capture.
- Result: both compiles passed; crown-cache tests passed (`2 passed`).
- Follow-up: `9c8f2aafb` adds per-backend counter reset/snapshot capture and
  fail-closed activation checks for crown-enabled physical arms.
- Suite config: `artifacts/task212-crown-suite.json`; CLI dry-run expanded nine
  control/crown/crown-width steps across 10k, 50k, and 100k and showed the
  capacity and width-pruning flags on the candidate arms.
- Benchmark command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config artifacts/task212-crown-suite.json --artifact-dir artifacts/bench-run-v3`.
- Benchmark head: `0a526ac1eb840a975ac00130201058b187f4057d` (release profile,
  unanimous across all three nodes).
- Corpus provenance: 10k query SHA `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  50k query SHA `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  100k query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Key results: control/crown/crown-width physical recall and mean latency were
  `0.9940/38.20`, `0.9990/35.00`, `0.9990/32.90` at 10k;
  `0.9595/50.60`, `0.9555/43.50`, `0.9555/45.00` at 50k; and
  `0.9145/54.20`, `0.9135/41.40`, `0.9135/41.50` at 100k.
- Structured artifacts: `bench-run-v3/results.jsonl` and
  `bench-run-v3/suite-manifest.json`; crown counter lines are in each arm's
  `distann-multinode-summary.log`.
