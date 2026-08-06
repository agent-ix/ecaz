# Task 215 production-wiring manifest

- Task bucket: `reviews/task-215/002-production-wiring/`
- Code checkpoint: `ea51a9c8b`
- Disposition: candidate checkpoint superseded by packet 003 STOP; not shipped
- Binary profile: normal PG18 release; no attribution feature
- Changed source: `src/am/ec_distann/mod.rs`,
  `src/tests/ec_distann_basic.rs`
- Defaults: BW 64, H 8, L 32; production head derivation yields 128 seeds
- Normal schema artifact: `artifacts/normal-schema.sql`; no attribution-only
  debug entry point is present
  at BW64 and 32 seeds at BW4
- Validation command: `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check --no-default-features --features pg18`
- Install command: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- Validation result: both commands passed

The repository-wide `cargo fmt --all -- --check` remains red on unrelated
pre-existing formatting drift outside this checkpoint; `git diff --check` for
the touched files passed. The normal-release A/B is packet 003.
