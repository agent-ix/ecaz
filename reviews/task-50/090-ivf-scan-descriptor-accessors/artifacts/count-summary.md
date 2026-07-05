# Task 50 Packet 090 Count Summary

Head SHA: `54c9021477ed70b3910c1dddaa2e159b43b9cff7`

Program coverage:

- P2 PostgreSQL handle views
- P5 heap source / tuple slot / snapshot contracts
- P10 scan opaque and raw ownership contracts
- Wave 2 IVF/RaBitQ production fanout

Direct unsafe movement:

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/` total direct unsafe blocks | 1953 | 1940 | -13 |
| `src/am/ec_ivf/scan.rs` | 36 | 23 | -13 |
| `src/` unsafe ledger rows | 1953 | 1940 | -13 |

Notes:

- Added checked IVF scan descriptor, index scan state, index-to-heap OID, active snapshot, and scan opaque accessors.
- Replaced repeated raw `IndexScanDesc`/opaque pointer reads in heap rerank relation/snapshot resolution, EXPLAIN counter extraction, and pg_test debug probes.
- Remaining IVF scan unsafe covers scan-owned palloc/pfree slices, PQ model loading, source-vector heap slot reader construction, debug AM callback wrappers, and residual order-by output pointer reads.

Task 50 is not complete. The regenerated ledger still contains `1940` current `src/` unsafe rows that must be removed or residual-registered.
