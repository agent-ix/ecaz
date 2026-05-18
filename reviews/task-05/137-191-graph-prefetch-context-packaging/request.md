# Review Request: Graph Prefetch Context Packaging

## Summary

- package the live graph prefetch frontier-selection operations behind a narrow `GraphTraversalPrefetchContext` in `src/am/scan.rs`
- remove the remaining raw closure bundle that reached back into `opaque` from `select_next_with_refill(...)`
- keep staged A3 behavior unchanged while tightening the last scan-owned graph selection shell

## What changed

- added `GraphTraversalPrefetchContext` to own the packaged graph-prefetch operations around:
  - candidate materialization into `GraphTraversalCursor`
  - emitted-element marking
  - expanded-source checks / marking
  - single-source refill
  - visible-seed top-up
- `prefetch_next_graph_traversal_result(...)` now:
  - builds a narrow context from `index_relation` and `opaque`
  - passes one packaged context runner into `with_visible_frontier_mut_and_bootstrap_expansion(...)`
  - delegates `select_next_with_refill(...)` through context methods instead of inline raw-pointer closures

## Why

- Review batch 182-190 called out the remaining `select_next_with_refill(...)` closure bundle as the next A3 ownership cut.
- After the previous direct-materialization slice, the graph path still reached back into scan-owned state through several inline closures for emitted/expanded/refill/top-up operations.
- This is the next bounded step: the live graph prefetch path now packages those operations behind one narrow context, which makes the remaining frontier-selection boundary more explicit without changing runtime behavior or exhaustion semantics.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review focus

- whether `GraphTraversalPrefetchContext` is the right near-term boundary for the remaining graph selection operations
- whether the context cleanly addresses the reviewer concern about raw closure reach-back into `opaque`
- whether the next A3 cut should move this packaged boundary out of `scan.rs`, or stop here and treat exhaustion policy as A4 scope
