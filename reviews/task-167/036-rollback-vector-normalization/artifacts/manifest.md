# Task 167 packet 036 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `036-rollback-vector-normalization`.
- Harness checkpoint: `caa8ad63f`.
- Trigger: packet 035 passed the clean-head synthetic gate but the isolated
  `mi` rollback index emitted the unit-normalization warning.
- Change isolation: CLI fixture SQL only; no extension product code changed.
- Coverage: initial rollback corpus, injected failing insert, and stable-id
  replacement UPDATE use the shared deterministic unit-vector expression.
- Validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- Validation result: 2 passed, 0 failed, 497 filtered in
  `validation-test.log` (SHA-256
  `3045f30a93787d345fc301ee75f0abf43f5ddeec4aa7f52fcb290d58044e754f`).
- Runtime status: pending final clean-head release rebuild/install and suite
  confirmation. No recall, latency, or storage closeout result is claimed yet.
