# Task 212 implementation artifacts

- Task bucket: `reviews/task-212/`; packet: `002-crown-cache-implementation`
- Code head: `a8b1699528e593b45f55fc25329199714d4627ff`
- Installed PG18 release extension: `a8b1699528e593b45f55fc25329199714d4627ff`
  (release profile, committed tree; three-node preflight was unanimous).
- Validation: PG18 focused physical handoff test passed after the lifecycle
  coverage was added; see `validation-followup.log`.
- Suite configs: `task212-crown-suite.json`, `task212-followup-suite.json`, and
  `task212-pruning-suite.json`.
- Capacity source of truth: `capacity-matrix-summary.md`, the compact
  per-arm summary logs under `bench-run-capacity-release-a8b169952/`, and
  `suite-manifest-r2.json` plus `results-r2.jsonl` for the two successful 100k
  reruns. The summary table is the merged nine-arm result record; the earlier
  partial-initialization records are not evidence.
- Single-variable pruning source of truth:
  `bench-run-pruning-10k-fixed/results.jsonl` and
  `bench-run-pruning-50-100k/results.jsonl` (the corrected 10k arm plus all
  four 50k/100k arms succeeded).
- Command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-212/002-crown-cache-implementation/artifacts/task212-capacity-suite.json --artifact-dir reviews/task-212/002-crown-cache-implementation/artifacts/bench-run-capacity-release-a8b169952 --continue-on-error`
- Pruning command (2026-08-02T15:25:30-07:00): `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-212/002-crown-cache-implementation/artifacts/task212-pruning-suite.json --artifact-dir reviews/task-212/002-crown-cache-implementation/artifacts/bench-run-pruning-50-100k --log-file reviews/task-212/002-crown-cache-implementation/artifacts/bench-run-pruning-50-100k/suite.log --continue-on-error`
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
- The width arm is a multi-variable activation demonstration, not a pruning
  attribution A/B: it changes `head_sample_exact/head_seed_count=32` to
  `persisted_head/head_seed_count=1` in addition to enabling pruning. Its
  recall/latency deltas must not be attributed to pruning alone. A promotion
  decision requires a single-variable `persisted_head`, equal-seed-count,
  pruning-off versus pruning-on A/B at capacity 2048 at every scale.
- Crown stats: after the gating fix, plain arms rank and consume zero crown
  seeds; they use the full head fan-out. Width activation was nonzero at every
  scale; recall-arm pruning
  was 0/67/71 shards at 10k/50k/100k, with 200/200/200 pruning activations.
  The 10k cap-1024 arm recorded activation even when no complete shard could
  be removed.
- Single-variable pruning A/B is now present in
  `bench-run-pruning-50-100k/` and the corrected 10k result is in
  `bench-run-pruning-10k-fixed/`. All six 50k/100k steps and the corrected 10k
  arm passed with `persisted_head`, `head_search_width=32`,
  `head_seed_count=32`, `crown_capacity=2048`, and `skip_single_control=true`.
  Physical recall / latency / storage results are: 10k off
  `0.9990 / 32.90 ms / 1.235333`, on `0.9990 / 33.00 ms / 1.235467`; 50k off
  `0.9545 / 43.30 ms / 1.332667`, on `0.9545 / 42.30 ms / 1.332667`; 100k
  off `0.9265 / 40.00 ms / 1.351160`, on
  `0.9265 / 42.30 ms / 1.351147`. The pruning arm activated 200 times on
  recall and 50 times on latency at each scale, but pruned zero shards at
  every scale. This is an activation and no-effect result, not evidence of a
  pruning latency win.
- The full fused 512/2048/4096 × 10k/50k/100k sizing sweep is complete;
  `capacity-matrix-summary.md` records the exact result table and the selected
  2048-entry opt-in capacity.
- Storage rows itemize `ec_distann_crown_cache` as bounded codes-only storage:
  `capacity=512, 2048, or 4096`, `entries=capacity`, `resident_bytes=bound`, and
  `within_capacity_bound=true`; coordinator unsharded bytes remain zero.
- Artifacts retained in this packet are only the suite manifest, structured
  results, and compact per-arm summary logs. Corpus data, predictions,
  operational logs, and PostgreSQL clusters are not committed/resident.
