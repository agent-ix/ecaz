# Review Request: Task 39 HNSW Page Coverage

Task: `plan/tasks/39-test-quality-measurement.md`

Implementation commit: `ce53f2540898342a745e52ddafe252366e21b848`

## Scope

This slice closes the HNSW AM page-codec coverage gap from the Task 39 review
checklist:

- imports `src/am/ec_hnsw/page.rs` into the `hardening/careful` coverage
  harness;
- exposes it through the harness-local `am::ec_hnsw::page` module path;
- runs the existing HNSW metadata, tuple codec, borrowed tuple-ref, and
  page-chain tests in the Task 39 coverage lane;
- raises the `am/ec_hnsw/page.rs` baseline from `0.00%` to `84.76%`;
- updates `docs/hardening.md` to record this packet as the ratchet source.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib`: passed,
  205 tests.
- `make coverage`: passed.
- `make coverage-baseline-check`: passed, 40 critical paths present.
- `scripts/check_coverage_delta.sh target/quality/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`:
  passed with `am/ec_hnsw/page.rs actual=84.76 baseline=84.76`.
- `git diff --check`: clean.

Key result: `am/ec_hnsw/page.rs` is now `84.76%` line coverage in the Task 39
coverage lane, clearing the reviewer target of at least 80% for that AM page
codec.

## Remaining Task 39 Gaps

This packet closes the HNSW page-codec coverage slice. Remaining structural
gaps are PG18/pgrx callback instrumentation feasibility, IVF/SPIRE page codec
coverage raises, SPIRE storage/coordinator coverage raises, storage guard
coverage/mutation, broader critical-module mutation triage, and scheduled CI
burn-in evidence.
