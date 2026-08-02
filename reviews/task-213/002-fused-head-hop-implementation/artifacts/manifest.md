# Task 213 implementation artifacts

- Task bucket: `reviews/task-213/`; packet: `002-fused-head-hop-implementation`
- Code head: `cc6a01c662b191e75194bf2c6b38222b6906924b`
- Installed PG18 release extension: `cc6a01c662b191e75194bf2c6b38222b6906924b-dirty`
  (`-dirty` reflects the uncommitted packet suite JSON while the extension was
  built; source code is the stated code head).
- Validation: PG18 cargo checks and four focused crown-cache tests passed;
  see `validation.log`.
- Suite config: `task213-fused-suite.json`.
- Final suite source of truth: `bench-run-final2/suite-manifest.json` and
  `bench-run-final2/results.jsonl` (all 6 steps succeeded).
- Command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-213/002-fused-head-hop-implementation/artifacts/task213-fused-suite.json --artifact-dir reviews/task-213/002-fused-head-hop-implementation/artifacts/bench-run-final2 --log-file reviews/task-213/002-fused-head-hop-implementation/artifacts/bench-run-final2/suite.log --continue-on-error`
- Corpus query SHA: 10k `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  50k `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  100k `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Physical isolated one-index-per-table A/B results (recall / recall-run
  mean ms / storage amplification; unfused, fused):
  - 10k `0.9940 / 40.32 / 1.235867`, `0.9985 / 34.91 / 1.235467`.
  - 50k `0.9595 / 53.91 / 1.332667`, `0.9585 / 41.57 / 1.332667`.
  - 100k `0.9145 / 54.73 / 1.351160`, `0.9300 / 40.81 / 1.351160`.
- Fused provenance is explicitly `seed_set_change=true` at every scale; it
  is not treated as an identity-preserving control. Fused recall counters
  served 6400 crown seeds and recorded 200 fused hops on each recall arm;
  latency counters served 1600 crown seeds and recorded 50 fused hops. Both
  variants recorded zero crown fallbacks.
- The physical retry path now classifies the typed epoch-mismatch error and
  reopens the active generation after resetting stale physical state; other
  search errors remain internal failures.
- Storage rows itemize `ec_distann_crown_cache` as bounded codes-only storage
  (`resident_bytes=resident_bytes_bound=434176` at the 2048-entry arms) with
  coordinator unsharded bytes zero.
- Artifacts retained in this packet are only the suite manifest, structured
  results, and compact per-arm summary logs. Corpus data, predictions,
  operational logs, and PostgreSQL clusters are not committed/resident.
