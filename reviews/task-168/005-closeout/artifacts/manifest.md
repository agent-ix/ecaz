# Task 168 Packet 005 — closeout: canonical-sweep confirmation + docs refresh

- Task: `plan/tasks/168-diskann-batched-beam-and-prefetch.md`; branch
  `task-168-diskann-batched-beam`, head at run time `11d285092`
  (all four phase slices landed/shelved).
- Host / backend: Intel desktop, PG18 pgrx tree (port 28818), db
  `tqvector_bench`; release backend (`build-profile.log` in packet 004 —
  same install, no rebuild between packet 004's decision arm and this run).
- Command: the **canonical lane config as-is**, diskann subset:
  `ecaz --host /home/peter/.pgrx --port 28818 bench suite run
  --config crates/ecaz-cli/suites/current/intel-local.json
  --artifact-dir reviews/task-168/005-closeout/artifacts
  --only load-10k-diskann --only recall-10k-diskann ... (12 steps:
  load/recall/latency/storage × 10k/50k/100k)`.
  100k substitutes for the canonical 1m axis (1m not staged locally);
  subsetting via `--only` per the CLAUDE.md convention.
- Fixture: fresh loads at prefixes `current_intel_real{10k,50k,100k}_diskann`
  from `data/staged-current/`, **no reloptions** — exercising the flipped
  rabitq default end-to-end (storage logs show rabitq-sized indexes with
  `reloptions={}`). Loader manifest-prefix warnings are the canonical
  config's `allow_manifest_mismatch` behavior.

## Key results (canonical defaults, W=4 beam, rabitq default)

| Scale | L=64 | L=200 | L=800 | Index |
|---|---|---|---|---|
| 10k | 0.9990 / 3.38 ms p50 | 1.0000 / 4.04 ms | 1.0000 / 5.74 ms | 4.1 MiB |
| 50k | 0.9700 / 3.67 ms | 0.9905 / 4.84 ms | 0.9965 / 9.41 ms | 20.6 MiB |
| 100k | 0.9360 / 4.28 ms | 0.9845 / 6.27 ms | 0.9975 / 14.0 ms | 41.1 MiB |

Full sweep in `results.jsonl` (+ recall/latency/storage logs). Index sizes
match the pre-task doc rows exactly (4.1 / 20.6 MiB) — same on-disk format,
scan-side wins only.

## Cumulative task effect (vs packet 001 W=1 baseline, same host/fixture family)

- 100k: L=200 7.31 → 6.35 ms mean (−13%), L=800 14.5 → 13.9 ms canonical
  (packet-004 pinned-fixture decision arm: 12.2 ms); recall at L=64
  0.9275 → 0.9360.
- 50k: L=800 10.3 → 9.57 ms; every recall cell equal or better.
- Landed: batched-beam W=4 default (packet 002), pooled-decode/allocation
  cleanups + TID hasher (packet 004), rabitq storage default (packet 004).
- Shelved with evidence: graph prefetch + block-grouped reads (packet 003).

## docs/benchmarks.md

`ec_diskann` section gains a Task 168 note (defaults, beam GUC, local-Intel
deltas, evidence pointer). The AWS-lane absolute cells were **not**
overwritten — they were measured on the AWS Intel/Graviton lanes and refresh
on their next canonical run.
