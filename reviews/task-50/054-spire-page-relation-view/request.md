# Task 50 Packet 054: SPIRE Page Relation View

This packet continues the accepted packet 030 comprehensive unsafe burndown plan. It extends the SPIRE page/store tranche by centralizing relation-level page operations behind a local `SpirePageRelation` view.

## Change

- Added `SpirePageRelation` inside `src/am/ec_spire/page.rs`.
- Centralized safe local methods for:
  - main-fork block counts
  - FSM free-space lookup
  - locked buffer reads
  - already-locked new-block reads
  - GenericXLog transaction start
- Rolled the view through metadata initialization, object tuple append, object tuple rewrite, and no-compact object tuple delete paths.
- Made the private object tuple append helpers safe by requiring `SpirePageRelation` instead of raw relation input.

The public raw-relation entry points remain unsafe. This packet only removes repeated internal direct unsafe blocks after the relation contract is established.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/page.rs` | 38 | 35 | -3 |
| `src/` total | 2228 | 2225 | -3 |

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2225` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `git diff --check HEAD^ HEAD` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. Current closeout audit count is `2225` direct unsafe blocks under `src/`; packet 030 still requires every unsafe row to be removed or residual-registered.

