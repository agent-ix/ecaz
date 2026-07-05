# Review Request: Task 68 Top-Graph Distance Cache

Code commit: `fe7d5e6892dc1e7154eb95d8e620b22bef070d10`

## Scope

This is the next Task 68 Phase 2 slice after packet
`005-zero-replica-fast-path-measurement`. That packet moved the 100k build from
`22482 ms` to `3362 ms`; the largest remaining SPIRE-specific phase was
top-graph/object-store at `935 ms` (`27.8 %` of total).

This slice keeps the change local to `src/am/ec_spire/build/top_graph.rs`:

- Precompute an in-memory `node_count * node_count` centroid distance matrix
  before invoking the Vamana builder.
- Fall back to the previous direct distance closure if the matrix would exceed
  a 64 MiB cap.
- Preserve the same IP-derived pseudo-distance expression and f32 accumulation
  order for each pair.
- Add a unit test proving matrix distances match the direct distance function
  exactly on a representative small graph.

No `unsafe` was added.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz --lib am::ec_spire::build --no-default-features --features pg18`

Result:

```text
test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 1875 filtered out
```

## Reviewer Ask

Please verify the memory cap and determinism argument. If this code packet is
acceptable, the follow-up measurement packet should repeat the same 10k/100k
Task 68 suite and compare against packet 005's fast-path split.
