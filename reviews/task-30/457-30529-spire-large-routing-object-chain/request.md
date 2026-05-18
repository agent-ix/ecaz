# Review Request: SPIRE Large Routing Object Chain

- Code commit: `56633ec3` (`Chain large SPIRE routing objects`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Agent: coder1

## Summary

The real 10k SPIRE recall gate exposed a hard build blocker: a 1536-dimensional
root routing object with `nlists = 32` encodes to roughly 197 KB, which cannot
fit in one relation object tuple.

This checkpoint keeps the existing routing object V1 logical codec, but stores
oversized relation-backed routing objects through a V2 chain:

- the placement points at a compact routing-chain metadata tuple;
- page-sized chain segment tuples store chunks of the original V1 bytes;
- reads reconstruct the V1 byte stream and pass it through the existing
  `SpireRoutingPartitionObject::decode` path;
- `read_object_header` and active tuple locator accounting understand the chain
  metadata so diagnostics and future cleanup do not lose segment tuples.

Small routing objects still use the original single-tuple path.

## Review Focus

1. Confirm that preserving the V1 logical routing bytes behind a V2 physical
   chain is the right minimal format change.
2. Check the segment tuple sizing policy, especially the conservative 7000-byte
   cap used to stay below PostgreSQL FSM request limits.
3. Verify that active object tuple locators include routing chain segments.

## Validation

- `cargo test local_store_relation_plan --lib`
- `cargo pgrx test pg18 test_ec_spire_large_routing_object_builds_and_scans`
- `cargo fmt --check`
- `git diff --check`

## Notes

This packet was created before rerunning the measured SPIRE recall/latency gate
so reviewers can inspect the storage fix separately from benchmark results.
