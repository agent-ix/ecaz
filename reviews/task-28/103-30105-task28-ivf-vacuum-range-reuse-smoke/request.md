# Task 28 IVF Vacuum Range-Reuse Churn Smoke

## Scope

This packet reruns the 5k synthetic IVF churn fixture from packet 30104 after:

- `746a8eea` preserves non-empty list block ranges during vacuum.
- `d54e1f40` makes insert search the existing list range from tail backward before allocating a new page.

This is a small PG18 smoke, not a production-scale vacuum claim.

## Result

The range-reuse insert path materially reduced refill growth versus packet 30104, but did not eliminate it for this distribution-shifting fixture.

Index size by phase:

| surface | after build | after delete vacuum | after refill |
| --- | ---: | ---: | ---: |
| nlists=8 | 448 kB | 448 kB | 464 kB |
| nlists=32 | 448 kB | 448 kB | 528 kB |
| nlists=64 | 448 kB | 448 kB | 576 kB |

For comparison, packet 30104 refill sizes were 648 kB, 656 kB, and 648 kB respectively on the same fixture before range-search insert reuse.

VACUUM times in this run:

- nlists=8: 17.094 ms
- nlists=32: 21.332 ms
- nlists=64: 27.812 ms

## Interpretation

The fix is directionally correct but not sufficient to close the vacuum/index-size item by itself. The fixture deletes ids 2501-5000 and refills with ids 5001-7500, so the refill population can have a different centroid/list distribution. Packet 30106 adds a same-distribution replacement smoke to separate that fixture effect from remaining storage mechanics.

## Artifacts

- `artifacts/ivf_vacuum_range_reuse_smoke.sql`
- `artifacts/ivf_vacuum_range_reuse_smoke.log`
- `artifacts/manifest.md`
