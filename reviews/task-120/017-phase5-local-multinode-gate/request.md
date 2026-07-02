# Task 120 Phase 5 Local Multi-Node Gate Review

## Request

Review the Task 120 Phase 5 local multi-node gate packet.

This packet is explicitly local multi-node: one coordinator PostgreSQL instance
plus three worker PostgreSQL instances on the same physical machine, with
distinct SPIRE node identities `2`, `3`, and `4`. The runs used static remote
placements, `EcSpireDistributedScan`, remote dispatch, and
`result_source=remote_heap_candidates`. This is not a single-node local scan.

AWS was not used for this packet.

## Code Under Review

Code commit: `c5448e08c781893ae6919b2325625f6248336d7b`

The harness changes add direct-local representative corpus loading and a
packet-local `ecaz bench suite` path for local multi-node Phase 5 evidence.

## Evidence

- Manifest: `artifacts/manifest.md`
- Summary: `artifacts/phase5-local-multinode-summary.md`
- Tiny distributed smoke: `artifacts/static-remote-smoke/`
- 10k suite: `artifacts/real10k-valid/bench-suite/results.jsonl`
- 50k suite: `artifacts/real50k/bench-suite/results.jsonl`
- 100k suite: `artifacts/real100k/bench-suite/results.jsonl`

All real-corpus suite runs passed through `ecaz bench suite`.

## Results

| Scale | Step | nprobe | recall@10 | p50 | p95 | p99 | Storage total | SPIRE index |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | default | 64 | 0.9850 | 42.148 ms | 49.005 ms | 57.794 ms | 168.4 MiB | 9.4 MiB |
| 10k | default | 96 | 0.9855 | 44.638 ms | 50.035 ms | 54.638 ms | 168.4 MiB | 9.4 MiB |
| 10k | rowcap25k | 96 | 0.9855 | 48.339 ms | 68.922 ms | 77.558 ms | 168.4 MiB | 9.4 MiB |
| 50k | default | 64 | 0.9850 | 54.194 ms | 62.366 ms | 71.122 ms | 835.6 MiB | 40.7 MiB |
| 50k | default | 96 | 0.9900 | 59.243 ms | 82.910 ms | 87.674 ms | 835.6 MiB | 40.7 MiB |
| 50k | rowcap25k | 96 | 0.9900 | 59.481 ms | 64.358 ms | 67.666 ms | 835.6 MiB | 40.7 MiB |
| 100k | default | 64 | 0.9730 | 78.257 ms | 117.404 ms | 125.179 ms | 1.6 GiB | 79.7 MiB |
| 100k | default | 96 | 0.9880 | 98.876 ms | 113.926 ms | 134.894 ms | 1.6 GiB | 79.7 MiB |
| 100k | rowcap25k | 96 | 0.9880 | 95.085 ms | 108.528 ms | 114.650 ms | 1.6 GiB | 79.7 MiB |

Remote production-read counters were clean for all real-corpus suite rows:
`status=ready`, `local_pid_sum=0`, `dispatch_sum=600`, `timeout_sum=0`,
`cancel_sum=0`, and `degraded_skip_sum=0`.

## Reviewer Focus

- Confirm this satisfies the required local multi-node distributed gate before
  any AWS run is considered.
- Check that the packet evidence proves remote placements and remote dispatch,
  not a single-node local scan.
- Check the caveats in `artifacts/manifest.md`: rowcap25k did not bind at these
  route counts, NDCG was not emitted by this suite step, and full arbitrary
  row materialization still reports `requires_remote_heap_resolution`.

## Remaining Task 120 Work

This packet does not authorize or claim an AWS result. Task 120 still requires
explicit user approval before any specific AWS 1M run, and AWS 1M evidence is
still required before any product-default or product-claim decision.
