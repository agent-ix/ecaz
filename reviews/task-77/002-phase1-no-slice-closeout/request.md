# Task 77 Closeout: Phase 1 Candidate Attribution No-Slice Decision

## Summary

Task 77 closes through the task's allowed no-slice path. The Phase 1
Intel-local packet shows the high-recall SPIRE candidate path is dominated by
approximate quantized scoring, not row materialization or heap-retained
candidate maintenance.

The evidence is in:

- benchmark packet:
  `benchmarks/task77-intel-local-candidate-cost-attribution/`
- benchmark manifest:
  `benchmarks/task77-intel-local-candidate-cost-attribution/manifest.md`
- suite manifest:
  `benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-manifest.json`
- attribution summary:
  `benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/funnel-attribution-summary.json`

## Decision

No SPIRE-local materialization slice should land in Task 77.

The materialization plus heap-append p50 totals are only:

- `3.000 ms` at tg64/nprobe64,
- `4.458 ms` at tg96/nprobe96,
- `5.777 ms` at tg128/nprobe128.

Even eliminating them completely would not clear the task's required `>=10%`
p50 win at the matched 100k high-recall point. Candidate scoring is the
dominant measured cost instead:

- tg64/nprobe64: `82.9%` of measured candidate-path time,
- tg96/nprobe96: `82.1%`,
- tg128/nprobe128: `83.2%`.

Object reads are also non-trivial at the high-recall points, including
`17.934 ms` p50 at tg96/nprobe96. Task 77 does not land an object-read slice
because reducing that cost requires reducing the candidate surface and/or
format-specific object access, not a bounded SPIRE-local candidate
materialization change. That work is part of the Task 78 RaBitQ-first candidate
reduction lane, where it can be evaluated as part of SPIRE latency
optimization against the same matched-recall evidence instead of being hidden
inside the Task 77 materialization closeout.

Task 78 is added as the follow-up RaBitQ-first SPIRE latency optimization lane.
It should start from candidate-count reduction; TurboQuant should remain in the
evidence matrix only as a comparison point.

## Validation

- `target/debug/ecaz --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-audit.log bench suite audit --config benchmarks/task77-intel-local-candidate-cost-attribution/suite.json`
- `target/debug/ecaz --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-run.log --database task77_spire_attribution --host /home/peter/.pgrx --port 28818 bench suite run --config benchmarks/task77-intel-local-candidate-cost-attribution/suite.json --manifest-output benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-manifest.json --results-output benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/results.jsonl`
- `target/debug/ecaz --log-file benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-status.log bench suite status --manifest benchmarks/task77-intel-local-candidate-cost-attribution/artifacts/suite-manifest.json`

Suite status: `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

PG18 clippy is run after this docs/packet update and recorded in
`artifacts/clippy-pg18.log`.

## AWS

No AWS run was started. Task 77 required AWS only after a local slice cleared
the matched-recall p50 gate; this closeout does not land such a slice.
