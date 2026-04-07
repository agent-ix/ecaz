# Request: Graph-Owned Layer-0 Beam Search Runner

Commit: `55ee557`

Summary:
- add `run_layer0_beam_search` in `src/am/graph.rs`
- wire `BeamSearch::run()` to graph-owned layer-0 successor loading
- add a pure graph-layer traversal test that proves best-first expansion and remaining frontier order

Please review:
- whether `graph.rs` is the right owner for the first real layer-0 beam traversal runner
- whether returning `BeamTrace<ItemPointer>` is the right seam before scan starts consuming this traversal directly
- whether this is the right next A2 step toward replacing the current bootstrap refill loop with actual graph traversal
