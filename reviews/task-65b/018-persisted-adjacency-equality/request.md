# Packet 018: Persisted Adjacency Equality

## Summary

This packet addresses the remaining packet 005 B-1 carryover: the build-level Task 65b scaffold tests now assert full persisted adjacency equality, not only metadata, BFS order, and degree summaries.

Commit under review:

- `7909480ee` - `Assert DiskANN persisted adjacency equality`

## What Changed

`src/am/ec_diskann/build.rs` now has a test helper:

- `decoded_node_adjacency(&BuildOutput) -> Vec<Vec<u32>>`

The helper decodes each persisted `VamanaNodeTuple`, takes the filled neighbor prefix, maps each neighbor TID back to its node id through `node_to_tid`, and returns the full per-node adjacency list.

The helper is asserted in:

- `task65b_worker_one_scaffold_matches_serial_output`
- `task65b_worker_zero_config_matches_plain_serial_output`

This directly covers the reviewer concern that matching `entry_point`, `node_to_tid`, `persistence_order`, `medoid`, and `final_in_degree` could still allow divergent per-node adjacency.

## Validation

Artifacts:

- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-test-build-task65b.log`
- `artifacts/manifest.md`

Commands:

- `cargo fmt --check`
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b`

Result:

- `cargo fmt --check`: pass
- focused build tests: `6 passed; 0 failed; 0 ignored; 0 measured; 1970 filtered out`

## Reviewer Mapping

Packet 005 seq 01 required:

- Add `assert_eq!(serial.graph.neighbors, worker_one.graph.neighbors)` at batch=1.
- Add the same adjacency-level proof for the worker-zero/fallback serial path.

The production `BuildOutput` intentionally does not retain the raw in-memory `VamanaGraph` after persistence, so this packet checks the stronger persisted form: decoded on-page neighbor slots mapped back to node ids. That proves the graph adjacency that the index actually stores is byte-shape equivalent for the compared paths.

## Remaining Outside This Packet

- Packet 001 measurement-floor carryovers remain for closeout.
- The real-100k recall-edge decision from the late batch sweep remains for closeout.
