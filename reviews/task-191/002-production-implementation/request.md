---
task: 191
packet: 002-production-implementation
role: coder
status: review_requested
head_sha: 677e2d1d5af25426023af19015902bde1aa4e314
date: 2026-07-20
---

# Review request: production lazy payload implementation

## Scope

This checkpoint makes deterministic global-ranked payload windows of exactly
10 the unconditional physical `ec_distann` scan path. The benchmark feature
retains an explicit eager control (`0`) and positive test window overrides;
normal builds expose no materialization-size tuning surface. No persisted
format, traversal, head, codec, placement, or protocol choice changes.

The implementation also closes the four Task 184 review carryovers:

- the semantic fixture forces incompressible `STORAGE EXTERNAL` payloads and
  attests their storage shape;
- stable materialized payloads survive ranked-prefix deepening, with a
  feature-only duplicate-remote-request counter that must remain zero;
- eager `output_merge` and lazy `materialize_output_associate` attribution are
  mutually exclusive; and
- the suite captures its Git descriptor before tracked artifact writes.

An initial semantic run exposed that the CLI omitted the explicit eager value
`0` after the production default changed from eager to lazy. Commit
`677e2d1d5` fixes both recall and latency child sessions and adds a regression
test. The accepted run below uses that corrected runner; the invalid run and
regenerable exhaust were not retained.

## Validation

- PG18 normal and benchmark-feature library checks passed during the code
  checkpoint.
- Stable-prefix deepening unit coverage passed.
- Four CLI materialization/parser/suite-report tests passed during the code
  checkpoint.
- The focused explicit-zero forwarding regression passes: 1 passed, 0 failed.
- The checked-in semantic suite audits cleanly and reports one succeeded step,
  no failed/skipped/stale/missing steps, and clean runner descriptor
  `677e2d1d5af25426023af19015902bde1aa4e314`.

## Semantic and failure findings

All eager-versus-production comparisons preserve output identity and ordering:

- LIMIT 5/10/15: payload reads 8/10/15 within bounds 10/10/20;
- reject first window: 20 reads within the fixed 1,024 deepening cap;
- reject multiple windows: 50 reads within the cap and zero duplicate remote
  candidate requests;
- NULL and genuinely external-TOAST projection/qual cases pass;
- mixed local/remote winners pass; and
- an owner failure after the first 10-row window aborts the later request.

The short 10k smoke is not the promotion gate, but confirms the A/B controls:

| Variant | Recall | Mean latency | Remote candidates/scan |
| --- | ---: | ---: | ---: |
| eager control (`0`) | 0.9950 | 43.60 ms | 31.33 |
| production lazy10 | 0.9950 | 25.30 ms | 6.67 |

Both arms report zero duplicate remote requests. Eager records three
`output_merge` samples and zero `materialize_output_associate` samples; lazy
records the inverse (six associate samples, zero merge samples).

## Review focus

Please review stable-prefix datum ownership/identity, later-window failure
behavior, the absence of a production tuning knob, feature-gate isolation, and
whether the semantic assertions fully cover the Task 191 contract. Full
10k/50k/100k promotion evidence follows in packet 003.

Evidence and exact commands are indexed by
[`artifacts/manifest.md`](artifacts/manifest.md).
