# Artifact Manifest

Head SHA: `6d38b4e1ff6848517151b7f5ab814d677e50f920`

Task bucket: `reviews/task-50/`

Packet: `reviews/task-50/034-hnsw-heap-slot-reader-rollout/`

Timestamp: `2026-05-20T09:35:14-07:00`

Lane: Task 50 unsafe burndown, P5 heap source / tuple slot / snapshot / scorer
contract rollout.

Storage format / rerank mode: code-structure packet; HNSW insert source-column
scoring, vacuum source-backed repair scoring, and grouped heap-f32 scan rerank
paths touched.

Isolation surface: local source-tree validation only; no database benchmark
fixtures or shared-table benchmark surfaces used.

## Artifacts

### `before-counts.log`

- Command:
  `for f in ...; do git show 09de24d814af405b743bf3f00f036843b2688eda:"$f" | rg -o "unsafe\\s*\\{" | wc -l; done`
- Purpose: direct unsafe counts for each touched file at the pre-slice code
  baseline.
- Key lines:
  - `src/am/ec_hnsw/insert.rs 93`
  - `src/am/ec_hnsw/vacuum.rs 68`
  - `src/am/ec_hnsw/scan.rs 158`
  - `src/am/ec_hnsw/source.rs 53`
  - `src/am/common/heap_slot.rs 13`

### `after-counts.log`

- Command: `make unsafe-block-count`
- Purpose: current direct unsafe counts for all `src/` files after the code
  slice.
- Key lines:
  - `157 src/am/ec_hnsw/scan.rs`
  - `86 src/am/ec_hnsw/insert.rs`
  - `66 src/am/ec_hnsw/vacuum.rs`
  - `53 src/am/ec_hnsw/source.rs`
  - `13 src/am/common/heap_slot.rs`
- Aggregate: `131` files, `2432` direct unsafe blocks.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Purpose: PG18-focused compile validation for touched HNSW code.
- Result: passed.
- Known unrelated warning: existing unused imports in `src/am/mod.rs`.

### `unsafe-ledger-after.jsonl`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/034-hnsw-heap-slot-reader-rollout/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/034-hnsw-heap-slot-reader-rollout make unsafe-ledger`
- Purpose: packet-local current unsafe ledger after this slice.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Purpose: captured ledger generation command output.

### `unsafe-ledger-check.log`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/034-hnsw-heap-slot-reader-rollout/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
- Purpose: prove the packet-local ledger covers the current `src/` unsafe
  inventory.
- Key line: `ledger covers 2432 current unsafe rows`
