# Manifest: Task 123 Closeout Decline Response

- Head SHA when prepared: `946a52c47`
- Task bucket: `reviews/task-123/013-closeout-decline-response/`
- Date: 2026-06-28
- Purpose: retract packet 012 closeout, index the packet 011 structured latency
  rows, and record the projection-failure diagnosis.

## Artifacts

### `latency-trace.jsonl`

- Source packet: `reviews/task-123/011-multi-instance-100k-timeline-rerun/`
- Source results:
  - `artifacts/n128-b4-200q-source/bench-suite/results-idonly.jsonl`
  - `artifacts/n1024-b2-200q-source/bench-suite/results-idonly.jsonl`
- Lane: contained local multi-instance, one coordinator plus three local worker
  PostgreSQL instances on one host
- Fixture: `ec_real_100k`
- Storage format: `rabitq`
- Projection: id-only
- Queries: 200
- Key result lines cited by `request.md`:
  - n128 b4/tr50/f8 np96: `latency_p50=5408.521 ms`,
    `latency_p95=5815.967 ms`, `recall@k=1.0000`
  - n1024 b2/tr50/f8 np64: `latency_p50=770.595 ms`,
    `latency_p95=860.296 ms`, `recall@k=1.0000`

### `projection-failure-diagnosis.md`

- Source log:
  `reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/coord-postgres.log`
- Failure: `id,source` projection failed before timing rows.
- Root signal in committed log: remote typed tuple payload was `12316` bytes
  while `ec_spire.max_remote_payload_bytes_per_row` was `1024`.
- Status: unresolved measurement blocker.

## Commands

Artifact extraction/check commands:

```text
sed -n '8,10p' reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n128-b4-200q-source/bench-suite/results-idonly.jsonl
sed -n '8,10p' reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/bench-suite/results-idonly.jsonl
rg -n -C 3 "remote_heap_resolution_failed|remote_payload_too_large" reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/coord-postgres.log
df -h /tmp/ecaz-task123 /home/peter/dev/ecaz
```
