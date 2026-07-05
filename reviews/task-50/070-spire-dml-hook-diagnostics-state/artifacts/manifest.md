# Task 50 SPIRE DML Hook Diagnostics State Artifacts

- head SHA: `d42efe27e9ed7b6b5f499e9d3f68adee07a48d66`
- task bucket: `reviews/task-50/070-spire-dml-hook-diagnostics-state/`
- timestamp: `2026-05-20T12:41:26-07:00`
- program / wave: P1 FFI and callback boundary contracts / Wave 2 SPIRE DML frontdoor
- touched file: `src/am/ec_spire/dml_frontdoor/mod.rs`
- storage format / rerank mode: not applicable
- isolated one-index-per-table vs shared-table surface: not applicable; compile/static unsafe burndown packet

## Artifacts

- `count-summary.md`
  - source of truth for before/after direct unsafe counts cited by `request.md`
  - result: `src/am/ec_spire/dml_frontdoor/mod.rs` 32 -> 30, `src/` total 2076 -> 2074

- `code-diff.patch`
  - command: `git diff --no-color HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor/mod.rs`
  - result: moves hook diagnostic state out of `static mut` into a safe mutex snapshot

- `git-diff-check.log`
  - command: `git diff --check HEAD^ HEAD`
  - result: passed

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed with the pre-existing unused SPIRE DML import warning in `src/am/mod.rs`

- `src-unsafe-block-count-after.log`
  - command: `make unsafe-block-count`
  - result: current `src/` direct unsafe inventory; `src/am/ec_spire/dml_frontdoor/mod.rs` has 30 rows after this packet

- `unsafe-ledger-after.jsonl`
  - command: `UNSAFE_LEDGER=reviews/task-50/070-spire-dml-hook-diagnostics-state/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/070-spire-dml-hook-diagnostics-state make unsafe-ledger`
  - result: wrote 2074 current `src/` unsafe ledger rows

- `unsafe-ledger-generate.log`
  - command log for generating `unsafe-ledger-after.jsonl`
  - result: passed

- `unsafe-ledger-check.log`
  - command: `UNSAFE_LEDGER=reviews/task-50/070-spire-dml-hook-diagnostics-state/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
  - result: `ledger covers 2074 current unsafe rows`
