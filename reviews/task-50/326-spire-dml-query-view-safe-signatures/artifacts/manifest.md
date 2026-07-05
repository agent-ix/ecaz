# Task 50 Packet 326 Artifact Manifest

- Head SHA: `a38e0cfaf264a9bf86784124a83b2c5f5a3e45bc`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/326-spire-dml-query-view-safe-signatures`
- Timestamp: `2026-05-21T21:47:47Z`
- Lane: SPIRE unsafe burndown
- Fixture / storage format / rerank mode: not applicable
- Surface: code review packet; no benchmark matrix
- Isolation: not applicable; no table/index benchmark surface

## Artifacts

### `git-diff-check.log`

- Command: `git diff --check HEAD~1..HEAD`
- Result: passed

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Note: emitted the pre-existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`.

### `no-unsafe-dml-query-shape-signatures.log`

- Command: `rg -n "unsafe fn dml_frontdoor_replacement_decision_catalog_row|unsafe fn dml_frontdoor_primitive_plan_expr_catalog_row|unsafe fn classify_dml_frontdoor_query|unsafe fn dml_frontdoor_target_relation_oid|unsafe \\{[[:space:]]*(am::spire_)?dml_frontdoor_replacement_decision_catalog_row|unsafe \\{[[:space:]]*am::spire_classify_dml_frontdoor_query|unsafe \\{[[:space:]]*am::spire_dml_frontdoor_target_relation_oid|unsafe \\{[[:space:]]*am::spire_dml_frontdoor_primitive_plan_expr_catalog_row" src/am/ec_spire/dml_frontdoor src/lib.rs src/tests/dml_frontdoor.rs`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1937`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/326-spire-dml-query-view-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/326-spire-dml-query-view-safe-signatures unsafe-ledger`
- Key result: `wrote 1356 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/326-spire-dml-query-view-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1356 current unsafe rows`
