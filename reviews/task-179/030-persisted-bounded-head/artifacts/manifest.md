# Packet 030 — Persisted bounded head

Task bucket: `reviews/task-179/`; packet `030-persisted-bounded-head/`.
Head SHA: `961056f6b63d267218196776c0c6bc5842453a1a`.
Lane: PG18; one coordinator and two distinct remote participant control indexes
over pooled loopback connections. Head state is coordinator-local and each
owner keeps isolated row-tier, graph, and directory relations. No benchmark
measurements are claimed.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Strict clippy, generated schema, persisted-head three-owner serving/abort/reclaim pass |

This is correctness evidence for bounded persisted seeding. It is not the
required real three-instance topology or 10k/50k/100k closeout benchmark.
