# Artifact manifest

- Head SHA: `afcc2d6afe1b19092038115d0bac2112ba5401c2`
- Implementation commit: `afcc2d6af` (`perf(distann): read physical graph without SPI`)
- Task bucket / packet: `reviews/task-179/049-direct-graph-reader`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local PG18 physical graph-read correctness
- Fixture: one coordinator/source control plus three physical owner controls,
  loopback transport, one Published generation distributed across all owners
- Storage format: distributed-control physical graph heap plus unique btree
  `vec_id` directory, frozen row-tier heap
- Rerank mode: exact frozen-row materialization
- Isolation surface: isolated source and owner-shell tables; no shared-table
  benchmark surface
- Timestamp: `2026-07-13T01:24:39-07:00`

All commands below ran from the exact implementation head. This packet is code
correctness evidence; performance evidence is deliberately deferred to the
required canonical A/B suite.

## Commands and results

### Live three-owner PG18 fixture

```text
cargo pgrx test pg18 test_distann_three_owner_physical_handoff
```

The fixture builds and publishes 30 rows across all three physical owners,
executes coordinator-local and remote graph expansion, materializes frozen
rows, and verifies the normal distributed CustomScan result and topology.

Result: `test result: ok. 1 passed; 0 failed; ...`. See
`direct-graph-reader-pg18.log`.

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

- `direct-graph-reader-pg18.log`: complete focused pgrx build/install and live
  three-owner test transcript.
- `clippy-pg18.log`: production-feature warnings-denied lint transcript.
- `clippy-legacy-seed-pg18.log`: benchmark-feature warnings-denied lint
  transcript.

No benchmark result, PostgreSQL server log, corpus, truth cache, run directory,
or polling output is committed.
