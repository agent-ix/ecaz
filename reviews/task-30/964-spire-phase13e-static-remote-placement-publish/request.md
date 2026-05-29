# Review Request: Static Remote Placement Publication

Task: Task 30 Phase 13e SPIRE AWS Production Gap Closure

Code commit: `6b4c1cfb10185d92fd4d77dcb4d8494a8725f582`

## Summary

This slice adds the first production coordinator placement publication surface
for Phase 13e:

- New SQL function `ec_spire_publish_static_remote_placement_nodes(index_oid, pids, node_ids)`.
- The function republishes the active internal SPIRE placement directory with
  explicit remote `node_id` ownership, rather than using
  `tests.ec_spire_test_rewrite_placement_node`.
- It rejects empty input, duplicate PIDs, local node assignments, negative
  values, length mismatches, and missing PIDs.
- `scripts/spire-aws/load.sh` now builds the coordinator SPIRE index before
  writing the distributed placement plan and loading remote shards.
- `scripts/spire-aws/register.sh` now publishes remote leaf ownership after
  descriptor registration and captures both remote-node and coordinator
  placement snapshots.
- New `scripts/spire-aws/publish-remote-placements.sql` assigns active
  coordinator leaf PIDs across plan remotes and calls the production function.

This removes the test-only placement rewrite dependency from the AWS operator
path. It is still not the final remote row materialization primitive: the
current script assigns leaf ownership round-robin after coordinator build. The
next slice must make remote shard materialization leaf-owned, so remote indexes
contain the exact leaf PIDs the coordinator routes.

## Evidence

See `artifacts/manifest.md`.

- `artifacts/cargo-check-ecaz-lib.log`: extension library compile passed.
- `artifacts/cargo-check-ecaz-cli.log`: CLI compile passed with one existing
  dead-code warning.
- `artifacts/bash-syntax-spire-aws.log`: AWS shell syntax passed.
- `artifacts/cargo-test-ecaz-lib-pg-symbol-limited.log`: attempted lib test
  execution compiled but could not run outside PostgreSQL due missing
  `LockBuffer`; not counted as behavioral proof.

## Reviewer Notes

Please focus on whether the operator function has the right safety boundaries
for production use, and whether the AWS script ordering is now correct:
coordinator build, distributed plan, remote shard load, descriptor registration,
then internal placement publication.
