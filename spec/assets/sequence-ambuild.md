# Sequence Diagram: ambuild (Bulk Index Build)

```mermaid
sequenceDiagram
    participant PG as PostgreSQL
    participant AM as tqhnsw ambuild
    participant HNSW as hnsw_rs::Hnsw
    participant Pages as Index Pages

    PG->>AM: ambuild(heap_rel, index_rel, index_info)
    AM->>AM: Parse amoptions (m, ef_construction)
    
    Note over AM: Phase 1 — Heap Scan + Graph Build
    
    AM->>PG: heap_beginscan()
    loop For each heap tuple
        PG-->>AM: (heap_tid, tqvector_datum)
        AM->>AM: Extract f32 embedding + tqvector code bytes
        AM->>AM: Cache (origin_id → heap_tid, code_bytes) in HashMap
        AM->>HNSW: insert(origin_id, f32_vector)
    end
    AM->>PG: heap_endscan()
    
    Note over AM: Phase 2 — Graph Serialization
    Note over AM: Pass 1: Write tuples, collect TID mapping
    
    loop For each point in hnsw graph
        AM->>HNSW: get_origin_id(), get_neighborhood_id()
        HNSW-->>AM: origin_id, neighbors_per_layer
        AM->>AM: Look up (heap_tid, code_bytes) from cache
        AM->>Pages: GenericXLogStart
        AM->>Pages: Write TqElementTuple (code bytes, heap_tid)
        AM->>Pages: Write TqNeighborTuple (placeholder TIDs)
        AM->>Pages: GenericXLogFinish
        AM->>AM: Record origin_id → index_page_tid in tid_map
    end
    
    Note over AM: Pass 2: Fix up neighbor TID pointers
    
    loop For each point in hnsw graph
        AM->>HNSW: get_neighborhood_id()
        HNSW-->>AM: neighbor origin_ids per layer
        AM->>AM: Resolve origin_ids → page TIDs via tid_map
        AM->>Pages: GenericXLogStart
        AM->>Pages: Update TqNeighborTuple with resolved TIDs
        AM->>Pages: GenericXLogFinish
    end
    
    Note over AM: Pass 3: Write metadata page
    
    AM->>Pages: GenericXLogStart
    AM->>Pages: Write page 0 (M, ef_construction, entry_point, dim)
    AM->>Pages: GenericXLogFinish
    
    AM->>AM: Drop hnsw_rs::Hnsw + HashMap
    AM-->>PG: Return index build result (tuple count)
```

## Key Design Decisions

1. **f32 vectors for build**: The hnsw_rs graph is built with raw f32 distance, not compressed code distance. This produces a higher-quality graph.
2. **Two-pass serialization**: Necessary because page TIDs are assigned during write, but neighbor tuples reference other points' TIDs.
3. **hnsw_rs is ephemeral**: The in-memory graph is dropped after serialization. Runtime operations use Postgres pages only.
