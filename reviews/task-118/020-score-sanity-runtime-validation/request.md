---
task: 118
packet: reviews/task-118/020-score-sanity-runtime-validation
checkpoint_sha: c64247c5d14dba08b0799f9df1ddcf5c79f1613c
branch: task-118-hnsw-quantized-recall-attribution
role: coder
date: 2026-06-21
---

# Review Request: Score-Sanity Runtime Validation Handoff

## Scope

This checkpoint retries the packet 009 synthetic score-correlation runtime test
on the current branch head and wires that test into the final Intel closeout
handoff.

The attempted command was:

```bash
cargo pgrx test pg18 test_ech_score_correlation_synthetic_known_ordering
```

On this slower AMD host, the command again remained at the `Compiling ecaz`
line and was interrupted to avoid leaving a long-running process active. This
does not validate the fixture, but it records a current-head retry and makes
the remaining requirement explicit for the Intel/normal PG18 host.

Updated:

- `reviews/task-118/010-intel-closeout-runbook/artifacts/intel-closeout-runbook.md`
- `reviews/task-118/011-final-closeout-audit-template/artifacts/final-closeout-audit-template.md`

## Validation

- Artifact: `artifacts/cargo-pgrx-test-pg18-score-sanity-rerun.log`
- Result: inconclusive; interrupted during compile on AMD.

The runbook now requires this final artifact before closeout:

`reviews/task-118/006-final-attribution-matrix/artifacts/cargo-pgrx-test-pg18-score-sanity-intel.log`

## Remaining Task 118 Closeout Work

Run the focused score-sanity `cargo pgrx test` on the Intel/normal PG18 host,
then run the Intel 10k/50k/100k suites and complete packet 006.
