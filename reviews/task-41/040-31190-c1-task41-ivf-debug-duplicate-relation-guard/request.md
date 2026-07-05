# Task 41 Review Request: IVF Duplicate Debug Relation Guard

## Scope

This checkpoint migrates `debug_ec_ivf_validate_no_duplicate_heap_tid` from
manual `index_open` / `index_close` to the shared `IndexRelationGuard`.

Code commit: `8c8de0c6ec2e73ccb85ab309b2d8dc4cc7437552`

## Safety Invariant

The debug helper needs an AccessShare relation handle while it reads IVF
metadata and checks whether a heap TID is absent from the index.
`IndexRelationGuard::access_share` now owns the relation handle and closes
it with the matching AccessShare lockmode on all Rust unwind paths.

## Baseline Impact

Unsafe comment baseline decreased:

- before: `4254`
- after: `4252`

This removes the manual relation open/close sites and leaves only the
metadata read / heap-TID walk unsafe operations in the helper.

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

- Confirm the guard lifetime covers both metadata read and duplicate check.
- Confirm using a fully-qualified guard path in the cfg-gated helper is
  preferable to a normal import that is unused in non-test builds.
