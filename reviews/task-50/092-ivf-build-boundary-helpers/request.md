# Task 50 Review Request: IVF Build Boundary Helpers

## Summary

This packet reviews commit
`002da64da2a1a487b9ffc4d24cd99cc48ac6626b`, which continues the Task 50
unsafe burndown in the IVF/RaBitQ production build path.

The slice removes `8` direct unsafe blocks from `src/am/ec_ivf/build.rs`
(`17 -> 9`) and moves the remaining PostgreSQL boundary operations behind
named helper contracts.

## What Changed

- Converted `ec_ivf_build_callback`, `ec_ivf_ambuild`, and
  `ec_ivf_ambuildempty` to use `pg_am_callback!`, so callback bodies delegate
  through the shared AM callback guard instead of wrapping each callback body
  in a broad direct unsafe block.
- Added `build_state_mut` and `table_index_build_scan` helpers for callback
  private data and heap build scan boundaries.
- Added `write_data_page` to own buffer/page/WAL mutation for IVF build data
  pages, while leaving the caller-side write loop safe.
- Made `build_index_tuple`, `decode_heap_tid`, and
  `resolve_indexed_vector_kind` safe callers by introducing narrower helper
  boundaries for datum construction, index-info access, heap tuple descriptor
  access, and type-name lifetime management.

## Evidence

- Code diff: `artifacts/code-diff.patch`
- Validation: `artifacts/cargo-check-pg18-bench.log`
- Whitespace check: `artifacts/git-diff-check.log`
- Unsafe count: `artifacts/src-unsafe-block-count-after.log`
- Count summary: `artifacts/count-summary.md`
- Ledger: `artifacts/unsafe-ledger-after.jsonl`
- Ledger generation/check logs:
  `artifacts/unsafe-ledger-generate.log`,
  `artifacts/unsafe-ledger-check.log`

## Result

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1936 | 1928 | -8 |
| `src/am/ec_ivf/build.rs` | 17 | 9 | -8 |
| `src/` unsafe ledger rows | 1936 | 1928 | -8 |

Validation:

- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with the existing unused SPIRE DML import warning in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-ledger-check`: passed; ledger covers `1928` current `src/`
  unsafe rows.

Task 50 is not complete. This packet is one checkpoint in the broader
comprehensive burndown plan in
`reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`.
