# Bench-Suite Emitter Fix Plan

Task 94 owns the latency-suite direct counter emission gap called out in
Task 92 Packet 014 feedback.

## Current State

The extension counter API already exposes both views:

- `ec_block_kernel_scoring_snapshot()`
- `ec_task87_candidate_batch_scoring_snapshot()`

The CLI also already has `snapshot_block_kernel_counters` and
`format_block_kernel_counter_lines`, which can format direct
`[block-kernel-counters]` rows plus the Task 87 compatibility rows.

The remaining gap is suite-level preservation of the direct rows in latency
artifacts and normalized suite outputs. Packet 014 artifacts showed only
`[task87-counters]` lines, so Task 99 would have to infer direct block-kernel
state indirectly.

## Phase 6 Change

When the T94 AM registration work reaches benchmark plumbing:

1. Keep the existing latency flag spelling for compatibility:
   `--task87-candidate-batch-counters`.
2. Ensure `bench latency` logs include direct `[block-kernel-counters]` lines
   whenever that flag is enabled.
3. Ensure `ecaz bench suite` captures those direct lines in the step artifact
   and, if `results.jsonl` is expected to carry counter rows, extend the suite
   parser with a `metric=block_kernel_counters` row shape instead of dropping
   the lines during table parsing.
4. Preserve `[task87-counters]` compatibility lines until downstream consumers
   are migrated.

## Validation

Local validation for the emitter fix should be CLI-level and not AWS-backed:

- a unit test for `format_block_kernel_counter_lines` showing both direct and
  compatibility lines;
- a suite parser test with raw latency output containing
  `[block-kernel-counters]` and `[task87-counters]`;
- a packet-local PG18 latency smoke only if the parser/formatter tests do not
  cover the exercised path.

The Graviton 4 measurement packet later in T94 must include direct
`[block-kernel-counters]` rows and measured runtime SVE2 vector length.

