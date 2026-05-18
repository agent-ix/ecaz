# Recall And Planner Gate Floors

Task 47 owns these first machine-readable gates. They are deliberately smaller
than the historical benchmark sweeps so they can burn in as PR or nightly lanes
before becoming full release criteria.

## Entry Points

| Command | Config | Cadence | Purpose |
| --- | --- | --- | --- |
| `make recall-gate` | `fixtures/gates/recall-gate-small.json` | PR candidate | Fast exact-KNN recall floor check on 10k real-corpus fixtures. |
| `make recall-gate-full` | `fixtures/gates/recall-gate-full.json` | Nightly candidate | Larger HNSW sweep over the same fixture and truth-cache path. |
| `make cross-am-gate` | `fixtures/gates/cross-am-gate-small.json` | PR candidate, report-first | Runs multiple AMs against the same exact truth cache so cross-AM drift is visible in one suite manifest. |
| `make cost-gate` | `fixtures/gates/cost-gate-small.json` | PR candidate, report-first | Captures cost-model snapshot rows and rejects non-positive modeled total cost. |

All gate configs write transient local output under `target/gates/`. Reviewable
evidence must be copied or rerun with packet-local artifact paths before it is
cited in a review request.

## Fixture Contract

The small gate configs assume PG18 has loaded matching real-10k fixtures:

| Prefix | Profile | Expected index |
| --- | --- | --- |
| `task47_hnsw_real10k` | `ec_hnsw` | `task47_hnsw_real10k_idx` |
| `task47_ivf_real10k` | `ec_ivf` | `task47_ivf_real10k_idx` |
| `task47_diskann_real10k` | `ec_diskann` | `task47_diskann_real10k_idx` |

The fixture source should be the deterministic DBpedia OpenAI real-10k corpus
described in `docs/recall-methodology.md`. Use one exact truth-cache file per
suite so AMs are compared against identical query/source rows.

## Current Floors

| Gate row | Floor | Source |
| --- | ---: | --- |
| HNSW, `k=10`, `ef_search=128` | `recall@k >= 0.89` | NFR-003 headline floor for the 50k shape, reused here as a conservative burn-in floor. |
| HNSW, `k=10`, `ef_search=200` | `recall@k >= 0.93` | NFR-003 headline floor for the 50k shape, reused here as a conservative burn-in floor. |
| IVF, `k=10`, `nprobe=48`, `rerank_width=750` | `recall@k >= 0.80` | Initial report-first burn-in floor pending a Task 47 calibration packet. |
| DiskANN, `k=10`, `list_size=200` | `recall@k >= 0.80` | Initial report-first burn-in floor pending a Task 47 calibration packet. |

Floor changes require a review packet that includes the raw recall table, exact
truth-cache descriptor, corpus identity, query count, profile reloptions, and a
short rationale for the new value.

## Cost Gate Status

`make cost-gate` currently proves that the cost snapshot functions are wired
into a repeatable suite lane and that modeled total cost remains positive. The
next Task 47 slice should replace the positivity check with a committed baseline
diff using per-node drift bands from `EXPLAIN (FORMAT JSON, COSTS ON)`.
