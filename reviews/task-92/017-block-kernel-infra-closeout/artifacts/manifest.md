# Task 92 Packet 017 Artifact Manifest

Head SHA: `84ab8c433b987bc135a40fdc8ec7934e9ae66a76`

Task bucket: `reviews/task-92/017-block-kernel-infra-closeout/`

Timestamp: `2026-06-09T06:56:27-07:00`

## Artifacts

### `acceptance-matrix.md`

- Purpose: maps Task 92 acceptance criteria to current source files and prior
  packet-local evidence.
- Key result: all Task 92 infrastructure criteria are marked met; Graviton 4
  runtime evidence is explicitly deferred to Tasks 93-98 because Task 92 ships
  no new real SVE2 kernel.

### `status-and-deferral-audit.log`

- Command:
  `rg -n "Status: complete|92-cross-quant-block-kernel-infrastructure|Task 92.s infrastructure closeout|Graviton 4 smoke evidence|status: ACCEPTED|Universal block kernel pattern|ADR-076 was accepted|SVE2" plan/tasks/92-cross-quant-block-kernel-infrastructure.md plan/tasks/README.md docs/block-kernel-development.md spec/adr/ADR-076-universal-block-kernel-pattern.md spec/adr/index.md`
- Purpose: prove complete status, accepted ADR-076, and Graviton 4/SVE2
  deferral wording are present in active docs.
- Key result: Task 92 status is complete; README row is complete; ADR-076 is
  accepted and indexed; docs say Task 92 closeout does not require AWS Graviton
  4 benchmarks.

### `infrastructure-symbol-audit.log`

- Command:
  `rg -n "BlockKernelCounterKey|ec_block_kernel_scoring|record_block_scalar_score_for|current_isa|select_highest_isa|score_block32|missing_kernel|kernel_status|format_block_kernel_counter_lines|task92-offpath-calibration" src crates/ecaz-cli docs spec/adr/ADR-076-universal-block-kernel-pattern.md plan/tasks/92-cross-quant-block-kernel-infrastructure.md`
- Purpose: static source audit for the Task 92 infrastructure surfaces.
- Key result: finds runtime ISA detection, LUT32 module-layout dispatch, block
  counter keys/snapshot SQL, off-path scalar accounting, CLI counter formatting,
  and suite `kernel_status` markers.

### `local-cargo-test-select-highest-isa.log`

- Command:
  `cargo test -p ecaz --no-default-features --features pg18 select_highest_isa`
- Purpose: focused local verification for runtime ISA selection.
- Key result: 4 tests passed, 0 failed, including
  `select_highest_isa_prefers_graviton4_sve2`.

### `local-cargo-test-candidate-batch.log`

- Command:
  `cargo test -p ecaz --no-default-features --features pg18 candidate_batch`
- Purpose: focused local verification for the shared batch container, LUT32
  batch scorer, and block-kernel counter attribution.
- Key result: 12 tests passed, 0 failed, including LUT32 scalar parity,
  surface counter recording, and scalar-tail attribution under `isa=scalar`.

### `local-cargo-test-cli-task92.log`

- Command: `cargo test -p ecaz-cli task92`
- Purpose: focused local verification for Task 92 suite config parsing.
- Key result: 2 tests passed, 0 failed:
  `parses_task92_quant_axis_smoke_config` and
  `parses_task92_offpath_calibration_config`.

### `local-cargo-test-cli-block-kernel-counter-lines.log`

- Command:
  `cargo test -p ecaz-cli block_kernel_counter_lines_include_transition_formats`
- Purpose: focused local verification for the CLI output contract that emits
  direct `[block-kernel-counters]` rows and Task 87 compatibility rows.
- Key result: 1 test passed, 0 failed.

### `git-diff-check.log`

- Command: `git diff --check`
- Purpose: whitespace sanity check.
- Key result: passed with exit code 0.

## Notes

No GitHub CI, AWS smoke tests, or AWS benchmarks were run for this closeout
packet.
