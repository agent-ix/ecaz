# Task 218 — P1 production lazy-10 attribution

Entry gate: Task 217’s same-generation lane implementation is committed in
`15834e2e4`; its A/A proof is the associated Task 217 packet.

This packet pre-registers the first and mandatory phase of Task 218. It runs a
feature-instrumented, three-owner, 100k physical control with the production
lazy-10 materialization setting. It records the true owner payload SQL,
endpoint, locator, payload-count, and executor-row budget before selecting any
candidate. MAT-16, MAT-21, and MAT-22 remain unselected until this denominator
is measured.

The committed SuiteConfig is the only runner. NFR-021/NFR-022 register the
sharded owner control as conforming; no single-instance or traversal-replica
arm is used as a decision control.

Static validation completed:

- `cargo check -p ecaz-cli` passed (one pre-existing dead-code warning).
- `ecaz bench suite audit --config .../task218-lazy10-attribution.json`
  passed.

Review closure requires the packet-local 100k `results.jsonl` and manifest
citations. If the lazy-10 addressable budget is too small to justify a
candidate, the correct disposition is STOP at P1; otherwise exactly one
candidate may be pre-registered in packet 002.
