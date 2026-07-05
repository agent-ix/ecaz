# Task 39 Review Request: IVF Page Coverage and Mutation

Code checkpoint: `a2a142fc38f047c3d1f10d2c4990c264609baa52`

## Summary

This packet closes the IVF page-codec coverage and mutation gap for
`src/am/ec_ivf/page.rs` in the pgrx-free `hardening/careful` lane.

Changes:

- Added `src/am/ec_ivf/page.rs` to the `hardening/careful` harness and to the
  careful-backed `make mutants` mapping.
- Gated PG buffer/WAL helpers behind `pg17`/`pg18` while keeping the pure IVF
  metadata, tuple, fit, `DataPage`, and small helper code testable without pgrx.
- Added IVF page tests for metadata format/rerank codecs, tuple layout
  constants, centroid/list-directory/posting/PQ-codebook roundtrips, posting
  flag/count rejection, page-chain updates, fit boundaries, physical TID
  advancement, posting free hints, and line-pointer counting.
- Raised the coverage baseline for `am/ec_ivf/page.rs` from `0.00%` to
  `95.86%` and documented the new baseline in `docs/hardening.md`.
- Replaced the derived posting deleted-flag expression with a literal to remove
  an equivalent shift mutant while preserving the wire value.

## Evidence

- Focused tests: `artifacts/careful-ivf-page-tests.log`
  - 21 passed, 0 failed.
- Coverage: `artifacts/coverage/summary.txt`
  - `am/ec_ivf/page.rs`: 1328 lines, 55 missed, `95.86%` line coverage.
- Initial mutation run: `artifacts/mutants/page.rs.mutants/mutants.out/missed.txt`
  - 221 mutants tested, 41 missed, 143 caught, 37 unviable.
- Intermediate mutation run: `artifacts/mutants-rerun/page.rs.mutants/mutants.out/missed.txt`
  - 221 mutants tested, 1 missed, 182 caught, 38 unviable.
- Final mutation run: `artifacts/mutants-final/page.rs.mutants/mutants.out/outcomes.json`
  - 220 mutants tested, 0 missed, 182 caught, 38 unviable.
- Production compile check: `artifacts/cargo-check-pg18-bench.log`
  - passed with the pre-existing `src/am/mod.rs` unused-import warning.
- Coverage baseline completeness: `artifacts/coverage-baseline-check.log`
  - `coverage baseline complete for 40 critical paths`.
- Whitespace check: `artifacts/git-diff-check.log`
  - no whitespace errors.

## Review Notes

Please focus on whether the fallback no-pgrx shims are narrow enough for the
careful harness, and whether the new tests pin the IVF wire layout and helper
semantics without changing the PG18 buffer/WAL behavior.
