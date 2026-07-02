# Task 133 IVF Stage-Latency Attribution Artifacts

- task bucket: `reviews/task-133/001-stage-attribution`
- code commits:
  - `27518cbb2` Stamp suite manifests with git + backend provenance
  - `a05babf74` Add IVF query stage-latency attribution counters (Task 133)
- base commit: `eced7e1bc` (task-125 closeout review tip)
- lane: local PG18 (Homebrew 18.3, aarch64-apple-darwin, Apple M5 Pro), `tqvector_bench`
- host cores: 6 "Super" @128 KB L1D + 12 "Performance" @64 KB L1D (hw.perflevel*)
- fixture: staged real corpus (dbpedia 1536-dim), `ec_ivf`, `storage_format=turboquant`,
  `bits=4`, `seed=42`, `nprobe=32`, k=10
- runner: `target/release/ecaz bench suite`
- installed backend: release dylib, `ecaz_build_git_sha() = a05babf74…` (verified via SQL
  before the run; suite manifests record backend git sha + runner git sha)
- isolation: fresh `task133_tq_ivf_real{10k,50k,100k}` one-index-per-prefix tables loaded
  by this suite (extension was dropped/recreated first, invalidating older prefixes)

## Fixture provenance (data/staged-current, regenerated this session)

Staged via hardlink of the task60 100k corpus + `ecaz corpus subset` (10k/50k derived
deterministically from the 100k corpus; queries = rows beyond the corpus split).
Recall at nprobe=32 reproduces task-125 packet 001 exactly (0.9734 / 0.9521 / 0.8969),
confirming fixture equivalence.

| prefix | corpus rows | corpus sha256 (16) | queries rows | queries sha256 (16) |
|---|---|---|---|---|
| ec_real_10k | 10000 | c67c5810b66d982d | 200 | a2c191bb742017d8 |
| ec_real_50k | 50000 | 56023baaa7bc42f7 | 1000 | 95ac7992578aa80b |
| ec_real_100k | 100000 | 07275cfd5a7a4b41 | 1000 | a7cbec6fc44f6c14 |

## Suite Config

- `tq-ivf-stage-suite.json` (bespoke: task-125 A/B config + `ivf_stage_counters: true`
  on latency steps; bespoke rather than the canonical lane config because this packet
  needs the stage-counter option and the task-125-comparable nprobe=32 point, not the
  full canonical sweep)

## Commands

Full suite (load + recall + latency + storage), stale-CLI run (stage lines missing from
latency logs — CLI predated the flag; kept for load/recall/storage provenance):

```sh
target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 \
  bench suite run --config reviews/task-133/001-stage-attribution/tq-ivf-stage-suite.json \
  --artifact-dir reviews/task-133/001-stage-attribution/artifacts \
  --manifest-output .../suite-manifest.json --results-output .../results.jsonl
```

Latency-only re-runs with the rebuilt CLI (`--only-tag latency`):
`suite-manifest-latency-rerun.json` / `results-latency-rerun.jsonl` (noisy: overlapped a
cargo rebuild; retained for the stage-share data) and
`suite-manifest-latency-quiet.json` / `results-latency-quiet.jsonl` (quiet machine —
citation source for stage shares).

## Key Results

Latency (mean per query, nprobe=32):

| scale | task-125 int16 baseline | first run (timers) | quiet re-run (timers) | pre-timer dylib (quiet) |
|---|---|---|---|---|
| 10k | 0.88 ms | 0.93 ms | 1.09 ms | 1.21 ms |
| 50k | 1.88 ms | 1.85 ms | 1.99 ms | 2.05 ms |
| 100k | 2.73 ms | 2.96 ms | 3.21 ms | 3.23 ms |

Timer-overhead verdict: **neutral** — the pre-timer dylib (built from `eced7e1bc`
in `.task-worktrees/pretimer-baseline`, verified via missing
`ecaz_build_git_sha()` + `ecaz_build_profile()=release`) measures the same or
slightly higher than the with-timer build on the same tables in the same quiet
session. The elevation of both builds vs the task-125 session is environmental
(tables re-created after extension drop; different physical layout / cache
state), not attributable to the timers. Kernel cross-check: 39.16 ms (pre-timer)
vs 39.71 ms (timers) @100k×32 scans.

Stage attribution (per query, from `[ivf-stage-counters]` lines in
`latency-ivf-tq-real*.log`; sanity: probe_plan sits outside the approximate-scan window;
approximate_scan ≈ posting_visit + topk_collect within ~3 µs at every scale):

| stage (µs/query) | 10k | 50k | 100k |
|---|---|---|---|
| approximate_scan | 951 | 1653 | 2697 |
| posting_visit | 878 | 1509 | 2481 |
| scratch_flush | 469 | 910 | 1355 |
| scorer_batch | 433 | 836 | 1252 |
| candidate_record (dedup+heap) | 28 | 58 | 84 |
| topk_collect | 72 | 142 | 213 |
| probe_plan | 48 | 89 | 127 |
| derived: page I/O + entry parse (visit − flush) | 409 | 599 | 1126 |
| derived: SoA copy (flush − scorer − record) | 8 | 16 | 19 |

Cross-check: scorer_batch (40.05 ms / 32 scans @100k) vs block-kernel counters
(39.49 ms) — agree within dispatch overhead.

(µs/query figures above from the `-rerun` pass; stage *shares* are stable across passes,
absolute latencies cited from the quiet pass.)

## Attribution summary (the ~40% non-scorer answer)

At 100k, of the 2.70 ms approximate-scan window: scorer 46%, **posting page I/O +
entry parse 42%**, top-k collect 8%, probe plan+record ~5%, SoA copy <1%.
Additionally ~0.5–0.6 ms/query falls outside the scan window entirely (centroid scoring,
LUT query prep, executor/gettuple overhead) — not yet decomposed.

Rerank stages are zero in this config (no heap_f32 rerank at nprobe=32 default).
