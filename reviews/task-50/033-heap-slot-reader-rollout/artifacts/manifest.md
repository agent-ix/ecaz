# Artifact Manifest

Head SHA: `590c57f61622c09fde10b5154be160d7d91cde00`

Task bucket: `reviews/task-50/`

Packet: `reviews/task-50/033-heap-slot-reader-rollout/`

Timestamp: `2026-05-20T09:28:44-07:00`

Lane: Task 50 unsafe burndown, P5 heap source / tuple slot / snapshot / scorer
contract rollout.

Storage format / rerank mode: code-structure packet; IVF `heap_f32`, SPIRE
exact heap rerank, and DiskANN heap-vector rerank/backlink/vacuum source lookup
call paths touched.

Isolation surface: local source-tree validation only; no database benchmark
fixtures or shared-table benchmark surfaces used.

## Artifacts

### `before-counts.log`

- Command:
  `for f in ...; do git show HEAD^:"$f" | rg -o "unsafe\\s*\\{" | wc -l; done`
- Purpose: direct unsafe counts for each touched file at the pre-slice code
  baseline.
- Key lines:
  - `src/am/common/heap_slot.rs 7`
  - `src/am/ec_hnsw/source.rs 51`
  - `src/am/ec_ivf/scan.rs 69`
  - `src/am/ec_diskann/routine.rs 64`
  - `src/am/ec_diskann/scan_state.rs 20`
  - `src/am/ec_spire/scan/relation.rs 29`
  - `src/am/ec_spire/update/materialization.rs 1`
  - `src/am/ec_spire/coordinator/hierarchy_snapshots.rs 48`

### `after-counts.log`

- Command: `make unsafe-block-count`
- Purpose: current direct unsafe counts for all `src/` files after the code
  slice.
- Key lines:
  - `68 src/am/ec_ivf/scan.rs`
  - `58 src/am/ec_diskann/routine.rs`
  - `53 src/am/ec_hnsw/source.rs`
  - `48 src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
  - `25 src/am/ec_spire/scan/relation.rs`
  - `20 src/am/ec_diskann/scan_state.rs`
  - `13 src/am/common/heap_slot.rs`
  - `1 src/am/ec_spire/update/materialization.rs`
- Aggregate: `131` files, `2442` direct unsafe blocks.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Purpose: PG18-focused compile validation for touched AM code.
- Result: passed.
- Known unrelated warning: existing unused imports in `src/am/mod.rs`.

### `unsafe-ledger-after.jsonl`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/033-heap-slot-reader-rollout/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/033-heap-slot-reader-rollout make unsafe-ledger`
- Purpose: packet-local current unsafe ledger after this slice.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Purpose: captured ledger generation command output.

### `unsafe-ledger-check.log`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/033-heap-slot-reader-rollout/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
- Purpose: prove the packet-local ledger covers the current `src/` unsafe
  inventory.
- Key line: `ledger covers 2442 current unsafe rows`
