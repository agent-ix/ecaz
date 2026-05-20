# Task 50 SPIRE DML Query Pointer Views Artifacts

- head SHA: `ffa04340af2d0b9aef2a4b3a38172fdb84643630`
- task bucket: `reviews/task-50/068-spire-dml-query-pointer-views/`
- timestamp: `2026-05-20T12:32:28-07:00`
- program / wave: P11 planner, node, list, and custom scan views / Wave 2 SPIRE DML frontdoor expression walkers
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`
- storage format / rerank mode: not applicable
- isolated one-index-per-table vs shared-table surface: not applicable; compile/static unsafe burndown packet

## Artifacts

- `count-summary.md`
  - source of truth for before/after direct unsafe counts cited by `request.md`
  - result: `src/am/ec_spire/dml_frontdoor/mod.rs` 37 -> 34, `src/` total 2081 -> 2078

- `code-diff.patch`
  - command: `git diff --no-color HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor/mod.rs`
  - result: consolidates repeated DML frontdoor `Query` pointer reads into a private helper

- `git-diff-check.log`
  - command: `git diff --check HEAD^ HEAD`
  - result: passed

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`

- `src-unsafe-block-count-after.log`
  - command: `make unsafe-block-count`
  - result: current `src/` direct unsafe inventory; `src/am/ec_spire/dml_frontdoor/mod.rs` has 34 rows after this packet

- `unsafe-ledger-after.jsonl`
  - command: `UNSAFE_LEDGER=reviews/task-50/068-spire-dml-query-pointer-views/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/068-spire-dml-query-pointer-views make unsafe-ledger`
  - result: wrote 2078 current `src/` unsafe ledger rows

- `unsafe-ledger-generate.log`
  - command log for generating `unsafe-ledger-after.jsonl`
  - result: passed

- `unsafe-ledger-check.log`
  - command: `UNSAFE_LEDGER=reviews/task-50/068-spire-dml-query-pointer-views/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
  - result: `ledger covers 2078 current unsafe rows`
