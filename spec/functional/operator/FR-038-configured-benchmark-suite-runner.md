---
id: FR-038
title: Configured Benchmark Suite Runner
type: FR
status: APPROVED
object: interface
relationships:
  - target: "ix://agent-ix/ecaz/US-017"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-037"
    type: "extends"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/NFR-015"
    type: "supports"
    cardinality: "N:1"
---
# FR-038: Configured Benchmark Suite Runner

## Description

Ecaz SHALL provide a configured benchmark suite runner under `ecaz bench suite` for repeatable load, recall, latency, storage, EXPLAIN, and custom benchmark sequences.

## Behavior

1. `ecaz bench suite run --config <path>` SHALL parse a JSON suite and expand each selected step into the ordinary `ecaz` command it represents.
2. `run --dry-run` SHALL write the manifest and print expanded commands without executing suite steps.
3. `run` SHALL execute selected steps sequentially and record per-step status,
   timing, exit code, command, and expected artifacts in
   `suite-manifest.json`.
4. `run --only <name>` SHALL restrict execution to matching step names, leaving all other steps marked skipped.
5. `run --only-tag <tag>` SHALL restrict execution to steps that declare matching tags.
6. `run --resume-from <manifest>` SHALL skip selected steps that already succeeded in the referenced manifest only when the config hash and expanded step command match the current run.
7. `run` SHALL stop after the first failed selected step unless `--continue-on-error` is set.
8. `run` SHOULD write normalized `results.jsonl` rows from completed recall,
   latency, storage, load, build-timing, and block-kernel-counter artifacts
   when those result families are emitted.
9. `run` SHALL evaluate configured thresholds against parsed result rows and fail the suite when any threshold is not satisfied.
10. Thresholds SHALL support exact-match row filters so multi-row sweeps can target a specific candidate row.
11. `audit --config <path>` SHALL validate suite shape and required load input files before a long run.
12. `status --manifest <path>` SHALL summarize completed, failed, skipped, dry-run, stale, and missing-artifact state.
13. `report --manifest <path>` SHALL emit a markdown report from manifest metadata and parsed result rows.
14. The legacy `ecaz bench suite --config <path> --dry-run` form SHALL remain accepted as a compatibility alias for the first dry-run slice.
15. Suite reports SHOULD preserve the access-method,
   quantizer/storage-format, option-set, dataset, environment, backend
   provenance, and metric fields required by `NFR-015`.
16. `run` SHALL preflight latency and recall suites against the connected
   backend by querying SQL-visible extension build profile metadata. The
   manifest SHALL record the backend build profile and SHOULD record the
   installed backend path and SHA256 when the local pgrx layout can be
   identified.
17. `run` SHALL refuse latency or recall steps against a debug-built backend
   unless `--allow-debug-backend` is explicitly passed. Debug-backend runs are
   valid only as diagnostic evidence and SHALL NOT be cited for product latency or
   recall claims.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-038-AC-1 | Dry-runs and executed runs produce a manifest with config SHA256, redacted connection metadata, expanded commands, tags, step selection, status, timing, and artifact paths | Test |
| FR-038-AC-2 | The runner supports the step kinds `load`, `recall`, `latency`, `storage`, `explain`, and `raw` | Test |
| FR-038-AC-3 | Suite audit and status commands are usable without connecting to PostgreSQL | Test |
| FR-038-AC-4 | The CLI README documents suite commands, schema conventions, dry-run/execution flow, and targeted tuning usage | Inspection |
| FR-038-AC-5 | Completed suite runs can produce normalized JSONL rows for recall, latency, storage, and load artifacts | Test |
| FR-038-AC-6 | Configured thresholds are recorded in the manifest and can fail an otherwise completed suite | Test |
| FR-038-AC-7 | Thresholds can target a specific row from a multi-row sweep, and resume rejects stale manifests | Test |
| FR-038-AC-8 | Suite reports include enough candidate identity and metric metadata to populate the benchmark reporting standard without hand-editing | Inspection |
| FR-038-AC-9 | Latency and recall suite runs record the backend build profile in the manifest before executing selected benchmark steps | Test |
| FR-038-AC-10 | Latency and recall suite runs fail fast on a debug backend unless `--allow-debug-backend` is present | Test |

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

### FR-038-AC-6

Configured thresholds are recorded in the manifest and can fail an otherwise completed suite.

### FR-038-AC-7

Thresholds can target a specific row from a multi-row sweep, and resume rejects stale manifests whose config hash or expanded command differs.

### FR-038-AC-8

Suite reports include enough candidate identity and metric metadata to populate
the benchmark reporting standard without hand-editing result semantics.

### FR-038-AC-9

Latency and recall suite runs record the backend build profile in the manifest
before executing selected benchmark steps.

### FR-038-AC-10

Latency and recall suite runs fail fast on a debug backend unless
`--allow-debug-backend` is present.

## Contract

```yaml
interface: ecaz bench suite
description: >-
  Configured benchmark suite runner. A JSON SuiteConfig expands into ordinary
  `ecaz` commands; the runner keeps the expansion visible in a manifest and
  optionally executes each selected step in sequence.
operations:
  - name: run
    inputs:
      - { flag: --config, type: path, required: true, semantics: JSON suite configuration file }
      - { flag: --dry-run, type: bool, semantics: write manifest and print expanded commands without executing }
      - { flag: --continue-on-error, type: bool, semantics: keep running remaining selected steps after a failure }
      - { flag: --only, type: string[], semantics: run only steps with this name (repeatable) }
      - { flag: --only-tag, type: string[], semantics: run only steps with this tag (repeatable) }
      - { flag: --resume-from, type: path, semantics: reuse successful step records from an earlier manifest when config hash and expanded command match }
      - { flag: --results-output, type: path, semantics: where to write normalized result rows; defaults to <artifact_dir>/results.jsonl }
      - { flag: --artifact-dir, type: path, semantics: override the config artifact directory for logs/manifest/results }
      - { flag: --manifest-output, type: path, semantics: where to write the manifest; defaults to <artifact_dir>/suite-manifest.json }
      - { flag: --allow-debug-backend, type: bool, semantics: permit latency/recall steps against a debug-built backend }
    output: suite-manifest.json (+ results.jsonl); thresholds evaluated against parsed rows
    semantics: >-
      execute selected steps sequentially, recording per-step status, timing,
      exit code, command, and expected artifacts; stop after first failure
      unless --continue-on-error; preflight latency/recall against the backend
      build profile and refuse a debug backend unless --allow-debug-backend
  - name: audit
    inputs:
      - { flag: --config, type: path, required: true }
    output: validation result (no PostgreSQL connection required)
    semantics: validate suite shape and required load input files before a long run
  - name: status
    inputs:
      - { flag: --manifest, type: path, required: true }
    output: completion summary (completed, failed, skipped, dry-run, stale, missing-artifact)
    semantics: summarize completion state from a suite manifest (no PostgreSQL connection required)
  - name: report
    inputs:
      - { flag: --manifest, type: path, required: true }
      - { flag: --results-output, type: path, semantics: also write normalized rows parsed from manifest artifacts }
    output: markdown report from manifest metadata and parsed result rows
    semantics: emit a markdown report carrying candidate identity and metric provenance (NFR-015)
legacy_alias:
  # `ecaz bench suite --config <path> --dry-run` (top-level flags on SuiteArgs)
  # remains accepted as a compatibility alias for the first dry-run slice.
  flags: [--config, --dry-run, --only, --manifest-output]
config_inputs:
  SuiteConfig:
    - { field: name, type: string }
    - { field: schema_version, type: u32 }
    - { field: artifact_dir, type: path, optional: true }
    - { field: defaults, type: SuiteDefaults, optional: true }
    - { field: thresholds, type: ThresholdConfig[], optional: true }
    - { field: steps, type: SuiteStep[] }
  ThresholdConfig:
    - { field: name, type: string }
    - { field: step, type: string }
    - { field: metric, type: string }
    - { field: filters, type: map<string,string>, semantics: exact-match row filters to target a single sweep row }
    - { field: field, type: string }
    - { field: op, type: enum[gt, gte, lt, lte, eq] }
    - { field: value, type: f64 }
outputs:
  - { artifact: suite-manifest.json, semantics: config SHA256, redacted connection metadata, expanded commands, tags, step selection, status, timing, artifact paths, recorded thresholds, backend build profile }
  - { artifact: results.jsonl, semantics: normalized rows for recall/latency/storage/load/build-timing/block-kernel-counter artifacts }
```

## Dependencies

- **Upstream**: US-017 (implements), FR-037 (extends), NFR-015 (supports)
- **Downstream**: none identified
