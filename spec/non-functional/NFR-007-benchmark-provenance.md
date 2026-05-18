---
id: NFR-007
title: Benchmark Provenance
type: non-functional-requirement
artifact_type: NFR
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-006"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-007: Benchmark Provenance

## Requirement

Any benchmark result used in README, docs, spec, task status, or review rationale SHALL identify the evidence source and the scope of the claim.

## Measurement Rules

1. Benchmark measurements SHALL store raw logs under `benchmarks/<topic>/artifacts/` and summarize them in `benchmarks/<topic>/manifest.md`. Code-review packets that include benchmark evidence MAY continue to live under `reviews/task-{id}/{ordinal}-<topic>/artifacts/` and SHALL cite the benchmark packet by path when one exists.
2. Artifact manifests SHALL record head SHA, topic, lane, fixture, storage format, rerank mode, command, timestamp, isolation/shared-table status, and cited key result lines.
3. Configured benchmark suites SHALL write a suite manifest that records config identity, selected steps, expanded commands, execution status, timing, and expected artifact paths.
4. Local development measurements SHALL be labeled as local evidence and SHALL NOT be described as product benchmark claims.
5. Product benchmark claims SHALL require dedicated controlled hardware and reproducible command/settings metadata.

## Acceptance Criteria

### NFR-007-AC-1

Every benchmark row in `docs/benchmarks.md` cites a source packet under `benchmarks/<topic>/` (or a code-review packet under `reviews/task-{id}/{ordinal}-<topic>/`) or clearly states that the evidence is historical/local.

### NFR-007-AC-2

Benchmark packets used for measurement claims include `manifest.md` and packet-local raw logs under `benchmarks/<topic>/artifacts/`. Code-review packets that cite benchmark evidence SHALL link to the owning `benchmarks/<topic>/` packet.

### NFR-007-AC-3

`spec/tests.md` records measurement gaps rather than marking unevidenced performance requirements complete.
