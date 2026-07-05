# Request: Graph-Owned Layer-0 Successor Candidates

Commit: `23f9ee5`

Summary:
- add `load_layer0_successor_candidates` in `src/am/graph.rs`
- move layer-0 successor `BeamCandidate` construction out of `src/am/scan.rs`
- keep the bootstrap refill width cap in `scan.rs`, after graph-owned successor discovery

Please review:
- whether `graph.rs` is now the right owner for layer-0 successor candidate discovery
- whether the `keep_neighbor_tid` plus score-closure split is the right boundary between graph loading and scan-owned visited/scoring policy
- whether keeping the width cap in `scan.rs` is still the correct contract for the next traversal-wiring step
