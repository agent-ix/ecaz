---
id: FR-071
title: "HNSW Configuration Surface"
artifact_type: FR
status: IMPLEMENTED
object: configuration
relationships:
  - target: "ix://agent-ix/ecaz/US-003"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-030"
    type: "constrains"
    cardinality: "1:1"
---
# [FR-071] HNSW Configuration Surface

## Description

This configuration object is the canonical inventory of every `ec_hnsw`
reloption and session GUC. Behavioral FRs (`FR-008`, `FR-009`, `FR-016`,
`FR-030`) reference these parameters; this FR owns the complete list so no
tuning surface ships undocumented.

Implementation anchor: `src/am/ec_hnsw/options.rs` (`register_gucs`).

## Parameters

Index reloptions (`CREATE INDEX ... WITH (...)`):

| Name | Type | Default | Range | Description |
| --- | --- | --- | --- | --- |
| `m` | int | 8 | 2-100 | Max neighbors per layer. |
| `ef_construction` | int | 64 | 10-1000 | Build-time beam width. |
| `ef_search` | int | 40 | 1-1000 | Per-index default scan breadth. |
| `build_source_column` | text | null | valid heap column | Optional `float4[]` column used only during bulk build. |
| `rerank_source_column` | text | null | valid heap column | Optional rerank payload source. |
| `storage_format` | text | per opclass | documented formats | Persisted code format selector. |

## Settings

Session GUCs (all `PGC_USERSET`):

| Name | Type | Default | Range/Values | Description |
| --- | --- | --- | --- | --- |
| `ec_hnsw.ef_search` | int | 40 | 1-1000 | Session override for scan breadth; at default the reloption stays authoritative. |
| `ec_hnsw.turboquant_exact_score_mode` | enum | `exact` | `exact`, `full_lut`, `tiled_lut`, `int8_approx` | Task 98 measurement switch selecting the TurboQuant exact-score strategy. |
| `ec_hnsw.candidate_batch_scoring` | bool | `on` | — | Task 87 batch-scoring route; disable only to A/B the structural CandidateBatch route against the pre-Task-87 scalar FullLut path. |
| `ec_hnsw.enable_parallel_build_concurrent_dsm` | bool | `on` | — | ADR-048 Phase-4 concurrent DSM graph assembly for eligible parallel builds; disable only as a diagnostic fallback. |
| `ec_hnsw.disable_binary_prefilter` | bool | `off` | — | Diagnostic A/B switch: skip ADR-031 binary-query preparation. |
| `ec_hnsw.force_binary_derivation` | bool | `off` | — | Diagnostic A/B switch: derive binary words from code bytes even when persisted sidecars exist. |

## Behavior

1. A non-default `ec_hnsw.ef_search` overrides the reloption; the default
   value (40) leaves the reloption authoritative (`FR-009` precedence rules).
2. `turboquant_exact_score_mode` applies only to TurboQuant scan storage;
   non-TurboQuant scans ignore it and report mode `exact`.
3. The two batch/diagnostic booleans (`candidate_batch_scoring`,
   `enable_parallel_build_concurrent_dsm`) are correctness-neutral: toggling
   them changes routing and performance, never result sets.
4. Benchmark packets that toggle any setting here record the override in the
   candidate-identity fields required by `NFR-015`.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-071-CON-1 | Every registered `ec_hnsw.*` GUC appears in this table; adding a GUC without updating this FR is a spec defect | Architecture | Docs audit against `register_gucs()` |
| FR-071-CON-2 | Diagnostic switches default to the production path (batching on, DSM on, prefilter enabled) | Technical | Unit test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-071-AC-1 | The Settings table matches the GUCs registered in `src/am/ec_hnsw/options.rs` one-to-one, including defaults and value sets | Docs audit |
| FR-071-AC-2 | `SET ec_hnsw.candidate_batch_scoring = off` changes counter attribution (scalar route) without changing query results | pg_test A/B |

## Dependencies

- **Upstream**: `FR-009` ef_search precedence model, ADR-031/ADR-048 diagnostics.
- **Downstream**: `FR-030` behavioral surface, `NFR-015` candidate identity fields.
