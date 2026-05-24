# Review Request: IVF Insert Relation Boundary

## Summary

This checkpoint follows up packet 159 by pushing the raw-relation contract through the IVF insert helper layer instead of repeating unsafe blocks around every page helper call.

Code commit: `40aa6ce22afef20521376015fc468bea24a71c8d`

## Scope

- Marked IVF insert helpers that carry raw `pg_sys::Relation` as unsafe:
  - `insert_into_trained_index`
  - `ensure_heap_tid_absent`
  - `bootstrap_empty_index`
  - `load_centroid_model`
  - `load_directory_entry`
- Removed redundant inner unsafe blocks around page helper calls now covered by the helper-level relation contract.
- Kept the PostgreSQL AM callback/debug entry points as the owner of the live-relation invariant.

## Completion Audit Note

This is a structural cleanup slice after packet 159: it keeps the reviewer-requested unsafe boundary explicit while reducing local unsafe-block clutter in the IVF insert path. It does not close the whole IVF cluster or Task 50.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed; existing unused-import warning remains in `src/am/mod.rs`.
- `git diff --check`: passed.
- `make unsafe-block-count`: passed; `src/am/ec_ivf/insert.rs` is now at 4 unsafe blocks.

See `artifacts/manifest.md` for packet-local command provenance and key output lines.
