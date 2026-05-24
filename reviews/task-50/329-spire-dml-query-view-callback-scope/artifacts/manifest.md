# Task 50 Packet 329 Artifact Manifest

- head SHA: `6e957292e6b1d7ecf2cd866d13cdcabdcc04d631`
- parent SHA: `2f2a098ad7ff07e8fd7e80fb3fc4895e33181068`
- task bucket: `reviews/task-50/`
- packet path: `reviews/task-50/329-spire-dml-query-view-callback-scope/`
- timestamp: `2026-05-21T15:32:00-07:00`
- lane: Task 50 unsafe burndown, P11 planner/node/list views and P13 test helper cleanup
- fixture / storage format / rerank mode: not applicable
- table surface: not applicable; query-view lifetime and test helper refactor

## Artifacts

- `unsafe-counts-before-after.log`
  - command: before/after `rg -n 'unsafe\s*\{'` counts for touched files plus current `src/` total
  - key lines:
    - `src/am/ec_spire/dml_frontdoor/mod.rs`: `37 -> 36`
    - `src/lib.rs`: `21 -> 21`
    - `src/tests/dml_frontdoor.rs`: `8 -> 4`
    - current `src/` total: `1349`
- `touched-file-unsafe-lines-after.log`
  - command: `rg -n 'unsafe\s*\{' src/am/ec_spire/dml_frontdoor/mod.rs src/lib.rs src/tests/dml_frontdoor.rs`
  - result: after-slice direct unsafe locations for the touched files
- `unsafe-ledger-after.jsonl`
  - command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/329-spire-dml-query-view-callback-scope/artifacts/unsafe-ledger-after.jsonl --packet reviews/task-50/329-spire-dml-query-view-callback-scope src`
  - result: wrote `1349` current `src` ledger rows
- `unsafe-ledger-generate.log`
  - command log for the ledger generation above
- `unsafe-ledger-check.log`
  - command: `make UNSAFE_LEDGER=reviews/task-50/329-spire-dml-query-view-callback-scope/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
  - result: passed; `ledger covers 1349 current unsafe rows`
- `git-diff-check.log`
  - command: `git diff --check HEAD~1..HEAD`
  - result: passed
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed
  - note: existing unused SPIRE DML re-export warning remains in `src/am/mod.rs`
- `cargo-test-dml-frontdoor-blocked.log`
  - command: `cargo test dml_frontdoor --no-default-features --features pg18,bench`
  - result: blocked before running tests by `undefined symbol: pg_re_throw`
- `cargo-pgrx-test-dml-primitive-plan-pg18-blocked.log`
  - command: `cargo pgrx test pg18 test_ec_spire_dml_frontdoor_primitive_plan_from_decision`
  - result: blocked before running the test by `undefined symbol: CacheRegisterRelcacheCallback`
