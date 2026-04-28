# Task 28 IVF Vacuum Compaction Checkpoint

## Summary

This packet records the A3 vacuum compaction code checkpoint at head
`20a2da6313a7558ab180cb21d647b63b630400fa`.

The previous IVF vacuum path rewrote fully-dead posting tuples as logical
tombstones. The new path lets the page rewrite callback return an explicit
`Delete` action. Vacuum now deletes fully-dead posting tuples with PostgreSQL's
`PageIndexTupleDeleteNoCompact`, which reclaims the tuple storage while
preserving existing line pointer numbers.

The no-compact primitive is intentional. A trial using compacting
`PageIndexTupleDelete` shifted directory tuple offsets when directory entries
and postings shared a block; the focused PG18 vacuum test caught that as
out-of-range directory TIDs. The landed version preserves stable item pointers
and passed the same PG18 vacuum path.

## Scope

- Added `IvfPostingRewrite::{Keep, Rewrite, Delete}` for page-level posting
  rewrites.
- Changed IVF vacuum to physically delete already-deleted or newly-empty
  posting tuples instead of rewriting them as tombstones.
- Preserved same-length rewrite behavior for partially-deleted postings.
- Did not add relation truncation; this checkpoint reclaims posting tuple
  storage/free space inside pages, not trailing relation blocks.

## Validation

Commands run:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::page::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_vacuum`
- `git diff --check`

Results:

- Page tests: 11 passed.
- IVF unit tests: 42 passed.
- PG18 IVF vacuum tests: 4 passed.
- `git diff --check`: clean.

## Follow-Up

A3 still needs scale evidence before claiming index-size convergence under
churn. The next vacuum slice should measure repeated delete/vacuum/insert
cycles and compare block growth/free-space reuse before considering relation
truncation.
