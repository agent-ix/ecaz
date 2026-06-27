---
id: FR-064
title: "Benchmark Suite Configuration Schema"
type: FR
status: IMPLEMENTED
object: data_schema
relationships:
  - target: "ix://agent-ix/ecaz/US-017"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-038"
    type: "constrains"
    cardinality: "1:1"
---
# [FR-064] Benchmark Suite Configuration Schema

## Description

`ecaz bench suite` SHALL accept JSON suite configurations with the structure
defined here. Checked-in suite configs under `crates/ecaz-cli/suites/` are the
reference corpus; `NFR-007` requires every benchmark packet to check its
`SuiteConfig` into the owning packet.

Implementation anchor: `SuiteConfig` and `SuiteStep` in
`crates/ecaz-cli/src/commands/bench/suite.rs`.

## Schema

```json
{
  "$id": "ix://agent-ix/ecaz/bench-suite-config",
  "title": "SuiteConfig",
  "type": "object",
  "required": ["name", "schema_version", "steps"],
  "properties": {
    "name": { "type": "string" },
    "schema_version": { "type": "integer", "const": 1 },
    "artifact_dir": { "type": "string", "description": "optional base directory for step artifacts" },
    "defaults": {
      "type": "object",
      "description": "fallbacks applied to steps that omit the field",
      "properties": {
        "profile": { "type": "string", "description": "access-method profile, e.g. ec_ivf, ec_hnsw" },
        "bits": { "type": "integer" },
        "seed": { "type": "integer" },
        "queries_limit": { "type": "integer" },
        "iterations": { "type": "integer" },
        "force_index": { "type": "boolean" },
        "sample_backend_memory": { "type": "boolean" },
        "memory_sample_interval_ms": { "type": "integer" },
        "pg": { "type": "integer", "description": "PostgreSQL major version, e.g. 18" },
        "socket_dir": { "type": "string" }
      }
    },
    "thresholds": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "step", "metric", "field", "op", "value"],
        "properties": {
          "name": { "type": "string" },
          "step": { "type": "string", "description": "step name the threshold targets" },
          "metric": { "type": "string", "description": "result-row metric family, e.g. recall, latency" },
          "filters": { "type": "object", "description": "exact-match row filters so multi-row sweeps target one candidate row (FR-038-AC-7)" },
          "field": { "type": "string", "description": "result-row value key, e.g. recall@k, p50" },
          "op": { "type": "string", "enum": ["gt", "gte", "lt", "lte", "eq"] },
          "value": { "type": "number" }
        }
      }
    },
    "steps": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["kind", "name"],
        "discriminator": "kind",
        "kinds": ["corpus-fetch", "corpus-prepare", "load", "recall", "cross-am", "latency", "spire-pipeline", "storage", "explain", "sidecar-rerank", "compare-pgvector", "compare-vectorscale", "raw"],
        "common_properties": {
          "name": { "type": "string" },
          "tags": { "type": "array", "items": { "type": "string" }, "description": "selected via run --only-tag" }
        },
        "kind_specific_examples": {
          "load": ["prefix", "corpus_file", "queries_file", "manifest_file", "allow_manifest_mismatch", "reloptions", "log_file"],
          "recall": ["prefix", "k", "sweep", "rerank_width", "truth_cache_file", "log_output"],
          "latency": ["prefix", "k", "sweep", "rerank_width", "log_output"]
        }
      }
    }
  }
}
```

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-064-CON-1 | Unknown step `kind` values fail parse; new step kinds extend the suite runner (`FR-038`), never per-packet shell scripts | Architecture | CLI unit test + `NFR-007` |
| FR-064-CON-2 | `audit --config` validates suite shape and required load input files without connecting to PostgreSQL | Technical | CLI unit test |
| FR-064-CON-3 | The manifest records the config SHA256 so resume can reject stale configs (`FR-038-AC-7`) | Technical | CLI unit test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-064-AC-1 | Every checked-in suite under `crates/ecaz-cli/suites/` parses against this schema | Test |
| FR-064-AC-2 | A threshold with `filters` selects exactly one row from a multi-row sweep | Test |
| FR-064-AC-3 | The documented step-kind list matches the `SuiteStep` enum verbatim | Inspection |

## Dependencies

- **Upstream**: `FR-038` behavior 1 (config parsing), `NFR-007` (config provenance).
- **Downstream**: `FR-065` suite manifest, `FR-066` results rows, `FR-070` run lifecycle.
