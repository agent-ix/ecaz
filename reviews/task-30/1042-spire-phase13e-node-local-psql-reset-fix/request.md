# Task 30 Review Request: Node-Local psql Reset Fix

## Summary

This fixes the node-local representative load failure captured in packet 1041. The AWS node-local reset/drop SQL path used `ecaz dev sql`, which depends on `.pgrx` and is appropriate for the operator host, not for EC2 nodes.

Changes:

- `load_remote_shards_node_local` now writes the remote drop SQL to a node-local `.sql` file and executes it with the node's PostgreSQL `psql`.
- `load_coordinator_representative_node_local` now writes the coordinator reset SQL to a node-local `.sql` file and executes it with `psql`.
- `preflight-representative-performance.sh` now fails closed unless the representative load script contains the node-local `psql` path.

`ecaz` remains the tool for node-local corpus load and inspect operations.

## Validation

- `artifacts/bash-n-rerun.log`
- `artifacts/representative-preflight-rerun.log`
- `artifacts/git-diff-check-rerun.log`

Key result:

```text
SPIRE representative performance preflight passed
```

No AWS rerun was started in this packet.
