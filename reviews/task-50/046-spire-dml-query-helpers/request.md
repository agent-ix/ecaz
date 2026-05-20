# Task 50 Packet 046: SPIRE DML Query Helpers

This packet continues the comprehensive unsafe burndown plan under P2/P11 for SPIRE DML frontdoor query and planner helper APIs.

## Change

- Converted SPIRE DML query/classification helper APIs from `unsafe fn` to safe functions where the helpers already keep PostgreSQL pointer dereferences internal and return owned Rust data:
  - `dml_frontdoor_plan_tree_replacement_expr`
  - `dml_frontdoor_observe_planner_query`
  - `dml_frontdoor_replacement_decision_catalog_row`
  - `dml_frontdoor_primitive_plan_expr_catalog_row`
  - `dml_frontdoor_classify_query_with_catalog_context`
  - `dml_frontdoor_classify_query_with_relation`
  - `dml_frontdoor_query_detail_with_relation`
  - `dml_frontdoor_query_detail_from_baserel`
  - `classify_dml_frontdoor_query`
  - `dml_frontdoor_target_relation_oid`
- Removed redundant caller-side unsafe wrappers in the planner hook and SQL diagnostic wrappers.
- Updated DML frontdoor tests so only genuinely unsafe test helpers remain behind the `dml_frontdoor_checked!` macro.

No new helper was introduced. This packet deletes caller-side unsafe wrappers around already-localized PostgreSQL query-tree reads.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 91 | 76 | -15 |
| `src/lib.rs` | 42 | 37 | -5 |
| `src/tests/dml_frontdoor.rs` | 5 | 5 | 0 |
| `src/` total | 2344 | 2324 | -20 |

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2324` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning also present in the previous packet.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. The current closeout audit still shows `2324` direct unsafe blocks under `src/`; the closeout gate from packet 030 requires every row to be removed or residual-registered.
