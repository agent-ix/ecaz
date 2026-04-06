# Code Review Pass 3: tqvector scan implementation

**Reviewer:** Claude Sonnet 4.6 (initial), Claude Opus 4.6 (second pass)  
**Date:** 2026-04-06  
**Commit:** `85c5b72` (head at time of review)

## Context

This review focuses on the current state of the scan implementation as the codebase approaches the transition from bootstrap linear scan to real HNSW graph traversal. The `BeamSearch` module and `graph.rs` primitives are complete. The bootstrap frontier machinery is the in-flight scaffold toward ordered traversal.

Validation status assumed:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review Files

### Fix Now
- [01 - next_bootstrap_expand_tid not gated cfg(test)](01-next-bootstrap-expand-tid-not-gated.md)
- [02 - score_scan_element_result acquires mutex + hash lookup + Arc clone per element scored](02-score-element-cached-per-call.md)

### Fix Before ef_search Wires In
- [03 - DebugCandidateFrontier stale 2-slot type alias](03-debug-frontier-stale-type-alias.md)

### Style / Fragility (Low Priority)
- [04 - with_visible_frontier_and_bootstrap_expansion raw pointer split-borrow](04-split-borrow-raw-pointer-fragility.md)
- [05 - amendscan casts opaque seven times](05-amendscan-repeated-cast.md)

### Notes (No Action Required)
- [06 - expand_one with empty neighbor function is intentional transition scaffolding](06-expand-one-empty-neighbors.md)
- [07 - BeamSearch forget_queued O(n) drain-rebuild](07-forget-queued-on.md)
- [08 - refill early return when frontier full](08-refill-early-return-comment.md)

## Finding Details

### 01 — `next_bootstrap_expand_tid` not gated `#[cfg(test)]` ✓ CONFIRMED

`scan.rs:582-605` — this function is called only from unit tests (lines 1538, 1554 inside `#[cfg(test)] mod tests`) but is compiled into production builds. It should be gated `#[cfg(test)]`.

**Correction from initial review:** The initial review characterized this as "duplicating what the persistent scheduler computes." That's imprecise. `next_bootstrap_expand_tid` is a **stateless recomputation** that builds a fresh throwaway `BeamSearch` to observe what the score-order policy *would* select, without mutating any persistent state. The production `top_up_bootstrap_frontier` uses the **stateful** persistent scheduler whose state may have drifted from a clean reset. The function exists so tests can query policy behavior without touching the persistent scheduler. It's a legitimate test helper — it just needs `#[cfg(test)]`.

**Fix:** Add `#[cfg(test)]` to the function definition. Do not delete it.

### 02 — `score_scan_element_result` hot-path overhead ✓ CONFIRMED, SEVERITY UNDERSTATED

`scan.rs:939-951` — calls `ProdQuantizer::cached()` per element scored. The initial review said "avoids one atomic refcount bump." The actual per-call cost is:

1. **Mutex lock** on a `Mutex<HashMap<(usize, u8, u64), Arc<ProdQuantizer>>>`
2. **HashMap entry lookup** with hash computation
3. **Arc clone** (atomic refcount increment)

At MAX_BOOTSTRAP_FRONTIER_CANDIDATES=3 this is negligible. At ef_search=40 with graph fan-out scoring hundreds of candidates, the mutex acquire per candidate is the real concern — not the atomic refcount.

`store_scan_prepared_query` (line 228) already calls `cached()` once during `amrescan`. Cache the returned `Arc<ProdQuantizer>` alongside the prepared query in `TqScanOpaque` and reuse it for the scan's duration.

**Fix:** Add a `*mut ProdQuantizer` field to `TqScanOpaque` (or store the `Arc` directly), populate it in `store_scan_prepared_query`, free in `amendscan`. Change `score_scan_element_result` to use the cached quantizer.

### 03 — `DebugCandidateFrontier` stale 2-slot type alias ✓ CONFIRMED, SEVERITY OVERSTATED

`scan_debug.rs:20` — `type DebugCandidateFrontier = [DebugCandidateSlot; 2]` — hard-coded to 2 slots while the frontier can hold 3+ candidates.

**Correction from initial review:** The initial review said this "silently ignores candidates beyond index 1." While `debug_candidate_frontier_snapshot` (line 143-146) does only read slots 0 and 1, the newer `DebugCandidateFrontierSlots` (Vec-based) and `DebugCandidateFrontierSlotConsume` types already provide full frontier visibility in the tests that need it. The 2-slot snapshot coexists with the complete Vec snapshot — tests are not blind to the third candidate.

The type alias is still stale and should be migrated before ef_search wires in and the frontier grows substantially, but the claim that tests "miss bugs" because of it is overstated.

**Fix:** Migrate remaining 2-slot snapshot consumers to the Vec-based `DebugCandidateFrontierSlots` type, or remove `DebugCandidateFrontier` if no pg tests still depend on it.

### 04 — `with_visible_frontier_and_bootstrap_expansion` raw pointer split-borrow (NEW)

`scan.rs:416-426` — casts `&VisibleCandidateFrontierState` and `&mut BeamSearch` to raw pointers to work around the borrow checker (both are behind pointers in `TqScanOpaque`, so Rust can't prove they don't alias). This is **sound** because `candidate_frontier` and `bootstrap_expansion` are disjoint heap allocations, but it's an implicit invariant that would silently break if either field were changed to alias or share storage.

Not a defect. Worth a `// SAFETY:` comment documenting the non-aliasing invariant.

### 05 — `amendscan` casts opaque seven times (NEW)

`scan.rs:180-186` — each `free_*` call independently casts `opaque` to `&mut TqScanOpaque`. This is fine (each cast's lifetime ends before the next begins), but a single cast-once-reuse pattern is clearer and easier to audit:

```rust
let opaque = &mut *opaque.cast::<TqScanOpaque>();
free_scan_candidate_frontier(opaque);
free_bootstrap_expansion(opaque);
// ...
```

Style issue, not a soundness issue.

### 06 — `expand_one` with empty neighbor function (RECLASSIFIED from "fix before ef_search")

`scan.rs:658-660` — the initial review recommended replacing `expand_one(|_| empty())` with a `pop_best()` call or adding `BeamSearch::consume_best()`. On second look, this is **intentional transition scaffolding**: when A3 wires in real graph search, the empty neighbor function will be replaced with the actual neighbor-loading callback, and the external `refill` call will move inside it. Using `expand_one` here pre-shapes the call site for that transition.

Adding `consume_best()` would create a dead-end API that gets replaced immediately when A3 lands. The better fix is a comment explaining the intent: `// empty neighbor function — real neighbor loading happens in the refill callback; this call site gains a real neighbor function when graph traversal wires in`.

**No code change needed.** Add a comment if desired.

### 07 — `BeamSearch::forget_queued` O(n) (UNCHANGED)

No action needed now. The real graph scan will use `BeamSearch::run()` / `expand_one()` in a tight loop — `forget_queued` is only needed for the dual-structure consume/refill pattern, which goes away with A3.

### 08 — `refill_candidate_frontier_from_source` silent early return (UNCHANGED)

`scan.rs:710-714` — early returns when `max_successor_candidates == 0`. Correct behavior, worth a short comment. Minor.

## Priority Summary

| Priority | Finding | Action |
|---|---|---|
| Fix now | #01 `next_bootstrap_expand_tid` | Add `#[cfg(test)]` |
| Fix now | #02 `score_scan_element_result` | Cache `Arc<ProdQuantizer>` in `TqScanOpaque` during `amrescan` |
| Fix before A3 | #03 `DebugCandidateFrontier` | Migrate to Vec-based snapshot type |
| Low | #04 split-borrow raw pointers | Add `// SAFETY:` comment |
| Low | #05 amendscan repeated cast | Refactor to single cast |
| No action | #06 expand_one empty neighbors | Intentional scaffolding |
| No action | #07 forget_queued O(n) | Goes away with A3 |
| No action | #08 refill early return | Add comment if desired |

## Plan / Architecture Review

The Sonnet pass also produced a plan/architecture review (in conversation, not yet written to file). The findings were validated and are summarized here for the record:

### Confirmed

1. **Task 05 A1 is complete but not marked.** scan.rs, insert.rs, graph.rs, search.rs, shared.rs, scan_debug.rs are all extracted.
2. **GUC not registered.** FR-009-AC-5 requires `SET tqhnsw.ef_search = 200` to work. No `_PG_init` / `GucRegistry` exists anywhere in the codebase. Only the reloption in `options.rs`.
3. **ADR-014 and ADR-015 are PROPOSED, should be DECIDED.** The implementation already follows both designs.
4. **Layer-0-only vs multi-layer needs an explicit resolution.** FR-009 Step 2 specifies multi-layer greedy descent; ADR-015 recommends layer-0-only. The current `graph.rs::load_graph_neighbors` returns all tids without layer slicing. FR-007 defines the layer-aware slot layout (`tids[0..2M]` for layer 0, `tids[2M..3M]` for layer 1, etc.), so the page format supports multi-layer — the graph.rs API doesn't expose it yet.
5. **A4 recall gate requires ordered emission.** Measuring recall on the current linear-scan output is meaningless. A4 can only run after ADR-015 Stages 2+3 (result buffering + linear fallback removed).

### Correction

The Sonnet review's A2 scope narrowing ("just wire BeamSearch + graph.rs together") understates the work slightly. A2 also requires:
- A layer-0 neighbor slicing helper in `graph.rs` (even for layer-0-only, you need to read `tids[0..min(2*M, count)]` not the full neighbor list, or deleted upper-layer neighbors could pollute traversal)
- Deciding how `score_scan_element_result` interacts with `BeamSearch` — the current function takes `&TqScanOpaque` but `BeamSearch::run()` takes a closure that can't borrow opaque
- Buffer pin discipline verification: `load_graph_element` and `load_graph_neighbors` each acquire and release a buffer lock per call; beam search will call these potentially hundreds of times per scan
