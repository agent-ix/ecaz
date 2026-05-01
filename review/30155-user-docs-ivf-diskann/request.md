# User Docs IVF and DiskANN Coverage

## Scope

Docs-only follow-up to make the public README and user docs cover the landed
IVF and DiskANN access methods.

## Changes

- Updated `docs/getting-started.md` to use the canonical `ecvector` flow and
  show HNSW, IVF, and DiskANN index creation.
- Rewrote `docs/usage.md` around current surfaces:
  - `ecvector` vs `tqvector`
  - `encode_to_ecvector`
  - HNSW, IVF, and DiskANN reloptions
  - session GUCs for `ec_hnsw`, `ec_ivf`, and `ec_diskann`
  - DiskANN's unit-normalized vector contract
- Rewrote `docs/benchmarks.md` to include:
  - existing HNSW recall targets and baseline rows
  - Task 28 IVF 10K/25K/100K/990K local results
  - Task 29 DiskANN final local readiness rows
  - source review packets and product-benchmark caveats
- Updated the README performance summary to point at the current HNSW, IVF,
  and DiskANN local evidence instead of only HNSW.

## Validation

- `git diff --check`
- Text scan over README/getting-started/usage/benchmarks for stale HNSW-only
  example wording and missing IVF/DiskANN benchmark coverage.

No Rust or pgrx tests were run because this is Markdown-only.

## Artifacts

No new measurement artifacts. Benchmark numbers are copied from existing
packet-local artifacts and summaries:

- `review/30145-task28-ivf-a10-current-closure/`
- `review/30119-task28-ivf-a9-100k-current-build/`
- `review/30131-task28-ivf-current-gate-status/`
- `review/30151-task28-ivf-local-landing-status/`
- `review/11109-task29d-final-readiness/`
