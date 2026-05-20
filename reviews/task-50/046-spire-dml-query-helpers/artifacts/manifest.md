# Task 50 Packet 046 Artifacts

- head SHA: `ec6cacf7929d3e1296c78f321bbccc34bb75fcb7`
- previous SHA: `f12eedda`
- task bucket: `reviews/task-50/046-spire-dml-query-helpers/`
- timestamp: `2026-05-20`
- code commit: `ec6cacf7 Make SPIRE DML query helpers safe`
- contract programs: P2 PostgreSQL Handle Views, P11 Planner / Node / List Views
- wave / tranche: Wave 2, SPIRE DML frontdoor expression/query helper fanout
- benchmarks: not run; this packet removes redundant caller-side unsafe wrappers and does not change candidate ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Artifacts

- `count-summary.md`
  - command basis:
    - `git show HEAD^:src/am/ec_spire/dml_frontdoor/mod.rs | rg "unsafe\s*\{" | wc -l`
    - `git show HEAD^:src/lib.rs | rg "unsafe\s*\{" | wc -l`
    - `git show HEAD^:src/tests/dml_frontdoor.rs | rg "unsafe\s*\{" | wc -l`
    - `make unsafe-block-count | awk '{s += $1} END {print s}'`
  - result: `src/` total `2344 -> 2324`; touched file deltas `91 -> 76`, `42 -> 37`, `5 -> 5`

- `src-unsafe-block-count-after.log`
  - command: `make unsafe-block-count`
  - result: after-count inventory with `2324` total direct unsafe blocks under `src/`

- `unsafe-ledger-after.jsonl`
  - command: `UNSAFE_LEDGER=reviews/task-50/046-spire-dml-query-helpers/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/046-spire-dml-query-helpers make unsafe-ledger`
  - result: `2324` current unsafe ledger rows

- `unsafe-ledger-generate.log`
  - command log for the ledger generation above

- `unsafe-ledger-check.log`
  - command: `UNSAFE_LEDGER=reviews/task-50/046-spire-dml-query-helpers/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
  - result: `ledger covers 2324 current unsafe rows`

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed; only the known pre-existing `src/am/mod.rs` unused import warning remains

- `code-diff.patch`
  - command: `git diff --no-color HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor/mod.rs src/lib.rs src/tests/dml_frontdoor.rs`
  - result: packet-local code diff for reviewer inspection
