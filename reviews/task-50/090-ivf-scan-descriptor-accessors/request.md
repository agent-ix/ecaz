# Task 50 Review Request: IVF Scan Descriptor Accessors

Code commit: `54c9021477ed70b3910c1dddaa2e159b43b9cff7`

## Scope

This packet advances the broad Task 50 unsafe burndown plan from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

Programs/tranches:

- P2 PostgreSQL handle views
- P5 heap source, tuple slot, and snapshot contracts
- P10 scan opaque and raw ownership contracts
- Wave 2 IVF/RaBitQ production fanout

## Change

This slice centralizes repeated IVF scan descriptor and scan opaque reads:

- Added checked helpers for `IndexScanDesc`, `IndexScanState`, active snapshot, index-to-heap OID resolution, and IVF scan opaque access.
- Reworked heap rerank relation/snapshot resolution to use those helpers instead of repeated direct pointer dereferences.
- Reworked EXPLAIN counter extraction and pg_test debug probes to consume the same checked accessor helpers.
- Reused existing scan-owned box helpers for prepared-query debug lengths.

The remaining IVF scan unsafe is still around scan-owned memory allocation/free, PQ model loading, heap slot reader construction, debug AM wrappers, and direct order-by output pointer reads.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1953 | 1940 | -13 |
| `src/am/ec_ivf/scan.rs` | 36 | 23 | -13 |
| `src/` unsafe ledger rows | 1953 | 1940 | -13 |

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: pass, with the known pre-existing unused SPIRE DML export warning in `src/am/mod.rs`
- `git diff --check`
  - log: `artifacts/git-diff-check.log`
  - result: pass
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: current `src/` total is `1940` direct unsafe blocks across `131` files
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1940` current `src/` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1940 current unsafe rows`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/count-summary.md`
- `artifacts/code-diff.patch`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/git-diff-check.log`
- `artifacts/src-unsafe-block-count-after.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`

## Residual Work

Task 50 is not complete. IVF still has direct unsafe in `scan.rs`, `page.rs`, `build.rs`, `vacuum.rs`, `options.rs`, `admin.rs`, `cost.rs`, `insert.rs`, and `routine.rs`. The broader Task 50 ledger also still covers SPIRE, HNSW, DiskANN, shared AM/storage, quant, tests, hardening, crates, and vendor disposition.
