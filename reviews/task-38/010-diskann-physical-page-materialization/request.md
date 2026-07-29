---
agent: codex
role: coder
model: gpt-5
date: 2026-07-28
task: 38
packet: 010-diskann-physical-page-materialization
status: review-requested
---

# Review Request: Task 38 DiskANN Physical-Page Materialization

Review code checkpoints `2100e7310` and `50b7690d8`, plus the packet-local
evidence described in `artifacts/manifest.md`.

## Scope

This packet responds to findings 9 and 10 in
`../009-authoritative-fault-full/feedback/2026-07-28-02-reviewer.md`.

Checkpoint `2100e7310` restores the OOM lane's timing oracle. The live workload
is marked in SQL, and immediately before SIGKILL an observer connection requires
the target PID to be `active` in `pg_stat_activity` with the marked AM workload
as its current query. A completed workload followed by the safety
`pg_sleep(10)` hold is rejected instead of being reported as a workload kill.

Checkpoint `50b7690d8` hardens production DiskANN physical-page
materialization:

- physical TID divergence is a returned error in release builds;
- missing first-page and physical-block lookups are returned errors rather than
  panics;
- unused line pointers have explicit state and `raw_tuple` rejects them;
- graph iteration visits only occupied physical offsets; and
- a PG18 end-to-end regression creates a real unused line pointer, appends a
  later node, materializes the chain, and forces a nearest-neighbour scan.

## PG18 Regression Evidence

- `artifacts/oom-activity-oracle-unit.log`: the focused OOM activity-oracle
  unit test passes.
- `artifacts/diskann-page-materialization-unit.log`: both focused physical-page
  unit tests pass.
- `artifacts/diskann-unused-line-pointer-pg18.log`: a fresh PG18 database
  creates the extension and executes
  `tests.test_ec_diskann_unused_line_pointer_scan()` successfully.

## Required DiskANN A/B Evidence

The checked-in `diskann-ab-suite.json` was executed only through
`ecaz bench suite` against release PG18 backends on this Apple M5:

- baseline: `147d44d05f0fe2d95a590a5730c6af55091d11c1`, the checkpoint
  immediately before the original DiskANN materialization fix;
- candidate: `50b7690d84024ec58bc570bc506cfa6719c23c9e`;
- matrix: baseline/candidate × 10k/50k/100k ×
  load/recall/latency/storage;
- isolation: separate fresh databases and one DiskANN index per corpus table;
  and
- query shape: 200 queries, `k=10`, list sizes
  `64/128/200/400/800`, forced index, post-recall warm latency.

All 24 selected steps succeeded. Across all 15 scale/list-size recall points,
`recall@k`, worst-query recall, and NDCG are identical. DiskANN index bytes and
bytes per row are also identical at every scale.

At list size 200:

| Scale | Recall baseline/candidate | Mean latency baseline | Mean latency candidate | DiskANN bytes baseline/candidate |
| --- | --- | ---: | ---: | ---: |
| 10k | 1.0000 / 1.0000 | 0.78 ms | 0.74 ms | 4,299,162 / 4,299,162 |
| 50k | 0.9905 / 0.9905 | 1.19 ms | 1.25 ms | 21,600,666 / 21,600,666 |
| 100k | 0.9845 / 0.9845 | 1.59 ms | 1.45 ms | 43,096,474 / 43,096,474 |

Across the complete 15-point latency sweep, candidate mean-latency deltas range
from `-17.4%` to `+7.2%`, consistent with run-to-run local variance and with no
systematic regression. The results support the intended conclusion: the
correctness hardening is recall-neutral and storage-neutral, with no evidenced
latency regression.

## Requested Review

Please verify:

- the OOM lane fails if its marked AM workload is no longer active immediately
  before SIGKILL;
- no release-compiled-out assertion or avoidable panic remains in the touched
  materialization path;
- unused physical offsets cannot be mistaken for empty tuple payloads;
- the PG18 regression actually crosses build/materialize/scan behavior; and
- the two suite manifests and `results.jsonl` files prove the required
  release-backend 10k/50k/100k recall, latency, and storage A/B.

No AWS, remote host, CI, Docker, Intel, or Linux-only command was run. This
packet closes the Apple-M5 findings only. Task 38 remains open for the 67
provider, remote-socket, and cgroup cases designated for Intel/Linux.
