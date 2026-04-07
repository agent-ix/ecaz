# Request: Graph-Owned Layer-0 Neighbor Loader

Commit: `e530a97`

Summary:
- add `load_layer0_neighbor_tids` in `src/am/graph.rs`
- move layer-0 neighbor tuple loading/filtering out of `src/am/scan.rs`
- keep successor-width limiting in scan selection so refill behavior stays unchanged after visited/deleted filtering

Please review:
- whether `graph.rs` is now the right owner for layer-0 neighbor tuple loading
- whether keeping width limiting in `scan.rs` is the right choice to preserve refill behavior after visited/deleted filtering
- whether this is a useful A2-forward primitive for later `BeamSearch` traversal wiring
