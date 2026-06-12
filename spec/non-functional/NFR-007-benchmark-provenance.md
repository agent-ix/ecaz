---
id: NFR-007
title: Benchmark Provenance
type: non-functional-requirement
artifact_type: NFR
quality_attribute: compliance
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/StR-006"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-007: Benchmark Provenance

## Statement

Any benchmark result used in README, docs, spec, task status, or review rationale SHALL identify the evidence source and the scope of the claim.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Benchmark-claim provenance compliance | 100% of cited benchmark claims trace to a source packet or are labeled historical/local | no exceptions | repo audit of `docs/benchmarks.md`, `spec/tests.md`, and packet manifests |

1. Benchmark measurements SHALL store raw logs under `benchmarks/<topic>/artifacts/` and summarize them in `benchmarks/<topic>/manifest.md`. Code-review packets that include benchmark evidence MAY continue to live under `reviews/task-{id}/{ordinal}-<topic>/artifacts/` and SHALL cite the benchmark packet by path when one exists.
2. Artifact manifests SHALL record head SHA, topic, lane, fixture, storage format, rerank mode, command, timestamp, isolation/shared-table status, and cited key result lines.
3. Configured benchmark suites SHALL write a suite manifest that records
   config identity, selected steps, expanded commands, execution status,
   timing, expected artifact paths, and benchmark backend provenance.
4. Promoted current benchmark lanes MAY live under `benchmarks/current/<lane>/` for mutable host-class snapshots, but each lane manifest SHALL cite the immutable source packet, head SHA, suite config path/hash, raw artifacts, and claim class.
5. Local development measurements SHALL be labeled as local evidence and SHALL NOT be described as product benchmark claims.
6. Product benchmark claims SHALL require dedicated controlled hardware and
   reproducible command/settings metadata.
7. Latency and recall claims SHALL identify whether the measured extension
   backend was a release or debug build. Debug-backend measurements SHALL be
   labeled diagnostic-only unless a requirement explicitly targets debug
   behavior.
8. When the suite runner can identify the installed extension library, the
   packet manifest or suite manifest SHOULD record its SHA256 so later review
   can distinguish release installs from `pg_test` debug overwrites.

## Verification

Compliance is checked by repo audit: every benchmark row in
`docs/benchmarks.md` is checked for a citation to a source packet under
`benchmarks/<topic>/` (or a code-review packet under
`reviews/task-{id}/{ordinal}-<topic>/`) or an explicit historical/local label;
packet manifests are reviewed for the required provenance fields (head SHA,
lane, fixture, command, timestamp, isolation status, backend build profile);
and `spec/tests.md` is checked to record measurement gaps rather than marking
unevidenced performance requirements complete.

## Acceptance Criteria

### NFR-007-AC-1

Every benchmark row in `docs/benchmarks.md` cites a source packet under `benchmarks/<topic>/` (or a code-review packet under `reviews/task-{id}/{ordinal}-<topic>/`) or clearly states that the evidence is historical/local.

### NFR-007-AC-2

Benchmark packets used for measurement claims include `manifest.md` and packet-local raw logs under `benchmarks/<topic>/artifacts/`. Code-review packets that cite benchmark evidence SHALL link to the owning `benchmarks/<topic>/` packet.

### NFR-007-AC-3

`spec/tests.md` records measurement gaps rather than marking unevidenced performance requirements complete.

### NFR-007-AC-4

Latency and recall benchmark packets cite either a suite manifest backend
profile field or an equivalent packet-local artifact proving the measured
backend build profile.
