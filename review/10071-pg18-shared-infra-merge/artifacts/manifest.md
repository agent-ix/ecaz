# Artifact Manifest

Packet: `10071-pg18-shared-infra-merge`
Head: `b5f98fc`

This packet makes no measurement claims.

Validation cited in `request.md` was run directly from the working tree:
- `cargo test --no-default-features --features pg17` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo test` — blocked by local `pgrx` PG18 setup
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — blocked by local `pgrx` PG18 setup
- `cargo pgrx test pg18` — blocked by local `pgrx` PG18 setup

Current environment blocker:
- `~/.pgrx/config.toml` only manages `pg17`
- `~/.pgrx/18.3` exists as a source tree, but there is no built `pgrx-install/bin/pg_config`
- all attempted PG18 validation currently fails with `Postgres 'pg18' is not managed by pgrx`
