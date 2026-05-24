# Count Summary

- code commit: `38822699757cde7a571fc483bb98d380cdfaefad`
- task bucket: `reviews/task-50/080-ivf-build-helper-boundaries/`
- touched production files:
  - `src/am/ec_ivf/build.rs`
  - `src/am/ec_ivf/insert.rs`
- program coverage: P3 IVF page/write contract, P6 IVF/RaBitQ datum and payload flow
- timestamp: `2026-05-20T13:19:57-07:00`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/build.rs` | 21 | 18 | -3 |
| `src/am/ec_ivf/insert.rs` | 14 | 13 | -1 |
| `src/` total | 2030 | 2026 | -4 |

## Notes

- Makes build-plan flush, data-page writing, detoast conversion, and indexed-vector type resolution helpers safe to call.
- Removes the remaining caller-side flush wrapper from IVF empty insert bootstrap.
- Leaves the actual PostgreSQL page/WAL/datum/type unsafe in helper bodies that own those contracts.
