# Task 121 Packet 028 Artifact Manifest

- Head SHA: `6a01c903e60edf127ac95d0bcc6e865435349edc`
- Task bucket: `reviews/task-121/028-revised-core-algorithm-status-sync`
- Timestamp: `2026-06-28T04:31:54-07:00`
- Packet type: status sync / closeout scope audit
- Primary evidence packet: `reviews/task-123/011-multi-instance-100k-timeline-rerun`
- Runner for cited measurements: `ecaz bench suite`
- Host lane: local four-instance PG18, Unix sockets, one coordinator plus three local remote PostgreSQL instances
- Corpus: `ec_real_100k`
- Storage format: `rabitq`
- Isolated surfaces: one coordinator table/index plus one remote table/index per local remote instance

## Evidence Sources

- Task 121 original closeout: `reviews/task-121/026-phase4-final-pareto-verdict/`
- Task 121 reviewer sign-off: `reviews/task-121/026-phase4-final-pareto-verdict/feedback/2026-06-26-01-reviewer.md`
- Task 123 200-query multi-instance rerun: `reviews/task-123/011-multi-instance-100k-timeline-rerun/`
- Task 123 packet 011 manifest: `reviews/task-123/011-multi-instance-100k-timeline-rerun/artifacts/manifest.md`

## Requirement Audit

| Requirement | Evidence | Status |
| --- | --- | --- |
| Preserve Task 121 route-stage DOE findings | Packet 026 closeout and reviewer sign-off | Satisfied |
| Re-check named route candidates on contained multi-instance executor | Task 123 packet 011 200-query local four-instance PG18 rerun | Satisfied for core algorithm |
| Use at least 200 queries for the reopened multi-instance check | Packet 011 `results-idonly.jsonl` and logs show `queries=200` | Satisfied |
| Avoid claiming true cross-network performance | Packet 011 and this request scope state local multi-instance only | Satisfied |
| Avoid claiming realistic payload transport | Packet 011 records `id,source` failure and uses id-only completed timings | Satisfied by explicit non-claim |

## Key Lines Cited

- `n128 b4/tr50/f8`, nprobe 8: 200 queries, p50 `662.821 ms`, p95 `923.969 ms`, recall@10 `0.9900`
- `n128 b4/tr50/f8`, nprobe 96: 200 queries, p50 `5408.521 ms`, p95 `5815.967 ms`, recall@10 `1.0000`
- `n1024 b2/tr50/f8`, nprobe 8: 200 queries, p50 `555.397 ms`, p95 `581.701 ms`, recall@10 `0.9290`
- `n1024 b2/tr50/f8`, nprobe 64: 200 queries, p50 `770.595 ms`, p95 `860.296 ms`, recall@10 `1.0000`
