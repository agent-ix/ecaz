---
id: FR-068
title: "IVF Scan Pipeline"
artifact_type: FR
status: IMPLEMENTED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/US-013"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-032"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-061"
    type: "reads"
    cardinality: "N:1"
---
# [FR-068] IVF Scan Pipeline

## Description

This process object defines the staged `ec_ivf` ordered-scan pipeline that
`FR-032` regulates behaviorally, including the interaction `FR-032` behavior 8
requires to stay documented: scratch-SoA batch decode versus suffix-max/cutoff
pruning, which is why batch-on/off remains a benchmark axis.

## Workflow

```mermaid
sequenceDiagram
    participant Exec as PostgreSQL executor
    participant Scan as ec_ivf scan
    participant Meta as Metadata + centroids (FR-061)
    participant Lists as Posting-list pages (FR-061)
    participant Codec as Index-local QuantCodec adapter
    participant Heap as Heap relation

    Exec->>Scan: amrescan(ORDER BY embedding <#> query LIMIT k)
    Scan->>Meta: read metadata block; load centroid chain
    Note over Scan: resolve nprobe (GUC ec_ivf.nprobe > reloption > ceil(sqrt(nlists))) and rerank_width (GUC ec_ivf.rerank_width > reloption)
    Scan->>Scan: score query against nlists centroids; select nprobe nearest lists
    Scan->>Codec: prepare query for the persisted storage format (turboquant / pq_fastscan / rabitq)

    loop For each selected posting list
        Scan->>Lists: read posting entries (codes, gammas, heap TIDs)
        alt batch route enabled
            Scan->>Codec: score_ip_batch over scratch-SoA decoded candidate batches
            Note over Codec: suffix-max / cutoff pruning interacts with batch width: a batch decodes fully before per-candidate cutoffs apply
        else per-candidate route
            Scan->>Codec: score_ip_candidate per posting with early cutoff
        end
        Codec-->>Scan: approximate scores + (surface=ivf, quant_kind, isa) counter rows
        Scan->>Scan: keep best rerank_width candidates across lists
    end

    alt rerank = heap_f32
        Scan->>Heap: fetch ecvector rows for top rerank_width candidates
        Scan->>Scan: exact f32 rerank
    else rerank = off
        Scan->>Scan: order by approximate score
    end
    Exec->>Scan: amgettuple x k
    Scan-->>Exec: heap TIDs in final order (LIMIT truncates)
```

## Behavior

Stage boundaries for attribution:

1. **Centroid selection** — `nlists` centroid scores; scales with `nlists`,
   not corpus size.
2. **Posting-list batch scoring** — the compressed-domain stage; all batch
   scores flow through `QuantCodec::score_ip_batch` and are observable as
   `surface=ivf` rows in the `FR-063` counter snapshot.
3. **Pruning interaction** — suffix-max/cutoff pruning can skip per-candidate
   work the batch route has already paid for in decode; whether batching wins
   depends on list length, payload format, and cutoff selectivity. Packets
   comparing the two MUST run both sides of the axis.
4. **Heap rerank** — bounded by `rerank_width` under `heap_f32`.

## Algorithm

1. Read the `FR-061` metadata block; reject magic/version mismatches.
2. Resolve `nprobe` and `rerank_width` per the precedence in `FR-032`.
3. Rank centroids by inner product against the prepared query; take `nprobe`.
4. For each selected list, walk its posting block range and score live
   entries through the codec adapter (batch or per-candidate route).
5. Maintain the global top-`rerank_width` candidate set across lists.
6. Apply the configured rerank mode and emit ordered heap TIDs.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-068-CON-1 | Compressed-domain scoring never bypasses the `QuantCodec` adapter for any storage format | Architecture | Code review (ADR-076) |
| FR-068-CON-2 | Batch-on/off remains a configured benchmark axis wherever pruning tradeoffs affect latency | Business | Suite config review (`FR-032` behavior 8) |
| FR-068-CON-3 | Deleted postings (flag bit 0) are skipped without contributing counter or score work | Technical | pg_test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-068-AC-1 | Batch and per-candidate routes return identical result sets for the same query and settings (pg_test A/B) | Test |
| FR-068-AC-2 | Batch-scored work appears as `surface=ivf` counter rows with the active quant kind and ISA (pg_test + suite result extraction) | Test |
| FR-068-AC-3 | An IVF latency packet can attribute a change to centroid selection, posting scoring, pruning, or rerank using this decomposition (packet audit) | Inspection |

## Dependencies

- **Upstream**: `FR-061` persisted format, `FR-014`/`FR-015` kernel and codec contracts.
- **Downstream**: `FR-032` behavioral requirements, `FR-072` configuration surface, Task 99 matrix evidence.
