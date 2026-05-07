# Review Request: SPIRE Empty Root/Control Page

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `b55dbadd Persist SPIRE empty root control`

## Summary

This checkpoint starts the relation-backed SPIRE persistence implementation at
the smallest live AM surface: an empty root/control page. It keeps populated
partition-object persistence blocked, but proves that `ec_spire` can create a
relation-backed index on an empty heap, persist the empty root/control state,
read it during live scan setup, and return an empty cursor instead of failing.

## Changed Files

- `src/am/ec_spire/build.rs`
- `src/am/ec_spire/meta.rs`
- `src/am/ec_spire/mod.rs`
- `src/am/ec_spire/page.rs`
- `src/am/ec_spire/scan.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## What Changed

- Added SPIRE-owned page helpers for initializing and reading the relation
  metadata block as the root/control page, using the existing PostgreSQL page
  and GenericXLog pattern already used by `ec_ivf`.
- `ambuild` now validates relation options, scans the heap, rejects populated
  heap builds with an explicit not-implemented error, and initializes the empty
  persisted root/control page for empty builds.
- `ambuildempty` now initializes the same empty persisted root/control page.
- `amrescan` now decodes the ORDER BY query, reads the persisted root/control
  page, and returns an empty candidate cursor when no active epoch is published.
- Added pg regression coverage for creating an `ec_spire` index on an empty
  table and scanning it with an ordered LIMIT query.
- Updated Task 30 status to reflect that empty relation-backed root/control
  persistence is live while populated object persistence remains open.

## Validation

- `cargo fmt`
  - Completed with the repository's existing stable-rustfmt warnings for
    unstable `imports_granularity` and `group_imports` settings.
- `cargo test --lib test_ec_spire_empty_build_scan_no_rows --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1063 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `183 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

## Notes For Reviewer

- This intentionally does not implement populated partition-object writes or
  published snapshot loading. Populated `ambuild` fails before root/control
  initialization so the current persistence checkpoint cannot create a partial
  non-empty index.
- The root/control page shape follows `ec_ivf`'s metadata-page mechanics, but
  the payload remains SPIRE-owned and versioned through
  `SpireRootControlState`.
- The untracked architecture-review feedback file
  `review/30219-spire-foundation-progress-status/feedback.md` remains local and
  was not staged or committed by this checkpoint.
