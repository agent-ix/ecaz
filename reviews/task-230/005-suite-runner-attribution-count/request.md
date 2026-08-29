---
task: 230
packet: 005-suite-runner-attribution-count
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 01
---

# Task 230 suite-runner attribution-count correction

Review the standalone runner correction at `0b15cf020`. Packet 004's authorized
suite stopped in its first 10k control arm before producing a valid result:
the release preflight passed at `35648e467`, but the CLI rejected 62 emitted
materialization-work rows because its invariant still expected 52.

## Root cause and correction

`DistannMaterializationWork::ALL` now contains 61 server-side metrics. Task 230
added ten hot/cold attribution counters, but
`run_physical_benchmarks` retained the earlier 51-metric comment and
52-row constant. The latency child appends exactly one `client_result_rows`
metric, so the valid total is 62 rows per concurrency group.

The code commit changes only the stale comment and constant from 51/52 to
61/62. It does not alter the frozen suite config, measurement options,
thresholds, or result interpretation.

## Validation and disposition of the failed attempt

- Focused CLI test:
  `cargo test -p ecaz-cli task230_io_projections_mirror_all_six_end_to_end_shapes`
  — 1 passed, 0 failed.
- The first attempted Packet 004 arm is invalid and will not be interpreted or
  resumed. Its key preflight and failure lines are copied into
  `artifacts/failed-first-arm-summary.log`.
- After review closure, the extension will be reinstalled in release mode and
  the matching CLI rebuilt at the accepted head. The frozen 20-step suite will
  restart from step 1 with a new empty results file and a clean run directory.

## Review request

Please verify that 62 is the exact contract implied by 61 server rows plus one
client row, and that the change is properly isolated. If DONE, Packet 004's
unchanged matrix may restart from a clean fixture.
