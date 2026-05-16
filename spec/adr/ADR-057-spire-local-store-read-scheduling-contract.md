---
id: ADR-057
title: "SPIRE Local Store Read Scheduling Contract"
status: ACCEPTED
impact: Affects Task 30 Phase 10 local multi-store execution
date: 2026-05-09
---
# ADR-057: SPIRE Local Store Read Scheduling Contract

## Status

Accepted.

## Context

SPIRE Phase 10 groups selected leaf and delta object routes by
`(node_id, local_store_id)` before candidate scoring. This preserves the
multi-store boundary introduced for local multi-NVMe layouts and gives the scan
path a concrete scheduling unit.

The current AM scan still runs inside one PostgreSQL backend and one
`amrescan` call. That backend owns the active relation handles, snapshot
validation, memory context, route diagnostics, object decoding, and candidate
accumulator. Introducing worker threads or asynchronous callbacks inside this
path would need a separate PostgreSQL resource-ownership design.

The code already overlaps what is safe through PostgreSQL storage primitives:
all selected leaf and delta placements are collected, grouped by local store,
and passed through relation block prefetch before object decoding and scoring.
On PG18, relation-backed stores use `read_stream` block prefetch; on older PG
targets they fall back to `PrefetchBuffer`. This is storage prefetch overlap,
not concurrent per-store execution.

## Decision

Phase 10 keeps `(node_id, local_store_id)` as the local-store scheduling unit
and keeps object decoding/candidate scoring sequential inside the single
backend scan.

The accepted local overlap primitive for this phase is PostgreSQL relation
prefetch/read-stream over all selected store groups before scoring begins. The
scan must not claim true parallel multi-NVMe execution until a later design
adds a safe executor model and benchmark evidence.

## Required Invariants

- Route grouping is keyed by `(node_id, local_store_id)`.
- Selected leaf and delta object placements are prefetched before their rows
  are decoded and scored.
- Candidate scoring remains deterministic and independent of store-group
  ordering except for explicit score and tie-break rules.
- Scan diagnostics report per-store route, prefetch, scanned-row, candidate,
  dedupe, winner, and truncation counts.
- Product claims must distinguish PostgreSQL prefetch overlap from true
  concurrent per-store execution.

## Rationale

Keeping the execution sequential inside one backend matches PostgreSQL's
relation, snapshot, buffer, and memory-context ownership model. It also keeps
the eager bounded scan contract from ADR-056 simple: `amrescan` owns all object
reads and `amgettuple` only drains the ranked cursor.

The current prefetch-before-scoring shape is still useful. It lets PG18 issue
block requests for the selected object set before the scan starts decoding
objects, and it gives future work a stable boundary where worker or async
execution could attach.

## Consequences

- Phase 10.4 is explicit that local multi-store reads are grouped and
  prefetched, not executed concurrently.
- Any future true multi-NVMe overlap needs a new ADR covering backend resource
  ownership, cancellation, error propagation, deterministic merge behavior,
  and measurement on actual multi-NVMe hardware.
- Benchmark packets may measure the current design as grouped prefetch
  execution, but must not label it as parallel store execution.
