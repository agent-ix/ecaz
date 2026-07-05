# Review Request: Concurrent DSM striped graph insertion

## Summary

Code checkpoint: `654ebb1` (`Stripe concurrent DSM graph insertion`)

This checkpoint changes the concurrent DSM graph insertion scheduler from one contiguous node range per participant to small striped global-order chunks:

- `insert_concurrent_dsm_graph_participant` now inserts multiple `[start, end)` chunks for its participant.
- The chunks interleave across participants in node-index order using a fixed 64-node stripe chunk.
- Existing contiguous partitioning remains available and covered for the lower-level partition insertion helper.

The intent is to reduce the topology skew seen in the tuned recall packet where one participant could run far ahead of the serial prefix while searches skip not-yet-ready DSM nodes. Striping keeps workers parallel, but bounds how far any participant's local insertion range can drift from the global insertion order.

## Review Focus

Please review:

- Whether the striped partition helper covers every node exactly once and has reasonable overflow / bounds handling.
- Whether using fixed 64-node chunks is acceptable as an initial policy for this checkpoint.
- Whether `insert_concurrent_dsm_graph_participant` should keep summing inserted counts across partitions, or whether there is a better progress-accounting shape before measurement work continues.

## Validation

Run on PG18 only, per current branch target:

- `cargo test concurrent_dsm_node_striped_partitions -- --nocapture`
- `cargo test build_parallel -- --nocapture`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_graph_opt_in`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18`
- `git diff --check`

All passed.

## Follow-up

This packet does not claim recall or latency improvement. The next packet should rerun the tuned recall validation against this striped scheduler and compare it to packet 652, especially the high-ef behavior where the prior concurrent DSM graph flattened at `ef_search=400`.
