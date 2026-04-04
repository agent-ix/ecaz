# Sequence Diagram: aminsert (Single Row Insert)

```mermaid
sequenceDiagram
    participant PG as PostgreSQL
    participant Insert as tqhnsw aminsert
    participant Pages as Index Pages
    participant Quant as ProdQuantizer

    PG->>Insert: aminsert(index_rel, values, isnull, heap_tid, ...)
    Insert->>Insert: Extract tqvector code bytes from datum
    
    Note over Insert: Step 1 — Layer Assignment
    Insert->>Insert: level = floor(-ln(random()) / ln(M))
    
    Note over Insert: Step 2 — Read Entry Point
    Insert->>Pages: ReadBuffer(page 0) → metadata
    Pages-->>Insert: entry_point_tid, max_level, M
    
    Note over Insert: Step 3 — Greedy Descent (upper layers)
    Insert->>Insert: current = entry_point
    
    loop For layer = max_level down to (level + 1)
        Insert->>Pages: ReadBuffer(current) → TqElementTuple.code
        Insert->>Pages: ReadBuffer(current.neighbor_tid) → TqNeighborTuple
        loop For each neighbor at this layer
            Insert->>Pages: ReadBuffer(neighbor) → TqElementTuple.code
            Insert->>Quant: score_ip_encoded_lite(new_code, neighbor_code)
            Quant-->>Insert: distance
        end
        Insert->>Insert: current = closest neighbor
    end
    
    Note over Insert: Step 4 — Beam Search (insertion layers)
    
    loop For layer = min(level, max_level) down to 0
        Insert->>Insert: Init candidate_set with current node
        loop While candidates remain
            Insert->>Insert: Pop closest candidate
            Insert->>Pages: ReadBuffer → TqNeighborTuple
            loop For each unvisited neighbor
                Insert->>Pages: ReadBuffer → TqElementTuple.code
                Insert->>Quant: score_ip_encoded_lite(new_code, neighbor_code)
                Quant-->>Insert: distance
                Insert->>Insert: Add to candidates if promising
            end
        end
        Insert->>Insert: Select top M neighbors (2M at layer 0) from candidates
        Insert->>Insert: Store selected neighbors for this layer
    end
    
    Note over Insert: Step 5 — Allocate New Tuples
    Insert->>Pages: Find page with free space (or extend relation)
    Insert->>Pages: GenericXLogStart
    Insert->>Pages: Write TqElementTuple (code, heap_tid, level)
    Insert->>Pages: Write TqNeighborTuple (selected neighbor TIDs per layer)
    Insert->>Pages: GenericXLogFinish
    
    Note over Insert: Step 6 — Update Back-Links
    loop For each selected neighbor (all layers)
        Insert->>Pages: GenericXLogStart
        Insert->>Pages: ReadBuffer → neighbor's TqNeighborTuple
        alt Neighbor has room (count < M or 2M)
            Insert->>Pages: Append new_point_tid to neighbor list
        else Neighbor is full
            Insert->>Insert: Find weakest existing connection
            Insert->>Quant: score_ip_encoded_lite (compare distances)
            alt New connection is stronger
                Insert->>Pages: Replace weakest with new_point_tid
            else New connection is weaker
                Insert->>Insert: Skip (don't add)
            end
        end
        Insert->>Pages: GenericXLogFinish
    end
    
    Note over Insert: Step 7 — Update Entry Point (if needed)
    alt new level > max_level
        Insert->>Pages: GenericXLogStart
        Insert->>Pages: Update metadata: entry_point = new_point, max_level = level
        Insert->>Pages: GenericXLogFinish
    end
    
    Insert-->>PG: return success
```

## Key Design Decisions

1. **Code-to-code scoring**: aminsert uses `score_ip_encoded_lite` (no LUT). Both sides are compressed codes stored on pages. This is less accurate than LUT-based scoring but avoids constructing a LUT for a single insert.
2. **Neighbor pruning**: When a neighbor's list is full, the weakest connection is replaced only if the new connection is stronger. This is the standard HNSW "select-neighbors-simple" heuristic.
3. **Lock ordering**: Page locks are acquired in ascending block number order to prevent deadlocks during back-link updates.
4. **GenericXLog per page**: Each page modification is its own GenericXLog transaction. If the server crashes mid-insert, partial updates are rolled back and the graph remains consistent (some back-links may be missing but the graph is still navigable).
