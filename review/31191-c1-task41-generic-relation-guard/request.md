# Task 41 Review Request: Generic Relation Guard

## Scope

This checkpoint adds a shared `RelationGuard` for PostgreSQL
`relation_open` / `relation_close` pairs and migrates the first one-shot
aux-store users:

- `src/storage/relation_guard.rs`
- `src/am/ec_spire/storage/relation_plan.rs`
- `src/tests/build.rs`

Code commit: `7bd67c24876301faefc362ded0871305264942a0`

## Safety Invariant

`RelationGuard::try_open` owns the relation handle returned by
`pg_sys::relation_open` and closes it with the same lockmode in `Drop`.
This gives aux-store relation users the same pgrx-unwind cleanup property
as the existing `IndexRelationGuard` and `HeapRelationGuard`.

This slice deliberately keeps the API minimal: only `try_open` and
`as_ptr` are exposed. A pgrx-erroring `open` helper can be added later when
production callers want that behavior.

## Baseline Impact

Unsafe comment baseline decreased:

- before: `4252`
- after: `4251`

The production aux-store creation path no longer manually pairs
`relation_open` and `relation_close`. The test reloptions assertion also
uses the shared guard, though the tracked baseline does not count that as a
net removal because nearby unsafe line numbers shifted.

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

- Confirm `RelationGuard` belongs beside `IndexRelationGuard` and
  `HeapRelationGuard`.
- Confirm `try_open`-only is the right initial API for callers that need to
  return `Result` instead of raising `pgrx::error!`.
- Confirm the aux-store metadata initialization relation remains open for
  the full metadata-page initialization call.
