# 03 — `DebugCandidateFrontier` stale 2-slot type alias

**Severity:** Low  
**File:** `src/am/scan_debug.rs:20, 143-146`

## Finding

`type DebugCandidateFrontier = [DebugCandidateSlot; 2]` is a fixed 2-element array from the old two-slot frontier era. The frontier is now `Vec`-backed and can hold `MAX_BOOTSTRAP_FRONTIER_CANDIDATES = 3` candidates (or more once `ef_search` wires in).

`debug_candidate_frontier_snapshot` (line 143-146) hard-codes reads of slots 0 and 1:
```rust
[visible_frontier_slot(opaque, 0), visible_frontier_slot(opaque, 1)]
    .map(debug_candidate_slot)
```

## Concrete concern

The 2-slot snapshot coexists with the Vec-based `DebugCandidateFrontierSlots` type (line 30) and `DebugCandidateFrontierSlotConsume` (lines 54-62) which already provide full frontier visibility in newer tests. Tests are not blind to the third candidate — the Vec-based types cover it.

The stale type alias is misleading to readers and will become more problematic when `ef_search` wires in and the frontier holds 40+ candidates, but existing test coverage is not degraded by it.

## Suggested shape

Migrate remaining 2-slot snapshot consumers (`DebugCandidateFrontierLifecycle`, `DebugCandidateFrontierConsume`) to the Vec-based `DebugCandidateFrontierSlots` type. Or remove `DebugCandidateFrontier` entirely if no pg tests still depend on the fixed-array return type.

## Impact

No correctness issue. Code clarity and future-proofing for the `ef_search` transition.
