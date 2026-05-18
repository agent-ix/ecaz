# Review Request: Task 41 Invariant #2 IVF scan opaque-owned slices

Code commit: `9e1e9d62a67df398a636d55fc402b53781460753`

## Summary

This Phase C slice tightens `src/am/ec_ivf/scan.rs` so palloc-backed scan
state is exposed through methods on `EcIvfScanOpaque`.

The raw query slice, debug selected-list slice, and posting-candidate cursor
are now owner-tied methods on the scan opaque. Heap rerank borrows the query
slice through `opaque.query_values()` and keeps counter mutation outside that
borrow. Debug helpers copy out through owner methods, and `amgettuple` advances
posting candidates through `opaque.next_posting_candidate()`.

## Scope

- Changed `src/am/ec_ivf/scan.rs` only.
- Preserved existing allocation and `pfree` points for query values, selected
  lists, centroid scores, and posting candidates.
- Did not change buffer/page access, heap slot ownership, AM callback control
  flow, or scan result ordering.

## Validation

- `cargo fmt --all --check`
- `cargo check --no-default-features --features pg18`
- `git diff --check HEAD~1 HEAD`

The PG18 cargo check completed successfully with the pre-existing unused import
warning in `src/am/mod.rs`. No pgrx runtime tests were run for this local
lifetime-shape refactor.

## Artifacts

- `artifacts/fmt-check.log`
- `artifacts/cargo-check-pg18.log`
- `artifacts/git-diff-check.log`
- `artifacts/code-diff-stat.log`
- `artifacts/ivf-scan-slice-inventory.log`
- `artifacts/manifest.md`

## Reviewer Focus

- Confirm raw slices over IVF palloc scan-state fields are now created only
  inside `EcIvfScanOpaque` owner methods.
- Confirm the query slice borrow in heap rerank cannot overlap mutable access
  to `opaque`.
- Confirm posting candidate cursor access remains bounded by
  `posting_candidate_count`.
