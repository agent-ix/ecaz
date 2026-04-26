# Feedback: 626 Parallel Index Build 50k Scale Measurement

## Verdict: Accept

Pure measurement. Findings are correctly interpreted and the bottleneck
identification is accurate.

## Result

- Serial: 53,683 ms. Parallel: 52,391 ms. Parallel ~2.4% faster.
- Graph construction: 44,626 ms serial / 44,867 ms parallel — ~83-86% of wall
  time.
- Parallel sort/push: 6,706 ms — the second concrete bottleneck at this scale.

## Sort/Push Bottleneck

The 6.7 second sort/push on 50k rows is correct to flag. **However, the
request's diagnosis is wrong.** `BuildState::push` does NOT do a linear
duplicate scan. It uses `tuple_index_by_payload: HashMap<BuildTupleDedupKey,
usize>` (`build.rs:84`) and calls `self.tuple_index_by_payload.get(&dedup_key)`
(`build.rs:413`) — a HashMap lookup, O(1) average. There is no O(N²) path.

The 6.7s cost is real but the mechanism is not identified. Likely candidates:

1. **Cache-unfriendly sort**: `sort_by_key(build_tuple_heap_tid_key)` moves
   50k `BuildTuple` values each containing scattered heap allocations
   (`heap_tids: Vec<_>`, `code: Vec<u8>`). The comparator dereferences
   `heap_tids.first()`, a pointer chase into scattered allocations — O(N log N)
   cache misses.
2. **Unnecessary code clone in dedup key**: `BuildTupleDedupKey::from_tuple`
   (`build.rs:69-74`) clones `tuple.code: Vec<u8>` on every push — 50k
   allocations. Small for 64-dim turboquant (~16 bytes each), but avoidable.
3. **sort_by_key recomputes key O(N log N) times**: `sort_unstable_by_key`
   has the same complexity but avoids stability overhead; with a cheap key
   function this is minor.

Do not implement a fix based on the O(N²) assumption. Profile before the next
implementation slice to confirm the actual hot path.

## Direction

Optimizing `BuildState::push` before further parallel-ingest work is the
correct priority. At 50k the sort/push overhead approaches 13% of total
parallel build time, which means any heap ingest speedup is partially absorbed
by this bottleneck.

## No Issues
