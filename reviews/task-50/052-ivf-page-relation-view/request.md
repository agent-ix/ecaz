# Task 50 Packet 052: IVF Page Relation View

This packet continues the accepted packet 030 comprehensive unsafe burndown plan. It advances the Wave 2 IVF/RaBitQ page storage tranche by moving repeated raw relation, buffer, FSM, and WAL operations behind a local IVF page relation view.

## Change

- Added `IvfPageRelation<'a>` inside `src/am/ec_ivf/page.rs`.
- Kept the raw `pg_sys::Relation` contract explicit at the unsafe constructor.
- Centralized safe local methods for:
  - relation OID reads used by posting free-space hints
  - main-fork block counts
  - FSM free-space lookup
  - locked buffer reads
  - already-locked new-block reads
  - GenericXLog transaction start
- Rolled the view through IVF posting append, directory/posting rewrite, and metadata initialize/read/update paths.
- Made the private posting append helpers safe by requiring `IvfPageRelation<'_>` instead of raw relation input.

This does not make public raw-relation entry points safe. The unsafe boundary remains at the PostgreSQL-facing caller contract, while repeated internal direct unsafe blocks are absorbed into the local view.

## Counts

| Scope | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_ivf/page.rs` | 56 | 42 | -14 |
| `src/` total | 2247 | 2233 | -14 |

## Ledger

- Generated `artifacts/unsafe-ledger-after.jsonl` for the post-change tree.
- `make unsafe-ledger-check` confirms the ledger covers all `2233` current direct unsafe rows under `src/`.
- Removed unsafe rows are represented by the before/after deltas above and the packet-local `code-diff.patch`.

## Validation

- `git diff --check HEAD^ HEAD` passed.
- `cargo check --all-targets --no-default-features --features pg18,bench` passed.
- The only compile warning is the known pre-existing `src/am/mod.rs` unused import warning.
- Benchmarks were not run because this packet does not change scan ordering, scoring math, payload bytes, WAL order, or hot-path allocation shape.

## Task 50 Status

Task 50 is not complete. Current closeout audit count is `2233` direct unsafe blocks under `src/`; packet 030 still requires every unsafe row to be removed or residual-registered.

