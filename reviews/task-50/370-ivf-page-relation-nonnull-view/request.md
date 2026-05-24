# Task 50 Review Request: IVF Page Relation NonNull View

## Summary

This slice advances the Task 50 IVF page tuple / buffer-page contract work.

Code commit: `59fad9a35ada90ffb67ad9e804c09fef51a1ed14`

Changed `src/am/ec_ivf/page.rs` so `IvfPageRelation` stores a
`NonNull<pg_sys::RelationData>` and private tuple/posting helpers take that
typed relation view instead of repeatedly constructing the page relation from a
raw `pg_sys::Relation`.

This removes the repeated direct unsafe blocks at IVF page call sites without
introducing a safe public raw PostgreSQL relation API.

## Unsafe Counts

- `src/am/ec_ivf/page.rs`: `29 -> 18`
- `src/` total direct unsafe blocks: `1198 -> 1187`

See `artifacts/unsafe-counts.log`.

## Plan Coverage

- Program: P3/P4, buffer-page-WAL and page tuple / line-pointer views.
- Wave/tranche: Wave 2, IVF page tuple and posting view cleanup.
- Disposition: repeated call-site `unsafe { IvfPageRelation::new(...) }`
  blocks were replaced by a non-null relation view constructed once and passed
  into private readers.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`
  passed; it reports the known pre-existing `src/am/mod.rs` unused SPIRE DML
  re-export warning.
- `git diff --check` passed.
- `rustfmt --check src/am/ec_ivf/page.rs` passed.
- Raw-boundary guard produced no matches.
- Generated unsafe ledger covers all `1187` current `src` unsafe rows.

Artifacts are under `artifacts/`; see `artifacts/manifest.md`.
