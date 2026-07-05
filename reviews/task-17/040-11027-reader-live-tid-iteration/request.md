# Review Request: Reader Live-TID Iteration + `first_live_tid`

Branch: `adr034-diskann-access-method`
Author: coder-2
Target: `src/am/diskann/reader.rs`

## What this packet is

Pure-Rust isolated slice inside `src/am/diskann/reader.rs`. Adds
three iteration primitives to `PersistedGraphReader`:

- `iter_tids()` — every occupied TID in the chain in
  `(block_number, offset_number)` order, no decoding.
- `iter_live_tids()` — every non-deleted tuple, decoded, in the
  same order, yielding `Result<(ItemPointer, VamanaNodeTuple)>`.
- `first_live_tid()` — lowest-block, lowest-offset live TID in the
  chain, or `None` if every tuple is tombstoned.

Unblocks two named open questions:

1. **Phase 6B medoid fallback** — the scan design doc
   (`plan/design/diskann-scan-pgrx.md`) flags that if the medoid
   TID points to a deleted element (ADR-047 §10 defers medoid
   migration to rebuild), the scan needs a fallback entry point.
   `first_live_tid()` is that primitive.

2. **ADR-047 pass-3 orphan detection** — packet 11025 G3 asks how
   orphan detection scans every live element's neighbor list. The
   expected resolution (option b) is an ordered block scan that
   reads every live element. `iter_live_tids()` is the scaffold for
   that pass.

## Why this is isolated pure-Rust work

- No edit to `src/am/*.rs` (native-build lane off-limits).
- No new module; additive methods on an existing reader.
- No pgrx, no storage changes — `DataPageChain::pages()` +
  `DataPage::tuple_count()` already expose enough.

## Design choices

- `iter_tids()` walks every slot — *including tombstoned ones* —
  because some future consumers (e.g., ADR-047 pass-1 dead-heap-TID
  discovery) will want tombstoned tuples too. Live filtering is
  opt-in via `iter_live_tids()`.
- `iter_live_tids()` yields `Result<_>` and stops on first decode
  error, so a corrupt tuple surfaces as an error instead of being
  silently skipped. This matches the existing `read_node` contract.
- `first_live_tid()` is a thin `fn` over `iter_live_tids()`, not a
  separate implementation. No premature optimization — the hot path
  is Phase 6B's `amgettuple`, which calls this at most once per
  scan (on medoid-deleted fallback), so a single decode per skipped
  tombstone is fine.
- No caching of the live TID on the reader — the reader is a
  borrowed handle, and the persisted chain never mutates through
  a shared reference. If a hot caller wants to memoize, that
  belongs to Phase 6B's `DiskannScanOpaque`.

## Tests

Four new tests (RD-013..RD-016). All use a new helper
`persisted_with_tombstones(n, max_degree, to_tombstone)` that
persists a chain-graph, then flips `deleted` on the named nodes via
`vacuum::mark_deleted` + re-encode + `update_raw_tuple` (ADR-045
Decision 3 fixed-length invariant).

- **RD-013** — `iter_tids()` yields `n` TIDs total, strictly
  increasing in `(block, offset)` order, and every persisted node's
  TID is contained in the output.
- **RD-014** — `iter_live_tids()` skips tombstoned TIDs (0, 3, 7 of
  12 nodes), preserves order on survivors, and yields `!deleted`
  tuples only.
- **RD-015** — `first_live_tid()` skips a leading tombstoned run
  (nodes 0, 1 of 10) and returns the same TID as
  `iter_live_tids().next()`.
- **RD-016** — all-tombstoned chain returns `None`.

## Verification

```
cargo check --lib     # clean
cargo test --lib am::diskann::reader  # 16 passed (was 12)
cargo test --lib am::diskann          # 81 passed (was 77)
```

## Non-changes (affirming choices)

- `read_node`, `neighbors`, `greedy_search_persisted*` signatures
  unchanged.
- No MVCC/visibility logic at the reader layer — tombstone = index
  `deleted` bit only. Heap visibility stays in the pgrx callback.
- `iter_live_tids()` does not expose an `impl Iterator + 'a` that
  borrows across decode — decode is synchronous per item, so the
  lifetime binding is trivial.

## Dependencies

- **Packet 11022** (Phase 5D persisted-graph reader) — the module
  this extends.
- **Packet 11021** (Phase 8A vacuum primitives) — `mark_deleted`
  used in the test helper.

## Companion work

- **Packet 11026** (`VisitedState` reuse) — shipped concurrently
  for the greedy hot path; this packet extends the reader's
  read-only surface orthogonally.
- **`plan/design/diskann-scan-pgrx.md`** — calls out
  `first_live_tid()` under "Open questions — Fallback entry
  point."

## Not doing in this packet

- Cursor-style mutable iteration. Callers that need to rewrite
  tuples in place (ADR-047 pass-2 neighbor repair) should use
  `get_page_mut` + their own loop.
- An `iter_dead_tids()` counterpart. Not needed yet; easy to add
  when a consumer lands.
- Paginated iteration or prefetching. Phase 6B's scan is eager-
  materialized (per the design doc); iteration cost is bounded by
  the number of tombstoned tuples encountered before the first live
  hit.
