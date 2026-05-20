# Count Summary

- code commit: `9b460c2bb78b1b8d447dd699ee6b2850ae88de96`
- task bucket: `reviews/task-50/082-ivf-tuple-chain-reader-boundaries/`
- touched production files:
  - `src/am/ec_ivf/admin.rs`
  - `src/am/ec_ivf/insert.rs`
  - `src/am/ec_ivf/page.rs`
  - `src/am/ec_ivf/quantizer.rs`
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_ivf/vacuum.rs`
- program coverage: P3 IVF page/tuple view contract, P6 IVF/RaBitQ payload flow
- timestamp: `2026-05-20T13:29:27-07:00`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/admin.rs` | 7 | 6 | -1 |
| `src/am/ec_ivf/insert.rs` | 10 | 7 | -3 |
| `src/am/ec_ivf/page.rs` | 35 | 35 | 0 |
| `src/am/ec_ivf/quantizer.rs` | 1 | 0 | -1 |
| `src/am/ec_ivf/scan.rs` | 40 | 37 | -3 |
| `src/am/ec_ivf/vacuum.rs` | 15 | 14 | -1 |
| `src/` total | 2013 | 2004 | -9 |

## Notes

- Makes IVF centroid, list-directory, and PQ-codebook tuple-chain readers safe to call.
- Removes caller-side unsafe from admin, insert, quantizer, scan, and vacuum tuple-chain traversal.
- Keeps the actual buffer/page tuple unsafe in the shared page reader boundary.
