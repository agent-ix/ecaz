---
id: FR-063
title: "Block-Kernel Counter Snapshot"
artifact_type: FR
status: IMPLEMENTED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/FR-014"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/NFR-015"
    type: "references"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-038"
    type: "references"
    cardinality: "N:1"
---
# [FR-063] Block-Kernel Counter Snapshot

## Description

The shared candidate-batch scoring layer SHALL expose one counter snapshot row
per `(surface, quant_kind, isa)` key, with the exact field names and label
sets defined here. This row shape is the contract between the in-extension
counters, the `[block-kernel-counters]` log lines parsed by `ecaz bench
suite`, the normalized `results.jsonl` rows, and the benchmark reporting
standard. Field-name drift between any of those surfaces is a defect.

Implementation anchor: `BlockKernelScoringSnapshot` in
`src/am/common/candidate_batch/counters.rs`; labels in `src/quant/isa.rs` and
`src/am/common/quant_codec.rs`.

## Schema

```json
{
  "$id": "ix://agent-ix/ecaz/block-kernel-counter-snapshot",
  "title": "Block-kernel scoring counter snapshot row",
  "type": "object",
  "key": ["surface", "quant_kind", "isa"],
  "key_cardinality": "5 surfaces x 7 quant kinds x 5 ISAs = 175 slots",
  "properties": {
    "surface": { "type": "string", "enum": ["hnsw", "ivf", "diskann", "spire", "unknown"] },
    "quant_kind": { "type": "string", "enum": ["turboquant", "turboquant_qjl", "turboquant_tiled_lut", "turboquant_int8", "rabitq", "grouped_pq", "binary"] },
    "isa": { "type": "string", "enum": ["scalar", "neon", "sve", "sve2", "avx2"], "description": "label of the ISA that actually scored; fallback stubs report scalar, never the host's highest capability" },
    "flushes": { "type": "integer", "description": "total batch flushes" },
    "candidates": { "type": "integer", "description": "total candidates scored" },
    "elapsed_nanos": { "type": "integer", "description": "total scoring time" },
    "kernel_flushes": { "type": "integer", "description": "flushes handled by the SIMD kernel path" },
    "kernel_candidates": { "type": "integer" },
    "kernel_elapsed_nanos": { "type": "integer" },
    "scalar_flushes": { "type": "integer", "description": "off-path scalar flushes: disabled kernels, unsupported widths, prevalidation rejections" },
    "scalar_candidates": { "type": "integer" },
    "scalar_elapsed_nanos": { "type": "integer" },
    "width_lt8_flushes": { "type": "integer", "description": "flushes with 1-7 candidates" },
    "width_8_15_flushes": { "type": "integer", "description": "flushes with 8-15 candidates" },
    "width_16_31_flushes": { "type": "integer", "description": "flushes with 16-31 candidates" },
    "width_ge32_flushes": { "type": "integer", "description": "flushes with >=32 candidates" }
  },
  "required": ["surface", "quant_kind", "isa", "flushes", "candidates", "elapsed_nanos", "kernel_flushes", "kernel_candidates", "kernel_elapsed_nanos", "scalar_flushes", "scalar_candidates", "scalar_elapsed_nanos", "width_lt8_flushes", "width_8_15_flushes", "width_16_31_flushes", "width_ge32_flushes"]
}
```

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-063-CON-1 | Counter label sets are closed enums; a new surface, quant kind, or ISA extends the enum here and in `NFR-015` before it appears in evidence | Architecture | Spec review |
| FR-063-CON-2 | Measured SVE/SVE2 vector length (for example `sve2-128`) is reported in packet prose or environment fields, never as the `isa` label value | Technical | Docs audit |
| FR-063-CON-3 | Shape mismatches and prevalidation failures increment no counter fields and mutate no output scores (ADR-076) | Technical | Unit test |
| FR-063-CON-4 | `ecaz bench suite` result extraction preserves these field names verbatim in normalized result rows | Technical | CLI unit test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-063-AC-1 | The in-extension snapshot, the `[block-kernel-counters]` log line, and the `results.jsonl` row expose identical field names for one flush | CLI unit test over a captured run |
| FR-063-AC-2 | A scalar-fallback host emits rows with `isa=scalar` and zero `kernel_*` fields | pg_test |
| FR-063-AC-3 | Width histogram fields sum to `flushes` for every row | Unit test |

## Dependencies

- **Upstream**: `FR-014` (counter surface requirement), ADR-076 (counter semantics).
- **Downstream**: `FR-066` results-row schema, `NFR-015` reporting standard, `docs/benchmark-reporting-standard.md`.
