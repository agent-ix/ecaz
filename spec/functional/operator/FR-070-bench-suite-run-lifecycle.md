---
id: FR-070
title: "Benchmark Suite Run Lifecycle"
artifact_type: FR
status: IMPLEMENTED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/US-017"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-038"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-065"
    type: "writes"
    cardinality: "N:1"
---
# [FR-070] Benchmark Suite Run Lifecycle

## Description

This process object defines the `ecaz bench suite run` lifecycle that
`FR-038` regulates behaviorally — in particular the order guarantee that
backend build-profile preflight happens before any latency/recall step
executes, and that manifest writes bracket execution so interrupted runs stay
resumable and auditable.

Implementation anchor: `crates/ecaz-cli/src/commands/bench/suite.rs`
(`run` entry, preflight at the `build_profile != "release"` guard,
`extract_result_rows`).

## Workflow

```mermaid
sequenceDiagram
    participant Op as Operator
    participant CLI as ecaz bench suite run
    participant PG as PostgreSQL backend
    participant FS as Artifact dir

    Op->>CLI: run --config suite.json [--only/--only-tag/--resume-from/--dry-run/--allow-debug-backend]
    CLI->>CLI: parse + validate SuiteConfig (FR-064); compute config SHA256
    CLI->>CLI: expand each selected step into its ordinary ecaz command
    alt suite contains latency or recall steps
        CLI->>PG: SELECT ecaz_build_profile()
        PG-->>CLI: build_profile (+ library path/SHA256 when identifiable)
        alt build_profile != "release" and no --allow-debug-backend
            CLI-->>Op: fail fast: refuse latency/recall on debug backend (FR-038-AC-10)
        end
    end
    CLI->>FS: write suite-manifest.json (FR-065) with backend provenance
    alt --dry-run
        CLI-->>Op: print expanded commands; stop (manifest records dry-run)
    end

    loop selected steps, in config order
        alt --resume-from manifest has this step succeeded with matching config SHA256 + command
            CLI->>CLI: skip (resume)
        else execute
            CLI->>PG: run expanded step command
            CLI->>FS: update step record (status, timing, exit code, artifacts)
            alt step failed and no --continue-on-error
                CLI-->>Op: stop after first failure (manifest preserves state)
            end
        end
    end

    CLI->>FS: extract normalized results.jsonl rows (FR-066) from completed artifacts
    CLI->>CLI: evaluate configured thresholds against result rows
    alt any threshold fails
        CLI-->>Op: suite fails with threshold_results recorded in manifest
    else
        CLI-->>Op: suite succeeds; report renders from manifest + rows
    end
```

## Behavior

Ordering guarantees:

1. Config validation and command expansion complete before any step runs.
2. Backend preflight completes (or refuses) before the first latency/recall
   step; the manifest records `backend.build_profile` either way.
3. The manifest exists on disk from the moment steps begin, so `status` and
   `resume` work after interruption.
4. Result extraction and threshold evaluation run only over steps whose
   status is succeeded.

## Algorithm

1. Parse config; reject unknown step kinds and missing load inputs (`audit`
   shares this validation without connecting).
2. Apply `--only` / `--only-tag` selection; unselected steps are recorded as
   skipped.
3. Preflight the backend when the selected set includes latency/recall.
4. Execute steps sequentially, recording per-step status/timing/exit
   code/artifacts as each finishes.
5. Extract result rows, evaluate thresholds, write the report inputs.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-070-CON-1 | No latency or recall step executes before backend preflight resolves (`FR-038` behaviors 16-17) | Technical | CLI unit test |
| FR-070-CON-2 | Resume never reuses a step whose config SHA256 or expanded command changed | Technical | CLI unit test (`FR-038-AC-7`) |
| FR-070-CON-3 | A failed threshold fails the suite even when every step succeeded | Technical | CLI unit test (`FR-038-AC-6`) |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-070-AC-1 | An interrupted run's manifest supports `status` classification and `--resume-from` continuation | Test |
| FR-070-AC-2 | A debug backend without the override flag stops the run before the first benchmark step, and the manifest shows why | Test |
| FR-070-AC-3 | Dry-run writes a complete manifest and executes nothing | Test |

## Dependencies

- **Upstream**: `FR-064` config schema, `ecaz_build_profile()` SQL surface.
- **Downstream**: `FR-065` manifest, `FR-066` result rows, `NFR-007` provenance citations.
