# Review Request: Task 41 Invariant #2 HNSW build parallel DSM scoped slices

Code commit: `8848b8dd6f7ac853990e1ba2e01d56c9e1f3742a`

## Summary

This Phase C slice handles the HNSW build-parallel DSM source/code slice
surface in `src/am/ec_hnsw/build_parallel.rs`.

The previous `concurrent_dsm_code_for_node` and
`concurrent_dsm_source_for_node` helpers returned `&'static` slices into DSM
graph storage. They now use scoped callbacks:

- `with_concurrent_dsm_code_for_node`
- `with_concurrent_dsm_source_for_node`

The score path computes the score inside nested callbacks, so DSM-backed code
and source slices cannot be returned or stored by the helper API.

## Scope

- Changed `src/am/ec_hnsw/build_parallel.rs` only.
- Preserved DSM layout, offsets, locking, score cache behavior, and score math.
- Did not rewrite neighbor-slot helpers or test-only DSM image assertions; the
  remaining DSM raw slices are local readback, lock-bounded slot views,
  message decoding, or test assertions.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

The PG18 cargo check completed successfully with the pre-existing unused import
warning in `src/am/mod.rs`. No pgrx runtime tests were run for this scoped
helper refactor.

## Artifacts

- `artifacts/fmt-check.log`
- `artifacts/cargo-check-pg18.log`
- `artifacts/git-diff-check.log`
- `artifacts/code-diff-stat.log`
- `artifacts/build-parallel-dsm-slice-inventory.log`
- `artifacts/manifest.md`

## Reviewer Focus

- Confirm DSM code/source slices can no longer escape through `&'static`
  return types.
- Confirm nested callback scoring preserves the previous score calculation.
- Confirm remaining DSM `from_raw_parts` sites are not ordinary palloc
  scan-state slices and are suitable for the later buffer/DSM ownership review
  rather than Phase C palloc cleanup.
