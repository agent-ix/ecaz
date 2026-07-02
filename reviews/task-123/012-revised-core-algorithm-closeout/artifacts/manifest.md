# Task 123 Packet 012 Artifact Manifest

- Head SHA: `6a01c903e60edf127ac95d0bcc6e865435349edc`
- Task bucket: `reviews/task-123/012-revised-core-algorithm-closeout`
- Timestamp: `2026-06-28T04:31:54-07:00`
- Packet type: closeout scope audit
- Primary evidence packet: `reviews/task-123/011-multi-instance-100k-timeline-rerun`
- Runner for cited measurements: `ecaz bench suite`
- Host lane: local four-instance PG18, Unix sockets, one coordinator plus three local remote PostgreSQL instances
- Corpus: `ec_real_100k`
- Storage format: `rabitq`
- Isolated surfaces: one coordinator table/index plus one remote table/index per local remote instance

## Feedback Resolution Audit

| Feedback item | Source | Resolution |
| --- | --- | --- |
| Rerun at >=200 queries | `reviews/task-123/009-multi-instance-phase-a-baseline/feedback/2026-06-27-01-reviewer.md` | Satisfied by packet 011 id-only reruns, both cells at 200 queries |
| Use contained multi-instance substrate | Task 121/123 amendments and packet 009 feedback | Satisfied by packet 011 local four-instance PG18 lane |
| Realistic projection / communications bytes | Packet 009/010 feedback | Attempted via `id,source`; failed with `remote_heap_resolution_failed`; out of this narrowed core-algorithm closeout |
| Full scoring/traversal/planning/communications taxonomy | Packet 010 feedback | Not claimed; transport/stage-attribution follow-up |
| PR #43 pre-materialization-prune A/B | Packet 009 seq-02 feedback | Not claimed; materialization/communications follow-up |
| No true cross-network measurement | User revised scope | Explicit non-claim |

## Key Results Cited

From `reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/manifest.md`:

- `n128 b4/tr50/f8`, nprobe 8: 200 queries, p50 `662.821 ms`, p95 `923.969 ms`, recall@10 `0.9900`
- `n128 b4/tr50/f8`, nprobe 96: 200 queries, p50 `5408.521 ms`, p95 `5815.967 ms`, recall@10 `1.0000`
- `n1024 b2/tr50/f8`, nprobe 8: 200 queries, p50 `555.397 ms`, p95 `581.701 ms`, recall@10 `0.9290`
- `n1024 b2/tr50/f8`, nprobe 64: 200 queries, p50 `770.595 ms`, p95 `860.296 ms`, recall@10 `1.0000`

Storage:

- `n128 b4/tr50/f8`: coordinator index `392.2 MiB`
- `n1024 b2/tr50/f8`: coordinator index `246.1 MiB`

Realistic projection status:

- `id,source` nested production read failed with `remote_heap_resolution_failed`
- Failure evidence: `reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/bench-suite/suite-run.log`
- Failure evidence: `reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/n1024-b2-200q-source/coord-postgres.log`
