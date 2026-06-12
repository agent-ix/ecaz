---
id: FR-072
title: "IVF Configuration Surface"
artifact_type: FR
status: IMPLEMENTED
object: configuration
relationships:
  - target: "ix://agent-ix/ecaz/US-013"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-031"
    type: "constrains"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-032"
    type: "constrains"
    cardinality: "1:1"
---
# [FR-072] IVF Configuration Surface

## Description

This configuration object is the canonical inventory of every `ec_ivf`
reloption and session GUC, including the Task 51 adaptive/diagnostic switches
that were previously undocumented in spec.

Implementation anchor: `src/am/ec_ivf/options.rs` (`register_gucs`).

## Configuration

Scope `creation` rows are index reloptions (`CREATE INDEX ... WITH (...)`);
`runtime` and `session` rows are session GUCs (all `PGC_USERSET`).

| Name | Scope | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `nlists` | creation | int | — | IVF cluster count. |
| `nprobe` | creation | int | — | Default posting lists probed per query; automatic `ceil(sqrt(nlists))` when unset. |
| `rerank_width` | creation | int | — | Default `heap_f32` rerank frontier width. |
| `training_sample_rows` | creation | int | — | Centroid training sample size. |
| `seed` | creation | int | — | Deterministic training/assignment seed. |
| `pq_group_size` | creation | int | — | Dimensions per grouped-PQ group (`pq_fastscan`). |
| `posting_slack_percent` | creation | int | — | Reserved slack pages for churn reuse. |
| `storage_format` | creation | text | — | `auto`, `turboquant`, `pq_fastscan`, `rabitq`. |
| `rerank` | creation | text | — | `auto`, `off`, `heap_f32`; `source_column` rejected until implemented. |
| `ec_ivf.nprobe` | session | int | -1 (unset) | Overrides relation `nprobe` when set to 1 or higher; -1 uses the relation value. Range: -1..max. |
| `ec_ivf.rerank_width` | session | int | -1 (unset) | Overrides relation `rerank_width` when set to 0 or higher; -1 uses the relation value. Range: -1..max. |
| `ec_ivf.adaptive_nprobe` | runtime | bool | `off` | Task 51 diagnostic: scans may halve nprobe when the centroid frontier shows the configured score gap. |
| `ec_ivf.adaptive_nprobe_score_gap_micros` | runtime | int | 0 | Inner-product score gap (x1e6) required between retained frontier and next centroid before adaptive reduction. Range: 0..max. |
| `ec_ivf.adaptive_nprobe_score_margin_ratio_bps` | runtime | int | 0 | Basis-point ratio signal; values > 0 switch adaptive nprobe to the ratio criterion. Range: 0..max. |
| `ec_ivf.scratch_soa_batch_decode` | runtime | bool | `off` | Task 51 experimental: batch posting-tuple field decode into scan-local structure-of-arrays buffers before scoring. |

## Behavior

1. `nprobe` resolution precedence: positive session GUC, then relation
   reloption, then automatic `ceil(sqrt(nlists))` (`FR-032` behavior 1).
2. `rerank_width` resolution precedence: session GUC at zero or higher, then
   relation reloption (`FR-032` behavior 3).
3. Adaptive-nprobe switches are deterministic for a given query and frontier;
   they trade recall for latency and are diagnostic, not product defaults.
4. `scratch_soa_batch_decode` interacts with suffix-max/cutoff pruning
   (`FR-068` Behavior item 3); benchmark packets toggling it keep
   batch-on/off as an explicit axis.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-072-CON-1 | Every registered `ec_ivf.*` GUC appears in this table; adding a GUC without updating this FR is a spec defect | Architecture | Docs audit against `register_gucs()` |
| FR-072-CON-2 | Adaptive and SoA switches default off; product benchmark lanes state their values explicitly | Business | `NFR-015` candidate identity fields |
| FR-072-CON-3 | Invalid reloption values raise ERROR at index creation (`FR-031-AC-2`) | Technical | pg_test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-072-AC-1 | The Settings table matches the GUCs registered in `src/am/ec_ivf/options.rs` one-to-one, including unset sentinels | Inspection |
| FR-072-AC-2 | Session `nprobe`/`rerank_width` overrides are reflected in scan/debug output (`FR-032-AC-2`) | Test |

## Dependencies

- **Upstream**: `FR-031`/`FR-032` behavioral requirements.
- **Downstream**: `FR-068` scan pipeline, `NFR-015` reporting fields.
