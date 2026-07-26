# Task 38 Packet 007 Artifact Manifest

- Code checkpoint: `374166bd3c88ddabf85f42d9fffb1ac0ed1fd7bc`
- Task bucket: `reviews/task-38/`
- Packet: `007-m5-mutation-controls`
- Host: local Apple M5, macOS arm64
- PostgreSQL: local PG18, Unix socket `/Users/peter/.pgrx`, port `28818`
- Database: `ecaz_fault_task38`
- Fixture isolation: one index per fixture table
- Remote/AWS/CI execution: none
- Run timestamp: `2026-07-26T20:32:07Z`

## Artifacts

### `all-fixtures-mutation-control-live.log`

- Command:
  `target/debug/ecaz --database ecaz_fault_task38 --host /Users/peter/.pgrx
  --port 28818 --log-file
  reviews/task-38/007-m5-mutation-controls/artifacts/all-fixtures-mutation-control-live.log
  dev fault mutation-control --rows 16`
- Exit: `0`
- SHA-256:
  `4937ee09955011f8f1a8d4046d46eed0aa64578d8dd8f7fd272372cbe9cd729a`
- Lines: `58`
- Lane / fixture / storage / rerank:
  - lane: cancellation and resource/palloc mutation controls
  - fixtures: HNSW, IVF, DiskANN, SPIRE, DistANN RaBitQ, DistANN TurboQuant,
    DistANN grouped-PQ
  - storage: real isolated PostgreSQL heap/index relations, one index per table
  - rerank: fixture-default; no benchmark rerank comparison
- Key results:
  - `7` `cancellation_mutation_control ... normal_cancel_oracle=rejected`
    markers.
  - `7` `resource_palloc_mutation_control ...
    normal_recovery_oracle=rejected` markers.
  - `9` `pg_buffercache_fixture_pins_ok=true pins=0` markers.
  - Final:
    `mutation_control_complete kind=All fixtures=7
    clean_postconditions=true`.
- The local PG18 role exposed `pg_buffercache`, so fixture-pin checks executed.
  `pg_stat_io` and `pg_stat_wal` queries were unavailable in this database and
  were explicitly skipped by the existing optional-counter policy. Required
  session, lock, prepared-xact, real-AM recovery, and cancellation SQLSTATE
  oracles still executed.

### `static-validation.log`

- Commands and results:
  - nightly `rustfmt --check` on
    `crates/ecaz-cli/src/commands/dev/fault.rs`: pass.
  - `git diff --check 6cc24bf3e..374166bd3`: pass, no output.
  - `cargo check -p ecaz-cli --message-format short`: pass in `2m50s`; one
    existing dead-code warning in `corpus/load.rs`.
  - `cargo build -p ecaz-cli --message-format short`: pass; final incremental
    build after the recovery-oracle tightening completed in `31.73s`.
  - `cargo clippy -p ecaz-cli --no-deps --message-format short`: exit `0` in
    `20m25s`; repository-existing warnings remain, with no warning in the new
    mutation-control code.

### `postcondition-audit-live.log`

- Command:
  `target/debug/ecaz --database ecaz_fault_task38 dev sql --socket-dir
  /Users/peter/.pgrx --port 28818 --log-output
  reviews/task-38/007-m5-mutation-controls/artifacts/postcondition-audit-live.log
  --sql <the three exact required leak_probe_sql queries>`
- Exit: `0`
- SHA-256:
  `2b6408707b54a41052bf41c88f01ab49c59dbd8f276b2a7d980097924dced21a`
- Key results:
  - `fault_sessions 0`
  - `fault_locks 0`
  - `prepared_xacts 0`

## Evidence Boundary

This packet proves the two repeatable negative controls on the local Apple M5
PG18 surface only. It makes no claim for Linux LD_PRELOAD provider behavior,
SPIRE/DistANN remote socket injection, cgroup v2 OOM execution, Intel behavior,
CI, or nightly promotion. Task 38 remains open for that designated Intel/Linux
evidence.
