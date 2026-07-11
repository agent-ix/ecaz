# Task 179 packet 005 artifact manifest

- Head SHA: `417d169859a1c9e37abdd885bf44cc811c21d14a`
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

### `cargo-test-handoff-fixes.log`

- Command: `cargo test --lib --no-default-features --features pg18 am::ec_distann::handoff`
- Purpose: final canonical wire, restartable hash, single-representation router, exact-retry, and real 8 MiB multi-owner capacity coverage at remediation HEAD.
- Key result: `12 passed; 0 failed`, including retained-capacity minus/exact/plus-one and unchanged retry bytes.

### `cargo-pgrx-test-streamed-handoff-fixes.log`

- Commands: initial `cargo pgrx test pg18 test_distann_s`, focused remediation reruns, then final `cargo test --lib --features 'pg18 pg_test' --no-default-features test_distann_s`.
- Purpose: installed PG18 validation for participant stage/seal, hostile-domain role switching, the zero-mutation matrix, supplied-MVCC/HOT source capture, and deterministic absent/vector/identity callback faults.
- Key final result: `6 passed; 0 failed` in the final combined run. Earlier failed fixture iterations remain in this log as an explicit debugging audit trail; they were corrected before the cited final run.

### `cargo-pgrx-test-streamed-handoff-head.log`

- Command: `cargo test --lib --features 'pg18 pg_test' --no-default-features test_distann_s`
- Purpose: clean exact-HEAD rerun of the six installed PG18 cases after the final router error-rollback audit.
- Key result: `6 passed; 0 failed` at `417d169859a1c9e37abdd885bf44cc811c21d14a`.

### `cargo-clippy-pg18-fixes.log`

- Command: `cargo clippy --lib --no-default-features --features 'pg18 pg_test' -- -D warnings`
- Purpose: strict production plus pg_test static validation at remediation HEAD.
- Key result: exit 0 with no warnings.

No corpus TSV, SSM output, tunnel state, polling exhaust, or temporary PostgreSQL files are committed. PostgreSQL `BufFile` contents are transaction/resource-owner scoped and deliberately absent from the packet.
