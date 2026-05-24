# Task 50 Packet 327 Artifact Manifest

- Head SHA: `e337e33b64e084ec0529ce06a2b4b5ccb48742ab`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/327-spire-dml-primitive-param-view-safe-signatures`
- Timestamp: `2026-05-21T21:52:44Z`
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

### `no-unsafe-dml-primitive-param-signatures.log`

- Command: `rg -n "unsafe fn dml_frontdoor_primitive_plan_pk_value_bytes|unsafe fn dml_frontdoor_primitive_invocation_from_plan|unsafe \\{[[:space:]]*dml_frontdoor_primitive_plan_pk_value_bytes|unsafe \\{[[:space:]]*am::spire_dml_frontdoor_primitive_plan_pk_value_bytes|unsafe \\{[[:space:]]*am::spire_dml_frontdoor_primitive_invocation_from_plan" src/am/ec_spire/dml_frontdoor src/tests/dml_frontdoor.rs`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1935`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/327-spire-dml-primitive-param-view-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/327-spire-dml-primitive-param-view-safe-signatures unsafe-ledger`
- Key result: `wrote 1355 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/327-spire-dml-primitive-param-view-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1355 current unsafe rows`
