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

## Configuration

Scope `creation` rows are index reloptions (`CREATE INDEX ... WITH (...)`);
`runtime` and `session` rows are session GUCs (all `PGC_USERSET`).

| Name | Scope | Type | Default | Description |
| --- | --- | --- | --- | --- |
| `m` | creation | int | 8 | Max neighbors per layer. Range: 2-100. |
| `ef_construction` | creation | int | 64 | Build-time beam width. Range: 10-1000. |
| `ef_search` | creation | int | 40 | Per-index default scan breadth. Range: 1-1000. |
| `build_source_column` | creation | text | null | Optional `float4[]` column used only during bulk build. Must be a valid heap column. |
| `rerank_source_column` | creation | text | null | Optional rerank payload source. Must be a valid heap column. |
| `storage_format` | creation | text | per opclass | Persisted code format selector. Values: documented formats. |
| `ec_hnsw.ef_search` | session | int | 40 | Session override for scan breadth; at default the reloption stays authoritative. Range: 1-1000. |
| `ec_hnsw.turboquant_exact_score_mode` | runtime | enum | `exact` | Task 98 measurement switch selecting the TurboQuant exact-score strategy. Values: `exact`, `full_lut`, `tiled_lut`, `int8_approx`. |
| `ec_hnsw.candidate_batch_scoring` | runtime | bool | `on` | Task 87 batch-scoring route; disable only to A/B the structural CandidateBatch route against the pre-Task-87 scalar FullLut path. |
| `ec_hnsw.enable_parallel_build_concurrent_dsm` | runtime | bool | `on` | ADR-048 Phase-4 concurrent DSM graph assembly for eligible parallel builds; disable only as a diagnostic fallback. |
| `ec_hnsw.disable_binary_prefilter` | runtime | bool | `off` | Diagnostic A/B switch: skip ADR-031 binary-query preparation. |
| `ec_hnsw.force_binary_derivation` | runtime | bool | `off` | Diagnostic A/B switch: derive binary words from code bytes even when persisted sidecars exist. |

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
| FR-071-AC-1 | The Settings table matches the GUCs registered in `src/am/ec_hnsw/options.rs` one-to-one, including defaults and value sets | Inspection |
| FR-071-AC-2 | `SET ec_hnsw.candidate_batch_scoring = off` changes counter attribution (scalar route) without changing query results (pg_test A/B) | Test |

## Dependencies

- **Upstream**: `FR-009` ef_search precedence model, ADR-031/ADR-048 diagnostics.
- **Downstream**: `FR-030` behavioral surface, `NFR-015` candidate identity fields.
