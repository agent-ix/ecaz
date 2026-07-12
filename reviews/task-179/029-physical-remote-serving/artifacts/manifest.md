# Packet 029 — Physical remote serving

Task bucket: `reviews/task-179/`; packet `029-physical-remote-serving/`.
Head SHA: `6f9b98bfabb5d25447435614b039bed20dadb99a`.
Lane: PG18; one coordinator and two distinct remote participant control indexes
over pooled loopback connections. Each owner has an isolated row-tier, graph,
and directory relation. No benchmark measurements are claimed.

| File | Command | Key result |
|---|---|---|
| `validation.log` | commands listed in the artifact | Generated overload schema, strict clippy, and focused three-owner physical CustomScan pass |

The focused test validates routing and binary row materialization across three
isolated physical owner generations. It is not the required real three-instance
fixture, persisted bounded-head evidence, or 10k/50k/100k closeout benchmark.
