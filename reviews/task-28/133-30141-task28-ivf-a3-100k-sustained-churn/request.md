# Review Request: Task 28 IVF A3 100k Sustained Churn

## Summary

This packet records the A3 sustained-churn closure measurement after commit
`2b72141b`.

The production IVF insert path no longer rereads the assigned posting list for
duplicate heap-TID validation on every insert. PostgreSQL heap TIDs are unique
for normal index inserts, and the explicit pg_test duplicate-validation helper
still covers the corruption-check path. The `ecaz stress ivf-vacuum-scale`
harness now supports:

- `--cycles`
- `--churn-rows`
- `--refill-after-vacuum`
- `--vector-period`
- `--same-slice-churn`

## A3 Gate Result

Acceptance picked before the closure run: 100k live rows, `nlists in {32,64}`,
10 delete/VACUUM/refill cycles, and final index growth below 10%.

The final same-slice run passes:

| nlists | cycle 1 index bytes | cycle 10 index bytes | growth |
|---:|---:|---:|---:|
| 32 | 9,060,352 | 9,060,352 | 0.0% |
| 64 | 9,166,848 | 9,166,848 | 0.0% |

Each cycle deletes 25k rows, runs `VACUUM (ANALYZE)`, then refills 25k rows,
returning to 100k live rows. The final page-ownership diagnostic reports no
tombstones and no cross-list posting pages:

| nlists | posting blocks | posting tuples | heap TID refs | deleted postings | cross-list blocks | mixed blocks |
|---:|---:|---:|---:|---:|---:|---:|
| 32 | 1047 | 100000 | 100000 | 0 | 0 | 0 |
| 64 | 1061 | 100000 | 100000 | 0 | 0 | 0 |

## Important Limitation

The rotating-window diagnostic is still not closed. With the same 100k rows and
10 cycles but a rotating delete window, n32 grew from 9,043,968 to 11,198,464
bytes and n64 grew from 9,175,040 to 11,829,248 bytes. Final diagnostics showed
100k live postings and zero tombstones, but extensive cross-list page ownership.

Interpretation: tuple-level vacuum replacement is working, but the current
single `head_block..tail_block` directory representation still widens ranges
when inserts exhaust a list's original contiguous capacity. The next A3 slice
should either add list-local extent tracking or reserve/build slack inside each
list's original range before making a broader rotating-window churn claim.

## Validation

- `cargo test -p ecaz-cli ivf_vacuum_scale`
- `cargo test -p ecaz --lib am::ec_ivf::insert::tests`
- `cargo pgrx test pg18 test_ec_ivf_insert_rejects_duplicate_heap_tid`
- `git diff --check`

## Artifacts

- `artifacts/ivf_a3_100k_same_slice_final.log`
- `artifacts/page_ownership_same_slice_final.log`
- `artifacts/ivf_a3_100k_sustained_churn.log`
- `artifacts/page_ownership_after_10cycle.log`
- `artifacts/manifest.md`
