# Task 167 packet 029 artifacts

- Packet: `reviews/task-167/029-parity-arm-attribution`.
- Product/harness head: `71396d0e6`.
- Changed source: `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`.
- New parity fields: `append_disabled_recall`, `append_enabled_recall`, and
  `append_enabled_minus_disabled`; the enabled arm and overall parity form the
  pass condition.
- Validation commands, both passed:
  `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check --no-default-features --features pg18`
  and
  `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check --no-default-features --features pg18,pg_test`.
- Runtime status: no result claimed. External fixture initialization is
  blocked by the read-only `/home/peter/.ecaz/clusters` filesystem.
