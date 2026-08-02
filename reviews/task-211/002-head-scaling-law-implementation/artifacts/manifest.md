# Task 211 implementation artifacts

- Task bucket: `reviews/task-211/`; packet: `002-head-scaling-law-implementation`
- Code head: `a08f6fe6080dd3023241c3cf38a822fac9bb44c2`
- Installed PG18 extension SHA: `0a526ac1eb840a975ac00130201058b187f4057d`
- Validation: PG18 library/CLI checks and the deterministic attestation test
  passed; see `validation.log`.
- Suite configs: `task211-head-law-suite.json` (three corrected 0.02 law arms)
  and `task211-control100-suite.json` (corrected 100k fixed-cap control).
- Commands:
  - `/home/peter/.cargo-target/debug/ecaz bench suite run --config reviews/task-211/002-head-scaling-law-implementation/artifacts/task211-head-law-suite.json --artifact-dir reviews/task-211/002-head-scaling-law-implementation/artifacts/bench-run-law-fixed --only law-10k --only law-50k --only law-100k`
  - `/home/peter/.cargo-target/debug/ecaz bench suite run --config reviews/task-211/002-head-scaling-law-implementation/artifacts/task211-control100-suite.json --artifact-dir reviews/task-211/002-head-scaling-law-implementation/artifacts/bench-run-control100-fixed`
- Source of truth: `bench-run-law-fixed/results.jsonl` and
  `bench-run-control100-fixed/results.jsonl`; final fixed-cap 10k/50k
  baselines are also present in the counter-enabled Task 212 suite results.
- Corpus query SHA: 10k `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  50k `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  100k `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Physical A/B (recall / mean ms; storage ratio):
  - 10k control `0.9940 / 39.50 / 1.235467`; law `0.9950 / 39.10 / 1.235467`; sampled records `200`; hops `11.90` vs `11.60` per latency scan.
  - 50k control `0.9595 / 52.70 / 1.332640`; law `0.9555 / 53.50 / 1.332693`; sampled records `1000`; hops `16.10` vs `14.20`.
  - 100k control `0.9145 / 53.30 / 1.351173`; law `0.9155 / 55.80 / 1.351147`; sampled records `2000`; hops `15.68` vs `14.96`.
- Decision: the law is implemented and measured, but the shipped default stays
  at fixed cap 4096 under the task stop condition; rate `0.02` remains opt-in.
- Physical surface: isolated one-index-per-table multinode arms. All external
  run directories were removed after capture. No corpus data is committed.
