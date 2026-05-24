# Artifact Manifest

Head SHA: `278c72a77d69c2003ad3559b18ac9cd85aeb3fdc`

Task bucket: `reviews/task-50/`

Packet: `reviews/task-50/037-read-stream-prefetch-helper/`

Timestamp: `2026-05-20T09:50:00-07:00`

Lane: Task 50 unsafe burndown, P9 read-stream and prefetch contract.

Storage format / rerank mode: code-structure packet; IVF heap-f32 rerank,
SPIRE heap rerank, and DiskANN heap rerank prefetch paths touched.

Isolation surface: local source-tree validation only; no database benchmark
fixtures or shared-table benchmark surfaces used.

## Artifacts

### `before-counts.log`

- Command:
  `for f in ...; do git show 70af3bf4:"$f" | rg -o "unsafe\\s*\\{" | wc -l; done`
- Purpose: direct unsafe counts for each touched file at the pre-slice code
  baseline.
- Key lines:
  - `src/am/common/stream.rs 4`
  - `src/am/ec_ivf/scan.rs 68`
  - `src/am/ec_spire/scan/relation.rs 25`
  - `src/am/ec_diskann/routine.rs 58`

### `after-counts.log`

- Command: `make unsafe-block-count`
- Purpose: current direct unsafe counts for all `src/` files after the code
  slice.
- Key lines:
  - `62 src/am/ec_ivf/scan.rs`
  - `56 src/am/ec_diskann/routine.rs`
  - `18 src/am/ec_spire/scan/relation.rs`
  - `9 src/am/common/stream.rs`
- Aggregate: `131` files, `2406` direct unsafe blocks.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Purpose: PG18-focused compile validation for touched stream-prefetch code.
- Result: passed.
- Known unrelated warning: existing unused imports in `src/am/mod.rs`.

### `unsafe-ledger-after.jsonl`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/037-read-stream-prefetch-helper/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/037-read-stream-prefetch-helper make unsafe-ledger`
- Purpose: packet-local current unsafe ledger after this slice.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Purpose: captured ledger generation command output.

### `unsafe-ledger-check.log`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/037-read-stream-prefetch-helper/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
- Purpose: prove the packet-local ledger covers the current `src/` unsafe
  inventory.
- Key line: `ledger covers 2406 current unsafe rows`
