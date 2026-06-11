---
id: FR-066
title: "Suite Normalized Results Row Schema"
artifact_type: FR
status: IMPLEMENTED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/US-017"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-038"
    type: "constrains"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-063"
    type: "consumes"
    cardinality: "N:1"
---
# [FR-066] Suite Normalized Results Row Schema

## Description

Completed suite runs SHALL emit `results.jsonl` with one JSON object per line
in the row shape defined here. Rows normalize heterogeneous step artifacts
(recall, latency, storage, load, build timing, block-kernel counters, parallel
workers, kernel-cell dispositions) into a single comparable structure that
`report`, thresholds, and `NFR-015` conformance all consume.

Implementation anchor: `ResultRow` and `extract_result_rows()` in
`crates/ecaz-cli/src/commands/bench/suite.rs`.

## Schema

```json
{
  "$id": "ix://agent-ix/ecaz/suite-results-row",
  "title": "results.jsonl row",
  "type": "object",
  "required": ["suite", "step", "kind", "metric", "artifact", "values"],
  "properties": {
    "suite": { "type": "string" },
    "step": { "type": "string", "description": "step name from the manifest" },
    "kind": { "type": "string", "description": "step kind, e.g. recall, latency, storage, load" },
    "metric": { "type": "string", "description": "result family: recall, latency, storage, load, build_timing, block_kernel_counters, parallel_workers, kernel_cell" },
    "artifact": { "type": "string", "description": "source artifact path the row was parsed from" },
    "values": {
      "type": "object",
      "additionalProperties": { "type": "string" },
      "description": "string-keyed, string-valued metric map; keys depend on metric family",
      "metric_family_keys": {
        "recall": ["recall@k", "k", "sweep axis fields such as nprobe/ef_search"],
        "latency": ["p50", "p95", "p99", "mean", "iterations", "sweep axis fields"],
        "block_kernel_counters": ["surface", "quant_kind", "isa", "flushes", "candidates", "elapsed_nanos", "kernel_flushes", "kernel_candidates", "kernel_elapsed_nanos", "scalar_flushes", "scalar_candidates", "scalar_elapsed_nanos", "width_lt8_flushes", "width_8_15_flushes", "width_16_31_flushes", "width_ge32_flushes"],
        "build_timing": ["per-phase timing fields parsed from ec_ivf/ec_diskann build logs, including workers_launched and parallel_effective_workers"],
        "kernel_cell": ["quant", "isa", "kernel_status"]
      }
    }
  }
}
```

The `block_kernel_counters` value keys SHALL match `FR-063` field names
verbatim.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-066-CON-1 | `values` is a flat string map; consumers parse numerics, the emitter never loses fields by typing them | Technical | CLI unit test |
| FR-066-CON-2 | Block-kernel counter rows preserve the `FR-063` field names with no renames or omissions | Technical | CLI unit test |
| FR-066-CON-3 | Threshold evaluation reads only `metric`, `values` keys, and `filters`-matched rows; no artifact re-parsing at threshold time | Technical | CLI unit test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-066-AC-1 | One executed suite produces rows for every succeeded step with a parseable artifact | CLI unit test |
| FR-066-AC-2 | A `[block-kernel-counters]` backend log line round-trips into a row whose values match `FR-063` | CLI unit test |
| FR-066-AC-3 | `report --manifest` renders its tables from these rows without hand-editing result semantics (`FR-038-AC-8`) | CLI smoke |

## Dependencies

- **Upstream**: `FR-065` manifest (step status and artifact paths), `FR-063` counter snapshot.
- **Downstream**: `FR-038` thresholds/report behaviors, `NFR-015` row-level conformance, benchmark packet tables.
