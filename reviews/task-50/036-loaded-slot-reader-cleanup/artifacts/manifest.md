# Artifact Manifest

Head SHA: `3165448cac1dec24f333eadaa1b17ff6208362db`

Task bucket: `reviews/task-50/`

Packet: `reviews/task-50/036-loaded-slot-reader-cleanup/`

Timestamp: `2026-05-20T09:45:06-07:00`

Lane: Task 50 unsafe burndown, P5 heap source / tuple slot / snapshot / scorer
contract cleanup.

Storage format / rerank mode: code-structure packet; loaded tuple slot access
only.

Isolation surface: local source-tree validation only; no database benchmark
fixtures or shared-table benchmark surfaces used.

## Artifacts

### `before-counts.log`

- Command:
  `for f in ...; do git show 317ceb08:"$f" | rg -o "unsafe\\s*\\{" | wc -l; done`
- Purpose: direct unsafe counts for each touched file at the pre-slice code
  baseline.
- Key lines:
  - `src/am/common/heap_slot.rs 13`
  - `src/am/ec_hnsw/build.rs 33`
  - `src/am/ec_hnsw/source.rs 47`
  - `src/am/ec_diskann/scan_state.rs 20`

### `after-counts.log`

- Command: `make unsafe-block-count`
- Purpose: current direct unsafe counts for all `src/` files after the code
  slice.
- Key lines:
  - `46 src/am/ec_hnsw/source.rs`
  - `32 src/am/ec_hnsw/build.rs`
  - `18 src/am/ec_diskann/scan_state.rs`
  - `7 src/am/common/heap_slot.rs`
- Aggregate: `131` files, `2416` direct unsafe blocks.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Purpose: PG18-focused compile validation for touched P5 slot reader code.
- Result: passed.
- Known unrelated warning: existing unused imports in `src/am/mod.rs`.

### `unsafe-ledger-after.jsonl`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/036-loaded-slot-reader-cleanup/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/036-loaded-slot-reader-cleanup make unsafe-ledger`
- Purpose: packet-local current unsafe ledger after this slice.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Purpose: captured ledger generation command output.

### `unsafe-ledger-check.log`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/036-loaded-slot-reader-cleanup/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
- Purpose: prove the packet-local ledger covers the current `src/` unsafe
  inventory.
- Key line: `ledger covers 2416 current unsafe rows`
