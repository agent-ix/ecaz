# Task 68 Top-Graph Cache Measurement

## Scope

- Code under measurement: `fe7d5e6892dc1e7154eb95d8e620b22bef070d10`
- Head SHA while writing packet: `1a4aca35e4aa7b0c6af88cc38751626c218780d4`
- Installed backend SHA-256: `4aa5e735046ceb62ac02a87c7d14e513030e98111dec515cd05aeccb5a3551a8`
- Suite config: `reviews/task-68/007-top-graph-cache-measurement/artifacts/suite.json`
- Database: `task68_spire_char`
- Surface: one index per table, existing packet 002 fixture tables, PG18 local socket
- Reloptions: `storage_format='turboquant'`, `boundary_replica_count=0`, top graph enabled, `recursive_fanout=8`, `nprobe=24`, `rerank_width=25`

## Result

Compared with packet 005's post-zero-replica fast-path baseline:

| fixture | rows | baseline total_ms | cache total_ms | delta | speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | 10000 | 372 | 384 | +3.2 % | 0.97x |
| 100k | 100000 | 3362 | 3236 | -3.7 % | 1.04x |

Top graph specifically:

| fixture | baseline top_graph_ms | cache top_graph_ms | delta |
| --- | ---: | ---: | ---: |
| 10k | 59 | 68 | +15.3 % |
| 100k | 935 | 847 | -9.4 % |

The 100k result is a real but modest improvement. It does not clear the
Task 68 Phase 2 `~5 % of total build wall time at 100k` gate when measured
against packet 005: `126 ms / 3362 ms = 3.7 %`.

## After Split

| fixture | heap_scan_ms | kmeans_ms | assignment_ms | recursive_kmeans_ms | draft_total_ms | draft_leaf_rows_ms | top_graph_ms | object_store_total_ms | publish_ms | total_ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 138 | 148 | 15 | 0 | 10 | 1 | 68 | 68 | 2 | 384 |
| 100k | 1234 | 489 | 573 | 1 | 90 | 19 | 847 | 847 | 0 | 3236 |

## Interpretation

The distance matrix removes repeated centroid inner-product work, but the
Vamana top-graph builder still spends most of this phase in graph search,
candidate management, pruning, and backlink maintenance. Because the measured
end-to-end 100k win is below the Phase 2 slice gate, this packet treats the
cache as a small improvement and shelves further top-graph work for Task 68
unless a reviewer wants a deeper Vamana-specific slice.

The largest remaining measured 100k phases are now:

| phase | ms | share of total |
| --- | ---: | ---: |
| heap scan | 1234 | 38.1 % |
| top graph / object-store total | 847 | 26.2 % |
| top-level assignment | 573 | 17.7 % |
| top-level k-means | 489 | 15.1 % |
| draft total | 90 | 2.8 % |

Heap scan remains PG callback input collection. Task 69 already covers the
shared training/assignment lane. No additional Task 68-specific P0 slice is
obvious from this split without opening a deeper Vamana/top-graph redesign.
