# Task 167 packet 040 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `040-diagnostic-instrumentation-isolation`.
- Code checkpoint: `44de4a131`.
- Feedback addressed:
  `reviews/task-167/039-post-insert-parity-gate/feedback/2026-08-22-01-reviewer.md`
  sections 5 and 7.3.
- Change isolation: extension diagnostic counters/GUC plus fixture enablement
  and evidence labels; no DistANN graph algorithm changed.
- Extension validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz stage_and_insert_resets_are_independent --lib --no-default-features --features pg18 --quiet`.
- Extension validation result: 1 passed, 0 failed, 2,574 filtered in
  `extension-validation-test.log` (SHA-256
  `c5c5b9d2e9e446ebd7e6c2730b77cd0f0b16fde838c744eef51b5ca180fc4ba6`).
- CLI validation command:
  `env CARGO_TARGET_DIR=/home/peter/.cargo-target cargo test -p ecaz-cli task167_ --no-default-features --quiet`.
- CLI validation result: 8 passed, 0 failed, 497 filtered in
  `cli-validation-test.log` (SHA-256
  `86ebad18cce57c2ccb1cc465263ebd89e5e0b10bb336e3feb26b7cc7dde9ff98`).
- Insert counter scope: per coordinator backend only. Remote-owner backend
  work is not included in the coordinator snapshot and all emitted insert-work
  and append A/B lines now say so explicitly.
- Runtime build and matrix evidence are pending.
