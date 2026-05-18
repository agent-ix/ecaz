# Review Request: Phase 8A — Tuple-level Vacuum Primitives

Branch: `adr034-diskann-access-method`
Author: coder-2
Companion to: 11014 (ADR-045), 11015 (Phase 5A), 11016 (Phase 5B),
11017 (Phase 5C-1), 11018 (Phase 5C-2)

## What this slice is

Pure-Rust deletion + neighbor-repair primitives on
`VamanaNodeTuple`. No pgrx, no page I/O, no locks — these are the
three in-place mutations the future pgrx three-pass vacuum callback
(Phase 8B, deferred with the native-build lane) will orchestrate
under ADR-047's lock ordering.

Splitting the primitives off from the pgrx orchestration puts the
deletion state machine at the right test layer: ten unit tests cover
every transition + the ADR-045 fixed-length invariant, without a
buffer manager in sight.

## Scope

- `src/am/diskann/vacuum.rs` — new file, 349 lines incl. 10 tests.
- `src/am/diskann/mod.rs` — `pub mod vacuum;` declaration.

No other source files touched.

## What changed

### Public API

```rust
pub fn mark_deleted(tuple: &mut VamanaNodeTuple);

pub fn strip_dead_primary_heaptid<P: Fn(ItemPointer) -> bool>(
    tuple: &mut VamanaNodeTuple,
    dead_pred: P,
) -> bool;

pub fn is_fully_dead(tuple: &VamanaNodeTuple) -> bool;

pub fn repair_neighbors(
    tuple: &mut VamanaNodeTuple,
    dead_set: &HashSet<ItemPointer>,
) -> usize;
```

### What each primitive does

1. **`mark_deleted`** (ADR-047 vacuum pass 3) — flips the tombstone
   bit. Does not clear neighbors or payload bodies; backlink
   discovery on tombstones is still load-bearing until the page is
   reaped. Idempotent.
2. **`strip_dead_primary_heaptid`** (ADR-047 vacuum pass 1) —
   overwrites `primary_heaptid` with `ItemPointer::INVALID` iff
   `dead_pred` returns true on the current value. Already-`INVALID`
   heaptids skip the predicate. Returns `true` on strip, `false`
   otherwise. Does **not** flip `deleted` — pass 3 owns that decision
   (needs to know about overflow chains).
3. **`is_fully_dead`** — `primary_heaptid == INVALID &&
   !has_overflow_heaptids`. For V1 builds (no overflow chain) this
   is exact; when Phase 7 adds overflow heaptids, pessimistically
   returns `false` when the flag is set so callers know to walk the
   chain.
4. **`repair_neighbors`** (ADR-047 vacuum pass 2 fill-half) — walks
   `neighbors[..neighbor_count]`, drops entries in `dead_set`,
   stably compacts survivors into the prefix, pads the tail with
   `INVALID`, updates `neighbor_count`. Returns removed count.
   Vec length stays at `R`.

### Tests (10, all green)

- **VC-001** `mark_deleted` is idempotent
- **VC-002** `mark_deleted` preserves neighbors + primary heaptid
- **VC-003** `strip_dead_primary_heaptid` honors predicate (both
  branches), returns correct boolean
- **VC-004** `strip` skips already-INVALID heaptids *without*
  invoking the predicate (uses `Cell<bool>` to observe)
- **VC-005** `is_fully_dead` — INVALID + no overflow ⇒ true; alive
  primary ⇒ false; overflow flag blocks fully-dead
- **VC-006** `repair_neighbors` removes dead, compacts live, pads
  INVALID, updates `neighbor_count`, preserves Vec length `R`
- **VC-007** empty `dead_set` is a no-op (clone-equality verified)
- **VC-008** repair is stable — survivor order matches input order
- **VC-009** **ADR-045 Decision 3** — repair preserves encoded
  length: `encode(r, w, c).len()` is identical before/after repair
  and equals `VamanaNodeTuple::encoded_len(r, w, c)`. Locks the
  fixed-length invariant that makes `update_raw_tuple` sound.
- **VC-010** full deletion state machine walk: alive → strip primary
  → repair (no-op on self) → `mark_deleted`. Confirms each pass is
  independent and doesn't clobber prior passes' state.

```
running 10 tests
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured;
             563 filtered out; finished in 0.00s
```

`cargo check --lib` clean (5 pre-existing dead-code warnings).

## Review focus

1. **Three primitives, three passes.** The split mirrors ADR-047's
   three-pass structure exactly:
   - pass 1 (dead heap rows) → `strip_dead_primary_heaptid`
   - pass 2 (neighbor-list repair) → `repair_neighbors` (fill half)
   - pass 3 (tombstones) → `mark_deleted` + `is_fully_dead`
   Reviewer confirm this is the right seam for the pure-Rust layer,
   and that Phase 8B's pgrx callback orchestrating these under lock
   is the natural continuation.
2. **`strip_dead_primary_heaptid` does not flip `deleted`.** The
   tombstone decision is pass 3's and depends on the overflow chain,
   which the primitive deliberately knows nothing about. Reviewer
   confirm the split is right — or argue for a combined
   `strip_and_maybe_tombstone` if you want the two bound together.
3. **`repair_neighbors` is fill-only; it does not append repair
   candidates.** Adding new neighbors to fill the freed slack
   requires a `greedy_search` under shared lock, which is the pgrx
   caller's job (ADR-047 vacuum pass 2 append-half). The primitive
   draws the line at the mutation that needs no lookup. Reviewer
   confirm, or argue for pushing the append step into this module
   with the search closure injected.
4. **`HashSet<ItemPointer>` as the dead-set API.** Pass 2 classifies
   every page's dead TIDs up front; membership-test is the only
   operation the primitive needs. Alternative was `&[ItemPointer]` +
   linear scan — rejected because a page with R=32 neighbors × ~100s
   of dead TIDs in the VACUUM is O(R·D) vs O(R). Reviewer confirm
   the collection choice.
5. **Stability of repair.** VC-008 asserts survivor order is
   preserved. Reviewer confirm this matters (it does for
   deterministic test fixtures and avoids reshuffling hot neighbors
   to the back when the prefix compaction runs).
6. **`is_fully_dead` is the pessimistic helper for Phase 7.** When
   Phase 7's overflow chain lands, callers walk the chain only when
   `has_overflow_heaptids` is set. V1 callers can treat the helper
   as exact. Reviewer flag if the semantics should be split into
   `is_fully_dead_v1` + `is_fully_dead_with_chain` instead.

## Questions to answer

- **Should `repair_neighbors` return `&[ItemPointer]` of survivors
  for the pgrx caller to re-sort by distance?** Currently returns
  only the count. Argument for: the caller may want to score the
  survivors against the medoid or against each other before the
  append step. Argument against: the survivors are still in
  `tuple.neighbors[..neighbor_count]`; the caller can read them
  directly. Held: count-only, caller reads the prefix.
- **Should the primitives take `&mut VamanaNodeTuple` or operate on
  the encoded byte slice directly?** Taking the decoded tuple keeps
  the invariants (`neighbor_count ≤ R`) expressible in Rust and
  tested cheaply. The pgrx caller already pays the decode cost
  because it must inspect the tuple to classify it anyway. Held:
  decoded tuple in, re-encode on write.
- **Should `strip_dead_primary_heaptid` batch multiple TIDs?** V1
  has one primary heaptid per node, so no. Phase 7 + overflow
  chains may want a batched version — but that lives in an overflow
  primitive, not this one.

## Not doing in this packet

- **pgrx vacuum callback.** Phase 8B — deferred with the
  native-build lane (same conflict surface as 5C-3 per
  `project_native_build_conflict_surface.md`).
- **Overflow heaptid chain primitives.** Phase 7 introduces them;
  this packet just respects the `has_overflow_heaptids` flag.
- **Append-half of pass 2.** Requires `greedy_search` under shared
  lock; lives in Phase 8B orchestration.
- **Page-level reaping.** Index-AM-level `amvacuumcleanup`
  responsibilities (free-space recovery, etc.) are Phase 8B scope.

## Dependencies

- **ADR-045 ACCEPTED** — Decision 3 fixed-length invariant is the
  invariant VC-009 guards.
- **Phase 5B (11016)** — uses `VamanaNodeTuple` shape and
  `encode` / `encoded_len` from the slim tuple layout.
- **ADR-047 PROPOSED** — three-pass vacuum structure is the design
  these primitives compose into. ADR-047 does not need to be
  ACCEPTED before this packet lands (no pgrx-facing choices baked
  in), but Phase 8B does require it ACCEPTED.

## Companion packets

- **11014** — ADR-045 page-layout discipline.
- **11016** — Phase 5B slim tuple (the surface these mutate).
- **11017** — Phase 5C-1 persist sequencer.
- **11018** — Phase 5C-2 build orchestrator.
- **Future** — Phase 8B pgrx vacuum callback (deferred with
  native-build lane merge).

## Definition of ready

- ADR-045 ACCEPTED.
- 10 VC tests green (verified locally).
- Reviewer confirms the three-primitive split and the
  strip/tombstone separation.
- Phase 8B does not start before this lands.

## Handoff notes

The module is intentionally small. Its purpose is to:

1. Put the deletion state machine at the right layer for cheap
   testing — a `HashSet` + a decoded tuple, no buffer manager.
2. Lock the fixed-length invariant at the primitive that most
   threatens it (neighbor repair).
3. Give Phase 8B a tight public API to orchestrate under ADR-047's
   lock ordering, so the pgrx callback becomes: classify → call
   strip → call repair + append (the append needs the search
   closure, lives in 8B) → call mark_deleted.

If reviewer prefers the primitives closer to the encoded-bytes
boundary (skip the decode), that's a meaningful refactor and
changes the test shape — flag early. Otherwise the shape matches
5A → 5B → 5C-1 → 5C-2's pattern: decoded in-memory operations with
encode/decode handled at the persist boundary.
