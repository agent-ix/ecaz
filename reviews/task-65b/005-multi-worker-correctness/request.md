# Task 65b Packet 005: Multi-Worker Correctness

## Scope

This packet reviews commit `374922cc1a9b4024323b4d3374105788bd848942`, which moves the Task 65b worker scaffold from worker-one parity into a deterministic multi-worker Vamana graph-build stepping stone.

The change:

- Adds `build_vamana_graph_with_parallel_epochs`, which evaluates each epoch's pivot proposals in parallel against an immutable neighbor-cache snapshot.
- Commits proposals through a single deterministic reducer ordered by original Vamana permutation ordinal.
- Allows `requested_workers > 1` and `batch_size > 1` through `BuildParallelConfig`.
- Keeps nonzero `flush_rate` rejected until the explicit flush-cadence slice lands.
- Extends parallel build stats and the ambuild timing NOTICE with epoch, proposal, reducer, and same-epoch candidate-read counters.
- Replaces the prior multi-worker rejection test with deterministic multi-worker coverage and adds a batch-size epoch-count test.

## Validation

Packet-local evidence is recorded in `artifacts/manifest.md`.

- `cargo test -p ecaz --lib --no-default-features --features pg18 task65b_`
  - `5 passed; 0 failed`
- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - passed

## Review Notes

This is still a correctness-first slice. The reducer is intentionally serial and each epoch clones the current neighbor cache to keep the concurrency semantics reviewable before tuning.

Not yet claimed by this packet:

- flush cadence / batch tuning
- fallback-path corpus smoke and byte-equality evidence
- real10k / real100k performance gates
- recall-gate and scaling-curve measurement packet
