# Task 167 packet 038 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `038-production-counter-separation`.
- Code checkpoint: `a49ffd92a`.
- Trigger: packet 037's first 10k attempt requested benchmark-only query-stage
  SQL functions from a production `pg18` extension build.
- Change isolation: CLI preflight and physical benchmark measurement harness;
  no extension product code changed.
- Behavior: Task 167 production insert-work counters are always reset and
  captured for both append A/B arms. The query-stage counter switch is
  validated independently against the extension feature list before the
  fixture starts an expensive corpus build.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 5 passed, 0 failed, 497 filtered in
  `validation-test.log` (SHA-256
  `04a541f88f8c48a9ad5ddae9a3f9660d05542c02d84cc52f2d1dd75eb61bc9c8`).
- Runtime build, audit, and corrected real-corpus evidence are pending.
