# Artifact manifest

- Head SHA: `754eb7b911bf5aa5e2c6e7d4adb8213d03ff5b06`
- Implementation commit: `754eb7b91` (`fix(distann): reject remote system columns`)
- Task bucket / packet: `reviews/task-179/046-system-column-projection`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local PG18 planner safety and three-owner physical read regression
- Fixture: one coordinator/source control plus three physical owner controls,
  loopback transport, one Published generation distributed across all owners
- Storage format: distributed-control physical graph/row/directory relations
- Rerank mode: exact frozen-row materialization
- Isolation surface: isolated source and owner-shell tables; no shared-table
  benchmark surface
- Timestamp: `2026-07-12T22:01:34-07:00`

The validation logs were produced from the exact code tree committed as
`754eb7b91`. This is a fail-closed planner correctness change; no corpus or
benchmark data is involved.

## Commands and results

### Focused live PG18 fixture

```text
cargo pgrx test pg18 test_distann_three_owner_physical_handoff
```

The existing fixture builds, publishes, and serves one generation across three
physical owners through `EcDistannDistributedScan`. It now additionally plans
two negative queries:

- target-list projection of `ctid`; and
- base qual on `xmin`.

Both must fail during planning with `EC_UNSUPPORTED_PROJECTION`; the normal
three-owner frozen-row query still serves all 30 rows across owner ordinals
0/1/2.

Result: `test result: ok. 1 passed; 0 failed; ...`. See
`system-column-projection-pg18.log`.

### Production PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0, `Finished dev profile ...`. See `clippy-pg18.log`.

## Artifact index

- `system-column-projection-pg18.log`: complete focused pgrx build/install and
  live test transcript, including the final one-test pass.
- `clippy-pg18.log`: production-feature warnings-denied lint transcript.

No PostgreSQL server log, corpus, truth cache, run directory, or polling output
is committed.
