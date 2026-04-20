# Review Request: VisitedState Reuse (Allocation-Free Scan Scratch)

Branch: `adr034-diskann-access-method`
Author: coder-2
Target: `src/am/diskann/reader.rs`, `src/am/diskann/scan.rs`

## What this packet is

Pure-Rust isolated refactor inside `src/am/diskann/`. Packets
**11022 Q2** (reader) and **11023 Q2** (scan) both flagged the
per-call `HashSet<ItemPointer>` allocation as a concern once
Phase 6B wires the scan into `amgettuple`'s hot path. This packet
introduces a caller-owned `VisitedState` scratch buffer and adds
`_with` variants of the greedy/scan entry points that take
`&mut VisitedState`, while keeping the original allocation-per-call
variants as thin wrappers for test convenience and non-hot callers.

Test count: `am::diskann::*` now 77 (was 74). Additions are
RD-011, RD-012, SC-011.

## Motivation

- Phase 6B's pgrx `amgettuple` cursor is invoked once per returned
  tuple but holds scan-level state across calls. A fresh
  `HashSet<ItemPointer>` allocation per scan is tolerable; a fresh
  allocation per `amgettuple` invocation is not. The scan-level
  state is where the scratch naturally lives.
- `greedy_descent` and `greedy_search_persisted` both want the
  same scratch, so a single `VisitedState` is reused across scan
  stages.
- Additive design: no existing caller signature changes.

## Changes

### `src/am/diskann/reader.rs`

New type:

```rust
#[derive(Debug, Default)]
pub struct VisitedState {
    pub(crate) in_frontier: HashSet<ItemPointer>,
    pub(crate) visited: HashSet<ItemPointer>,
}

impl VisitedState {
    pub fn new() -> Self { Self::default() }
    pub fn clear(&mut self) { /* clears both sets, retains capacity */ }
    pub fn reserve(&mut self, additional: usize) { /* grows both */ }
}
```

Split:

```rust
pub fn greedy_search_persisted<D>(reader, entry_point, list_size, query_dist)
    -> Result<PersistedGreedyResult, String>
    where D: Fn(ItemPointer) -> f32;
// Now a thin wrapper over:
pub fn greedy_search_persisted_with<D>(
    reader,
    scratch: &mut VisitedState,
    entry_point,
    list_size,
    query_dist,
) -> Result<PersistedGreedyResult, String>;
```

The `_with` variant calls `scratch.clear()` on entry, so callers
can reuse the same buffer across scans without manual reset.

Two new tests:

- **RD-011** — reuse a single `VisitedState` across two greedy
  searches with different distance closures. Result must match
  two fresh-allocated calls exactly.
- **RD-012** — `clear()` empties both sets and `reserve()` grows
  capacity without disturbing contents on subsequent insert.

### `src/am/diskann/scan.rs`

Imports `VisitedState` from `reader`. Removed the now-unused
`use std::collections::HashSet;` from module scope (re-imported
inside `#[cfg(test)] mod tests` because synthetic fixtures use it).

Split:

```rust
pub fn greedy_descent<Pre>(reader, entry_point, list_size, prefilter)
    -> Result<Vec<ScanCandidate>, String>;
pub fn greedy_descent_with<Pre>(
    reader,
    scratch: &mut VisitedState,
    entry_point,
    list_size,
    prefilter,
) -> Result<Vec<ScanCandidate>, String>;

pub fn vamana_scan<Pre, Re>(reader, params, prefilter, rerank)
    -> Result<Vec<ScanResult>, String>;
pub fn vamana_scan_with<Pre, Re>(
    reader,
    scratch: &mut VisitedState,
    params,
    prefilter,
    rerank,
) -> Result<Vec<ScanResult>, String>;
```

`vamana_scan_with` calls `greedy_descent_with` with the passed
scratch; the rerank stage does not use the scratch (its dedup
rides on the greedy frontier's natural dedup).

One new test:

- **SC-011** — reuse a single `VisitedState` across two
  `vamana_scan_with` calls with different prefilter+rerank
  closures. Each result must equal the corresponding fresh call.

## Why `pub(crate)` on the fields

`VisitedState` is the canonical scratch for the private greedy
loop inside the module tree. Exposing the `HashSet` fields
publicly would leak the implementation choice. `pub(crate)` lets
the reader implementation touch them while keeping the public
surface to `new` / `clear` / `reserve`.

## Non-changes (affirming choices)

- `greedy_search_persisted` and `vamana_scan` still exist with
  their original signatures. No caller updates required.
- The frontier `Vec<TidCandidate>` is still allocated per call —
  it's small (bounded by `L_search`) and sorted in place, so
  reusing it would add complexity for minor savings. Revisit if
  a flamegraph justifies it.
- The prefilter/rerank closure shape is unchanged. Phase 6B still
  binds them at the pgrx boundary.

## Dependencies

- **Packet 11022** (Phase 5D reader) — introduces
  `greedy_search_persisted` and `PersistedGraphReader`.
- **Packet 11023** (Phase 6A scan shell) — introduces
  `vamana_scan` and `greedy_descent`.

Both Q2 items from those packets are addressed here.

## Verification

```
cargo check --lib     # clean (pre-existing quant warnings only)
cargo test --lib am::diskann
    test result: ok. 77 passed; 0 failed
```

## Not doing in this packet

- Reusing the frontier `Vec<TidCandidate>` — noted above.
- Pooling `VisitedState` across scans at a higher layer —
  belongs to the Phase 6B pgrx wiring packet.
- Benchmarks — no representative workload yet; Phase 6B will
  provide the first end-to-end call site to measure against.
