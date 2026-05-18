## Feedback: ADR-030 v2 Grouped Hot Payload Cache Seam

Read `CachedGraphElement`, `CachedGroupedSearchCode`, and `from_graph_tuple_ref` in
`src/am/scan.rs`, plus the pg_tests in `src/lib.rs`:
- `cached_graph_element_from_grouped_tuple_ref_keeps_grouped_hot_payloads`
- `cached_graph_element_from_scalar_tuple_ref_has_no_grouped_hot_payloads`

### What's right

- Cache entry now carries what the grouped scorer will need: `reranktid`,
  `grouped_search_code`. Without this, the first scorer packet would have to either
  re-decode tuples or reshape the cache — both worse.
- `CachedGroupedSearchCode` as an explicit carrier type, not a raw `Vec<u8>`. That
  lets the scorer depend on a named shape, and keeps the door open for a future
  packed-LUT representation to be added to the cache without shuffling callsites.
- Positive and negative tests: grouped entries have the payloads, scalar entries
  assert empty. That catches the two obvious regressions.

### Concerns

1. **Cache ownership.** If `CachedGroupedSearchCode` owns its bytes (`Vec<u8>`),
   cache entries copy search code on every cache fill. That's the right conservative
   choice while the cache is being shaped, but in the hot path this copy is probably
   avoidable (graph tuple ref bytes live in a pinned buffer during scan). Worth
   revisiting after the scorer lands and the real hot-path allocation profile is
   visible. Not a blocker.

2. **Invariant coupling.** A cache entry has either grouped hot payloads, or scalar
   exact payload, but never both. Is this encoded in the type, or is it a runtime
   invariant with two separate `Option` fields? If the latter, an exhaustive
   enum (`CachedGraphElementBody::Scalar { .. } | GroupedHot { .. }`) would make the
   invariant compile-time-checked. Worth considering before the scorer packet.

### Observation

This packet is the second half of the cache story started in 324-325. Together with
328's typed score-input, it means the scorer packet can land without reshaping scan
state again. Good sequencing.
