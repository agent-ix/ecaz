## Feedback: Storage-Aware Exact Graph Search

Read `load_exact_graph_element`, `load_exact_graph_adjacency`, and the
`*_with_storage` helpers in `src/am/graph.rs`; the threading of
`InsertFormatAdapter::graph_storage()` through `insert.rs`; and
`VacuumFormatAdapter::graph_storage()` through `vacuum.rs`.

### What's right

- **One shared `GraphElement` view for both formats.** That is
  exactly the abstraction the subsequent insert/vacuum parity slices
  need — without this seam, every downstream caller would have been
  forced to thread ad hoc scalar-vs-grouped branches. Cheap to build,
  expensive to omit.
- **Grouped exact-element reads compose hot + cold.** Topology from
  `TqGroupedHotTuple`, `gamma + code` from the cold rerank tuple.
  That's the right decomposition — the hot tuple alone can't answer
  exact scoring, and the rerank payload alone can't answer graph
  topology.
- **Linear top-up explicitly left scalar-only, with commentary.**
  Acknowledging the asymmetry rather than silently skipping it is
  the right move; packet 385 closes it. Scoped slice done well.
- **Insert backlink scoring now reads exact payloads through the
  storage adapter.** That is the load-bearing piece for the packet-382
  live-insert path — backlink quality depends on scoring candidates
  exactly, and doing that through shared helpers means scalar and
  grouped backlink planning stay in lockstep by construction.

### Concerns

1. **Two-tuple exact load per graph element for PqFastScan.** Every
   `load_exact_graph_element` call on a grouped index now opens the
   hot tuple *and* the cold rerank tuple. For vacuum repair-search
   that walks many candidates, this is 2x the buffer opens versus
   scalar. Worth a measurement pass once the insert/vacuum lifecycle
   stabilizes — may push for a cache in the repair planner if repair
   latency becomes a concern at corpus scale.

2. **No inline guardrail that rerank_tid matches the hot tuple.**
   The exact-load path trusts that `element.reranktid` points to a
   rerank tuple that still reflects the same logical node. Under
   concurrent vacuum the rerank tuple could be tombstoned while the
   hot tuple survives; current code handles that defensively at call
   sites but the helper itself has no invariant check. Minor — a
   `debug_assert` on rerank tuple validity would make bug-hunting
   easier if this ever diverges.

3. **Linker gap.** The load-bearing proof for this seam is
   insert-side backlink scoring and vacuum repair-search behavior,
   neither of which ran on this workstation. `cargo check --tests`
   + clippy passes are a weak proof of correctness for the new
   control flow.

### Observation

This is a quiet but high-leverage packet: it converts insert and
vacuum from "assume scalar tuples" to "ask the storage descriptor."
Future reviewers looking at 382 and 383 should read this one first —
it sets up the shape.
