# Task 122 Packet 009 Artifact Manifest

- head SHA: `61945d6907d0ba4292cda12cd4f8fe385ad317ce`
- task bucket: `reviews/task-122/009-sidecar-tq-stage2-suite`
- timestamp: `2026-06-27T14:51:35Z`
- runner: `./target/release/ecaz bench suite`
- backend: PG18, `/Users/peter/.pgrx`, port `28818`, database `tqvector_bench`
- build profile: release
- fixture: `data/staged-current/ec_real_{10k,50k,100k}_{corpus,queries,manifest}.json/tsv`
- profile: `ec_ivf`
- base index: RaBitQ, `rerank=off`, `quant_bits=1`, `nlists=64`, `nprobe=32`
- sidecar variants: `f32`, `rabitq8`, `turboquant4`
- read modes: `free`, `tid-sorted`
- query count: `100`
- k: `10`
- matrix:
  - candidate_k `50/100`, final exact f32 width `10`, nprobe `32/64`
  - candidate_k `100`, final exact f32 width `25/50`, nprobe `32/64`
- table isolation: isolated one-prefix-per-scale tables

## Commands

Release CLI build:

```sh
cargo build -p ecaz-cli --release > reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/cargo-build-ecaz-cli-release.log 2>&1
```

Backend check:

```sh
./target/release/ecaz dev sql \
  --pg 18 \
  --db tqvector_bench \
  --socket-dir /Users/peter/.pgrx \
  --port 28818 \
  --raw \
  --sql "SELECT ecaz_build_profile();" \
  --log-output reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/backend-profile-check.log
```

Base sidecar suite:

```sh
./target/release/ecaz bench suite audit \
  --config reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/task122-sidecar-tq-stage2-suite.json \
  --log-file reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/suite-audit-r2.log

./target/release/ecaz bench suite run \
  --config reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/task122-sidecar-tq-stage2-suite.json \
  --dry-run \
  --log-file reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/suite-dry-run-r2.log

./target/release/ecaz bench suite run \
  --config reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/task122-sidecar-tq-stage2-suite.json \
  --host /Users/peter/.pgrx \
  --port 28818 \
  --log-file reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/suite-run-r2.log
```

Final-rerank width suite:

```sh
./target/release/ecaz bench suite audit \
  --config reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/task122-sidecar-tq-stage2-width-suite.json \
  --log-file reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/width-suite-audit.log

./target/release/ecaz bench suite run \
  --config reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/task122-sidecar-tq-stage2-width-suite.json \
  --dry-run \
  --log-file reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/width-suite-dry-run.log

./target/release/ecaz bench suite run \
  --config reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/task122-sidecar-tq-stage2-width-suite.json \
  --host /Users/peter/.pgrx \
  --port 28818 \
  --log-file reviews/task-122/009-sidecar-tq-stage2-suite/artifacts/width-suite-run.log
```

## Artifacts

- `task122-sidecar-tq-stage2-suite.json`: checked-in base suite config.
- `task122-sidecar-tq-stage2-width-suite.json`: checked-in final width sweep config.
- `suite/suite-manifest.json`: base suite manifest; records `12` succeeded steps.
- `suite/results.jsonl`: structured base suite results.
- `width-suite/suite-manifest.json`: width suite manifest; records `6` succeeded steps.
- `width-suite/results.jsonl`: structured final width sweep results.
- `sidecar-summary.txt`: compact extracted base sidecar rows.
- `sidecar-width-summary.txt`: compact extracted width rows.
- `suite-report.md` and `width-suite-report.md`: suite report outputs.
- `suite/*.log` and `width-suite/*.log`: step-local load, sidecar, and storage logs.
- `backend-profile-check.log`: release backend confirmation.
- `cargo-build-ecaz-cli-release.log`: release CLI build log.

No corpus TSVs, truth caches, or generated ground-truth files are committed in
this packet.

## Key Status Lines

```text
[suite:task122-sidecar-tq-stage2-suite] completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
[suite:task122-sidecar-tq-stage2-width-suite] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Decision Rows

Product-shaped comparison uses `tid-sorted` sidecar reads, `candidate_k=100`,
and compares full f32 sidecar rerank (`final_rerank_k=10` after exact f32
sidecar scoring all candidates) to TQ stage-2 with exact f32 over the sidecar
top `25`.

```text
scale  nprobe  baseline f32 recall / p95   TQ->f32@25 recall / p95   TQ sidecar bytes touched
10k    32      1.0000 / 2.389 ms           1.0000 / 1.855 ms         75.39 KiB
10k    64      1.0000 / 2.898 ms           1.0000 / 2.176 ms         75.39 KiB
50k    32      0.9960 / 4.822 ms           0.9960 / 3.885 ms         75.39 KiB
50k    64      1.0000 / 7.835 ms           1.0000 / 6.819 ms         75.39 KiB
100k   32      0.9730 / 8.713 ms           0.9730 / 7.953 ms         75.39 KiB
100k   64      1.0000 / 13.815 ms          1.0000 / 13.517 ms        75.39 KiB
```

RaBitQ8 stage-2 is very close, but uses twice the sidecar bytes:

```text
scale  nprobe  RaBitQ8->f32@25 p95   TQ->f32@25 p95   sidecar bytes touched
10k    32      1.923 ms              1.855 ms          151.17 KiB vs 75.39 KiB
50k    64      6.876 ms              6.819 ms          151.17 KiB vs 75.39 KiB
100k   64      13.522 ms             13.517 ms         151.17 KiB vs 75.39 KiB
```

Final width `10` was too narrow for TQ stage-2 at 50k/100k:

```text
scale  candidate_k  nprobe  TQ->f32@10 recall
50k    100          64      0.9420
100k   100          64      0.9570
```

Base RaBitQ `rerank=off` index storage:

```text
scale  total     ec_ivf index
10k    162.0 MiB 2.9 MiB
50k    806.5 MiB 11.6 MiB
100k   1.6 GiB   22.5 MiB
```

Persisted sidecar table size at 100k:

```text
f32:         585.94 MiB
rabitq8:     147.63 MiB
turboquant4:  73.62 MiB
```

## Interpretation

TQ stage-2 has a measurable path, but this packet is still a modeled sidecar
harness rather than an in-engine product implementation. The evidence supports
promoting a follow-up that implements RaBitQ frontier -> TQ stage-2 -> exact
heap f32 width 25 inside the AM pipeline, with counters for f32 fetches and
materialized rows. It does not support replacing RaBitQ frontier generation or
claiming a durable product win from the sidecar harness alone.
