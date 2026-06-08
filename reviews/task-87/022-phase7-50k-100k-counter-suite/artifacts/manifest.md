# Task 87 Packet 022 Artifact Manifest

- head SHA: `3e0c0969ec8b55e90be6435168cfbe29728c591e`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/022-phase7-50k-100k-counter-suite/`
- timestamp: `2026-06-08T16:30:06-07:00`
- runner: `ecaz bench suite`
- suite config: `reviews/task-87/022-phase7-50k-100k-counter-suite/phase7-50k-100k-counter-suite.json`
- suite status: `completed=19 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- database: `postgres`
- socket dir: `/home/peter/.pgrx`
- port: `28818`
- storage surface: existing isolated real-corpus indexes from packet 013/014 setup; no shared-table benchmark surface introduced by this packet.

## Commands

Audit:

```sh
target/debug/ecaz bench suite audit --config reviews/task-87/022-phase7-50k-100k-counter-suite/phase7-50k-100k-counter-suite.json --log-file reviews/task-87/022-phase7-50k-100k-counter-suite/artifacts/suite-audit.log
```

Run:

```sh
target/debug/ecaz bench suite run --config reviews/task-87/022-phase7-50k-100k-counter-suite/phase7-50k-100k-counter-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-87/022-phase7-50k-100k-counter-suite/artifacts/run-manifest.json --results-output reviews/task-87/022-phase7-50k-100k-counter-suite/artifacts/results.jsonl --log-file reviews/task-87/022-phase7-50k-100k-counter-suite/artifacts/run.log
```

Status:

```sh
target/debug/ecaz bench suite status --manifest reviews/task-87/022-phase7-50k-100k-counter-suite/artifacts/run-manifest.json
```

## Artifact Index

- `suite-audit.log`: suite audit output; key line: `[suite:task87-phase7-50k-100k-counter-suite] audit passed: 19 steps`.
- `run.log`: full suite run log.
- `run-manifest.json`: structured suite manifest emitted by `ecaz bench suite run`.
- `results.jsonl`: structured parsed suite results.
- `status.log`: status output; key line: `[suite:task87-phase7-50k-100k-counter-suite] completed=19 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `run/*.log`: packet-local logs for the individual precheck, recall, latency, storage, SPIRE pipeline, and HNSW probe steps.

## Result Summary

### real50k IVF

- lane / fixture: `task67_local_50k_ivfrabitq`
- storage format: `rabitq`
- rerank mode: `rerank_width=25`, `nprobe=64`
- recall off/on: `0.9300` / `0.9300`
- latency off p50/p95/p99: `12.2/13.8/15.3 ms`
- latency on p50/p95/p99: `12.3/15.5/18.0 ms`
- counters: all `surface=ivf` counters are zero in off and on logs; this is a RaBitQ surface and not a Task 87 TurboQuant no-QJL LUT32 route.
- storage: total `840.9 MiB`; indexes `47.1 MiB`

### real50k SPIRE

- lane / fixture: `task87_phase6_real50k_spire`
- storage format: `turboquant`
- rerank mode: `rerank_width=25`, `nprobe=24`
- recall off/on: `0.9690` / `0.9690`
- latency off p50/p95/p99: `21.997/25.240/27.240 ms`
- latency on p50/p95/p99: `18.751/21.833/23.164 ms`
- counters off: all zero.
- counters on: `surface=spire flushes=4800 candidates=1739476 elapsed_ms=2006.536739 lut32_flushes=4800 lut32_candidates=1739476`
- storage: total `834.3 MiB`; indexes `40.5 MiB`

### real50k HNSW Probe

- lane / fixture: `current_intel_real50k_hnsw`
- rerank/search mode: `ef_search=128`
- latency p50/p95/p99: `5.73/23.7/34.1 ms`
- counters: all `surface=hnsw` counters are zero.
- interpretation: this existing real-corpus HNSW profile does not exercise the Task 87 common candidate-batch scorer and remains on the accepted Phase 5 structural route for this task.

### real100k IVF

- lane / fixture: `task28_ivf_tq100k_n64w25`
- storage format: TurboQuant no-QJL 4-bit
- rerank mode: `rerank_width=25`, `nprobe=64`
- recall off/on: `1.0000` / `1.0000`
- latency off p50/p95/p99: `172.7/183.2/186.5 ms`
- latency on p50/p95/p99: `146.2/168.0/179.2 ms`
- counters off: all zero.
- counters on: `surface=ivf flushes=78200 candidates=20000000 elapsed_ms=23574.111606 lut32_flushes=78200 lut32_candidates=20000000`
- storage: total `1.6 GiB`; indexes `89.5 MiB`

### real100k SPIRE

- lane / fixture: `task74_intel_spire_highrecall_tg128_b0`
- storage format: `turboquant`
- rerank mode: `rerank_width=25`, `nprobe=24`
- recall off/on: `0.9100` / `0.9100`
- latency off p50/p95/p99: `41.179/48.845/51.872 ms`
- latency on p50/p95/p99: `35.062/40.653/46.962 ms`
- counters off: all zero.
- counters on: `surface=spire flushes=4800 candidates=3842410 elapsed_ms=4486.740935 lut32_flushes=4800 lut32_candidates=3842410`
- storage: total `1.6 GiB`; indexes `81.8 MiB`

### real100k HNSW Probe

- lane / fixture: `current_intel_real100k_hnsw`
- rerank/search mode: `ef_search=128`
- latency p50/p95/p99: `7.58/43.2/72.8 ms`
- counters: all `surface=hnsw` counters are zero.
- interpretation: this existing real-corpus HNSW profile does not exercise the Task 87 common candidate-batch scorer and remains on the accepted Phase 5 structural route for this task.
