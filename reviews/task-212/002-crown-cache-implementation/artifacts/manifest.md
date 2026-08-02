# Task 212 implementation artifacts

- Task bucket: `reviews/task-212/`; packet: `002-crown-cache-implementation`
- Code head: `cc6a01c662b191e75194bf2c6b38222b6906924b`
- Installed PG18 release extension: `cc6a01c662b191e75194bf2c6b38222b6906924b-dirty`
  (`-dirty` reflects the uncommitted packet suite JSON while the extension was
  built; source code is the stated code head).
- Validation: PG18 cargo checks and four focused crown-cache tests passed;
  see `validation.log`.
- Suite config: `task212-crown-suite.json`.
- Final suite source of truth: `bench-run-final2/suite-manifest.json` and
  `bench-run-final2/results.jsonl` (all 9 steps succeeded).
- Command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-212/002-crown-cache-implementation/artifacts/task212-crown-suite.json --artifact-dir reviews/task-212/002-crown-cache-implementation/artifacts/bench-run-final2 --log-file reviews/task-212/002-crown-cache-implementation/artifacts/bench-run-final2/suite.log --continue-on-error`
- Corpus query SHA: 10k `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`,
  50k `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`,
  100k `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`.
- Physical isolated one-index-per-table A/B results (recall / recall-run
  mean ms / storage amplification):
  - 10k control `0.9940 / 39.41 / 1.235467`, crown
    `0.9940 / 41.39 / 1.235600`, crown-width
    `0.9975 / 51.45 / 1.235467`.
  - 50k control `0.9595 / 52.78 / 1.332693`, crown
    `0.9595 / 53.45 / 1.332667`, crown-width
    `0.9550 / 77.22 / 1.332667`.
  - 100k control `0.9145 / 51.30 / 1.351173`, crown
    `0.9145 / 52.11 / 1.351147`, crown-width
    `0.9200 / 98.29 / 1.351160`.
- Plain crown seed digests matched control at all three scales (`same_seed=true`);
  width arms are explicitly labeled `seed_set_change=true`.
- Crown stats: plain arms served 6400 recall and 1600 latency seeds with zero
  fallbacks. Width activation was nonzero at every scale; recall-arm pruning
  was 0/67/71 shards at 10k/50k/100k, with 200/200/200 pruning activations.
  The 10k cap-1024 arm recorded activation even when no complete shard could
  be removed.
- Storage rows itemize `ec_distann_crown_cache` as bounded codes-only storage:
  `capacity=1024 or 2048`, `entries=capacity`, `resident_bytes=bound`, and
  `within_capacity_bound=true`; coordinator unsharded bytes remain zero.
- Artifacts retained in this packet are only the suite manifest, structured
  results, and compact per-arm summary logs. Corpus data, predictions,
  operational logs, and PostgreSQL clusters are not committed/resident.
