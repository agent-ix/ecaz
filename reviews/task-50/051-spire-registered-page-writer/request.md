# Task 50 Packet 051: SPIRE Registered Page Writer

This packet continues the accepted packet 030 comprehensive unsafe burndown plan. It advances P3 for the SPIRE production page-write path by centralizing WAL-registered page mutation behind a local wrapper.

## Change

- Added `SpireRegisteredPage` inside `src/am/ec_spire/page.rs`.
- Moved registered-page operations into wrapper methods:
  - page initialization
  - special-area copy
  - free-space read
  - FSM free-space record
  - page item append
  - max-offset read
  - no-compact tuple delete
- Replaced caller-side unsafe blocks in metadata initialization, object tuple delete, existing-block append, and new-block append paths.

The wrapper is intentionally local to the SPIRE page module. It does not make raw relation entry points safe; it only removes repeated direct `pg_sys` page mutations after the caller has already acquired a locked buffer and registered it with `GenericXLogTxn::register_locked_buffer_full_image`.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/page.rs` | 48 | 38 | -10 |
| `src/` total | 2257 | 2247 | -10 |

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2247` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. Current closeout audit count is `2247` direct unsafe blocks under `src/`; packet 030 still requires every unsafe row to be removed or residual-registered.

