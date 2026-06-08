# Task 87 Packet 012: Phase 6 Suite Preparation

## Summary

This packet asks for review of the runnable Phase 6 measurement suite shape
for Task 87. It does not claim Phase 6 closeout yet.

Current head for this packet:

- `8d5fab4aebe35e094475588d1ba3bb89e19e813b` - `Add Task 87 IVF route reachability packet`

The packet adds:

- `phase6-suite.json`, an `ecaz bench suite` config for the Task 87
  aggregate off/on matrix.
- `artifacts/prepare-isolated-surfaces.sql`, invoked by the suite as a raw
  step to create one-index-per-table surfaces from already-loaded real
  corpora.
- packet-local audit and dry-run artifacts proving the suite expands to the
  expected commands.

## Matrix Shape

The suite covers the Phase 6 aggregate surface:

- AMs: HNSW, IVF, SPIRE.
- Corpus sizes: real10k, real50k, real100k.
- Cells:
  - HNSW recall + latency with `ec_hnsw.candidate_batch_scoring=off/on`.
  - IVF recall + latency with `ec_ivf.scratch_soa_batch_decode` off/on.
  - SPIRE pipeline query metrics + recall with
    `ec_spire.candidate_batch_scoring=off/on`.
  - Storage once per AM/corpus isolated surface.

DiskANN remains represented by the accepted Task 87 Stop Condition packet
(`reviews/task-87/005-phase4-diskann-stop-condition/`) and the Task 91
handoff approved in packet 009.

## Isolation

The setup SQL creates dedicated `task87_phase6_real{10,50,100}k_{am}`
corpus/query tables and one AM index per table. This keeps Phase 6 evidence
aligned with the task-file and review-packet rule that benchmark cells should
avoid shared-table surface ambiguity.

The source data is already loaded in the local PG18 database:

- real10k: existing Task 28/30 loaded real-corpus surfaces.
- real50k: existing Task 67 loaded real-corpus surfaces.
- real100k: existing Task 67 loaded real-corpus surfaces.

The suite setup is idempotent (`CREATE TABLE IF NOT EXISTS`,
`CREATE INDEX IF NOT EXISTS`) so a failed or interrupted long run can resume.

## Validation

See `artifacts/manifest.md` for packet-local log metadata.

- `artifacts/cargo-build-ecaz-cli.log`
  - `cargo build -p ecaz-cli`
  - result: succeeded with one existing dead-code warning.
- `artifacts/corpus-list-pg18.log`
  - confirms the local PG18 database has candidate loaded real-corpus
    prefixes.
- `artifacts/catalog-prefixes.log`
  - confirms relevant real-corpus prefixes with HNSW, IVF, and SPIRE
    indexes.
- `artifacts/selected-index-reloptions.log`
  - records the source reloptions mirrored by the suite setup.
- `artifacts/suite-audit.log`
  - `ecaz bench suite audit --config ...`
  - result: `audit passed: 41 steps`
- `artifacts/suite-dry-run.log`
  - `ecaz bench suite run --dry-run ...`
  - result: expanded all 41 steps, including off/on GUCs for HNSW and SPIRE
    and the IVF scratch SoA on/off flag.
- `artifacts/suite-dry-run-manifest.json`
  - normalized dry-run manifest emitted by the suite runner.

## Review Focus

- Confirm the suite shape is acceptable for the Task 87 Phase 6 aggregate
  matrix.
- Confirm using suite-owned raw SQL for isolated-surface setup is acceptable
  under the FR-038 runner rule.
- Confirm DiskANN remains satisfied by packet 005's accepted Stop Condition
  plus packet 009's Task 91 handoff, rather than being reintroduced into the
  long matrix.
