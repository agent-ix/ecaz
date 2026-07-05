# Feedback: 455-c1-native-build-query-score-cache

Reviewed against head `e15624b` and subsequent follow-ons.

## Verdict

Approved. Scope and seam are correct.

## Review focus answers

1. **Is the cache scope right — one insertion, query-side only, excluding
   backlink rewrites?**

   Yes. Keying by `query_idx == new_node_idx` matches exactly the pocket that
   rescores the same `(new_node, existing_node)` pair across entry-candidate
   creation, upper-layer expansion, and layer-0 expansion. Excluding backlink
   rewrites is the right call because those score from the target's
   perspective — the follow-on packet 456 handles that seam separately, which
   is the cleaner split.

2. **Next surface — backlink rescoring?**

   Yes, and packet 456 confirms that was the right next step.

## Notes

- `Vec<Option<f32>>` sized to `state.heap_tuples.len()` per insertion is the
  correct shape: indices are dense and small, lookups are O(1), and the
  allocation is short-lived. For very large builds this is N*8 bytes allocated
  and freed per insertion. If a future packet measures BUILD allocator pressure
  as a hotspot, this scorer could be hoisted out of the insertion loop and
  `.fill(None)` reused between insertions. Not required for merge.
- Threading the scorer through `populate_native_upper_layer_forward_slots` /
  `load_native_successor_candidates` is clean; it replaces three function
  parameters with one and makes the query-identity invariant local.
- Behavior preservation: cached value is exactly the prior `score_between`
  result, so ordering and tie-breaks are unaffected. Confirmed by reading the
  diff.

## Blockers

None.
