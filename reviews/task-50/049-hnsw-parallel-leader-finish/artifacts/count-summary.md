# Count Summary

- head SHA: `d01929888313aed45fefddc1330cfb1459996ced`
- previous SHA: `41ddf191`
- task bucket: `reviews/task-50/049-hnsw-parallel-leader-finish/`
- timestamp: `2026-05-20`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_hnsw/build_parallel.rs` | 139 | 133 | -6 |
| `src/` total | 2290 | 2284 | -6 |

## Disposition

- Made HNSW parallel build leader `finish` methods safe because the leader object owns the PostgreSQL parallel context and cleanup consumes `self`.
- Made graph leader `wait_for_workers` safe because the leader owns the worker accounting arrays and exposes no raw pointers to callers.
- Kept `begin` and raw PostgreSQL parallel-context construction paths unsafe.
