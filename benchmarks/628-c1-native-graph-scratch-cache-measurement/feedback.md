# Feedback: 628 Native Graph Scratch Cache Measurement

## Verdict: Accept

Both cache changes are correct. The ~38% serial speedup on 50k is a real
structural improvement.

## Code Review

**`NativeBuildVisitedSet`** (replaces `HashSet<usize>` in layer search scratch):
Same generation-stamp pattern as `NativeBuildQueryScoreCache` from packet 624.
Generation 0 is never a valid current generation (wrap clears and resets to 1).
`begin_search` is the clear boundary per search invocation. The
`expect("native build visited set should cover all graph nodes")` is correct —
the set is sized to `state.heap_tuples.len()` and all valid node indices are
within that range.

**`NativeBacklinkTargetScorer::cache: Vec<(usize, f32)>`** (replaces
`HashMap<usize, f32>`): The capacity is `m.saturating_mul(2).saturating_add(1)`.
For m=6 this is 13 entries. The cache is cleared on `reset_target` (per
backlink target, not per search), so it holds at most the neighbor count of the
current target. At 13 entries, linear scan is faster than HashMap hash+probe.
This is the right trade-off — HashMap overhead (heap allocation, hash
computation) on a 13-element set is pure overhead.

**`Option<NativeBacklinkTargetScorer>`** → owned value with `reset_target`:
Removes the `Option` unwrap path on every backlink use. The scorer is now
pre-allocated once and reset when the target changes. `current_target_idx:
Option<usize>` tracks the reset condition. Cleaner than the previous
`Option<scorer>` with `expect` on every access.

**`NativeBuildLayerSearchScratch::new(state.heap_tuples.len(), ...)`**: Visited
set capacity is now the full graph node count rather than a heuristic capacity
from packet 625. This is correct for the generation-stamp visited set — the
vec must cover all possible node indices. For the old `HashSet` path the
heuristic capacity was a starting point and the set could grow; for the
generation-stamp vec it must be pre-sized.

## Result

- Serial: 48,926 ms (627) → 30,333 ms (~38.0% faster).
- Parallel: 46,103 ms (627) → 30,845 ms (~33.1% faster).
- Serial graph phase: 47,023 ms (627) → 28,556 ms (~39.2% reduction).
- Parallel graph phase: 45,105 ms (627) → 29,816 ms (~33.9% reduction).

The `HashSet` allocation and hash overhead in the visited-set hot path was
the dominant remaining graph cost at this fixture size. The Vec+generation
version removes that allocation path entirely.

## Next Direction

After this change, graph construction is still 94% of serial build time.
The serial HNSW insertion search is now the dominant remaining cost. The
next packet should either:
1. Start a design packet for partial graph assembly parallelization (workers
   build disjoint graph regions, leader merges boundaries), or
2. Identify the next hot path within serial HNSW insertion (candidate pruning,
   backlink selection, neighbor count).

The dedicated parallel heap scan is correctly characterized as unable to
overcome the remaining serial graph phase on its own.

## No Issues
