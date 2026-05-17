## Feedback: ADR-030 v2 Grouped Read Scaffolding

Read `GraphTupleRef` in `src/am/graph.rs` along with `load_grouped_graph_element` and
`load_grouped_graph_adjacency`.

### What's right

- `GraphTupleRef::Scalar(...) | GroupedHot(...)` is the right top-level seam. All
  format-aware accessors live on this enum. Downstream cache code (packet 327) does
  not need to know about tuple byte layout.
- Accessors are typed: `level()`, `deleted()`, `heaptid_count()`, `collect_heaptids()`,
  `neighbortid()`, `reranktid()`, `binary_word_count()`, `collect_binary_words()`,
  `exact_payload()`, `grouped_search_code()`. Each has a clear format-awareness story.
- `exact_payload()` returns `Option` so grouped-v2 cleanly signals "no exact payload
  in the hot tuple" without a panic path. This is the seam that packet 326 uses for
  `ExactUnavailable`.

### Code-level concern (hot path)

**`binary_word_count()` on `GroupedHot` does `tuple.collect_binary_words().len()`.**

That allocates a Vec just to compute a length. In the scalar branch, it's byte-length
divided by 8 — cheap. On the grouped branch, it's a Vec allocation per call. Today
that's fine because grouped scans are rejected, but when the scorer lands, this will
be on the inner loop of the approximate scorer. Before the scorer packet:

1. Add a public `binary_word_count()` accessor to `TqGroupedHotTupleRef` that reads
   the count without materializing the slice.
2. Update `GraphTupleRef::binary_word_count()` to delegate to it on the grouped
   branch.

Same pattern for `heaptid_count()` if it also materializes under the hood — confirm.

### Test coverage

The read scaffolding is exercised by `test_grouped_v2_graph_reads_load_entry_and_neighbors`
in `src/lib.rs`. That validates decode at the entry-and-neighbors boundary, which is
where the scan descriptor meets graph layout. Good anchor point for future regression
tests.

### Observation

The `GraphTupleRef` enum is the right long-term abstraction. Resist the temptation to
add format-conditional branches at any callsite — push them into accessors on this
type.
