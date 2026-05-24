# Artifact Manifest

Head SHA: `8ca545d3c4185a5a9fb5b299b4a1db4ef593f953`

Task bucket: `reviews/task-50/`

Packet: `reviews/task-50/035-hnsw-dead-heap-helper-removal/`

Timestamp: `2026-05-20T09:40:10-07:00`

Lane: Task 50 unsafe burndown, P5 heap source / tuple slot / snapshot / scorer
contract cleanup.

Storage format / rerank mode: code-structure packet; obsolete HNSW helper API
removal only.

Isolation surface: local source-tree validation only; no database benchmark
fixtures or shared-table benchmark surfaces used.

## Artifacts

### `before-counts.log`

- Command:
  `for f in ...; do git show 7384b944c4603b9a21522ddcffa4ef45222127ff:"$f" | rg -o "unsafe\\s*\\{" | wc -l; done`
- Purpose: direct unsafe counts for each touched file at the pre-slice code
  baseline.
- Key lines:
  - `src/am/ec_hnsw/source.rs 53`
  - `src/am/common/heap_slot.rs 13`

### `after-counts.log`

- Command: `make unsafe-block-count`
- Purpose: current direct unsafe counts for all `src/` files after the code
  slice.
- Key lines:
  - `47 src/am/ec_hnsw/source.rs`
  - `13 src/am/common/heap_slot.rs`
- Aggregate: `131` files, `2426` direct unsafe blocks.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Purpose: PG18-focused compile validation for touched HNSW source helper code.
- Result: passed.
- Known unrelated warning: existing unused imports in `src/am/mod.rs`.

### `unsafe-ledger-after.jsonl`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/035-hnsw-dead-heap-helper-removal/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/035-hnsw-dead-heap-helper-removal make unsafe-ledger`
- Purpose: packet-local current unsafe ledger after this slice.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Purpose: captured ledger generation command output.

### `unsafe-ledger-check.log`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/035-hnsw-dead-heap-helper-removal/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
- Purpose: prove the packet-local ledger covers the current `src/` unsafe
  inventory.
- Key line: `ledger covers 2426 current unsafe rows`
