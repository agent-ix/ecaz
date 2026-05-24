# Artifact Manifest

Head SHA: `6c5b39cc9f8b316b11cb3b889a2e3506c9368e53`

Task bucket: `reviews/task-50/`

Packet: `reviews/task-50/038-spire-relation-store-prefetch/`

Timestamp: `2026-05-20T09:54:19-07:00`

Lane: Task 50 unsafe burndown, P9 read-stream and prefetch contract.

Storage format / rerank mode: code-structure packet; SPIRE relation-backed
object store prefetch paths touched.

Isolation surface: local source-tree validation only; no database benchmark
fixtures or shared-table benchmark surfaces used.

## Artifacts

### `before-counts.log`

- Command:
  `for f in ...; do git show 5b8cb1fc:"$f" | rg -o "unsafe\\s*\\{" | wc -l; done`
- Purpose: direct unsafe counts for each touched file at the pre-slice code
  baseline.
- Key lines:
  - `src/am/ec_spire/storage.rs 0`
  - `src/am/ec_spire/storage/relation_store.rs 52`

### `after-counts.log`

- Command: `make unsafe-block-count`
- Purpose: current direct unsafe counts for all `src/` files after the code
  slice.
- Key lines:
  - `38 src/am/ec_spire/storage/relation_store.rs`
- Aggregate: `131` files, `2392` direct unsafe blocks.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Purpose: PG18-focused compile validation for touched SPIRE relation-store
  code.
- Result: passed.
- Known unrelated warning: existing unused imports in `src/am/mod.rs`.

### `unsafe-ledger-after.jsonl`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/038-spire-relation-store-prefetch/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/038-spire-relation-store-prefetch make unsafe-ledger`
- Purpose: packet-local current unsafe ledger after this slice.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Purpose: captured ledger generation command output.

### `unsafe-ledger-check.log`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/038-spire-relation-store-prefetch/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
- Purpose: prove the packet-local ledger covers the current `src/` unsafe
  inventory.
- Key line: `ledger covers 2392 current unsafe rows`
