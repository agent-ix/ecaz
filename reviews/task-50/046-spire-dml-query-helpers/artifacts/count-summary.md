# Count Summary

- head SHA: `ec6cacf7929d3e1296c78f321bbccc34bb75fcb7`
- previous SHA: `f12eedda`
- task bucket: `reviews/task-50/046-spire-dml-query-helpers/`
- timestamp: `2026-05-20`

## Direct Unsafe Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 91 | 76 | -15 |
| `src/lib.rs` | 42 | 37 | -5 |
| `src/tests/dml_frontdoor.rs` | 5 | 5 | 0 |
| `src/` total | 2344 | 2324 | -20 |

## Disposition

- Removed caller-side unsafe wrappers around SPIRE DML query helpers that now expose safe APIs.
- No unsafe was moved into a new helper in this packet.
- Remaining direct unsafe in the touched files is covered by the generated after-ledger artifact for later residual classification.
