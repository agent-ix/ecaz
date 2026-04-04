# Sequence Diagram: HNSW Index Scan (amrescan + amgettuple)

```mermaid
sequenceDiagram
    participant Client as SQL Query
    participant PG as PostgreSQL Planner/Executor
    participant Scan as tqhnsw scan callbacks
    participant Pages as Index Pages
    participant Quant as ProdQuantizer

    Client->>PG: SELECT ... ORDER BY col <#> $query LIMIT 10
    PG->>Scan: ambeginscan(index_rel, nkeys, norderbys)
    Scan->>Scan: Allocate TqScanState (BinaryHeap, LUT=None)
    Scan-->>PG: scan descriptor

    PG->>Scan: amrescan(scan, orderbys=[query_tqvector])
    
    Note over Scan: Step 1 — LUT Preparation
    Scan->>Scan: Extract query tqvector from orderbys
    Scan->>Quant: prepare_ip_query(query_codes)
    Quant-->>Scan: LUT [dim × num_centroids] f32 array
    Scan->>Scan: Store LUT in TqScanState
    
    Note over Scan: Step 2 — Greedy Descent (upper layers)
    Scan->>Pages: Read metadata page 0 → entry_point_tid, max_level
    
    loop For layer = max_level down to 1
        Scan->>Pages: ReadBuffer(current_node) → TqElementTuple
        Scan->>Pages: ReadBuffer(neighbor_tid) → TqNeighborTuple
        loop For each neighbor at this layer
            Scan->>Pages: ReadBuffer(neighbor) → TqElementTuple.code
            Scan->>Quant: score_ip_encoded(LUT, neighbor_code)
            Quant-->>Scan: distance
        end
        Scan->>Scan: Move to closest neighbor, descend layer
    end
    
    Note over Scan: Step 3 — Beam Search (layer 0)
    Scan->>Scan: Init candidate_set + visited_set + result_set
    
    loop While candidates remain
        Scan->>Scan: Pop closest unscored candidate
        Scan->>Pages: ReadBuffer → TqNeighborTuple (layer 0 neighbors)
        loop For each unvisited neighbor
            Scan->>Pages: ReadBuffer → TqElementTuple.code
            Scan->>Quant: score_ip_encoded(LUT, code)
            Quant-->>Scan: distance
            Scan->>Scan: Add to candidate_set if closer than worst in result_set
            Scan->>Scan: Track top ef_search results in result_set
        end
    end
    
    Note over Scan: Step 4 — Load Results
    Scan->>Scan: Sort result_set by distance → BinaryHeap
    Scan-->>PG: amrescan complete

    loop For each result (up to LIMIT)
        PG->>Scan: amgettuple(scan, direction)
        Scan->>Scan: Pop best from BinaryHeap
        Scan->>Pages: Read TqElementTuple → heap_tid
        Scan-->>PG: xs_heaptid = heap_tid, return true
        PG->>PG: Fetch heap tuple, return to client
    end

    PG->>Scan: amgettuple(scan, direction)
    Scan-->>PG: return false (exhausted)

    PG->>Scan: amendscan(scan)
    Scan->>Scan: Free TqScanState, LUT, BinaryHeap
```

## Key Design Decisions

1. **LUT prepared once**: The LUT is built from the query codes in amrescan and reused for every distance calculation in the search. Zero allocation per scoring call.
2. **Greedy descent on upper layers**: Only one node is tracked (no beam). This matches standard HNSW — upper layers are sparse, greedy is sufficient.
3. **Beam search on layer 0**: Full beam with ef_search candidates. This is where recall is determined.
4. **Buffer pins released immediately**: Each page is pinned only during read, then released. No long-held pins.
