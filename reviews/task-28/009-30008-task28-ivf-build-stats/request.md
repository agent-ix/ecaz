# Review Request: Task 28 IVF Build Stats Readback

Scope: Phase 3 build-stats checkpoint. Persisted `ec_ivf` directory tuples can
now be read back from index pages so PG tests can verify per-list build counts
instead of relying only on metadata and relation size.

Task: `plan/tasks/28-ivf-access-method.md` Phase 3

Branch: `task28-ivf`

Head SHA: `fa06d282100abca4db07fd224117e4625acae16a`

Owner: coder2

Files:

- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy.
- No test suite was run for this narrow readback checkpoint. The existing
  populated-build PG test was extended but not re-run after the PG18-only
  compile check.
- No measurement claim is made in this packet.

## Summary

This slice adds persisted directory readback for the IVF build path:

- Adds a page-level helper that reads an `IvfListDirectoryTuple` from a physical
  PostgreSQL index page and returns the next physical TID.
- Adds a PG-test-only `debug_ec_ivf_directory_summary` helper that walks the
  directory entries from metadata `directory_head`, validates list ordering, and
  returns `nlists`, empty-list count, live/dead totals, and
  inserted-since-build totals.
- Extends the non-empty IVF build PG test to assert directory totals for a
  three-list build.
- Marks the Phase 3 build-stats task complete in the IVF plan.

## Review Focus

Please review for:

- Whether the physical-contiguous directory walk is acceptable for this stage,
  given the current build writer inserts all directory tuples consecutively
  after postings.
- Whether `next_physical_tuple_tid` should be replaced with explicit tuple refs
  or directory page links before Phase 4 scan routing.
- Whether the directory summary helper should remain PG-test-only, or whether
  these counters should become a user-facing diagnostic surface later.
- Whether the empty-index behavior, returning all lists as empty when
  `directory_head` is invalid and live tuple count is zero, matches the intended
  metadata contract.

## Non-Goals

This packet does not implement populated IVF scans, nearest-list routing,
candidate scoring, live insert, vacuum, planner costing, directory updates, or
any measurement claim.
