# Task 231 first decision-run stall diagnostic

- Timestamp: `2026-08-30T02:21:00-07:00`.
- Measurement extension SHA: `1f88b553e140628e5d70f72632599f15705b1f25`.
- Suite receipt checkpoint: `641901239143a133cf027824efda171896afe18b`.
- Step: `task231-warm-10k-a-fixed-second`.
- Disposition: invalid/incomplete arm; interrupted after proving a distributed
  self-deadlock. It must not be used as A/B evidence and will be rerun from a
  fresh fixture after the defect is fixed and reviewed.

The control-first arm completed. The matched fixed-stride arm passed release,
checksum, build, topology, serving, recall, latency, and graph-diagnostic
checks, then stalled in the routed 160-row insert workload. The coordinator
backend remained in the outer `INSERT` while an owner-2 backend executing
`ec_distann_apply_physical_backlink` waited on the fixed node-store relation.

Read-only diagnosis on owner 2 (`127.0.0.1:46411`) showed:

```text
prepared xid=793 gid=ec_distann_insert_48227_1_2_1_1229_2
prepared lock=ShareRowExclusiveLock granted=true relation=_ecdz_node_17810_71717171717141718171717171717171 virtualxid=20/8
waiter pid=1190788 query=SELECT ec_distann_apply_physical_backlink(...)
waiter lock=ShareRowExclusiveLock granted=false relation=_ecdz_node_17810_71717171717141718171717171717171 virtualxid=20/13
```

The prepared transaction is an earlier remote write for the same top-level
insert. `FixedStrideDmlContext::open` retains its self-conflicting
`ShareRowExclusiveLock` through transaction end. The coordinator then issues a
backlink amendment to the same owner in a separately prepared transaction, so
the second request can never acquire the lock while the first prepared
transaction waits for the coordinator's commit callback. PostgreSQL cannot
detect this as a local deadlock because the coordinator dependency is outside
its lock graph.

Diagnostic commands were read-only `ecaz dev sql` queries against
`pg_stat_activity`, `pg_prepared_xacts`,
`ec_distann_remote_prepared_xact_intent`, and `pg_locks`. The suite was stopped
with SIGINT after the lock cycle was proven. `suite-manifest.json` correctly
retains only the precheck and control arm as succeeded; the fixed arm remains
pending.
