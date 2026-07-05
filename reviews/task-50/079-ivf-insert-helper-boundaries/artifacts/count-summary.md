# Count Summary

- code commit: `0a046d51e7c107a253c5c9f1d0e083c8d807225a`
- task bucket: `reviews/task-50/079-ivf-insert-helper-boundaries/`
- touched production file: `src/am/ec_ivf/insert.rs`
- program coverage: P2 PostgreSQL relation helpers, P3 IVF page/write contract, P6 IVF/RaBitQ payload flow
- timestamp: `2026-05-20T13:16:29-07:00`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/insert.rs` | 20 | 14 | -6 |
| `src/` total | 2036 | 2030 | -6 |

## Notes

- Makes module-private IVF insert helpers safe to call when they already own the live relation and metadata contracts internally.
- Removes redundant caller-side unsafe around empty bootstrap locking, bootstrap flush, trained insert, centroid loading, directory lookup, and debug duplicate-TID validation.
- Leaves the actual relation locks, metadata reads, page writes, PQ model load, and page-chain scans inside the helper bodies that own those contracts.
