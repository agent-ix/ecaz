# Artifact Manifest

Head SHA: `4b17638f85e19deff77f3cd3067737f66f2582c7`

Task bucket: `reviews/task-50/`

Packet: `reviews/task-50/039-ivf-read-stream-visitor/`

Timestamp: `2026-05-20T09:59:01-07:00`

Lane: Task 50 unsafe burndown, P9 read-stream and prefetch contract.

Storage format / rerank mode: code-structure packet; IVF posting-page
read-stream traversal touched.

Isolation surface: local source-tree validation only; no database benchmark
fixtures or shared-table benchmark surfaces used.

## Artifacts

### `before-counts.log`

- Command:
  `for f in ...; do git show 5a54fcf6:"$f" | rg -o "unsafe\\s*\\{" | wc -l; done`
- Purpose: direct unsafe counts for each touched file at the pre-slice code
  baseline.
- Key lines:
  - `src/am/common/stream.rs 9`
  - `src/am/ec_ivf/page.rs 90`

### `after-counts.log`

- Command: `make unsafe-block-count`
- Purpose: current direct unsafe counts for all `src/` files after the code
  slice.
- Key lines:
  - `72 src/am/ec_ivf/page.rs`
  - `16 src/am/common/stream.rs`
- Aggregate: `131` files, `2381` direct unsafe blocks.

### `cargo-check-pg18-bench.log`

- Command:
  `cargo check --all-targets --no-default-features --features pg18,bench`
- Purpose: PG18-focused compile validation for touched IVF/common stream code.
- Result: passed.
- Known unrelated warning: existing unused imports in `src/am/mod.rs`.

### `unsafe-ledger-after.jsonl`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/039-ivf-read-stream-visitor/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/039-ivf-read-stream-visitor make unsafe-ledger`
- Purpose: packet-local current unsafe ledger after this slice.

### `unsafe-ledger-generate.log`

- Command: same as `unsafe-ledger-after.jsonl`.
- Purpose: captured ledger generation command output.

### `unsafe-ledger-check.log`

- Command:
  `env UNSAFE_LEDGER=reviews/task-50/039-ivf-read-stream-visitor/artifacts/unsafe-ledger-after.jsonl make unsafe-ledger-check`
- Purpose: prove the packet-local ledger covers the current `src/` unsafe
  inventory.
- Key line: `ledger covers 2381 current unsafe rows`
