# Recall And Planner Gate Floors

Task 47 owns these first machine-readable gates. They are deliberately smaller
than the historical benchmark sweeps so they can burn in as PR or nightly lanes
before becoming full release criteria.

## Entry Points

| Command | Config | Cadence | Purpose |
| --- | --- | --- | --- |
| `make recall-gate` | `fixtures/gates/recall-gate-small.json` | PR | Fast exact-KNN recall floor check on generated small fixtures. |
| `make recall-gate-full` | `fixtures/gates/recall-gate-full.json` | Nightly | Larger HNSW sweep over the same fixture and truth-cache path. |
| `make cross-am-gate` | `fixtures/gates/cross-am-gate-small.json` | PR candidate, report-first | Runs multiple AMs against the same exact truth cache and writes per-query top-k predictions so cross-AM drift is visible in one suite manifest. |
| `make cost-gate` | `fixtures/gates/cost-gate-small.json` | PR | Captures cost-model snapshot rows, enforces positive modeled total cost, and compares against `fixtures/cost-queries/baseline.json`. |

All gate configs write transient local output under `target/gates/`. Reviewable
evidence must be copied or rerun with packet-local artifact paths before it is
cited in a review request.

## Fixture Contract

The small gate configs assume PG18 has loaded matching small gate fixtures:

| Prefix | Profile | Expected index |
| --- | --- | --- |
| `task47_gate_hnsw` | `ec_hnsw` | `task47_gate_hnsw_m16_idx` |
| `task47_gate_ivf` | `ec_ivf` | `task47_gate_ivf_idx` |
| `task47_gate_diskann` | `ec_diskann` | `task47_gate_diskann_idx` |

PR CI uses a generated 512-row corpus plus 64 queries at 32 dimensions so the
gate fits the hosted-runner budget. Larger real-corpus calibrations still use
the deterministic DBpedia OpenAI corpus described in `docs/recall-methodology.md`.
Use one exact truth-cache file per suite so AMs are compared against identical
query/source rows.

## Current Floors

| Gate row | Floor | Source |
| --- | ---: | --- |
| HNSW, `k=10`, `ef_search=128` | `recall@k >= 0.84` | Burn-in floor from `reviews/task-47/002-live-small-gate`; the 512-row synthetic CI fixture measured `0.8500`. |
| HNSW, `k=10`, `ef_search=200` | `recall@k >= 0.93` | NFR-003 headline floor for the 50k shape, reused here as a conservative burn-in floor. |
| IVF, `k=10`, `nprobe=48`, `rerank_width=750` | `recall@k >= 0.84` | Burn-in floor from `reviews/task-47/002-live-small-gate`; the 512-row synthetic CI fixture measured `0.8500`. |
| DiskANN, `k=10`, `list_size=200` | `recall@k >= 0.55` | Burn-in floor from `reviews/task-47/002-live-small-gate`; the 512-row synthetic CI fixture measured `0.5660`. |
| HNSW vs DiskANN, `k=10` | `jaccard@k >= 0.10` | Report-first cross-AM floor. `jaccard@k` is averaged per-query top-k membership intersection over union. |
| HNSW vs DiskANN, `k=10` | `kendall_tau@k >= -1.00` | Report-first validity range. `kendall_tau@k` is averaged over the union of both top-k lists with missing entries ranked at `k+1`. |

Floor changes require a review packet that includes the raw recall table, exact
truth-cache descriptor, corpus identity, query count, profile reloptions, and a
short rationale for the new value. Cross-AM floor changes also require the
per-query prediction JSON files and the generated cross-AM metric table so a
reviewer can inspect whether drift is membership loss, rank-order movement, or
both.

## Cost Gate Status

`make cost-gate` first runs the configured explain suite, then
`scripts/check_cost_baseline.py` compares the normalized `planner_cost` rows in
`target/gates/cost-small/results.jsonl` against
`fixtures/cost-queries/baseline.json`. The current drift band is 15% relative
or 0.05 absolute, whichever is larger, over modeled startup cost, modeled total
cost, selectivity, correlation, index pages, and reltuples.

Baseline changes are explicit: rerun the gate, inspect the raw explain logs, and
only then update the fixture with `--accept-drift` in a Task 47 review packet.
