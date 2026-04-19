# Review Request: C1 Native Build Backlink Score Cache

Current head at execution: `fcfffd0`

## Context

Packet `455` removed repeated query-side scoring while the native serial
builder searches for forward neighbors.

The next repeated-work pocket was backlink rewrite planning. After the pending
selection list is sorted by target node and layer, the builder can hit the same
target node multiple times in sequence and rescore the same `(target, candidate)`
pairs for each rewritten layer slice.

## What changed

In `src/am/build.rs`:

1. Added `NativeBacklinkTargetScorer`, a tiny cache scoped to one backlink
   target node.
2. Reused that cache across consecutive pending backlink rewrites for the same
   `selection.node_idx`.
3. Left the existing pending ordering and backlink replacement logic unchanged.

The only work reduced here is repeated `metric.score_between(state, target, candidate)`
calls while one target node absorbs multiple backlink updates.

## Why this is safe

- No persisted page or tuple layout changed.
- No rewrite ordering changed.
- No candidate admission or tie-break rule changed.
- The cache is reset whenever the pending stream advances to a new target node.

This is still strictly serial native BUILD behavior preservation with less
repeated rescoring.

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Validation ran sequentially for this checkpoint.

## Review focus

1. Is target-local backlink rescoring cache the right next optimization seam, or
   do you want to stop here and keep subsequent changes focused on measurement?
2. Do you want a direct unit test for the cache reuse seam itself, or is the
   existing helper coverage sufficient because the ranking logic is unchanged?
