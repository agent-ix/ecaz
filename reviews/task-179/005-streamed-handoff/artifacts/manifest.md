# Task 179 packet 005 artifact manifest

- Head SHA: `c36aada77`
- Task bucket: `reviews/task-179/005-streamed-handoff/`
- Lane: PG18 debug validation
- Fixture: focused physical-generation handoff and frozen-source fixtures
- Storage format: ec_distann v5 control / physical generation v1
- Rerank mode: not applicable
- Isolation: one logical control and one isolated physical generation per fixture; no shared-table benchmark surface
- Timestamp: 2026-07-10 America/Los_Angeles

## Artifacts

### `cargo-test-handoff-core.log`

- Command: `cargo test --lib handoff_ --no-default-features --features pg18`
- Purpose: canonical wire, restartable owner hash, bounded router, source spool, and participant helper unit coverage.
- Key result: `13 passed; 0 failed` (12 ec_distann handoff/router/hash cases plus one existing shared handoff-lifecycle filter match).

### `cargo-test-format-fixtures.log`

- Command: `cargo test --test on_disk_fixtures distann_ --no-default-features --features pg18 && cargo test --test upgrade_matrix --no-default-features --features pg18`
- Purpose: TC-050 independent decoders, golden/layout/endian coverage, and compatibility-matrix validation.
- Key result: `13 passed; 0 failed` DistANN independent-format fixtures; `2 passed; 0 failed` upgrade-matrix checks.

### `cargo-check-pg18.log`

- Command: `cargo check --lib --no-default-features --features 'pg18 pg_test'`
- Purpose: production and pg_test compile surface.
- Key result: exit 0; PG18 production + pg_test library compile completed.

### `cargo-pgrx-test-streamed-handoff.log`

- Command: `cargo pgrx test pg18 test_distann_s`
- Purpose: one installation followed by the three focused PG18 cases whose names begin `test_distann_s`: participant stage, participant seal, and frozen-source capture/graph/routing.
- Key result: `3 passed; 0 failed`: atomic stage/replay/directory, physical seal/Ready replay including empty owner, and frozen source capture/graph/two-pass routing.

No corpus TSV, SSM output, tunnel state, polling exhaust, or temporary PostgreSQL files are committed. PostgreSQL `BufFile` contents are transaction/resource-owner scoped and deliberately absent from the packet.
