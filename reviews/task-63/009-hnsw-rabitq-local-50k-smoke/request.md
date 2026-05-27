# Review Request: HNSW RaBitQ Local 50k Smoke

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- head: `ba58dc7fb70cc7e743e3988f91d76555f1138374`
- packet: `reviews/task-63/009-hnsw-rabitq-local-50k-smoke/`

## Summary

This packet records a local 50k HNSW-only `ecaz bench suite` smoke after the
binary RaBitQ search-code change from packet 008. It is local tuning evidence,
not final Task 63 acceptance evidence.

The run stayed within the local workstation limits: HNSW only, 50k only, three
storage formats (`turboquant`, `pq_fastscan`, `rabitq`), and ef_search
40/100/200.

## Validation

Packet-local evidence is under `artifacts/local-50k-bin1/`.

- suite status: completed 14, failed 0, skipped 0.
- host: local PG18 socket `/home/peter/.pgrx`, port 28818.
- corpus: DBpedia-derived `ec_real_50k`, 50k corpus rows, 200 measured queries.

Build index seconds:

| format | build index | total load |
| --- | ---: | ---: |
| `turboquant` | 897.12s | 1071.95s |
| `pq_fastscan` | 934.31s | 1124.27s |
| `rabitq` | 898.07s | 1071.90s |

Recall@10:

| format | ef=40 | ef=100 | ef=200 |
| --- | ---: | ---: | ---: |
| `turboquant` | 0.8700 | 0.9155 | 0.9315 |
| `pq_fastscan` | 0.8965 | 0.9540 | 0.9740 |
| `rabitq` | 0.7955 | 0.8820 | 0.9065 |

Latency p50 / p95 / p99:

| format | ef=40 | ef=100 | ef=200 |
| --- | --- | --- | --- |
| `turboquant` | 19.4 / 24.8 / 44.8 ms | 29.3 / 33.8 / 45.2 ms | 46.1 / 56.0 / 67.1 ms |
| `pq_fastscan` | 24.6 / 31.6 / 40.3 ms | 36.4 / 42.3 / 52.4 ms | 53.2 / 62.1 / 69.9 ms |
| `rabitq` | 48.6 / 89.8 / 112.6 ms | 88.2 / 115.5 / 136.4 ms | 157.0 / 174.0 / 200.8 ms |

HNSW index storage:

| format | index size | bytes/row |
| --- | ---: | ---: |
| `turboquant` | 65.1 MiB | 1365.6 B |
| `pq_fastscan` | 65.2 MiB | 1368.1 B |
| `rabitq` | 65.1 MiB | 1365.6 B |

## Local Read

The 1-bit RaBitQ layout continues to remove the earlier 4-bit storage
regression at 50k: RaBitQ storage is effectively tied with TurboQuant and
PqFastScan. It does not show a useful local operating point in this run:
PqFastScan has higher recall and lower latency at every measured ef_search.

This should inform tuning and the faster-host benchmark decision, but it does
not replace the required publishable 50k/100k matrix on the newer Intel and m5
laptop hosts.
