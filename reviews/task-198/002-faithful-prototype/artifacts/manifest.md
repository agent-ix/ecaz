# Task 198 packet 002 artifact manifest

- Head SHA: `b9398a1ba`
- Task bucket: `reviews/task-198/002-faithful-prototype/`
- Lane: PG18 compile/unit validation; faithful benchmark-only prototype
- Storage format: FR-084 v1 heap `(vec_id, owner_ordinal, graph_record,
  exact_vector)` plus unique `vec_id` B-tree
- Rerank mode: exact final distance; owner-side payload materialization
- Topology: code/unit validation only; runtime three-owner evidence is owned by
  packets 003 and 004
- Timestamp: 2026-07-23 America/Los_Angeles

## Artifacts

### `pg18-unit.log`

Command:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test --no-default-features --features pg18,distann-head-attribution-benchmark traversal_replica
```

Key result: `3 passed; 0 failed`.

### `cli-unit.log`

Command:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo test -p ecaz-cli distann
```

Key result: `31 passed; 0 failed`.

No corpus data or benchmark result is claimed by this packet.
