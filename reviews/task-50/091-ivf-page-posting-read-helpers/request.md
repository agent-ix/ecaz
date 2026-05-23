# Task 50 Review Request: IVF Page Posting Read Helpers

Code commit: `d3a45863ec37eee4a9f3d54c4cbde7738064c435`

## Scope

This packet advances the broad Task 50 unsafe burndown plan from
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.

Programs/tranches:

- P2 PostgreSQL handle views
- P3 buffer, page, and WAL transaction contracts
- P4 page tuple and line-pointer views
- P9 read stream / posting block helpers
- Wave 2 IVF/RaBitQ production fanout

## Change

This slice removes repeated unsafe from IVF page posting readers and tuple line-pointer access:

- Added `read_posting_block` to route fallback posting block readers through `IvfPageRelation::read_main`.
- Replaced three direct `LockedBufferGuard::read_main` unsafe blocks in posting block visitors.
- Added `page_item_id_ref` and replaced repeated raw item-id dereferences in tuple reader/writer paths.

The remaining IVF page unsafe is still lower-level PostgreSQL page/WAL/buffer and raw tuple-byte boundary code.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1940 | 1936 | -4 |
| `src/am/ec_ivf/page.rs` | 33 | 29 | -4 |
| `src/` unsafe ledger rows | 1940 | 1936 | -4 |

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  - log: `artifacts/cargo-check-pg18-bench.log`
  - result: pass, with the known pre-existing unused SPIRE DML export warning in `src/am/mod.rs`
- `git diff --check`
  - log: `artifacts/git-diff-check.log`
  - result: pass
- `make unsafe-block-count`
  - log: `artifacts/src-unsafe-block-count-after.log`
  - result: current `src/` total is `1936` direct unsafe blocks across `131` files
- `make unsafe-ledger`
  - log: `artifacts/unsafe-ledger-generate.log`
  - result: generated `1936` current `src/` ledger rows
- `make unsafe-ledger-check`
  - log: `artifacts/unsafe-ledger-check.log`
  - result: `ledger covers 1936 current unsafe rows`

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

Task 50 is not complete. IVF still has direct unsafe in page, scan, build, vacuum, options, admin, cost, insert, and routine files. The broader ledger also still covers SPIRE, HNSW, DiskANN, shared AM/storage, quant, tests, hardening, crates, and vendor disposition.
