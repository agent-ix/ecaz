# Task 41 Review Request: HNSW Build Heap Relation Guards

## Scope

This checkpoint migrates HNSW build heap relation opens from manual
`table_open` / `table_close` pairs to the shared `HeapRelationGuard`.

Touched file:

- `src/am/ec_hnsw/build.rs`

Code commit: `a1478b4f2fc2f6f733332f715276b9f0ab102d1b`

## Safety Invariant

The heap relation is opened with AccessShare only when the index relation
resolves to a valid heap relation. `HeapRelationGuard::try_access_share`
owns the matching `table_close`, and the guard lifetime covers the indexed
vector attribute lookup and rerank-source validation calls.

## Baseline Impact

Unsafe comment baseline decreased:

- before: `4245`
- after: `4241`

This removes four tracked unsafe sites from HNSW build heap relation
open/close handling.

## Validation

See `artifacts/validation.md`.

Commands run:

- `cargo fmt`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
- `git diff --check`
- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

## Review Focus

- Confirm AccessShare is still the correct lock for both heap metadata
  validation paths.
- Confirm the guard lifetime covers the full unsafe relation use in each
  call.
