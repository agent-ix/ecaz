---
id: FR-067
title: "DiskANN Scan Pipeline"
artifact_type: FR
status: IMPLEMENTED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/US-014"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-035"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-062"
    type: "reads"
    cardinality: "N:1"
---
# [FR-067] DiskANN Scan Pipeline

## Description

This process object defines the staged `ec_diskann` ordered-scan pipeline that
`FR-035` regulates behaviorally. The stage decomposition is normative because
`FR-035-AC-5` requires scan-optimization packets to attribute end-to-end
latency changes to the dominant stage (scoring-share, frontier management,
heap rerank, or I/O) rather than reporting only aggregate percentiles.

## Workflow

```mermaid
sequenceDiagram
    participant Exec as PostgreSQL executor
    participant Scan as ec_diskann scan
    participant Meta as Metadata block (FR-062)
    participant Graph as Node tuples (FR-062)
    participant Codec as Index-local QuantCodec adapter
    participant Heap as Heap relation

    Exec->>Scan: amrescan(ORDER BY embedding <#> query LIMIT k)
    Scan->>Meta: read entry_point (medoid), codec kind, payload flags
    Scan->>Codec: prepare query (per persisted search codec: binary / grouped_pq / rabitq / turboquant)
    Note over Scan: resolve scan breadth from list_size reloption or ec_diskann.list_size GUC

    loop Beam traversal until frontier converges
        Scan->>Graph: read frontier node tuples (neighbors, search codes, binary words)
        alt prefilter kind = binary_sidecar (auto with sidecars present)
            Scan->>Codec: Hamming-prefilter candidate batch (binary words)
        else prefilter kind = grouped_pq, or TurboQuant search codes
            Scan->>Codec: score_ip_batch over the candidate batch (lut32 / grouped-PQ / RaBitQ block kernels; ec_diskann.candidate_batch_scoring gates the batch route)
        end
        Codec-->>Scan: approximate scores + (surface=diskann, quant_kind, isa) counter rows
        Scan->>Scan: update beam frontier and visited set, keep best list_size candidates
    end

    Scan->>Scan: select top rerank_budget candidates by approximate score
    Scan->>Heap: fetch ecvector rows for exact f32 rerank
    Scan->>Scan: exact-rerank, order final candidates
    Exec->>Scan: amgettuple x k
    Scan-->>Exec: heap TIDs in exact-score order (LIMIT truncates)
```

## Behavior

Stage boundaries for attribution:

1. **Query preparation** — codec query prep; excluded from per-candidate
   scoring share.
2. **Frontier management** — beam bookkeeping, visited tracking, node tuple
   page reads (I/O attribution lives here).
3. **Prefilter batch scoring** — every compressed-domain score in this stage
   flows through `QuantCodec::score_ip_batch` and is observable as
   `surface=diskann` rows in the `FR-063` counter snapshot.
4. **Heap rerank** — bounded by `rerank_budget`; exact f32 scoring against
   heap rows.

## Algorithm

1. Read the `FR-062` metadata block; reject version/codec mismatches before
   any scoring.
2. Resolve effective scan breadth (`list_size`) and prefilter kind (`auto`
   prefers persisted binary sidecars, falls back to grouped-PQ behavior).
3. Run best-first beam search from the medoid: pop the closest unexpanded
   candidate, batch-score its unvisited neighbors through the codec adapter,
   push survivors.
4. Stop when the beam cannot improve the current top-`list_size` set.
5. Exact-rerank the top `rerank_budget` survivors from heap data and emit in
   exact-score order.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-067-CON-1 | No scan stage calls ISA-specific kernel functions directly; all compressed-domain scoring goes through the index-local `QuantCodec` adapter | Architecture | Code review (ADR-076) |
| FR-067-CON-2 | Disabling `ec_diskann.candidate_batch_scoring` preserves results byte-for-byte while routing scoring to the per-candidate path | Technical | pg_test A/B |
| FR-067-CON-3 | Stage attribution in packets uses counter rows plus stage timings, not inference from aggregate p50 deltas | Business | Packet review (`FR-035-AC-5`) |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-067-AC-1 | A DiskANN scan-optimization packet can name the dominant stage for a latency change using this stage decomposition (packet audit) | Inspection |
| FR-067-AC-2 | Batch-scored candidates appear as `surface=diskann` counter rows with the active quant kind and ISA (pg_test + suite result extraction) | Test |
| FR-067-AC-3 | The same query returns identical rows with batching on and off (pg_test A/B) | Test |

## Dependencies

- **Upstream**: `FR-062` persisted format, `FR-015`/`FR-014` codec and kernel contracts.
- **Downstream**: `FR-035` behavioral requirements, Task 99 matrix evidence, `FR-073` configuration surface.
