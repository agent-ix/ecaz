---
id: FR-065
title: "Suite Manifest Schema"
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
  - target: "ix://agent-ix/ecaz/NFR-007"
    type: "references"
    cardinality: "N:1"
---
# [FR-065] Suite Manifest Schema

## Description

Every `ecaz bench suite run` (including dry runs) SHALL write a
`suite-manifest.json` with the structure defined here. The manifest is the
provenance record `NFR-007` cites for backend build-profile proof, step
status, and resume identity.

Implementation anchor: `SuiteManifest`, `BackendPreflight`, `StepRecord`, and
`ThresholdResult` in `crates/ecaz-cli/src/commands/bench/suite.rs`.

## Schema

```json
{
  "$id": "ix://agent-ix/ecaz/suite-manifest",
  "title": "suite-manifest.json",
  "type": "object",
  "required": ["suite", "schema_version", "config", "config_sha256", "dry_run", "generated_at_unix_ms", "connection", "steps"],
  "properties": {
    "suite": { "type": "string" },
    "schema_version": { "type": "integer" },
    "config": { "type": "string", "description": "config path" },
    "config_sha256": { "type": "string", "description": "resume identity; stale configs are rejected" },
    "dry_run": { "type": "boolean" },
    "generated_at_unix_ms": { "type": "integer" },
    "connection": {
      "type": "object",
      "required": ["database", "password_configured"],
      "properties": {
        "database": { "type": "string" },
        "host": { "type": "string" },
        "port": { "type": "integer" },
        "user": { "type": "string" },
        "password_configured": { "type": "boolean", "description": "redaction: presence flag only, never the secret" }
      }
    },
    "backend": {
      "type": "object",
      "description": "backend preflight provenance (FR-038-AC-9); absent only for runs without latency/recall preflight",
      "required": ["build_profile"],
      "properties": {
        "build_profile": { "type": "string", "description": "from SELECT ecaz_build_profile(); latency/recall runs fail fast unless 'release' or --allow-debug-backend" },
        "sha256": { "type": "string", "description": "installed extension library SHA256 when the local pgrx layout is identifiable" },
        "path": { "type": "string", "description": "installed backend library path when identifiable" }
      }
    },
    "steps": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "kind", "command", "selected"],
        "properties": {
          "name": { "type": "string" },
          "kind": { "type": "string" },
          "command": { "type": "array", "items": { "type": "string" }, "description": "expanded ecaz command" },
          "selected": { "type": "boolean" },
          "quant": { "type": "string" },
          "isa": { "type": "string" },
          "kernel_status": { "type": "string", "enum": ["valid", "missing_kernel", "structurally_absent", "invalid_config"], "description": "Task 99 matrix cell disposition for kernel-targeted steps" },
          "pgoptions": { "type": "string" },
          "tags": { "type": "array", "items": { "type": "string" } },
          "expected_artifacts": { "type": "array", "items": { "type": "string" } },
          "status": { "type": "string", "enum": ["dry-run", "pending", "skipped", "running", "succeeded", "failed"] },
          "started_at_unix_ms": { "type": "integer" },
          "finished_at_unix_ms": { "type": "integer" },
          "duration_ms": { "type": "integer" },
          "exit_code": { "type": "integer" },
          "parallel_workers_before": { "type": "integer" },
          "parallel_workers_after": { "type": "integer" },
          "parallel_workers_delta": { "type": "integer" }
        }
      }
    },
    "threshold_results": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "step", "metric", "field", "op", "expected", "passed", "message"],
        "properties": {
          "name": { "type": "string" },
          "step": { "type": "string" },
          "metric": { "type": "string" },
          "filters": { "type": "object" },
          "field": { "type": "string" },
          "op": { "type": "string", "enum": ["gt", "gte", "lt", "lte", "eq"] },
          "expected": { "type": "number" },
          "actual": { "type": "number" },
          "passed": { "type": "boolean" },
          "message": { "type": "string" }
        }
      }
    }
  }
}
```

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-065-CON-1 | Connection metadata is redacted: the manifest records whether a password was configured, never credential material | Security | CLI unit test |
| FR-065-CON-2 | Latency/recall manifests record `backend.build_profile` before benchmark steps execute (`FR-038-AC-9`) | Technical | CLI unit test |
| FR-065-CON-3 | Resume honors a prior manifest only when `config_sha256` and the expanded step command match (`FR-038-AC-7`) | Technical | CLI unit test |
| FR-065-CON-4 | `kernel_status` uses the closed disposition enum so Task 99 matrix cells are machine-readable | Architecture | Spec review |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-065-AC-1 | A dry run and an executed run both produce manifests valid against this schema | Test |
| FR-065-AC-2 | A debug-backend latency run without `--allow-debug-backend` fails before any step executes and the refusal cites `build_profile` | Test |
| FR-065-AC-3 | `status --manifest` can classify completed, failed, skipped, dry-run, stale, and missing-artifact states from manifest fields alone | Test |

## Dependencies

- **Upstream**: `FR-064` config schema, `FR-038` behaviors 3/16/17.
- **Downstream**: `FR-066` results rows, `NFR-007` provenance citations, packet manifests.
