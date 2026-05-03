---
id: FR-038
title: Configured Benchmark Suite Runner
type: functional-requirement
artifact_type: FR
status: APPROVED
object_type: interface
relationships:
  - target: "ix://agent-ix/tqvector/US-017"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/tqvector/FR-037"
    type: "extends"
    cardinality: "N:1"
---
# FR-038: Configured Benchmark Suite Runner

## Requirement

Ecaz SHALL provide a configured benchmark suite runner under `ecaz bench suite` for repeatable load, recall, latency, storage, EXPLAIN, and custom benchmark sequences.

## Behavior

1. `ecaz bench suite run --config <path>` SHALL parse a JSON suite and expand each selected step into the ordinary `ecaz` command it represents.
2. `run --dry-run` SHALL write the manifest and print expanded commands without executing suite steps.
3. `run` SHALL execute selected steps sequentially and record per-step status, timing, exit code, command, and expected artifacts in `suite-manifest.json`.
4. `run --only <name>` SHALL restrict execution to matching step names and SHALL leave other steps marked skipped.
5. `run --only-tag <tag>` SHALL restrict execution to steps that declare matching tags.
6. `run --resume-from <manifest>` SHALL skip selected steps that already succeeded in the referenced manifest.
7. `run` SHALL stop after the first failed selected step unless `--continue-on-error` is set.
8. `run` SHOULD write normalized `results.jsonl` rows from completed recall, latency, storage, and load artifacts.
9. `audit --config <path>` SHALL validate suite shape and required load input files before a long run.
10. `status --manifest <path>` SHALL summarize completed, failed, skipped, dry-run, stale, and missing-artifact state.
11. `report --manifest <path>` SHALL emit a markdown report from manifest metadata and parsed result rows.
12. The legacy `ecaz bench suite --config <path> --dry-run` form SHALL remain accepted as a compatibility alias for the first dry-run slice.

## Acceptance Criteria

### FR-038-AC-1

Suite dry-runs and executed runs produce a manifest with config SHA256, redacted connection metadata, expanded commands, tags, step selection, step status, timing, and artifact paths.

### FR-038-AC-2

The runner supports the configured step kinds needed by current Task 31 IVF work: `load`, `recall`, `latency`, `storage`, `explain`, and `raw`.

### FR-038-AC-3

Suite audit and status commands are usable without connecting to PostgreSQL.

### FR-038-AC-4

The CLI README documents suite commands, schema conventions, dry-run/execution flow, and targeted tuning usage.

### FR-038-AC-5

Completed suite runs can produce normalized JSONL rows for recall, latency, storage, and load artifacts.
