# Review request — Task 211 P1/P2: head scaling law implementation

- Task: `plan/tasks/211-ec-distann-head-scaling-law.md`
- Packet: `reviews/task-211/002-head-scaling-law-implementation/`
- Code head: `a08f6fe60` (`fix(distann): apply head sizing to physical fixtures`)
- Date: 2026-08-01. Coder: Codex

## What to review

This checkpoint implements the build-time sampling law, deterministic v3
attestation, explicit cap override, and suite provenance/counter plumbing.
The physical fixture now applies the law to the actual index build.

## Validation

PG18 checks and the focused attestation test passed. The corrected 0.02 law
arms resolve to 200/1000/2000 sampled records at 10k/50k/100k. Physical
recall / mean latency / storage ratio, compared with the final fixed-cap
counter baseline, is:

| scale | fixed-cap | 0.02 law | law sample / hops (fixed → law) |
| --- | --- | --- | --- |
| 10k | 0.9940 / 39.50 / 1.235467 | 0.9950 / 39.10 / 1.235467 | 200 / 11.90 → 11.60 |
| 50k | 0.9595 / 52.70 / 1.332640 | 0.9555 / 53.50 / 1.332693 | 1000 / 16.10 → 14.20 |
| 100k | 0.9145 / 53.30 / 1.351173 | 0.9155 / 55.80 / 1.351147 | 2000 / 15.68 → 14.96 |

The evidence does not show a consistent win at all scales, so the shipped
default remains fixed cap 4096; the law is available as an opt-in reloption.
The structured sources are `artifacts/bench-run-law-fixed/results.jsonl`,
`artifacts/bench-run-control100-fixed/results.jsonl`, and the final counter
baseline in the Task 212 packet, with provenance summarized in
`artifacts/manifest.md`.

## Status

Open — implementation and evidence complete; awaiting outside reviewer feedback.
