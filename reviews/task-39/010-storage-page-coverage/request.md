# Review Request: Task 39 Storage Page Coverage

Task: `plan/tasks/39-test-quality-measurement.md`

Implementation commit: `a9859d4c`

## Scope

This slice closes the reviewer-noted gap where `src/storage/page.rs` was below
the 80% coverage floor:

- adds `hardening/careful` tests for page size/free-space accounting;
- covers invalid `ItemPointer` lengths and invalid tuple lookup/update TIDs;
- covers in-place tuple updates, zero-page append no-op behavior, and
  oversized tuple rejection at both `DataPage` and `DataPageChain` layers;
- raises the `storage/page.rs` baseline from `76.57%` to `97.90%`;
- updates `docs/hardening.md` so the coverage table cites this packet as the
  ratchet source.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib`: passed,
  164 tests.
- `make coverage`: passed.
- `make coverage-baseline-check`: passed, 40 critical paths present.
- `scripts/check_coverage_delta.sh target/quality/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`:
  passed with `storage/page.rs actual=97.90 baseline=97.90`.
- `git diff --check`: clean.

Key result: `storage/page.rs` is now `97.90%` line coverage in the Task 39
coverage lane, clearing the reviewer target of at least 80%.

## Remaining Task 39 Gaps

This packet closes the specific `storage/page.rs` coverage gap. Remaining
structural gaps are unchanged: PG18/pgrx callback instrumentation feasibility,
AM page codec coverage raises, SPIRE storage/coordinator coverage raises,
storage guard coverage/mutation, broader critical-module mutation triage, and
scheduled CI burn-in evidence.
