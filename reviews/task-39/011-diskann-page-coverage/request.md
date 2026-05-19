# Review Request: Task 39 DiskANN Page Coverage

Task: `plan/tasks/39-test-quality-measurement.md`

Implementation commit: `9f39c99bd4980a4b2214df1d1c40008911d881c8`

## Scope

This slice closes the DiskANN AM page-codec coverage gap from the Task 39
review checklist:

- imports `src/am/ec_diskann/page.rs` into the `hardening/careful` coverage
  harness;
- exposes it through the harness-local `am::ec_diskann::page` module path;
- runs the existing Vamana metadata encode/decode tests in the Task 39
  coverage lane;
- raises the `am/ec_diskann/page.rs` baseline from `0.00%` to `97.35%`;
- updates `docs/hardening.md` to record this packet as the ratchet source.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib`: passed,
  172 tests.
- `make coverage`: passed.
- `make coverage-baseline-check`: passed, 40 critical paths present.
- `scripts/check_coverage_delta.sh target/quality/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`:
  passed with `am/ec_diskann/page.rs actual=97.35 baseline=97.35`.
- `git diff --check`: clean.

Key result: `am/ec_diskann/page.rs` is now `97.35%` line coverage in the Task
39 coverage lane, clearing the reviewer target of at least 80% for that AM page
codec.

## Remaining Task 39 Gaps

This packet closes the DiskANN page-codec coverage slice. Remaining structural
gaps are PG18/pgrx callback instrumentation feasibility, HNSW/IVF/SPIRE page
codec coverage raises, SPIRE storage/coordinator coverage raises, storage guard
coverage/mutation, broader critical-module mutation triage, and scheduled CI
burn-in evidence.
