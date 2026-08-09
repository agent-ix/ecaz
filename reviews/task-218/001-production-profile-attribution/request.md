# Task 218 — P1 production lazy-10 attribution

Entry gate: Task 217’s same-generation lane implementation is committed in
`15834e2e4`; its A/A proof is the associated Task 217 packet.

This packet records the first and mandatory phase of Task 218. It runs a
feature-instrumented, three-owner, 100k physical control with the production
lazy-10 materialization setting. It records the true owner payload SQL,
endpoint, locator, payload-count, and executor-row budget before selecting any
candidate. MAT-16, MAT-21, and MAT-22 remained unselected until this denominator
was measured; MAT-21 is isolated in packet 002.

The committed SuiteConfig is the only runner. NFR-021/NFR-022 register the
sharded owner control as conforming; no single-instance or traversal-replica
arm is used as a decision control.

Validation and evidence completed:

- `cargo check -p ecaz-cli` passed (one pre-existing dead-code warning).
- `ecaz bench suite audit --config .../task218-lazy10-attribution.json`
  passed.
- `ecaz bench suite status` completed one step with zero failures, skipped
  steps, missing artifacts, or stale artifacts.
- Production lazy-10 recall was 0.9275 and warm latency was 21.00 ms.
- `custom_scan_total` was 18.826614 ms/scan; owner endpoint work was 9.100528
  ms/scan and owner payload SQL work was 8.751874 ms/scan. Remote executor
  rows were 6.66/scan.

The compact cited lines are in `artifacts/run/100k/attribution-evidence.log`
and the complete structured output is `artifacts/run/results.jsonl`. The
addressable budget justified exactly one candidate, MAT-21, which is measured
in packet 002.
