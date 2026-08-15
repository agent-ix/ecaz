# Task 167 packet 028 artifacts

- Packet: `reviews/task-167/028-production-ab-concurrency`.
- Product head: `8f0334661ab05149190dfb10a6d8c8dff9947508`.
- Code change: remove the `pg_test` compile gates around remote propagation of
  `ec_distann.debug_disable_append_when_room`; production owner sessions now
  receive the coordinator's A/B setting.
- Harness/config change: packet 026's suite config enables concurrency drills
  at 50k and 100k and uses fresh external run-directory names.
- Validation commands, both passed:
  `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check --no-default-features --features pg18`
  and
  `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check --no-default-features --features pg18,pg_test`.
- Runtime status: no current-head result is claimed. The required external
  cluster root `/home/peter/.ecaz/clusters` is on a read-only filesystem, so
  fixture setup fails before preflight.
- No benchmark artifacts exist for this checkpoint; the next run must use
  `ecaz bench suite` and store its manifest/results under packet 026.
