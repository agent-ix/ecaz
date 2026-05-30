# Task 68 Zero-Replica Fast Path Measurement

## Scope

- Code under measurement: `c8f98a71da07e8d1417642fcbbe558ce0ae942d9`
- Packet commit carrying this evidence: `cd98e26ba7a8a8f1ecb2da8715dd782961161e2a`
- Installed backend SHA-256: `0a47749823f1bea04783eee15ce670ca03e3933a0ec1be5548ae98b92f5bd6ec`
- Suite config: `reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/suite.json`
- Database: `task68_spire_char`
- Surface: one index per table, existing packet 002 fixture tables, PG18 local socket
- Reloptions: `storage_format='turboquant'`, `boundary_replica_count=0`, top graph enabled, `recursive_fanout=8`, `nprobe=24`, `rerank_width=25`

## Result

| fixture | rows | nlists | baseline packet | baseline total_ms | fast path total_ms | reduction | speedup |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 10k | 10000 | 32 | `002-characterization` | 806 | 372 | 53.8 % | 2.17x |
| 100k | 100000 | 128 | `003-timing-drilldown` | 22482 | 3362 | 85.0 % | 6.69x |

The 100k direct drilldown target collapsed:

| 100k field | before ms | after ms | reduction |
| --- | ---: | ---: | ---: |
| `draft_total_ms` | 19248 | 92 | 99.5 % |
| `draft_leaf_rows_ms` | 19182 | 25 | 99.9 % |

## After Split

| fixture | heap_scan_ms | kmeans_ms | assignment_ms | recursive_kmeans_ms | draft_total_ms | draft_leaf_rows_ms | top_graph_ms | object_store_total_ms | publish_ms | total_ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 137 | 149 | 14 | 0 | 10 | 3 | 59 | 59 | 0 | 372 |
| 100k | 1252 | 495 | 580 | 1 | 92 | 25 | 935 | 935 | 7 | 3362 |

The fast path removes the zero-replica boundary reroute loop from the measured
configuration. For `boundary_replica_count=0`, primary leaf PID is already known
from the top-level centroid assignment, so leaf-row placement now uses the
existing identity placement helper and preserves source-order allocation.

## Updated Ranking

The original P0 target, `draft_leaf_rows_ms`, is no longer material after the
fast path. On the 100k fixture the largest remaining measured phases are:

| phase | ms | share of total |
| --- | ---: | ---: |
| heap scan | 1252 | 37.2 % |
| top graph / object-store total | 935 | 27.8 % |
| top-level assignment | 580 | 17.3 % |
| top-level k-means | 495 | 14.7 % |
| draft total | 92 | 2.7 % |

Task 69 has already moved common training and assignment to the shared
parallel helpers. The remaining Task 68-specific candidate is top graph/object
store, while heap scan is a PostgreSQL input collection cost rather than a
SPIRE draft construction path.
