# Request: Graph Beam Search Batch Review (158-165)

Batch review covering the graph-owned layer-0 beam search implementation.

Scope:
- Review 158: graph-owned layer-0 neighbor loader
- Review 159: graph-owned layer-0 successor candidates
- Review 160: graph-owned layer-0 beam search runner
- Review 161: seed bootstrap entry frontier from layer-0 beam trace
- Review 162: refill bootstrap frontier from layer-0 beam trace
- Review 163: seed bootstrap frontier directly from layer-0 beam trace
- Review 164: unify entry seeding on layer-0 beam trace
- Review 165: beam-driven bootstrap top-up from visible frontier

Files:
- `src/am/graph.rs`
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
