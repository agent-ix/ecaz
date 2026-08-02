# Task 213 implementation artifacts

- Task bucket: `reviews/task-213/`; packet: `002-fused-head-hop-implementation`
- Code head: `a08f6fe6080dd3023241c3cf38a822fac9bb44c2`
- Installed PG18 extension SHA: `0a526ac1eb840a975ac00130201058b187f4057d`
- Validation: PG18 library/CLI checks and crown support tests (`2 passed`);
  see `validation.log`.
- Suite config: `task213-fused-suite.json`; final source of truth is
  `bench-run-counters/results.jsonl` and `bench-run-counters/suite-manifest.json`.
- Command: `/home/peter/.cargo-target/debug/ecaz bench suite run --config reviews/task-213/002-fused-head-hop-implementation/artifacts/task213-fused-suite.json --artifact-dir reviews/task-213/002-fused-head-hop-implementation/artifacts/bench-run-counters --resume-from reviews/task-213/002-fused-head-hop-implementation/artifacts/bench-run-counters/suite-manifest.json`
- Corpus query SHA: 10k `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  50k `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  100k `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Physical A/B (recall / mean ms / storage ratio; unfused, fused):
  - 10k `0.9990 / 35.40 / 1.235467`, `0.9990 / 34.20 / 1.235600`; hops `9.90 / 9.90` per latency scan.
  - 50k `0.9555 / 45.30 / 1.332667`, `0.9555 / 45.70 / 1.332667`; hops `13.60 / 13.60`.
  - 100k `0.9135 / 44.20 / 1.351147`, `0.9135 / 41.10 / 1.351173`; hops `12.48 / 12.48`.
- Activation: crown served `6400` recall seeds and `1600` latency seeds with
  `0` fallbacks on each arm; fused arms recorded `fused_head_hops=200` on
  recall and `50` on latency. Fused provenance marks the seed-set change
  explicitly, and measured recall is unchanged versus unfused at every scale.
- Storage rows report `coordinator_resident_unsharded_bytes=0`.
- Physical surface: isolated one-index-per-table multinode arms. All external
  run directories were removed after capture. No corpus data is committed.
