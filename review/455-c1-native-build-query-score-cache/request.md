# Review Request: C1 Native Build Query Score Cache

Current head at execution: `e15624b`

## Context

This checkpoint is the first post-replacement native BUILD optimization slice.

The native serial builder was still recomputing the same query-to-node graph
scores several times while inserting one new node:

- entry candidate scoring
- upper-layer successor expansion
- layer-0 successor expansion

That repeated work was especially wasteful for higher `ef_construction`
settings because one insertion revisits the same existing nodes across multiple
search phases.

## What changed

In `src/am/build.rs`:

1. Added `NativeBuildQueryScorer`, a per-insertion cache keyed by existing node
   index.
2. Threaded that scorer through:
   - native entry candidate creation
   - upper-layer successor loading
   - layer-0 successor loading
3. Left backlink rewrite scoring unchanged. That path scores from the target
   node’s perspective, so it does not share the same query index and should not
   reuse this cache.

## Why this is safe

- No persisted page or tuple layout changed.
- No tie-break ordering changed.
- The cached value is exactly the prior `metric.score_between(...)` result for
  the same `(new_node_idx, existing_node_idx)` pair.
- Backlink selection semantics remain untouched.

This is strictly a repeated-work reduction inside one serial BUILD insertion.

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Validation ran sequentially for this checkpoint.

## Review focus

1. Does the cache scope look right to you: one insertion, query-side only,
   excluding backlink rewrites?
2. If this looks good, the next likely optimization surface is backlink rewrite
   rescoring rather than the main search path.
