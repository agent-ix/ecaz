# Task 204 review request: storage-step arm fidelity

## Scope

This checkpoint moves ec_distann storage measurement into the physical arm
loop, emits replica relation and cache measurements into structured suite rows,
adds per-node resident rows and the summed/per-node ratio rows, and teaches the
suite parser to retain those rows in `results.jsonl`.

The implementation was bundled into the earlier Task 203 docs checkpoint by
`d27e2fdde`; the attribution and file list are recorded in
`reviews/task-203/001-decision-reaudit/artifacts/commit-bundling-note.md`.
The follow-up wiring/fix checkpoint is `615fd72b2d6d31d7bec9020eabcfa8fa34d39a68`.

## Validation

- PG18 scan-focused tests: 13 passed, 0 failed; see
  `artifacts/pg18-focused.log`.
- Corrected reread of Task 198/199: see
  `artifacts/corrected-198-199-reread.md`.
- Required 100k two-arm demonstration: suite config is checked in, but the
  local audit is blocked by missing staged corpus files; see
  `artifacts/benchmark-preflight.log`. This packet does not claim the task's
  benchmark gate is satisfied until that run is executed.

## Review focus

Please review arm attribution, the coordinator-side relation accounting,
mandatory ratio/per-node rows, and the structured parser path. The packet is
open pending the required two-arm 100k evidence.
