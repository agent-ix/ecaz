# Artifact manifest

- Head SHA: `2bf203e4c7ed8091932bd2c01b134591d21bca73`
- Implementation commit: `2bf203e4c` (`test(distann): add legacy seed benchmark control`)
- Task bucket / packet: `reviews/task-179/047-legacy-seed-benchmark-harness`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local PG18 legacy owner-scan control validation
- Fixture: one coordinator/source control plus three physical owner controls,
  loopback transport, one Published generation distributed across all owners
- Storage format: distributed-control physical graph/row/directory relations;
  the persisted bounded head remains built and stored in both feature modes
- Rerank mode: exact frozen-row materialization
- Isolation surface: isolated source and owner-shell tables; no shared-table
  benchmark surface
- Timestamp: `2026-07-12T22:38:49-07:00`

All validation commands below ran from the exact implementation head. This is
harness evidence, not the persisted-head performance A/B.

## Commands and results

### Feature-enabled live three-owner PG18 fixture

```text
cargo pgrx test pg18 --features distann-legacy-seed-benchmark test_distann_three_owner_physical_handoff
```

The fixture builds, publishes, and reads 30 rows distributed across three
physical owners. With the feature enabled it asserts
`ec_distann_physical_seed_strategy() = 'owner_scan'`, so the successful remote
read exercises local and remote full-owner seed scans before the unchanged
distributed graph search.

Result: `test result: ok. 1 passed; 0 failed; ...`. See
`legacy-seed-three-owner-pg18.log`.

### Structured suite parser regression

```text
cargo test -p ecaz-cli distann_physical_topology_and_gate_are_structured
```

Result: `test result: ok. 1 passed; 0 failed; ...`. The parsed physical recall
and latency rows both require `seed_strategy=persisted_head`. See
`suite-parser-test.log`.

### Production PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0, `Finished dev profile ...`. See `clippy-pg18.log`.

### Benchmark-feature PG18 clippy

```text
cargo clippy --lib --no-default-features --features "pg18 distann-legacy-seed-benchmark" -- -D warnings
```

Result: exit 0, `Finished dev profile ...`. See
`clippy-legacy-seed-pg18.log`.

## Artifact index

- `legacy-seed-three-owner-pg18.log`: complete focused pgrx build/install and
  live three-owner test transcript.
- `suite-parser-test.log`: focused CLI structured-row parser regression.
- `clippy-pg18.log`: production-feature warnings-denied lint transcript.
- `clippy-legacy-seed-pg18.log`: benchmark-feature warnings-denied lint
  transcript.

No corpus, truth cache, PostgreSQL server log, run directory, benchmark result,
or polling output is committed.
