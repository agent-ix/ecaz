# Count Summary

- code commit: `623d3411843c5d1931b51594bfe9180bc006d9a7`
- task bucket: `reviews/task-50/077-ivf-scan-helper-boundaries/`
- touched production file: `src/am/ec_ivf/scan.rs`
- program coverage: P2 PostgreSQL scan handle views, P5 heap source/slot contracts, P6 IVF/RaBitQ scan payload flow
- timestamp: `2026-05-20T13:10:26-07:00`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/scan.rs` | 46 | 41 | -5 |
| `src/` total | 2048 | 2043 | -5 |

## Notes

- Makes IVF scan heap relation, snapshot, selected-probe plan, and directory loading helpers safe to call from their module-private orchestration sites.
- Leaves the actual scan descriptor, snapshot, active snapshot, and directory page unsafe blocks inside the helper bodies that own those pointer contracts.
- Keeps heap rerank setup, selected-list validation, and directory-chain ordering checks unchanged.
