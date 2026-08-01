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
- Required 100k two-arm demonstration: passed with the staged `ec_real_100k`
  corpus. The suite completed successfully; see the packet-local
  `artifacts/run-final/results.jsonl`, `artifacts/run-final/suite-manifest.json`,
  and `artifacts/run-final/storage-two-arm-100k/distann-multinode-summary.log`.
  `skip_recall` and `skip_single_control` are deliberate because this packet
  is the Task 204 storage-fidelity demonstration; both physical storage arms
  and their latency measurements still ran.

## Review focus

Please review arm attribution, the coordinator-side relation accounting,
mandatory ratio/per-node rows, and the structured parser path. The packet is
open for outside review with the required two-arm 100k evidence attached.

## Feedback follow-up

Checkpoint `045ce69e7` makes the storage-ratio row mandatory for every
physical step and adds a focused regression test for that assertion. The
focused CLI `distann_` run passes 44 tests. The Task 205 bounded-L rerun also
exercises the corrected parser at 10k/50k/100k and emits all nine storage-ratio
rows plus per-arm growth rows; see
`reviews/task-205/004-l-bounded-rerun/artifacts/run-v2/results.jsonl`.
Raw fixed-roster growth remains measurement-only pending a stable NFR-021
owner-record metric, rather than being presented as a hard 2.0 gate.
