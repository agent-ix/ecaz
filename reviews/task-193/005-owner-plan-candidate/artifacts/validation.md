# Task 193/194 candidate validation

- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo
  clippy --all-targets --no-default-features --features "pg18
  distann-head-attribution-benchmark" -- -D warnings`: passed.
- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo
  build -p ecaz-cli`: passed (one unrelated pre-existing dead-code warning in
  `corpus/load.rs`).
- Focused `ecaz-cli` owner-plan/fixed-work parser and suite-addressing tests:
  2 passed.
- Both checked-in candidate suites passed `ecaz bench suite audit`.
- `cargo pgrx install --release --pg-config
  /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features
  --features "pg18 distann-head-attribution-benchmark"`: passed.
- `target/release/libecaz.so`: 24,244,984 bytes,
  `ec58009be20adf9db45af01fcc9bf0a947b9ec893ee6541f9c47d194f5ea8031`.
- Installed `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`:
  24,244,984 bytes, same SHA-256. The installed binary is therefore the
  release artifact, not the debug-profile library implicated by packet 005.
