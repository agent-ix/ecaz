# Artifact manifest — Task 194 packet 008

- Task bucket / packet: `reviews/task-194/008-nine-way-completion-audit/`.
- Implementation SHA: `1b5e201a9`.
- Canonical suite config:
  `reviews/task-194/002-nine-way-attribution/artifacts/suite/task194-suite.json`.
- Lane: local Intel, three isolated PG18 owner instances; trained exact
  landmark head, RaBitQ stored neighbor values, exact co-located row rerank,
  lazy10, BW=4/H=100.
- Protocol: 200 recall queries / 2,000 trials and 10 warmups + 50 measured
  latency iterations, stage/work attribution enabled.
- Validation:
  - strict normal PG18 clippy with warnings denied: passed;
  - strict PG18 attribution-feature clippy with warnings denied: passed;
  - focused reconciliation parser test: 1 passed.
  - canonical suite audit: passed, one step.
- Installed extension preflight: target and installed PG18 libraries are both
  24,271,128 bytes with SHA-256
  `1f08db214b8ed61e1197307754f343947d02327098a8b83190bea9fc5f21fdb7`.
- Planned command: `target/debug/ecaz bench suite run --config
  reviews/task-194/002-nine-way-attribution/artifacts/suite/task194-suite.json
  --database tqvector_bench --log-file
  reviews/task-194/008-nine-way-completion-audit/artifacts/suite-run.log`.

The manifest/results, compact summary, and physical recall/latency logs will be
added after the run. Operational node logs, fixture transcript, and
single-control raw logs will not be committed.

## Pre-run files

- `release-install.log`: release install transcript.
- `suite-audit.log`: suite shape/input audit.
