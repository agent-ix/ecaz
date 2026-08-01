# Task 213 implementation artifacts

- Implementation commits: `4fe5d5c53`, `9c8f2aafb`
- Task bucket: `reviews/task-213/`
- Packet: `002-fused-head-hop-implementation`
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
- Follow-up: `9c8f2aafb` reports `fused_head_hops` and fails a fused arm when
  the activation counter is zero; seed-set changes are labeled explicitly.
- Suite config: `artifacts/task213-fused-suite.json`; CLI dry-run expanded six
  crown-on unfused/fused steps across 10k, 50k, and 100k and showed the fused
  flag only on fused arms.
- Benchmark command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config artifacts/task213-fused-suite.json --artifact-dir artifacts/bench-run-v3`.
- Benchmark head: `0a526ac1eb840a975ac00130201058b187f4057d` (release profile,
  unanimous across all three nodes).
- Corpus provenance: 10k query SHA `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  50k query SHA `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  100k query SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Key results: unfused/fused physical recall and mean latency were
  `0.9990/33.90` vs `0.9990/34.80` at 10k,
  `0.9555/44.60` vs `0.9555/44.60` at 50k, and
  `0.9135/40.80` vs `0.9135/41.30` at 100k; storage ratios were
  `1.235467`, `1.332667`, and `1.351173`.
- Structured artifacts: `bench-run-v3/results.jsonl` and
  `bench-run-v3/suite-manifest.json`; activation counter lines are in each
  arm's `distann-multinode-summary.log`.
