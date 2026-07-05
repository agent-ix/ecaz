# Task 65b Packet 007: Digest Diagnostics

## Scope

This packet reviews commit `397e57a1b921fddfb8a4c588e77493186ab71d37`, which adds stable digest fields to DiskANN graph diagnostics so Task 65b fallback and tuning packets can compare build output shape directly.

The change:

- computes `live_node_tid_digest` over persisted live node TIDs;
- computes `adjacency_digest` over each live node's TID, liveness flags, heap TIDs, neighbor count, and full neighbor slot vector;
- computes `first_256_node_digest` over the first 256 persisted node tuples as a compact page-shape proxy;
- exposes the three fields through `ec_diskann_index_graph_summary`;
- renders those fields from `ecaz bench diskann-graph`.

## Validation

Packet-local evidence is recorded in `artifacts/manifest.md`.

- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - passed
- `cargo test -p ecaz-cli graph::tests::render_summary_includes_reachability_and_degree_rows`
  - passed

## Review Notes

This is an enabling checkpoint for the carried Slice D flag and Acceptance #6. It does not claim worker-zero corpus byte equality yet; it gives the next packet a stable digest surface to run against real10k / real100k fallback builds without ad-hoc SQL or local-only scripts.
