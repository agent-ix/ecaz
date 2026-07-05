# Review Request: SPIRE multi-store scan width coverage

- coder: coder1
- date: 2026-05-14
- code commit: 4299c892 `Cover SPIRE multi-store scan widths`
- topic: SPIRE phase 12c.15.a/b three-store and four-store scan fixtures

## Scope

This slice adds focused multi-store scan coverage without expanding any already
large test file.

Changed file:

- `src/tests/scan.rs`

## What Changed

Added a compact helper, `assert_ec_spire_multistore_scan_width_sql`, plus two
pg_test fixtures:

- `test_ec_spire_three_store_scan_width_sql`
- `test_ec_spire_four_store_scan_width_sql`

Each fixture builds a deterministic 96-row SPIRE index with `nlists = 12`,
`nprobe = 12`, `rerank_width = 12`, and either `local_store_count = 3` or
`local_store_count = 4`.

The shared assertion checks:

- `ec_spire_index_placement_snapshot` reports exactly the expected store IDs
- `ec_spire_index_scan_local_store_read_overlap_harness` visits the same full
  store ID set
- every visited store has a nonzero read batch
- an ordered SPIRE scan still returns the exact query-vector row first

## Test File Size Discipline

The touched test file remains below the 2500-line target:

```text
1329 src/tests/scan.rs
```

No new file was needed for this narrow 12c.15 scan-width slice. The helper
keeps the 3-store and 4-store fixtures from duplicating the setup SQL.

## Validation

Passed:

```text
cargo fmt --check
git diff --check -- src/tests/scan.rs
cargo test --features "pg18 pg_test" --no-default-features test_ec_spire_three_store_scan_width_sql --no-run
```

`cargo fmt --check` emitted the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` and `group_imports`, but exited
successfully.

I did not run the pg_test binaries. Earlier runtime attempts in this branch
still hit the local PostgreSQL backend symbol boundary before executing tests;
this slice was validated with the narrow compile-only target.

## Review Focus

Please check whether asserting full placement width, full scan-harness width,
and a successful ordered scan is sufficient for 12c.15.a/b, or whether reviewers
want a follow-up that captures per-store object byte totals as an additional
multi-NVMe balance signal.
