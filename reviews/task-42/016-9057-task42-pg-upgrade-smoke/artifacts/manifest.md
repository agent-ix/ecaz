# Artifact Manifest: Task 42 PG Upgrade Smoke

- head SHA: `a4be6a557bfcae06f4e8e12c9c28d5773e3f13da`
- packet/topic: `9057-task42-pg-upgrade-smoke`
- timestamp: `2026-05-17T22:05:12Z`
- lane: Task 42 PG18 same-binary `pg_upgrade` smoke
- fixture: `ecaz_pg_upgrade_smoke` table, 4 `ecvector(4)` rows, one `ec_hnsw` index
- storage format: HNSW default storage format
- rerank mode: not applicable
- surface isolation: isolated one-index-per-table surface

## Artifacts

| File | Command | Key Result |
| --- | --- | --- |
| `make-pg-upgrade-smoke.log` | `make pg-upgrade-smoke PG_UPGRADE_SMOKE_FLAGS="--skip-install --run-dir target/pg-upgrade-smoke-task42-local-3 --smoke-log target/pg-upgrade-smoke-task42-local-3.log"` | `pre_top2=1,2`; `post_top2=1,2`; `pre_index_count=1`; `post_index_count=1`; `pg_amcheck=passed`; `PG18 pg_upgrade smoke passed` |
| `old-postgres.log` | emitted by `scripts/run_pg_upgrade_smoke_pg18.sh` | source PG18 cluster started cleanly for fixture creation |
| `new-postgres.log` | emitted by `scripts/run_pg_upgrade_smoke_pg18.sh` | upgraded PG18 cluster started cleanly for post-upgrade checks |
| `bash-n-pg-upgrade-smoke.log` | `bash -n scripts/run_pg_upgrade_smoke_pg18.sh` | shell syntax check passed |
| `cargo-fmt-check.log` | `cargo fmt --all -- --check` | rustfmt check passed; existing stable-toolchain warnings about unstable rustfmt options are present |
| `cargo-test-cli-pg-upgrade-parse.log` | `cargo test -p ecaz-cli cli_parses_pg_upgrade_smoke_command` | `1 passed`; existing unused-import warning in `src/am/mod.rs` is present |
