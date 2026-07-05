# Review Request: Task 28 IVF Build Smoke Coverage

Scope: Phase 3 smoke-coverage checkpoint. PG build tests now cover the
persisted shape of empty, singleton, tiny multi-row, duplicate-heavy, and
multi-page-list `ec_ivf` builds.

Task: `plan/tasks/28-ivf-access-method.md` Phase 3

Branch: `task28-ivf`

Head SHA: `14b73196dd2ea68534bcd083ab18bc52b1dbbbae`

Owner: coder2

Files:

- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy.
- The new PG tests were compiled but not run. No test suite was executed for
  this checkpoint because the repository policy now asks agents not to run tests
  unless necessary.
- No measurement claim is made in this packet.

## Summary

This slice closes Phase 3 build smoke coverage:

- Extends the empty-index build test to verify directory-summary readback
  reports every configured list as empty with zero live/dead/drift counters.
- Adds a singleton build test that verifies one live row, trained metadata, and
  a non-empty directory.
- Keeps the existing tiny multi-row build test and routes it through shared IVF
  test helpers.
- Adds duplicate-heavy coverage where all rows choose one centroid and the
  remaining lists stay empty.
- Adds a multi-page-list build test with a 512-dimensional fixture that proves
  one IVF list can span more than one data page.
- Updates the task plan to mark Phase 3 build smoke tests complete and moves
  status to Phase 4 query prep next.

## Review Focus

Please review for:

- Whether the duplicate-heavy fixture should assert the exact empty-list count
  at this stage, given deterministic tie-breaking sends identical vectors to
  list 0.
- Whether the multi-page fixture is large enough to be stable across PostgreSQL
  page-layout changes while still staying cheap to run.
- Whether the shared `ec_ivf_index_oid` and `ec_ivf_index_blocks` helpers are
  the right level of test abstraction for the current PG test module.
- Whether any of these smoke cases should be converted into lower-level Rust
  build-state tests instead of PG build tests.

## Non-Goals

This packet does not implement populated IVF scans, nearest-list routing,
candidate scoring, live insert, vacuum, planner costing, or any measurement
claim.
