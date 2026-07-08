# Review request — Task 166 M4 packet 002: distann full sweep (10k/50k/100k)

**Branch:** `task-165-ec-distann-m3`. The ec_distann column of the M4 4-way gate.

## What landed

A `ecaz bench suite` config running ec_distann at 10k (load/recall/latency/
storage) and its results. Confirms ec_distann is a fully-wired bench profile
(profiles.rs `EC_DISTANN`, default_sweep [16,32,64,100,200]) and runs cleanly
through the standard runner (FR-038), no bespoke sweeper.

## Evidence (`artifacts/`)

10k recall@10 = 0.9935 / 0.9990 / 0.9995 / 1.0000 / 1.0000 across the sweep;
ndcg@k ~1.0; warm q-time 2.8–11 ms. results.jsonl + per-step logs committed.

## Next (M4 gate)

Full ec_distann 10k/50k/100k sweep, paired with the standard intel-local
hnsw/ivf/spire results, into the 4-way M4 comparison + verdict.

## Ask

Confirm the harness wiring is correct and the 10k recall is sane. Not closing.
