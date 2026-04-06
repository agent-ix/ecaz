# 02 — refill_candidate_frontier_from_source redundant visited check

**Severity:** Medium  
**File:** `src/am/scan.rs:536–548`

## Finding

In `refill_candidate_frontier_from_source`, the closure passed to `collect_successor_candidates` checks `visited_contains_element(opaque, neighbor_tid)` at line 538, then loads the graph element, and checks `visited_contains_element(opaque, neighbor.tid)` again at line 548.

Since `load_graph_element(index_relation, neighbor_tid, code_len)` returns a `GraphElement` with `tid: element_tid` (i.e., `neighbor.tid == neighbor_tid`), these two checks test the same value.

```rust
// Line 538: first check
if visited_contains_element(opaque, neighbor_tid) {
    return None;
}

let neighbor = load_graph_element(index_relation, neighbor_tid, ...);

// Line 548: same check, neighbor.tid == neighbor_tid
if ... || visited_contains_element(opaque, neighbor.tid) {
    return None;
}
```

## Impact

The redundant check is harmless at the current frontier size (3 candidates). When the frontier grows to `ef_search` candidates, this becomes a wasted `HashSet::contains` call per neighbor per expansion. More importantly, the second check is misleading — it suggests `neighbor.tid` might differ from `neighbor_tid`, which it cannot.

## Suggestion

Remove the second `visited_contains_element` check. The `deleted` and `heaptids.is_empty()` guards on line 546 are still needed.
