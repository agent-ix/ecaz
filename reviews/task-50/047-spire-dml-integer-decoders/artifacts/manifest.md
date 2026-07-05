# Task 50 Packet 047 Artifacts

- head SHA: `5afb7b4efa20d92fee088da0e858621772fef814`
- previous SHA: `32d7516a`
- task bucket: `reviews/task-50/047-spire-dml-integer-decoders/`
- timestamp: `2026-05-20`
- code commit: `5afb7b4e Make SPIRE DML integer decoders safe`
- contract programs: P6 Datum / Varlena / Vector Payload Contracts, P11 Planner / Node / List Views
- wave / tranche: Wave 2, SPIRE DML frontdoor value-decoder cleanup
- benchmarks: not run; this packet only removes redundant caller-side unsafe wrappers around integer decoding and does not change candidate ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Artifacts

- `count-summary.md`
  - result: `src/am/ec_spire/dml_frontdoor/mod.rs` `76 -> 73`; `src/` total `2324 -> 2321`

- `src-unsafe-block-count-after.log`
  - command: `make unsafe-block-count`
  - result: after-count inventory with `2321` total direct unsafe blocks under `src/`

- `unsafe-ledger-after.jsonl`
  - command: `UNSAFE_LEDGER=reviews/task-50/047-spire-dml-integer-decoders/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/047-spire-dml-integer-decoders make unsafe-ledger`
  - result: `2321` current unsafe ledger rows

- `unsafe-ledger-generate.log`
  - command log for the ledger generation above

- `unsafe-ledger-check.log`
  - command: `UNSAFE_LEDGER=reviews/task-50/047-spire-dml-integer-decoders/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
  - result: `ledger covers 2321 current unsafe rows`

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed; only the known pre-existing `src/am/mod.rs` unused import warning remains

- `code-diff.patch`
  - command: `git diff --no-color HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor/mod.rs`
  - result: packet-local code diff for reviewer inspection
