# Task 50 Heap Slot Contract Seed Artifacts

- head SHA: `23bed0833706d348b786af9f17d0ef1dba8cd80f`
- task bucket: `reviews/task-50/032-heap-slot-contract-seed/`
- timestamp: `2026-05-20`
- contract program: P5 heap source, tuple slot, snapshot, and scorer contracts
- code commit: `23bed083` (`Seed common heap slot unsafe contract`)

## Artifacts

- `before-counts.log`
  - command: `git show HEAD^:<file> | rg -c 'unsafe\s*\{'` for existing touched files; new helper recorded as `0`
  - result:
    - `src/am/ec_spire/scan/relation.rs`: 35
    - `src/am/ec_diskann/scan_state.rs`: 24
    - `src/am/ec_hnsw/source.rs`: 52
    - `src/am/common/heap_slot.rs`: 0

- `after-counts.log`
  - command: `make unsafe-block-count | rg 'src/am/ec_spire/scan/relation.rs|src/am/ec_diskann/scan_state.rs|src/am/ec_hnsw/source.rs|src/am/common/heap_slot.rs'`
  - result:
    - `src/am/ec_spire/scan/relation.rs`: 29
    - `src/am/ec_diskann/scan_state.rs`: 20
    - `src/am/ec_hnsw/source.rs`: 51
    - `src/am/common/heap_slot.rs`: 7
  - net touched-file direct unsafe movement: 111 -> 107

- `unsafe-ledger-after.jsonl`
  - command: `UNSAFE_LEDGER=reviews/task-50/032-heap-slot-contract-seed/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/032-heap-slot-contract-seed make unsafe-ledger`
  - result: 2445 current direct unsafe rows under `src/`

- `unsafe-ledger-check.log`
  - command: `UNSAFE_LEDGER=reviews/task-50/032-heap-slot-contract-seed/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
  - result: `ledger covers 2445 current unsafe rows`

- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed with an existing unused-import warning in `src/am/mod.rs`

- `cargo-clippy-pg18.log`
  - command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - result: failed on existing repository-wide clippy debt unrelated to this slice; first failure is unused imports in `src/am/mod.rs`

- `cargo-clippy-pg18.exit`
  - records the failed clippy status because the log was captured through `tee`

