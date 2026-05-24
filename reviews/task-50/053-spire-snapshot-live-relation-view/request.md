# Task 50 Packet 053: SPIRE Snapshot Live Relation View

This packet continues the accepted packet 030 comprehensive unsafe burndown plan. It advances P2 for SPIRE coordinator diagnostics by rolling more snapshot paths through the existing `SpireLiveIndexRelation` view.

## Change

- Extended `SpireLiveIndexRelation` with safe local methods for relation OID, relation object store open, and object tuple scanning.
- Replaced repeated direct unsafe blocks in:
  - relation storage snapshot root/manifest/object-store reads
  - epoch snapshot tuple scanning
  - physical cleanup manifest scanning and protected-directory object-store opens
  - leaf snapshot root/manifest/object-store reads
- Kept the raw `pg_sys::Relation` contract at `live_index_relation` / `SpireLiveIndexRelation::new`.

This is a handle-view rollout, not a public API relaxation: raw relation entry points remain unsafe while repeated diagnostic internals use the typed live-index view.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/snapshots.rs` | 42 | 37 | -5 |
| `src/` total | 2233 | 2228 | -5 |

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2228` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `git diff --check HEAD^ HEAD` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. Current closeout audit count is `2228` direct unsafe blocks under `src/`; packet 030 still requires every unsafe row to be removed or residual-registered.

