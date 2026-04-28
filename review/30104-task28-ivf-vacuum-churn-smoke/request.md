# Task 28 IVF vacuum churn smoke

## Scope

This packet records a small local PG18 churn smoke after commit `c45e22c5`.

It is intentionally a diagnostic packet, not an A3 completion claim.

Fixture:

- Synthetic 4D `ecvector` tables.
- 5,000 initial rows per table.
- Three IVF indexes with identical shape except `nlists in {8, 32, 64}`.
- Churn: delete rows `2501..5000`, run `VACUUM (ANALYZE)`, then insert 2,500 new rows to return to 5,000 live rows.
- SQL runner: `ecaz-cli dev sql`.
- Raw artifacts: `artifacts/ivf_vacuum_churn_smoke.sql`, `artifacts/ivf_vacuum_churn_smoke.log`.

## Results

Index size after build:

- nlists=8: `448 kB`
- nlists=32: `448 kB`
- nlists=64: `448 kB`

Index size after delete + VACUUM:

- nlists=8: `448 kB`
- nlists=32: `448 kB`
- nlists=64: `448 kB`

Index size after refill to 5,000 live rows:

- nlists=8: `648 kB`
- nlists=32: `648 kB`
- nlists=64: `632 kB`

VACUUM wall times from psql timing:

- nlists=8: `16.109 ms`
- nlists=32: `20.567 ms`
- nlists=64: `45.931 ms`

Live rows after refill:

- nlists=8: `5000`
- nlists=32: `5000`
- nlists=64: `5000`

## Interpretation

The page-local compaction checkpoint is not enough for A3's index-size convergence requirement. The index does not grow during delete + VACUUM, but it grows when rows are reinserted to the original live count.

Likely cause: vacuum repairs the list tail backward to the last live posting block, which makes trailing empty posting pages unreachable from that list. Live insert appends at the current list tail or relation end, so it does not reuse those detached trailing pages.

## Next Work

A3 still needs a real free-page/trailing-page reuse strategy. Two viable directions:

- Keep per-list reusable block metadata so inserts can reuse vacuum-emptied blocks before extending the relation.
- Compact/rewrite lists so live postings move into earlier blocks and trailing relation pages become truncatable.

This packet also does not satisfy A2's required 1M-row peak-memory measurement. A2's code shape is streaming, but the large-scale memory packet still needs to run separately.
