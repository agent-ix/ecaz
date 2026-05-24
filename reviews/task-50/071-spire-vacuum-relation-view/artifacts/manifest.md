# Task 50 SPIRE Vacuum Relation View Artifacts

- head SHA: `160a8fcde6cf287604cf739401c35c962b639349`
- task bucket: `reviews/task-50/071-spire-vacuum-relation-view/`
- timestamp: `2026-05-20T12:46:40-07:00`
- program / wave: P1 callback boundary, P2 PostgreSQL handle views, P3 page/WAL/publish contracts / Wave 2 SPIRE production fanout
- touched file: `src/am/ec_spire/vacuum/mod.rs`
- storage format / rerank mode: not applicable
- isolated one-index-per-table vs shared-table surface: not applicable; compile/static unsafe burndown packet

## Artifacts

- `count-summary.md`
  - source of truth for before/after direct unsafe counts cited by `request.md`
  - result: `src/am/ec_spire/vacuum/mod.rs` 31 -> 26, `src/` total 2074 -> 2069

- `code-diff.patch`
  - command: `git diff --no-color HEAD^ HEAD -- src/am/ec_spire/vacuum/mod.rs`
  - result: introduces a private SPIRE vacuum live relation view and routes repeated relation helper calls through it

- `git-diff-check.log`
  - command: `git diff --check HEAD^ HEAD`
  - result: passed

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`

- `src-unsafe-block-count-after.log`
  - command: `make unsafe-block-count`
  - result: current `src/` direct unsafe inventory; `src/am/ec_spire/vacuum/mod.rs` has 26 rows after this packet

- `unsafe-ledger-after.jsonl`
  - command: `UNSAFE_LEDGER=reviews/task-50/071-spire-vacuum-relation-view/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/071-spire-vacuum-relation-view make unsafe-ledger`
  - result: wrote 2069 current `src/` unsafe ledger rows

- `unsafe-ledger-generate.log`
  - command log for generating `unsafe-ledger-after.jsonl`
  - result: passed

- `unsafe-ledger-check.log`
  - command: `UNSAFE_LEDGER=reviews/task-50/071-spire-vacuum-relation-view/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
  - result: `ledger covers 2069 current unsafe rows`
