# Task 185 packet 004 artifact manifest

Date: 2026-07-23 (America/Los_Angeles)

This is a decision-only packet. Its immutable measurement source is
`reviews/task-185/003-fixed-cap-screen/`, especially:

- `artifacts/fixed-cap-screen-100k-suite.json`;
- `artifacts/run/suite-manifest.json`;
- `artifacts/run/results.jsonl`;
- `artifacts/run/report.md`; and
- `artifacts/manifest.md`.

Runner head, extension head, release identity, corpus/query identity,
three-owner exact/disjoint topology, commands, result checksums, recall,
latency, storage, and construction values are recorded in packet 003's
manifest and are not duplicated as new measurements here.

No full-scale suite was run. Task 185 permits 10k/50k/100k confirmation only
for one useful 100k candidate. All four 100k cells tied at recall 0.9625;
gateway membership was set-identical to control; basin diversification added
roughly 46--48 ms/query. Advancing either candidate would violate that
pre-registered conditional gate.

Decision: **STOP** Task 185 and enter conditional Task 186 with the unchanged
Task 182 `training_landmarks_exact` cap-4,096 control.
