# Artifact Manifest: Task 51 Round Closeout

- head SHA at closeout: `a22c4d57982c5c5dfb1a9e428ea9dfb8320d6972`
- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/023-round-closeout/`
- lane: Task 51 closeout, IVF and RaBitQ only
- vchord / pgvectorscale: not run
- AWS final gate packet: `reviews/task-51/017-aws-current-head-final-gate/`
- AWS benchmark packet: `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/`
- final AWS stack state: `down`
- final retained snapshot: `snap-0758119609e81ab7f`
- running AWS cost: `$0.00/hr`

## Evidence Packets

- Exp 1 / counter baseline: `reviews/task-51/001-ivf-explain-counter-cleanup/`, `reviews/task-51/007-local-ivf-rabitq-scale-counters/`, `reviews/task-51/011-aws-ivf-rabitq-final-gate/`, `reviews/task-51/017-aws-current-head-final-gate/`
- Exp 2 / geometry: `reviews/task-51/006-local-ivf-rabitq-geometry/`, `reviews/task-51/007-local-ivf-rabitq-scale-counters/`
- Exp 3 / scratch SoA: `reviews/task-51/015-ivf-scratch-soa-batch-decode/`, `reviews/task-51/019-ivf-scratch-chunked-bits1/`
- Exp 4 / heap rerank locality: `reviews/task-51/021-heap-rerank-locality-gate/`
- Exp 5 / adaptive nprobe: `reviews/task-51/012-ivf-adaptive-nprobe/`, `reviews/task-51/014-local-ivf-adaptive-nprobe-smoke/`, `reviews/task-51/018-local-ivf-adaptive-nprobe-ratio/`
- Exp 6 / posting layout v2 gate: rejected through Exp 3 evidence; no format change started
- Exp 7 / sidecar measurement: `reviews/task-51/008-ivf-rabitq-sidecar-rerank-bench/`, `reviews/task-51/016-ivf-sidecar-real-io/`, `reviews/task-51/020-sidecar-tid-sorted-assumptions/`, `reviews/task-51/022-sidecar-concurrency-smoke/`, `reviews/task-51/017-aws-current-head-final-gate/`

## Final AWS Evidence

`benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/suite-status-local-after-pull.log`:

```text
[suite:task51-aws-ivf-rabitq-current-head-final-gate] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

`benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/artifacts/cloud-status-after-down.log`:

```text
profile:  10k-medium
state:    down
snapshot: snap-0758119609e81ab7f
cost:     ~$0.00/hr running, ~$4.00/mo retained storage
```
