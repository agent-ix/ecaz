# Task 87 Packet 012: Phase 6 Suite Preparation + Real10k Slice

## Summary

This packet asks for review of the runnable Phase 6 measurement suite shape
for Task 87 and the first completed real10k measurement slice. It does not
claim Phase 6 closeout yet.

Current head for this packet:

- `bbe3ef9e8b376c632267c0b39089d3b4ac483373` - `Update Task 87 Phase 6 suite setup`

The packet adds:

- `phase6-suite.json`, an `ecaz bench suite` config for the Task 87
  aggregate off/on matrix.
- `artifacts/prepare-isolated-surfaces.sql`, invoked by the suite as a raw
  step to create the missing Task 87 suite-owned surfaces from already-loaded
  real corpora.
- packet-local audit and dry-run artifacts proving the suite expands to the
  expected commands.
- packet-local real10k HNSW/IVF/SPIRE off/on artifacts.

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

## Surface Selection

The setup SQL creates dedicated `task87_phase6_real10k_{am}` corpus/query
tables and a missing `task87_phase6_real50k_spire` surface. The 50k and 100k
HNSW/IVF cells, plus the 100k SPIRE cell, reuse existing one-AM real-corpus
surfaces that already have the required indexes.

This avoids rebuilding large HNSW indexes during suite setup while keeping the
measurement cells on single-AM index tables rather than mixed AM surfaces.

The source data is already loaded in the local PG18 database:

- real10k: suite-owned surfaces copied from existing Task 28/30 loaded
  real-corpus tables.
- real50k: existing Task 67 HNSW/IVF surfaces plus a suite-owned SPIRE surface.
- real100k: existing current/Task 28/Task 74 HNSW/IVF/SPIRE surfaces.

The suite setup is idempotent (`CREATE TABLE IF NOT EXISTS`,
`CREATE INDEX IF NOT EXISTS`) so a failed or interrupted long run can resume.

The SPIRE pipeline cells are capped at 100 queries and use AM-specific truth
cache files because the suite-owned SPIRE query table is separately loaded from
the HNSW/IVF real10k tables. The HNSW/IVF cells share
`truth-real10k-k10.json`; SPIRE uses `truth-real10k-spire-k10.json`.

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
- `artifacts/setup-run.log`
  - setup-only suite execution for `precheck-host` and
    `prepare-isolated-surfaces`
  - result: succeeded after the suite was revised to avoid rebuilding large
    HNSW indexes.
- `artifacts/run/precheck-host.log`
  - packet-local precheck output from the setup run.
- `artifacts/run/prepare-isolated-surfaces.log`
  - packet-local setup SQL output; confirms the 50k SPIRE index completed with
    `total_ms=306144`.
- `artifacts/refresh-spire-pipeline-functions.{sql,log}`
  - narrow local PG18 refresh for the SPIRE diagnostic wrappers used by
    `ecaz bench spire-pipeline`; result: 15 missing wrappers created.
- `artifacts/real10k-run.{log,manifest.json}`
  - first real10k run; completed HNSW and IVF cells, then failed before SPIRE
    because the local PG18 extension catalog was missing SPIRE diagnostic
    wrappers.
- `artifacts/run/truth-real10k-spire-generate.log`
  - generated the SPIRE-specific 100-query truth cache.
- `artifacts/real10k-spire-rerun.{log,manifest.json,results.jsonl}`
  - reran only the real10k SPIRE off/on pipeline cells and SPIRE storage after
    refreshing wrappers and switching SPIRE to its own truth cache.

## Real10k Slice Results

All real10k cells used PG18 on `/home/peter/.pgrx:28818`.

| AM | off recall | on recall | off latency | on latency | storage |
| --- | ---: | ---: | ---: | ---: | ---: |
| HNSW | 0.6550, mean q-time 35.75 ms | 0.6550, mean q-time 35.16 ms | p50 32.6 ms, p95 44.2 ms | p50 31.6 ms, p95 44.1 ms | total 171.8 MiB, indexes 13.0 MiB |
| IVF | 1.0000, mean q-time 121.33 ms | 1.0000, mean q-time 120.25 ms | p50 119.6 ms, p95 135.9 ms | p50 117.4 ms, p95 128.7 ms | total 168.2 MiB, indexes 9.4 MiB |
| SPIRE | 1.0000, p50 168.137 ms, p95 187.051 ms | 1.0000, p50 106.142 ms, p95 122.507 ms | pipeline query metrics | pipeline query metrics | total 167.0 MiB, indexes 8.2 MiB |

Notes:

- HNSW and IVF recall is byte-equal across off/on for this slice.
- SPIRE candidate-batch on improves the pipeline p50 from 168.137 ms to
  106.142 ms on this real10k surface while preserving recall@k 1.0000.
- The SPIRE endpoint identity reports local tuple transport ready and remote
  serving status `requires_rabitq_storage_format`, which is expected for this
  local TurboQuant surface and does not affect the local pipeline cell.

## Review Focus

- Confirm the suite shape is acceptable for the Task 87 Phase 6 aggregate
  matrix.
- Confirm the real10k slice is acceptable as the first Phase 6 measurement
  checkpoint.
- Confirm using suite-owned raw SQL for missing-surface setup is acceptable
  under the FR-038 runner rule.
- Confirm the SPIRE-specific truth cache and 100-query cap are acceptable for
  separately loaded SPIRE real-corpus surfaces.
- Confirm reusing existing one-AM 50k/100k surfaces is acceptable after the
  initial all-isolated setup proved too expensive for HNSW rebuilds.
- Confirm DiskANN remains satisfied by packet 005's accepted Stop Condition
  plus packet 009's Task 91 handoff, rather than being reintroduced into the
  long matrix.
