# Task 28 IVF Vacuum Same-Distribution Replacement Smoke

## Scope

This packet checks the range-reuse vacuum/insert path on a controlled same-distribution replacement fixture. The initial 5k rows are two copies of a 2500-row vector population; the test deletes one copy, vacuums, and inserts another copy of the same population.

Head under test: `d54e1f40`.

## Result

Same-distribution replacement fully reused space for nlists=8, but nlists=32 and nlists=64 still grew after refill.

Index size by phase:

| surface | after build | after delete vacuum | after refill |
| --- | ---: | ---: | ---: |
| nlists=8 | 448 kB | 448 kB | 448 kB |
| nlists=32 | 448 kB | 448 kB | 464 kB |
| nlists=64 | 448 kB | 448 kB | 536 kB |

VACUUM times in this run:

- nlists=8: 17.023 ms
- nlists=32: 22.669 ms
- nlists=64: 65.239 ms

## Interpretation

The nlists=8 result proves the new path can reuse vacuumed posting pages through a full delete/vacuum/refill cycle without relation growth. The nlists=32 and nlists=64 growth means the vacuum item is not fully closed yet. Remaining likely work is finer-grained page reuse for mixed-list/mixed-tuple pages and/or explicit free-space metadata instead of relying only on list-range tail-backward search.

## Artifacts

- `artifacts/ivf_vacuum_replacement_reuse_smoke.sql`
- `artifacts/ivf_vacuum_replacement_reuse_smoke.log`
- `artifacts/manifest.md`
