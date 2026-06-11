---
id: FR-073
title: "DiskANN Configuration Surface"
artifact_type: FR
status: IMPLEMENTED
object: configuration
relationships:
  - target: "ix://agent-ix/ecaz/US-014"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-034"
    type: "constrains"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-035"
    type: "constrains"
    cardinality: "1:1"
---
# [FR-073] DiskANN Configuration Surface

## Description

This configuration object is the canonical inventory of every `ec_diskann`
reloption and session GUC, including the previously spec-undocumented
`candidate_batch_scoring` batching switch and `scan_profile_notice`
measurement switch.

Implementation anchor: `src/am/ec_diskann/options.rs` (`register_gucs`).

## Parameters

Index reloptions (`CREATE INDEX ... WITH (...)`):

| Name | Type | Description |
| --- | --- | --- |
| `graph_degree` | int | Max neighbors per Vamana node (R). |
| `build_list_size` | int | Vamana build-phase beam (L). |
| `list_size` | int | Default scan breadth. |
| `rerank_budget` | int | Exact heap-rerank bound before LIMIT truncation. |
| `top_k` | int | Expected result depth used by costing. |
| `alpha` | real | Vamana pruning factor. |
| `storage_format` | text | Currently `pq_fastscan` (`FR-034` behavior 3). |

## Settings

Session GUCs (all `PGC_USERSET`):

| Name | Type | Default | Range/Values | Description |
| --- | --- | --- | --- | --- |
| `ec_diskann.list_size` | int | -1 (unset) | -1, 1-10000 | Overrides relation `list_size` when set to 1-10000; -1 uses the relation value. |
| `ec_diskann.prefilter_kind` | enum | `auto` | `auto`, `binary_sidecar`, `grouped_pq` | `auto` uses persisted binary sidecars when present and falls back to grouped-PQ; `grouped_pq` forces the legacy prefilter for emergency rollback. |
| `ec_diskann.candidate_batch_scoring` | bool | `on` | — | Task 93 block-kernel CandidateBatch prefilter route; disable only to A/B against the per-candidate scoring path. |
| `ec_diskann.scan_profile_notice` | bool | `off` | — | Task 70 developer switch: amrescan emits one NOTICE with scan setup, graph read/decode, prefilter scoring, frontier maintenance, heap prefetch, exact rerank, result expansion, and total timing. |

## Behavior

1. Scan breadth resolves from the relation `list_size` unless the session GUC
   is set in 1-10000 (`FR-035` behavior 1).
2. `prefilter_kind = binary_sidecar` requires persisted binary sidecars and
   errors without them; `auto` degrades gracefully (`FR-035` behaviors 2-3).
3. `candidate_batch_scoring` is correctness-neutral: it moves scoring between
   the batch route (block kernels, `surface=diskann` counter rows) and the
   per-candidate route without changing result sets.
4. `scan_profile_notice` output is the per-stage timing evidence the `FR-067`
   stage attribution rules consume; it is diagnostic-only and not a product
   latency claim by itself.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-073-CON-1 | Every registered `ec_diskann.*` GUC appears in this table; adding a GUC without updating this FR is a spec defect | Architecture | Docs audit against `register_gucs()` |
| FR-073-CON-2 | `candidate_batch_scoring` defaults on; benchmark packets toggling it record both sides of the axis | Business | `NFR-015` + packet review |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-073-AC-1 | The Settings table matches the GUCs registered in `src/am/ec_diskann/options.rs` one-to-one, including the 1-10000 list_size range | Docs audit |
| FR-073-AC-2 | `SET ec_diskann.candidate_batch_scoring = off` preserves query results while moving counter attribution to the scalar/per-candidate path | pg_test A/B |

## Dependencies

- **Upstream**: `FR-034`/`FR-035` behavioral requirements.
- **Downstream**: `FR-067` scan pipeline stage attribution, Task 99 matrix A/B evidence.
